// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use glib::source;
use nix::sys::signal;
use std::sync::mpsc;

use hinawa::{FwNodeExt, FwNodeExtManual, SndUnitExt, SndUnitExtManual};

use alsactl::{CardExt, CardExtManual, ElemValueExtManual};

use core::dispatcher;
use core::card_cntr;

use super::model::BebobModel;

enum Event {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Elem(alsactl::ElemId, alsactl::ElemEventMask),
    Timer,
    StreamLock(bool),
}

pub struct BebobUnit<'a> {
    unit: hinawa::SndUnit,
    model: BebobModel<'a>,
    card_cntr: card_cntr::CardCntr,
    rx: mpsc::Receiver<Event>,
    tx: mpsc::SyncSender<Event>,
    dispatchers: Vec<dispatcher::Dispatcher>,
    timer: Option<dispatcher::Dispatcher>,
}

impl<'a> Drop for BebobUnit<'a> {
    fn drop(&mut self) {
        // Finish I/O threads.
        self.dispatchers.clear();
    }
}

impl<'a> BebobUnit<'a> {
    const NODE_DISPATCHER_NAME: &'a str = "node event dispatcher";
    const SYSTEM_DISPATCHER_NAME: &'a str = "system event dispatcher";
    const TIMER_DISPATCHER_NAME: &'a str = "interval timer dispatcher";

    const TIMER_NAME: &'a str = "metering";
    const TIMER_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);

    pub fn new(card_id: u32) -> Result<Self, Error> {
        let unit = hinawa::SndUnit::new();
        unit.open(&format!("/dev/snd/hwC{}D0", card_id))?;

        if unit.get_property_type() != hinawa::SndUnitType::Bebob {
            let label = "ALSA bebob driver is not bound to the unit.";
            return Err(Error::new(FileError::Inval, label));
        }

        let node = unit.get_node();
        let data = node.get_config_rom()?;
        let (vendor, model) = ta1394::config_rom::parse_entries(&data).ok_or_else(|| {
            let label = "Fail to detect information of unit";
            Error::new(FileError::Noent, label)
        })?;
        let model = BebobModel::new(vendor.vendor_id, model.model_id)?;

        let card_cntr = card_cntr::CardCntr::new();
        card_cntr.card.open(card_id, 0)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        Ok(BebobUnit {
            unit,
            model,
            card_cntr,
            rx,
            tx,
            dispatchers: Vec::new(),
            timer: None,
        })
    }

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

    pub fn listen(&mut self) -> Result<(), Error> {
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

    pub fn run(&mut self) {
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
                        let _= self.model.dispatch_elem_event(&self.unit, &mut self.card_cntr,
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
                    let _ = self.model.measure_elems(&self.unit, &mut self.card_cntr);
                }
                Event::StreamLock(locked) => {
                    let _ = self.model.dispatch_stream_lock(&self.unit, &mut self.card_cntr, locked);
                }
            }
        }
    }
}
