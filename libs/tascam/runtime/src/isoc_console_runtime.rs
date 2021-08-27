// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};
use glib::source;

use nix::sys::signal;
use std::sync::mpsc;

use hinawa::{FwNodeExt, SndUnitExt, SndTscmExt, SndTscmExtManual};

use alsactl::{CardExt, CardExtManual, ElemValueExtManual};
use alsaseq::{UserClientExt, EventCntrExt, EventCntrExtManual};

use core::dispatcher;
use core::card_cntr;
use card_cntr::{CtlModel, MeasureModel};

use super::fw1884_model::Fw1884Model;
use super::fw1082_model::Fw1082Model;
use super::protocol::{BaseProtocol, GetPosition, DetectPosition, DetectAction, ChooseSingle};
use super::protocol::{GetValue, ComputeValue};

use super::seq_cntr;

enum ConsoleUnitEvent {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Elem((alsactl::ElemId, alsactl::ElemEventMask)),
    Interval,
    SeqAppl(alsaseq::EventDataCtl),
    Surface((u32, u32, u32)),
}

enum ConsoleModel {
    Fw1884(Fw1884Model),
    Fw1082(Fw1082Model),
}

pub struct IsocConsoleRuntime {
    unit: hinawa::SndTscm,
    model: ConsoleModel,
    card_cntr: card_cntr::CardCntr,
    seq_cntr: seq_cntr::SeqCntr,
    rx: mpsc::Receiver<ConsoleUnitEvent>,
    tx: mpsc::SyncSender<ConsoleUnitEvent>,
    dispatchers: Vec<dispatcher::Dispatcher>,
    timer: Option<dispatcher::Dispatcher>,
    measure_elems: Vec<alsactl::ElemId>,

    req: hinawa::FwReq,
    msg_map: Vec<(u32, u32)>,
    led_states: std::collections::HashMap<u16, bool>,
    button_states: std::collections::HashMap::<(u32, u32), bool>,
    bank_state: usize,
    transport_state: usize,
}

impl Drop for IsocConsoleRuntime {
    fn drop(&mut self) {
        let node = self.unit.get_node();
        self.led_states.iter().filter(|&(_, &state)| state).for_each(|(&pos, _)|{
            let _ = self.req.bright_led(&node, pos, false);
        });

        self.dispatchers.clear();
    }
}

impl<'a> IsocConsoleRuntime {
    const NODE_DISPATCHER_NAME: &'a str = "node event dispatcher";
    const SYSTEM_DISPATCHER_NAME: &'a str = "system event dispatcher";
    const TIMER_DISPATCHER_NAME: &'a str = "interval timer dispatcher";

    const TIMER_NAME: &'a str = "metering";
    const TIMER_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);

    pub fn new(unit: hinawa::SndTscm, name: &str, sysnum: u32) -> Result<Self, Error> {
        let model = match name {
            "FW-1884" => ConsoleModel::Fw1884(Fw1884Model::new()),
            "FW-1082" => ConsoleModel::Fw1082(Fw1082Model::new()),
            _ => {
                let label = format!("Unsupported model name: {}", name);
                return Err(Error::new(FileError::Nxio, &label));
            }
        };

        let card_cntr = card_cntr::CardCntr::new();
        card_cntr.card.open(sysnum, 0)?;

        let seq_cntr = seq_cntr::SeqCntr::new(name)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();

        Ok(IsocConsoleRuntime {
            unit,
            model,
            card_cntr,
            seq_cntr,
            tx,
            rx,
            dispatchers,
            timer: None,
            measure_elems: Vec::new(),
            req: hinawa::FwReq::new(),
            msg_map: Vec::new(),
            led_states: std::collections::HashMap::new(),
            button_states: std::collections::HashMap::new(),
            bank_state: 0,
            transport_state: 0,
        })
    }

    fn launch_node_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = Self::NODE_DISPATCHER_NAME.to_string();
        let mut dispatcher = dispatcher::Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_snd_unit(&self.unit, move |_| {
            let _ = tx.send(ConsoleUnitEvent::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.connect_control(move |_, index, before, after| {
            let _ = tx.send(ConsoleUnitEvent::Surface((index, before, after)));
        });

        let tx = self.tx.clone();
        dispatcher.attach_fw_node(&self.unit.get_node(), move |_| {
            let _ = tx.send(ConsoleUnitEvent::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.get_node().connect_bus_update(move |node| {
            let generation = node.get_property_generation();
            let _ = tx.send(ConsoleUnitEvent::BusReset(generation));
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    fn launch_system_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = Self::SYSTEM_DISPATCHER_NAME.to_string();
        let mut dispatcher = dispatcher::Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_signal_handler(signal::Signal::SIGINT, move || {
            let _ = tx.send(ConsoleUnitEvent::Shutdown);
            source::Continue(false)
        });

        let tx = self.tx.clone();
        dispatcher.attach_snd_card(&self.card_cntr.card, |_| {})?;
        self.card_cntr
            .card
            .connect_handle_elem_event(move |_, elem_id, events| {
                let _ = tx.send(ConsoleUnitEvent::Elem((elem_id.clone(), events)));
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
                            let data = ConsoleUnitEvent::SeqAppl(ctl_data);
                            let _ = tx.send(data);
                        }
                    });
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.launch_system_event_dispatcher()?;

        self.seq_cntr.open_port()?;

        match &mut self.model {
            ConsoleModel::Fw1884(m) => m.load(&mut self.unit, &mut self.card_cntr)?,
            ConsoleModel::Fw1082(m) => m.load(&mut self.unit, &mut self.card_cntr)?,
        }

        self.init_surface()?;

        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::TIMER_NAME,
            0,
        );
        let _ = self.card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        match &mut self.model {
            ConsoleModel::Fw1884(m) => m.get_measure_elem_list(&mut self.measure_elems),
            ConsoleModel::Fw1082(m) => m.get_measure_elem_list(&mut self.measure_elems),
        }

        Ok(())
    }

    fn start_interval_timer(&mut self) -> Result<(), Error> {
        let mut dispatcher = dispatcher::Dispatcher::run(Self::TIMER_DISPATCHER_NAME.to_string())?;
        let tx = self.tx.clone();
        dispatcher.attach_interval_handler(Self::TIMER_INTERVAL, move || {
            let _ = tx.send(ConsoleUnitEvent::Interval);
            source::Continue(true)
        });

        self.timer = Some(dispatcher);

        Ok(())
    }

    fn stop_interval_timer(&mut self) {
        if let Some(dispatcher) = &self.timer {
            drop(dispatcher);
            self.timer = None;
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            let ev = match self.rx.recv() {
                Ok(ev) => ev,
                Err(_) => continue,
            };

            match ev {
                ConsoleUnitEvent::Shutdown => break,
                ConsoleUnitEvent::Disconnected => break,
                ConsoleUnitEvent::BusReset(generation) => {
                    println!("IEEE 1394 bus is updated: {}", generation);
                }
                ConsoleUnitEvent::Elem((elem_id, events)) => {
                    if elem_id.get_name() != Self::TIMER_NAME {
                        let _ = match &mut self.model {
                            ConsoleModel::Fw1884(m) =>
                                self.card_cntr.dispatch_elem_event(&mut self.unit, &elem_id, &events, m),
                            ConsoleModel::Fw1082(m) =>
                                self.card_cntr.dispatch_elem_event(&mut self.unit, &elem_id, &events, m),
                        };
                    } else {
                        let mut elem_value = alsactl::ElemValue::new();
                        if self.card_cntr.card.read_elem_value(&elem_id, &mut elem_value).is_ok() {
                            let mut vals = [false];
                            elem_value.get_bool(&mut vals);
                            if vals[0] {
                                let _ = self.start_interval_timer();
                            } else {
                                self.stop_interval_timer();
                            }
                        }
                    }
                }
                ConsoleUnitEvent::Interval => {
                    match &mut self.model {
                        ConsoleModel::Fw1884(m) =>{
                            let _ = self.card_cntr.measure_elems(&mut self.unit, &self.measure_elems, m);
                        }
                        ConsoleModel::Fw1082(m) => {
                            let _ = self.card_cntr.measure_elems(&mut self.unit, &self.measure_elems, m);
                        }
                    };
                }
                ConsoleUnitEvent::SeqAppl(ctl_data) => {
                    let _ = self.dispatch_seq_event(ctl_data);
                }
                ConsoleUnitEvent::Surface((index, before, after)) => {
                    let _ = self.dispatch_surface_event(index, before, after);
                }
            }
        }

        Ok(())
    }

    fn update_led_if_needed(&mut self, pos: u16, state: bool) -> Result<(), Error> {
        let node = self.unit.get_node();

        match self.led_states.get(&pos) {
            Some(&s) => {
                if s != state {
                    self.req.bright_led(&node, pos, state)?;
                }
            }
            None => self.req.bright_led(&node, pos, state)?,
        }

        self.led_states.insert(pos, state);

        Ok(())
    }

    fn init_led(&mut self) -> Result<(), Error> {
        match self.model {
            ConsoleModel::Fw1884(_) => Fw1884Model::SIMPLE_LEDS,
            ConsoleModel::Fw1082(_) => Fw1082Model::SIMPLE_LEDS,
        }.iter().try_for_each(|entries| {
            entries.get_position(|pos| {
                self.update_led_if_needed(pos, false)
            })
        })?;

        match self.model {
            ConsoleModel::Fw1884(_) => Fw1884Model::STATELESS_BUTTONS,
            ConsoleModel::Fw1082(_) => Fw1082Model::STATELESS_BUTTONS,
        }.iter().try_for_each(|(_, entries)| {
            entries.get_position(|pos| {
                self.update_led_if_needed(pos, false)
            })
        })?;

        match self.model {
            ConsoleModel::Fw1884(_) => Fw1884Model::TOGGLED_BUTTONS,
            ConsoleModel::Fw1082(_) => Fw1082Model::TOGGLED_BUTTONS,
        }.iter().try_for_each(|&(_, entries)| {
            entries.get_position(|pos| {
                self.update_led_if_needed(pos, false)
            })
        })?;

        match self.model {
            ConsoleModel::Fw1884(_) => Fw1884Model::BANK_LEDS,
            ConsoleModel::Fw1082(_) => Fw1082Model::BANK_LEDS,
        }.choose_single(self.bank_state, |pos, state| {
            self.update_led_if_needed(pos, state)
        })?;

        self.transport_state = 2;
        match self.model {
            ConsoleModel::Fw1884(_) => Fw1884Model::TRANSPORT_LEDS,
            ConsoleModel::Fw1082(_) => Fw1082Model::TRANSPORT_LEDS,
        }.choose_single(self.transport_state, |pos, state| {
            self.update_led_if_needed(pos, state)
        })?;

        Ok(())
    }

    fn init_msg_map(&mut self) {
        match self.model {
            ConsoleModel::Fw1884(_) => Fw1884Model::SIMPLE_LEDS,
            ConsoleModel::Fw1082(_) => Fw1082Model::SIMPLE_LEDS,
        }.iter().enumerate().for_each(|(i, _)| {
            let key = (std::u32::MAX, i as u32);
            self.msg_map.push(key);
        });

        match self.model {
            ConsoleModel::Fw1884(_) => Fw1884Model::STATELESS_BUTTONS,
            ConsoleModel::Fw1082(_) => Fw1082Model::STATELESS_BUTTONS,
        }.iter().for_each(|(key, _)| {
            self.msg_map.push(*key);
        });

        match self.model {
            ConsoleModel::Fw1884(_) => Fw1884Model::TOGGLED_BUTTONS,
            ConsoleModel::Fw1082(_) => Fw1082Model::TOGGLED_BUTTONS,
        }.iter().for_each(|(key, _)| {
            self.msg_map.push(*key);
        });

        match self.model {
            ConsoleModel::Fw1884(_) => Fw1884Model::BANK_CURSORS,
            ConsoleModel::Fw1082(_) => Fw1082Model::BANK_CURSORS,
        }.iter().for_each(|key| {
            self.msg_map.push(*key);
        });

        match self.model {
            ConsoleModel::Fw1884(_) => Fw1884Model::TRANSPORT_BUTTONS,
            ConsoleModel::Fw1082(_) => Fw1082Model::TRANSPORT_BUTTONS,
        }.iter().for_each(|key| {
            self.msg_map.push(*key);
        });

        match self.model {
            ConsoleModel::Fw1884(_) => Fw1884Model::SIMPLE_BUTTONS,
            ConsoleModel::Fw1082(_) => Fw1082Model::SIMPLE_BUTTONS,
        }.iter().for_each(|key| {
            self.msg_map.push(*key);
        });

        match self.model {
            ConsoleModel::Fw1884(_) => Fw1884Model::INPUT_FADERS,
            ConsoleModel::Fw1082(_) => Fw1082Model::INPUT_FADERS,
        }.iter().for_each(|(key, _)| {
            self.msg_map.push(*key);
        });

        match self.model {
            ConsoleModel::Fw1884(_) => Fw1884Model::DIALS,
            ConsoleModel::Fw1082(_) => Fw1082Model::DIALS,
        }.iter().for_each(|(key, _)| {
            self.msg_map.push(*key);
        });
    }

    fn init_surface(&mut self) -> Result<(), Error> {
        self.init_led()?;

        match self.model {
            ConsoleModel::Fw1884(_) => Fw1884Model::TOGGLED_BUTTONS,
            ConsoleModel::Fw1082(_) => Fw1082Model::TOGGLED_BUTTONS,
        }.iter().for_each(|&(key, _)| {
            self.button_states.insert(key, false);
        });

        self.init_msg_map();

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
        match self.model {
            ConsoleModel::Fw1884(_) => Fw1884Model::SIMPLE_LEDS,
            ConsoleModel::Fw1082(_) => Fw1082Model::SIMPLE_LEDS,
        }.detect_position(index, |pos| {
            self.update_led_if_needed(pos, state)
        })?;

        Ok(())
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
        match self.model {
            ConsoleModel::Fw1884(_) => Fw1884Model::STATELESS_BUTTONS,
            ConsoleModel::Fw1082(_) => Fw1082Model::STATELESS_BUTTONS,
        }.detect_action(index, before, after, |key, pos, state| {
            self.update_led_if_needed(pos, state)?;
            self.xfer_seq_event(&key, state.compute_value())
        })?;

        match self.model {
            ConsoleModel::Fw1884(_) => Fw1884Model::TOGGLED_BUTTONS,
            ConsoleModel::Fw1082(_) => Fw1082Model::TOGGLED_BUTTONS,
        }.detect_action(index, before, after, |key, pos, state| {
            if state {
                let s = match self.button_states.get(&key) {
                    Some(s) => !s,
                    None => return Ok(())
                };
                self.update_led_if_needed(pos, s)?;
                self.xfer_seq_event(&key, s.compute_value())?;
                self.button_states.insert(*key, s);
            }
            Ok(())
        })?;

        let (bank_cursors, bank_leds) = match self.model {
            ConsoleModel::Fw1884(_) => (Fw1884Model::BANK_CURSORS, Fw1884Model::BANK_LEDS),
            ConsoleModel::Fw1082(_) => (Fw1082Model::BANK_CURSORS, Fw1082Model::BANK_LEDS),
        };
        bank_cursors.detect_action(index, before, after, |idx, key, state| {
            if state  {
                if idx > 0 && self.bank_state < 3 {
                    self.bank_state += 1;
                } else if idx == 0 && self.bank_state > 0 {
                    self.bank_state -= 1;
                }

                bank_leds.choose_single(self.bank_state, |pos, state| {
                    self.update_led_if_needed(pos, state)
                })?;
            }
            self.xfer_seq_event(key, state.compute_value())
        })?;

        let (transport_buttons, transport_leds) = match self.model {
            ConsoleModel::Fw1884(_) => (Fw1884Model::TRANSPORT_BUTTONS, Fw1884Model::TRANSPORT_LEDS),
            ConsoleModel::Fw1082(_) => (Fw1082Model::TRANSPORT_BUTTONS, Fw1082Model::TRANSPORT_LEDS),
        };
        transport_buttons.detect_action(index, before, after, |idx, key, state| {
            self.xfer_seq_event(key, state.compute_value())?;
            if state {
                self.transport_state = idx;
                transport_leds.choose_single(self.transport_state, |pos, state| {
                    self.update_led_if_needed(pos, state)
                })
            } else {
                Ok(())
            }
        })?;

        let simple_buttons = match self.model {
            ConsoleModel::Fw1884(_) => Fw1884Model::SIMPLE_BUTTONS,
            ConsoleModel::Fw1082(_) => Fw1082Model::SIMPLE_BUTTONS,
        };
        simple_buttons.detect_action(index, before, after, |_, key, state| {
            self.xfer_seq_event(&key, state.compute_value())
        })?;

        let (input_sensors, input_faders) = match self.model {
            ConsoleModel::Fw1884(_) => (Fw1884Model::INPUT_SENSORS, Fw1884Model::INPUT_FADERS),
            ConsoleModel::Fw1082(_) => (Fw1082Model::INPUT_SENSORS, Fw1082Model::INPUT_FADERS),
        };
        input_sensors.detect_action(index, before, after, |idx, _, state| {
            if !state {
                let states = self.unit.get_state()?;
                let (key, val) = input_faders.get_value(states, idx);
                self.xfer_seq_event(&key, val as i32)?;
            }
            Ok(())
        })?;

        match self.model {
            ConsoleModel::Fw1884(_) => Fw1884Model::DIALS,
            ConsoleModel::Fw1082(_) => Fw1082Model::DIALS,
        }.detect_action(index, before, after, |key, val| {
            self.xfer_seq_event(key, val as i32)
        })?;
        Ok(())
    }
}

pub trait ConsoleData<'a> {
    const SIMPLE_LEDS: &'a [&'a [u16]];
    const STATELESS_BUTTONS: &'a [((u32, u32), &'a [u16])];
    const TOGGLED_BUTTONS: &'a [((u32, u32), &'a [u16])];
    const SIMPLE_BUTTONS: &'a [(u32, u32)];
    const DIALS: &'a [((u32, u32), u8)];

    const BANK_LEDS: &'a [&'a [u16]] = &[
        &[127, 140],
        &[159, 172],
        &[191, 204],
        &[223, 236],
    ];

    const BANK_CURSORS: &'a [(u32, u32)] = &[
        (9, 0x00080000),    // bank L
        (9, 0x00100000),    // bank R
    ];

    const TRANSPORT_LEDS: &'a [&'a [u16]] = &[
        &[13],      // rew
        &[45],      // fwd
        &[242],     // stop
        &[17],      // play
        &[146],     // record
    ];

    const TRANSPORT_BUTTONS: &'a [(u32, u32)] = &[
        (9, 0x08000000),    // rew
        (9, 0x10000000),    // fwd
        (9, 0x20000000),    // stop
        (9, 0x40000000),    // play
        (9, 0x80000000),    // record
    ];

    const INPUT_SENSORS: &'a [(u32, u32)] = &[
        (5, 0x00010000),    // input 1
        (5, 0x00020000),    // input 2
        (5, 0x00040000),    // input 3
        (5, 0x00080000),    // input 4
        (5, 0x00100000),    // input 5
        (5, 0x00200000),    // input 6
        (5, 0x00400000),    // input 7
        (5, 0x00800000),    // input 8
        (5, 0x01000000),    // master
    ];

    const INPUT_FADERS: &'a [((u32, u32), u8)] = &[
        ((0, 0x0000ffff), 0),     // input 1
        ((0, 0x0ffff000), 16),    // input 2
        ((1, 0x0000ffff), 0),     // input 3
        ((1, 0xffff0000), 16),    // input 4
        ((2, 0x0000ffff), 0),     // input 5
        ((2, 0xffff0000), 16),    // input 6
        ((3, 0x0000ffff), 0),     // input 7
        ((3, 0xffff0000), 16),    // input 8
        ((4, 0x0000ffff), 0),     // master
    ];
}
