// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;
use glib::source;
use nix::sys::signal;
use std::sync::mpsc;

use hinawa::{FwNodeExt, FwNodeExtManual, SndUnitExt, SndEfwExt};

use alsactl::CardExt;

use crate::dispatcher;
use crate::card_cntr;
use card_cntr::CtlModel;

use super::model;

enum Event {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Elem((alsactl::ElemId, alsactl::ElemEventMask)),
}

pub struct EfwUnit {
    unit: hinawa::SndEfw,
    model: model::EfwModel,
    card_cntr: card_cntr::CardCntr,
    rx: mpsc::Receiver<Event>,
    tx: mpsc::SyncSender<Event>,
    dispatchers: Vec<dispatcher::Dispatcher>,
}

impl Drop for EfwUnit {
    fn drop(&mut self) {
        // Finish I/O threads.
        self.dispatchers.clear();
    }
}

impl<'a> EfwUnit {
    const NODE_DISPATCHER_NAME: &'a str = "node event dispatcher";
    const SYSTEM_DISPATCHER_NAME: &'a str = "system event dispatcher";

    pub fn new(card_id: u32) -> Result<Self, Error> {
        let unit = hinawa::SndEfw::new();
        unit.open(&format!("/dev/snd/hwC{}D0", card_id))?;

        let node = unit.get_node();
        let data = node.get_config_rom()?;
        let model = model::EfwModel::new(&data)?;

        let card_cntr = card_cntr::CardCntr::new();
        card_cntr.card.open(card_id, 0)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();

        Ok(EfwUnit {
            unit,
            model,
            card_cntr,
            rx,
            tx,
            dispatchers,
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

        dispatcher.attach_snd_card(&self.card_cntr.card, |_| {})?;
        let tx = self.tx.clone();
        self.card_cntr
            .card
            .connect_handle_elem_event(move |_, elem_id, events| {
                let elem_id: alsactl::ElemId = elem_id.clone();
                let _ = tx.send(Event::Elem((elem_id, events)));
            });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.launch_system_event_dispatcher()?;

        self.model.load(&self.unit, &mut self.card_cntr)?;

        Ok(())
    }

    pub fn run(&mut self) {
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
                    let _ = self.card_cntr.dispatch_elem_event(
                        &self.unit,
                        &elem_id,
                        &events,
                        &mut self.model,
                    );
                }
            }
        }
    }
}
