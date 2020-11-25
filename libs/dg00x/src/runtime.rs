// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::source;
use glib::Error;
use nix::sys::signal;
use std::sync::mpsc;
use std::thread;

use hinawa::{SndUnitExt, SndDg00xExt};
use hinawa::{FwNodeExt, FwNodeExtManual};

use alsactl::CardExt;

use core::dispatcher;
use core::card_cntr;
use card_cntr::{CtlModel, NotifyModel};
use core::RuntimeOperation;

use super::model::Dg00xModel;

enum Event {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Elem((alsactl::ElemId, alsactl::ElemEventMask)),
    StreamLock(bool),
}

pub struct Dg00xRuntime {
    unit: hinawa::SndDg00x,
    model: Dg00xModel,
    card_cntr: card_cntr::CardCntr,
    rx: mpsc::Receiver<Event>,
    tx: mpsc::SyncSender<Event>,
    dispatchers: Vec<dispatcher::Dispatcher>,
    notified_elems: Vec<alsactl::ElemId>,
}

impl<'a> Drop for Dg00xRuntime {
    fn drop(&mut self) {
        // Finish I/O threads.
        self.dispatchers.clear();
    }
}

impl RuntimeOperation<u32> for Dg00xRuntime {
    fn new(card_id: u32) -> Result<Self, Error> {
        let unit = hinawa::SndDg00x::new();
        unit.open(&format!("/dev/snd/hwC{}D0", card_id))?;

        let card_cntr = card_cntr::CardCntr::new();
        card_cntr.card.open(card_id, 0)?;

        let node = unit.get_node();
        let data = node.get_config_rom()?;
        let model = Dg00xModel::new(&data)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();
        let notified_elems = Vec::new();

        Ok(Dg00xRuntime {
            unit,
            model,
            card_cntr,
            rx,
            tx,
            dispatchers,
            notified_elems,
        })
    }

    fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.launch_system_event_dispatcher()?;

        self.model.load(&self.unit, &mut self.card_cntr)?;
        self.model.get_notified_elem_list(&mut self.notified_elems);

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
                    let _ = self.card_cntr.dispatch_elem_event(
                        &self.unit,
                        &elem_id,
                        &events,
                        &mut self.model,
                    );
                }
                Event::StreamLock(locked) => {
                    let _ = self.card_cntr.dispatch_notification(&self.unit, &locked,
                                                            &self.notified_elems, &mut self.model);
                }
            }
        }

        Ok(())
    }
}

impl<'a> Dg00xRuntime {
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
}
