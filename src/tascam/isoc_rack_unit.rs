// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;
use glib::source;

use nix::sys::signal;
use std::sync::mpsc;

use hinawa::{FwNodeExt, SndUnitExt};

use alsactl::CardExt;

use crate::dispatcher;
use crate::card_cntr;
use crate::card_cntr::CtlModel;

use super::fw1804_model::Fw1804Model;

enum RackUnitEvent {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Elem((alsactl::ElemId, alsactl::ElemEventMask)),
}

pub struct IsocRackUnit<'a> {
    unit: hinawa::SndTscm,
    model: Fw1804Model<'a>,
    card_cntr: card_cntr::CardCntr,
    rx: mpsc::Receiver<RackUnitEvent>,
    tx: mpsc::SyncSender<RackUnitEvent>,
    dispatchers: Vec<dispatcher::Dispatcher>,
}

impl<'a> Drop for IsocRackUnit<'a> {
    fn drop(&mut self) {
        self.dispatchers.clear();
    }
}

impl<'a> IsocRackUnit<'a> {
    const NODE_DISPATCHER_NAME: &'a str = "node event dispatcher";
    const SYSTEM_DISPATCHER_NAME: &'a str = "system event dispatcher";

    pub fn new(unit: hinawa::SndTscm, _: String, sysnum: u32) -> Result<Self, Error> {
        let model = Fw1804Model::new();

        let card_cntr = card_cntr::CardCntr::new();
        card_cntr.card.open(sysnum, 0)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();

        Ok(IsocRackUnit {
            unit,
            model,
            card_cntr,
            tx,
            rx,
            dispatchers,
        })
    }

    fn launch_node_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = Self::NODE_DISPATCHER_NAME.to_string();
        let mut dispatcher = dispatcher::Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_snd_unit(&self.unit, move |_| {
            let _ = tx.send(RackUnitEvent::Disconnected);
        })?;

        let tx = self.tx.clone();
        dispatcher.attach_fw_node(&self.unit.get_node(), move |_| {
            let _ = tx.send(RackUnitEvent::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.get_node().connect_bus_update(move |node| {
            let generation = node.get_property_generation();
            let _ = tx.send(RackUnitEvent::BusReset(generation));
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    fn launch_system_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = Self::SYSTEM_DISPATCHER_NAME.to_string();
        let mut dispatcher = dispatcher::Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_signal_handler(signal::Signal::SIGINT, move || {
            let _ = tx.send(RackUnitEvent::Shutdown);
            source::Continue(false)
        });

        let tx = self.tx.clone();
        dispatcher.attach_snd_card(&self.card_cntr.card, |_| {})?;
        self.card_cntr
            .card
            .connect_handle_elem_event(move |_, elem_id, events| {
                let _ = tx.send(RackUnitEvent::Elem((elem_id.clone(), events)));
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
                RackUnitEvent::Shutdown => break,
                RackUnitEvent::Disconnected => break,
                RackUnitEvent::BusReset(generation) => {
                    println!("IEEE 1394 bus is updated: {}", generation);
                }
                RackUnitEvent::Elem((elem_id, events)) => {
                    let _ = self.card_cntr.dispatch_elem_event(&self.unit, &elem_id, &events, &mut self.model);
                }
            }
        }
    }
}
