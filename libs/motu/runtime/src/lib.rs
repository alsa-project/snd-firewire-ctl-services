// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
mod model;
mod ultralite_mk3;
mod audioexpress;
mod h4pre;
mod f828mk3;
mod ultralite;
mod traveler;
mod f8pre;
mod f828mk2;

mod common_proto;
mod v3_proto;
mod v2_proto;

mod common_ctls;

mod v3_ctls;
mod v3_port_ctls;

mod v2_ctls;
mod v2_port_ctls;

use glib::{Error, FileError};
use glib::source;
use nix::sys::signal;
use std::sync::mpsc;
use std::convert::TryFrom;

use hinawa::{FwNodeExt, FwNodeExtManual, SndUnitExt, SndMotuExt};
use alsactl::CardExt;

use core::RuntimeOperation;
use core::dispatcher;
use core::card_cntr;

use ieee1212_config_rom::*;
use motu_protocols::config_rom::*;

use model::MotuModel;

enum Event {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Elem((alsactl::ElemId, alsactl::ElemEventMask)),
    Notify(u32),
}

pub struct MotuRuntime{
    unit: hinawa::SndMotu,
    model: MotuModel,
    card_cntr: card_cntr::CardCntr,
    rx: mpsc::Receiver<Event>,
    tx: mpsc::SyncSender<Event>,
    dispatchers: Vec<dispatcher::Dispatcher>,
}

impl<'a> Drop for MotuRuntime {
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

impl<'a> RuntimeOperation<u32> for MotuRuntime {
    fn new(card_id: u32) -> Result<Self, Error> {
        let unit = hinawa::SndMotu::new();
        unit.open(&format!("/dev/snd/hwC{}D0", card_id))?;

        let node = unit.get_node();
        let data = node.get_config_rom()?;
        let config_rom = ConfigRom::try_from(data)
            .map_err(|e| {
                let msg = format!("Malformed configuration ROM detected: {}", e.to_string());
                Error::new(FileError::Nxio, &msg)
            })?;
        let unit_data = config_rom.get_unit_data()
            .ok_or_else(|| {
                Error::new(FileError::Nxio, "Unexpected content of configuration ROM.")
            })?;
        let model = MotuModel::new(unit_data.model_id, unit_data.version)?;

        let card_cntr = card_cntr::CardCntr::new();
        card_cntr.card.open(card_id, 0)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();

        Ok(MotuRuntime {
            unit,
            model,
            card_cntr,
            rx,
            tx,
            dispatchers,
        })
    }

    fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.launch_system_event_dispatcher()?;

        self.model.load(&self.unit, &mut self.card_cntr)?;

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
                    let _ = self.model.dispatch_elem_event(&self.unit, &mut self.card_cntr,
                                                           &elem_id, &events);
                }
                Event::Notify(msg) => {
                    let _ = self.model.dispatch_notification(&self.unit, &msg, &mut self.card_cntr);
                }
            }
        }
        Ok(())
    }
}

impl<'a> MotuRuntime {
    const NODE_DISPATCHER_NAME: &'a str = "node event dispatcher";
    const SYSTEM_DISPATCHER_NAME: &'a str = "system event dispatcher";

    fn launch_node_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = Self::NODE_DISPATCHER_NAME.to_string();
        let mut dispatcher = dispatcher::Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_snd_unit(&self.unit, move |_| {
            let _ = tx.send(Event::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.connect_notified(move |_, msg| {
            let t = tx.clone();
            let _ = std::thread::spawn(move || {
                // Just after notification, the target device tends to return RCODE_BUSY against
                // read request. Here, wait for 100 msec to avoid it.
                std::thread::sleep(std::time::Duration::from_millis(100));
                let _ = t.send(Event::Notify(msg));
            });
        });

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
            let _ = tx.send(Event::Elem((elem_id.clone(), events)));
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }
}
