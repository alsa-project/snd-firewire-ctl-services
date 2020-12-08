// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;
use glib::source;

use nix::sys::signal;

use std::sync::mpsc;

use hinawa::FwNodeExt;
use hinawa::{SndDice, SndDiceExt, SndUnitExt};

use core::RuntimeOperation;
use core::dispatcher;

enum Event {
    Shutdown,
    Disconnected,
    BusReset(u32),
}

pub struct DiceRuntime{
    unit: SndDice,
    rx: mpsc::Receiver<Event>,
    tx: mpsc::SyncSender<Event>,
    dispatchers: Vec<dispatcher::Dispatcher>,
}

impl RuntimeOperation<u32> for DiceRuntime {
    fn new(card_id: u32) -> Result<Self, Error> {
        let unit = SndDice::new();
        let path = format!("/dev/snd/hwC{}D0", card_id);
        unit.open(&path)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();

        Ok(DiceRuntime{unit, rx, tx, dispatchers})
    }

    fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.launch_system_event_dispatcher()?;

        Ok(())
    }

    fn run(&mut self) -> Result<(), Error> {
        loop {
            if let Ok(ev) = self.rx.recv() {
                match ev {
                    Event::Shutdown => break,
                    Event::Disconnected => break,
                    Event::BusReset(generation) => {
                        println!("IEEE 1394 bus is updated: {}", generation);
                    }
                }
            }
        }
        Ok(())
    }
}

impl Drop for DiceRuntime {
    fn drop(&mut self) {
        // Finish I/O threads.
        self.dispatchers.clear();
    }
}

impl<'a> DiceRuntime {
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

        self.dispatchers.push(dispatcher);

        Ok(())
    }
}
