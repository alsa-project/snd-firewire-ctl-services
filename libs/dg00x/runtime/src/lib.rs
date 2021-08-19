// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
mod model;

mod protocol;

mod monitor_ctl;

use glib::source;
use glib::{Error, FileError};
use nix::sys::signal;
use std::sync::mpsc;
use std::thread;
use std::convert::TryFrom;

use hinawa::{SndUnitExt, SndDg00xExt};
use hinawa::{FwNodeExt, FwNodeExtManual};

use alsactl::{CardExt, CardExtManual, ElemEventMask, ElemIfaceType, ElemId, ElemValue, ElemValueExtManual};

use core::dispatcher::*;
use core::card_cntr::*;
use core::RuntimeOperation;

use ieee1212_config_rom::ConfigRom;
use ta1394::config_rom::Ta1394ConfigRom;

use model::*;

enum Event {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Elem((ElemId, ElemEventMask)),
    StreamLock(bool),
    Timer,
}

enum Model {
    Digi002(Digi002Model),
    Digi003(Digi003Model),
}

pub struct Dg00xRuntime {
    unit: hinawa::SndDg00x,
    model: Model,
    card_cntr: CardCntr,
    rx: mpsc::Receiver<Event>,
    tx: mpsc::SyncSender<Event>,
    dispatchers: Vec<Dispatcher>,
    notified_elems: Vec<ElemId>,
    timer: Option<Dispatcher>,
    measured_elem_id_list: Vec<ElemId>,
}

impl<'a> Drop for Dg00xRuntime {
    fn drop(&mut self) {
        // At first, stop event loop in all of dispatchers to avoid queueing new events.
        for dispatcher in &mut self.dispatchers {
            dispatcher.stop();
        }

        // Next, consume all events in queue to release blocked thread for sender.
        for _ in self.rx.try_iter() {}

        // Finally Finish I/O threads.
        self.dispatchers.clear();
    }
}

// NOTE: Additionally, in model ID field:
//   0x000001: the console models
//   0x000002: the rack models
const SPECIFIER_ID_DIGI002: u32 = 0x0000a3;
const SPECIFIER_ID_DIGI002_RACK: u32 = 0x0000a4;
const SPECIFIER_ID_DIGI003: u32 = 0x0000aa;
const SPECIFIER_ID_DIGI003_RACK: u32 = 0x0000ab;

impl RuntimeOperation<u32> for Dg00xRuntime {
    fn new(card_id: u32) -> Result<Self, Error> {
        let unit = hinawa::SndDg00x::new();
        unit.open(&format!("/dev/snd/hwC{}D0", card_id))?;

        let card_cntr = CardCntr::new();
        card_cntr.card.open(card_id, 0)?;

        let node = unit.get_node();
        let rom = node.get_config_rom()?;
        let config_rom = ConfigRom::try_from(rom)
            .map_err(|e| {
                let label = format!("Malformed configuration ROM detected: {}", e);
                Error::new(FileError::Nxio, &label)
            })?;
        let model_data = config_rom.get_model()
            .ok_or({
                let msg = "Configuration ROM is not for 1394TA standard";
                Error::new(FileError::Nxio, &msg)
            })?;

        let model = match model_data.specifier_id {
            SPECIFIER_ID_DIGI002 |
            SPECIFIER_ID_DIGI002_RACK => Model::Digi002(Default::default()),
            SPECIFIER_ID_DIGI003 |
            SPECIFIER_ID_DIGI003_RACK => Model::Digi003(Default::default()),
            _ => Err(Error::new(FileError::Nxio, "Not supported."))?,
        };

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();
        let notified_elems = Vec::new();
        let measured_elem_id_list = Vec::new();
        let timer = None;

        Ok(Dg00xRuntime {
            unit,
            model,
            card_cntr,
            rx,
            tx,
            dispatchers,
            notified_elems,
            measured_elem_id_list,
            timer,
        })
    }

    fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.launch_system_event_dispatcher()?;

        match &mut self.model {
            Model::Digi002(m) => m.load(&mut self.unit, &mut self.card_cntr),
            Model::Digi003(m) => m.load(&mut self.unit, &mut self.card_cntr),
        }?;

        match &mut self.model {
            Model::Digi002(m) => m.get_notified_elem_list(&mut self.notified_elems),
            Model::Digi003(m) => m.get_notified_elem_list(&mut self.notified_elems),
        }

        match &mut self.model {
            Model::Digi002(m) => m.get_measure_elem_list(&mut self.measured_elem_id_list),
            Model::Digi003(m) => m.get_measure_elem_list(&mut self.measured_elem_id_list),
        }

        if self.measured_elem_id_list.len() > 0 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::TIMER_NAME, 0);
            let _ = self.card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;
        }

        Ok(())
    }

    fn run(&mut self) -> Result<(), Error> {
        loop {
            let ev = match self.rx.recv() {
                Ok(ev) => ev,
                Err(_) => continue,
            };

            match ev {
                Event::Shutdown | Event::Disconnected => break,
                Event::BusReset(generation) => {
                    println!("IEEE 1394 bus is updated: {}", generation);
                }
                Event::Elem((elem_id, events)) => {
                    if elem_id.get_name() != Self::TIMER_NAME {
                        let _ = match &mut self.model {
                            Model::Digi002(m) => self.card_cntr.dispatch_elem_event(
                                &mut self.unit,
                                &elem_id,
                                &events,
                                m,
                            ),
                            Model::Digi003(m) => self.card_cntr.dispatch_elem_event(
                                &mut self.unit,
                                &elem_id,
                                &events,
                                m,
                            ),
                        };
                    } else {
                        let mut elem_value = ElemValue::new();
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
                Event::StreamLock(locked) => {
                    let _ = match &mut self.model {
                        Model::Digi002(m) => self.card_cntr.dispatch_notification(
                            &mut self.unit,
                            &locked,
                            &self.notified_elems,
                            m,
                        ),
                        Model::Digi003(m) => self.card_cntr.dispatch_notification(
                            &mut self.unit,
                            &locked,
                            &self.notified_elems,
                            m,
                        ),
                    };
                }
                Event::Timer => {
                    let _ = match &mut self.model {
                        Model::Digi002(m) => self.card_cntr.measure_elems(
                            &mut self.unit,
                            &self.measured_elem_id_list,
                            m,
                        ),
                        Model::Digi003(m) => self.card_cntr.measure_elems(
                            &mut self.unit,
                            &self.measured_elem_id_list,
                            m,
                        ),
                    };
                }
            }
        }

        Ok(())
    }
}

impl<'a> Dg00xRuntime {
    const NODE_DISPATCHER_NAME: &'a str = "node event dispatcher";
    const SYSTEM_DISPATCHER_NAME: &'a str = "system event dispatcher";
    const TIMER_DISPATCHER_NAME: &'a str = "interval timer dispatcher";

    const TIMER_NAME: &'a str = "metering";
    const TIMER_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);

    fn launch_node_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = Self::NODE_DISPATCHER_NAME.to_string();
        let mut dispatcher = Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_snd_unit(&self.unit, move |_| {
            let _ = tx.send(Event::Disconnected);
        })?;

        let tx = self.tx.clone();
        dispatcher.attach_fw_node(&self.unit.get_node(), move |_| {
            let _ = tx.send(Event::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.get_node().connect_bus_update(move |node| {
            let _ = tx.send(Event::BusReset(node.get_property_generation()));
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    fn launch_system_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = Self::SYSTEM_DISPATCHER_NAME.to_string();
        let mut dispatcher = Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_signal_handler(signal::Signal::SIGINT, move || {
            let _ = tx.send(Event::Shutdown);
            source::Continue(false)
        });

        let tx = self.tx.clone();
        dispatcher.attach_snd_card(&self.card_cntr.card, |_| {})?;
        self.card_cntr
            .card
            .connect_handle_elem_event(move |_, elem_id, events| {
                let _ = tx.send(Event::Elem((elem_id.clone(), events)));
            });

        let tx = self.tx.clone();
        self.unit.connect_lock_status(move |_, locked| {
            let t = tx.clone();
            let _ = thread::spawn(move || {
                // The notification of stream lock is not strictly corresponding to actual
                // packet streaming. Here, wait for 500 msec to catch the actual packet
                // streaming.
                thread::sleep(std::time::Duration::from_millis(500));
                let _ = t.send(Event::StreamLock(locked));
            });
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    fn start_interval_timer(&mut self) -> Result<(), Error> {
        let mut dispatcher = Dispatcher::run(Self::TIMER_DISPATCHER_NAME.to_string())?;
        let tx = self.tx.clone();
        dispatcher.attach_interval_handler(Self::TIMER_INTERVAL, move || {
            let _ = tx.send(Event::Timer);
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
}
