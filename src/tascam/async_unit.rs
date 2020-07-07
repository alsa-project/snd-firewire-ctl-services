// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};
use glib::source;

use nix::sys::signal;
use std::sync::{mpsc, Arc, Mutex};

use hinawa::{FwNodeExt, FwRespExt, FwRespExtManual};

use crate::dispatcher;

use super::protocol::{BaseProtocol, ExpanderProtocol, GetPosition};
use super::fe8_model::Fe8Model;

enum AsyncUnitEvent {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Surface((u32, u32, u32)),
}

pub struct AsyncUnit {
    node: hinawa::FwNode,
    resp: hinawa::FwResp,
    rx: mpsc::Receiver<AsyncUnitEvent>,
    tx: mpsc::SyncSender<AsyncUnitEvent>,
    dispatchers: Vec<dispatcher::Dispatcher>,
    req: hinawa::FwReq,
    state_cntr: Arc<Mutex<[u32; 32]>>,
    led_states: std::collections::HashMap::<u16, bool>,
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

    pub fn new(node: hinawa::FwNode, _: String) -> Result<Self, Error> {
        let resp = hinawa::FwResp::new();

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();

        Ok(AsyncUnit {
            node,
            resp,
            tx,
            rx,
            dispatchers,
            req: hinawa::FwReq::new(),
            state_cntr: Arc::new(Mutex::new([0;32])),
            led_states: std::collections::HashMap::new(),
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
        self.resp.connect_requested(move |resp, tcode| {
            // This application can handle any write request.
            if tcode != hinawa::FwTcode::WriteQuadletRequest
                && tcode != hinawa::FwTcode::WriteBlockRequest
            {
                return hinawa::FwRcode::AddressError;
            }

            let frames = resp.get_req_frames();
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

        Ok(())
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.register_address_space()?;

        self.init_led()?;

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
                AsyncUnitEvent::Surface((_, _, _)) => (),
            }
        }
    }
}

pub trait ConsoleData<'a> {
    const FW_LED: &'a [u16];
}
