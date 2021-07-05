// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use glib::source;
use nix::sys::signal;
use std::sync::mpsc;

use hinawa::{FwNodeExt, FwNodeExtManual, SndUnitExt, SndUnitExtManual};

use alsactl::{CardExt, CardExtManual, ElemValueExtManual};

use core::RuntimeOperation;
use core::dispatcher;
use core::card_cntr;

use ieee1212_config_rom::ConfigRom;
use ta1394::config_rom::Ta1394ConfigRom;

use super::model::BebobModel;

use std::convert::TryFrom;

enum Event {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Elem(alsactl::ElemId, alsactl::ElemEventMask),
    Timer,
    StreamLock(bool),
}

pub struct BebobRuntime<'a> {
    unit: hinawa::SndUnit,
    model: BebobModel<'a>,
    card_cntr: card_cntr::CardCntr,
    rx: mpsc::Receiver<Event>,
    tx: mpsc::SyncSender<Event>,
    dispatchers: Vec<dispatcher::Dispatcher>,
    timer: Option<dispatcher::Dispatcher>,
}

impl<'a> Drop for BebobRuntime<'a> {
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

impl<'a> RuntimeOperation<u32> for BebobRuntime<'a> {
    fn new(card_id: u32) -> Result<Self, Error> {
        let unit = hinawa::SndUnit::new();
        unit.open(&format!("/dev/snd/hwC{}D0", card_id))?;

        if unit.get_property_type() != hinawa::SndUnitType::Bebob {
            let label = "ALSA bebob driver is not bound to the unit.";
            return Err(Error::new(FileError::Inval, label));
        }

        let node = unit.get_node();
        let raw = node.get_config_rom()?;

        let config_rom = ConfigRom::try_from(raw)
            .map_err(|e| {
                let label = format!("Malformed configuration ROM detected: {}", e);
                Error::new(FileError::Nxio, &label)
            })?;

        let (vendor, model) = config_rom.get_vendor()
            .and_then(|vendor| {
                config_rom.get_model()
                    .map(|model| (vendor, model))
            })
            .ok_or(Error::new(FileError::Nxio, "Configuration ROM is not for 1394TA standard"))?;

        let model = BebobModel::new(vendor.vendor_id, model.model_id)?;

        let card_cntr = card_cntr::CardCntr::new();
        card_cntr.card.open(card_id, 0)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        Ok(BebobRuntime {
            unit,
            model,
            card_cntr,
            rx,
            tx,
            dispatchers: Vec::new(),
            timer: None,
        })
    }

    fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.launch_system_event_dispatcher()?;

        self.model.load(&self.unit, &mut self.card_cntr)?;

        if self.model.measure_elem_list.len() > 0 {
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0,
                                                       Self::TIMER_NAME, 0);
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
                Event::Shutdown => break,
                Event::Disconnected => break,
                Event::BusReset(generation) => {
                    println!("IEEE 1394 bus is updated: {}", generation);
                }
                Event::Elem(elem_id, events) => {
                    if elem_id.get_name() != Self::TIMER_NAME {
                        let _= self.model.dispatch_elem_event(&mut self.unit, &mut self.card_cntr,
                                                              &elem_id, &events);
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
                Event::Timer => {
                    let _ = self.model.measure_elems(&mut self.unit, &mut self.card_cntr);
                }
                Event::StreamLock(locked) => {
                    let _ = self.model.dispatch_stream_lock(&self.unit, &mut self.card_cntr, locked);
                }
            }
        }
        Ok(())
    }
}

impl<'a> BebobRuntime<'a> {
    const NODE_DISPATCHER_NAME: &'a str = "node event dispatcher";
    const SYSTEM_DISPATCHER_NAME: &'a str = "system event dispatcher";
    const TIMER_DISPATCHER_NAME: &'a str = "interval timer dispatcher";

    const TIMER_NAME: &'a str = "metering";
    const TIMER_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);

    fn launch_node_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = Self::NODE_DISPATCHER_NAME.to_string();
        let mut dispatcher = dispatcher::Dispatcher::run(name)?;

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
        let mut dispatcher = dispatcher::Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_signal_handler(signal::Signal::SIGINT, move || {
            let _ = tx.send(Event::Shutdown);
            source::Continue(false)
        });

        let tx = self.tx.clone();
        dispatcher.attach_snd_card(&self.card_cntr.card, |_| {})?;
        self.card_cntr.card.connect_handle_elem_event(move |_, elem_id, events| {
            let elem_id: alsactl::ElemId = elem_id.clone();
            let _ = tx.send(Event::Elem(elem_id, events));
        });

        let tx = self.tx.clone();
        self.unit.connect_lock_status(move |_, locked| {
            let t = tx.clone();
            let _ = std::thread::spawn(move || {
                // The notification of stream lock is not strictly corresponding to actual
                // packet streaming. Here, wait for 500 msec to catch the actual packet
                // streaming.
                std::thread::sleep(std::time::Duration::from_millis(500));
                let _ = t.send(Event::StreamLock(locked));
            });
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    fn start_interval_timer(&mut self) -> Result<(), Error> {
        let mut dispatcher = dispatcher::Dispatcher::run(Self::TIMER_DISPATCHER_NAME.to_string())?;
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
