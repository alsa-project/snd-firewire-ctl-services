// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};
use glib::source;

use nix::sys::signal;
use std::sync::{mpsc, Arc, Mutex};

use hinawa::{FwNodeExt, FwRespExt, FwRespExtManual};

use alsaseq::{UserClientExt, EventCntrExt, EventCntrExtManual};

use crate::dispatcher;

use super::seq_cntr;

use super::protocol::{BaseProtocol, ExpanderProtocol, GetPosition, DetectAction, DetectPosition};
use super::protocol::{GetValue, ComputeValue};

use super::fe8_model::Fe8Model;

enum AsyncUnitEvent {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Surface((u32, u32, u32)),
    SeqAppl(alsaseq::EventDataCtl),
}

pub struct AsyncUnit {
    node: hinawa::FwNode,
    resp: hinawa::FwResp,
    seq_cntr: seq_cntr::SeqCntr,
    rx: mpsc::Receiver<AsyncUnitEvent>,
    tx: mpsc::SyncSender<AsyncUnitEvent>,
    dispatchers: Vec<dispatcher::Dispatcher>,
    req: hinawa::FwReq,
    state_cntr: Arc<Mutex<[u32; 32]>>,
    led_states: std::collections::HashMap::<u16, bool>,
    button_states: std::collections::HashMap::<(u32, u32), bool>,
    msg_map: Vec<(u32, u32)>,
}

impl Drop for AsyncUnit {
    fn drop(&mut self) {
        self.led_states.iter().for_each(|(&pos, &state)| {
            if state {
                let _ = self.req.bright_led(&self.node, pos, false);
            }
        });
        let _ = self.req.enable_notification(&self.node, false);
        self.resp.release();
        self.dispatchers.clear();
    }
}

impl<'a> AsyncUnit {
    const NODE_DISPATCHER_NAME: &'a str = "node event dispatcher";

    pub fn new(node: hinawa::FwNode, name: String) -> Result<Self, Error> {
        let resp = hinawa::FwResp::new();

        let seq_cntr = seq_cntr::SeqCntr::new(name)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();

        Ok(AsyncUnit {
            node,
            resp,
            seq_cntr,
            tx,
            rx,
            dispatchers,
            req: hinawa::FwReq::new(),
            state_cntr: Arc::new(Mutex::new([0;32])),
            led_states: std::collections::HashMap::new(),
            button_states: std::collections::HashMap::new(),
            msg_map: Vec::new(),
        })
    }

    fn launch_node_event_dispatcher(&mut self) -> Result<(), Error> {
        // Use a dispatcher.
        let name = Self::NODE_DISPATCHER_NAME.to_string();
        let mut dispatcher = dispatcher::Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_fw_node(&self.node, move |_| {
            let _ = tx.send(AsyncUnitEvent::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.node.connect_bus_update(move |node| {
            let generation = node.get_property_generation();
            let _ = tx.send(AsyncUnitEvent::BusReset(generation));
        });

        let tx = self.tx.clone();
        dispatcher.attach_signal_handler(signal::Signal::SIGINT, move || {
            let _ = tx.send(AsyncUnitEvent::Shutdown);
            source::Continue(false)
        });

        let tx = self.tx.clone();
        dispatcher.attach_snd_seq(&self.seq_cntr.client)?;
        self.seq_cntr
            .client
            .connect_handle_event(move |_, ev_cntr| {
                let _ = (0..ev_cntr.count_events())
                    .filter(|&i| {
                        // At present, controller event is handled.
                        ev_cntr.get_event_type(i).unwrap_or(alsaseq::EventType::None) == alsaseq::EventType::Controller
                    }).for_each(|i| {
                        if let Ok(ctl_data) = ev_cntr.get_ctl_data(i) {
                            let data = AsyncUnitEvent::SeqAppl(ctl_data);
                            let _ = tx.send(data);
                        }
                    });
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    fn register_address_space(&mut self) -> Result<(), Error> {
        // Reserve local address to receive async messages from the
        // unit within private space.
        let mut addr = 0x0000ffffe0000000 as u64;
        while addr < 0x0000fffff0000000 {
            if let Err(_) = self.resp.reserve(&self.node, addr, 0x80) {
                addr += 0x80;
                continue;
            }

            break;
        }
        if !self.resp.get_property_is_reserved() {
            let label = "Fail to reserve address space";
            return Err(Error::new(FileError::Nospc, label));
        }

        let tx = self.tx.clone();
        let state_cntr = self.state_cntr.clone();
        let node_id = self.node.get_property_node_id();
        self.resp.connect_requested2(move |_, tcode, _, src, _, _, _, frames| {
            // This application can handle any write request.
            if tcode != hinawa::FwTcode::WriteQuadletRequest
                && tcode != hinawa::FwTcode::WriteBlockRequest
            {
                return hinawa::FwRcode::TypeError;
            }

            if src != node_id {
                return hinawa::FwRcode::TypeError;
            }

            let len = frames.len() / 4;

            // Operate states under mutual exclusive lock.
            if let Ok(mut states) = state_cntr.clone().lock() {
                (0..len).for_each(|mut i| {
                    i *= 4;
                    let index = frames[i + 1] as usize;
                    let doublet = [frames[i + 2], frames[i + 3]];
                    let state = u16::from_be_bytes(doublet) as u32;

                    if states[index] != state {
                        // Avoid change from initial state.
                        if states[index] != 0 {
                            let ev = (index as u32, states[index], state);
                            let _ = tx.send(AsyncUnitEvent::Surface(ev));
                        }

                        states[index] = state;
                    }
                });
            }

            hinawa::FwRcode::Complete
        });
        // Register the address to the unit.
        addr |= (self.node.get_property_local_node_id() as u64) << 48;
        self.req.register_notification_addr(&self.node, addr)?;

        self.req.enable_notification(&self.node, true)?;

        Ok(())
    }

    fn update_led_if_needed(&mut self, pos: u16, state: bool) -> Result<(), Error> {
        match self.led_states.get(&pos) {
            Some(&s) => {
                if s != state {
                    self.req.bright_led(&self.node, pos, state)?;
                }
            }
            None => self.req.bright_led(&self.node, pos, state)?,
        }

        self.led_states.insert(pos, state);

        Ok(())
    }


    fn init_led(&mut self) -> Result<(), Error> {
        Fe8Model::FW_LED.get_position(|pos| {
            self.update_led_if_needed(pos, true)
        })?;

        Fe8Model::SIMPLE_LEDS.iter().try_for_each(|entries| {
            entries.get_position(|pos| {
                self.update_led_if_needed(pos, false)
            })
        })?;

        Fe8Model::TOGGLED_BUTTONS.iter().try_for_each(|&(key, entries)| {
            entries.get_position(|pos| {
                self.update_led_if_needed(pos, false)?;
                self.button_states.insert(key, false);
                Ok(())
            })
        })?;

        Ok(())
    }

    fn init_msg_map(&mut self) {
        Fe8Model::SIMPLE_LEDS.iter().enumerate().for_each(|(i, _)| {
            let key = (std::u32::MAX, i as u32);
            self.msg_map.push(key);
        });

        Fe8Model::TOGGLED_BUTTONS.iter().for_each(|&(key, _)| {
            self.msg_map.push(key);
        });

        Fe8Model::INPUT_FADERS.iter().for_each(|&(key, _)| {
            self.msg_map.push(key);
        });

        Fe8Model::DIALS.iter().for_each(|&(key, _)| {
            self.msg_map.push(key);
        });
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.register_address_space()?;

        self.seq_cntr.open_port()?;

        self.init_led()?;

        self.init_msg_map();

        Ok(())
    }

    pub fn run(&mut self) {
        loop {
            let ev = match self.rx.recv() {
                Ok(ev) => ev,
                Err(_) => continue,
            };

            match ev {
                AsyncUnitEvent::Shutdown | AsyncUnitEvent::Disconnected => break,
                AsyncUnitEvent::BusReset(generation) => {
                    println!("IEEE 1394 bus is updated: {}", generation);
                }
                AsyncUnitEvent::Surface((index, before, after)) => {
                    let _ = self.dispatch_surface_event(index, before, after);
                }
                AsyncUnitEvent::SeqAppl(ctl_data) => {
                    let _ = self.dispatch_seq_event(ctl_data);
                }
            }
        }
    }

    fn xfer_seq_event(&mut self, key: &(u32, u32), value: i32) -> Result<(), Error> {
        if let Some(param) = self.msg_map.iter().position(|e| e == key) {
            self.seq_cntr.schedule_event(param as u32, value)
        } else {
            Ok(())
        }
    }

    fn dispatch_surface_event(&mut self, index: u32, before: u32, after: u32) -> Result<(), Error>
    {
        Fe8Model::TOGGLED_BUTTONS.detect_action(index, before, after, |key, pos, state| {
            if state {
                let s = match self.button_states.get(key) {
                    Some(s) => !s,
                    None => return Ok(()),
                };

                self.update_led_if_needed(pos, s)?;
                self.xfer_seq_event(key, s.compute_value())?;
                self.button_states.insert(*key, s);
            }
            Ok(())
        })?;

        Fe8Model::INPUT_SENSORS.detect_action(index, before, after, |idx, _, state| {
            if !state {
                let (key, val) = match self.state_cntr.lock() {
                    Ok(s) => {
                        let states: &[u32;32] = &s;
                        Fe8Model::INPUT_FADERS.get_value(states, idx)
                    }
                    Err(_) => return Ok(()),
                };
                self.xfer_seq_event(&key, val as i32)?;
            }
            Ok(())
        })?;

        Fe8Model::DIALS.detect_action(index, before, after, |key, val| {
            self.xfer_seq_event(key, val as i32)
        })?;

        Ok(())
    }

    fn dispatch_seq_event(&mut self, ctl_data: alsaseq::EventDataCtl) -> Result<(), Error>
    {
        if ctl_data.get_channel() != 0 {
            let label = format!("Channel {} is not supported yet.", ctl_data.get_channel());
            return Err(Error::new(FileError::Inval, &label));
        }

        let key = (std::u32::MAX, ctl_data.get_param());
        let index = match self.msg_map.iter().position(|k| k == &key) {
            Some(p) => p,
            None => return Ok(()),
        };

        let state = ctl_data.get_value() > 0;
        Fe8Model::SIMPLE_LEDS.detect_position(index, |pos| {
            self.update_led_if_needed(pos, state)
        })?;

        Ok(())
    }
}

pub trait ConsoleData<'a> {
    const FW_LED: &'a [u16];
    const SIMPLE_LEDS: &'a [&'a [u16]];
    const TOGGLED_BUTTONS: &'a [((u32, u32), &'a [u16])];
    const INPUT_SENSORS: &'a [(u32, u32)];
    const INPUT_FADERS: &'a [((u32, u32), u8)];
    const DIALS: &'a [((u32, u32), u8)];
}
