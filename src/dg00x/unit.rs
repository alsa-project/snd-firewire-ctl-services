// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::source;
use glib::Error;
use nix::sys::signal;
use std::sync::mpsc;

use hinawa::{SndUnitExt, SndDg00xExt};
use hinawa::FwNodeExt;

use crate::dispatcher;

enum Event {
    Shutdown,
    Disconnected,
    BusReset(u32),
}

pub struct Dg00xUnit {
    unit: hinawa::SndDg00x,
    rx: mpsc::Receiver<Event>,
    tx: mpsc::SyncSender<Event>,
    dispatchers: Vec<dispatcher::Dispatcher>,
}

impl<'a> Drop for Dg00xUnit {
    fn drop(&mut self) {
        // Finish I/O threads.
        self.dispatchers.clear();
    }
}

impl<'a> Dg00xUnit {
    const NODE_DISPATCHER_NAME: &'a str = "node event dispatcher";
    const SYSTEM_DISPATCHER_NAME: &'a str = "system event dispatcher";

    pub fn new(card_id: u32) -> Result<Self, Error> {
        let unit = hinawa::SndDg00x::new();
        unit.open(&format!("/dev/snd/hwC{}D0", card_id))?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();

        Ok(Dg00xUnit {
            unit,
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

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.launch_system_event_dispatcher()?;

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
            }
        }
    }
}
