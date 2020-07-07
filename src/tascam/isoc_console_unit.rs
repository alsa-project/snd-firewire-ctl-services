// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};
use glib::source;

use nix::sys::signal;
use std::sync::mpsc;

use hinawa::{FwNodeExt, SndUnitExt};

use crate::dispatcher;

use super::fw1884_model::Fw1884Model;
use super::fw1082_model::Fw1082Model;

enum ConsoleUnitEvent {
    Shutdown,
    Disconnected,
    BusReset(u32),
}

enum ConsoleModel{
    Fw1884(Fw1884Model),
    Fw1082(Fw1082Model),
}

pub struct IsocConsoleUnit {
    unit: hinawa::SndTscm,
    model: ConsoleModel,
    rx: mpsc::Receiver<ConsoleUnitEvent>,
    tx: mpsc::SyncSender<ConsoleUnitEvent>,
    dispatchers: Vec<dispatcher::Dispatcher>,
}

impl<'a> Drop for IsocConsoleUnit {
    fn drop(&mut self) {
        self.dispatchers.clear();
    }
}

impl<'a> IsocConsoleUnit {
    const NODE_DISPATCHER_NAME: &'a str = "node event dispatcher";
    const SYSTEM_DISPATCHER_NAME: &'a str = "system event dispatcher";

    pub fn new(unit: hinawa::SndTscm, name: String, _: u32) -> Result<Self, Error> {
        let model = match name.as_str() {
            "FW-1884" => ConsoleModel::Fw1884(Fw1884Model::new()),
            "FW-1082" => ConsoleModel::Fw1082(Fw1082Model::new()),
            _ => {
                let label = format!("Unsupported model name: {}", name);
                return Err(Error::new(FileError::Nxio, &label));
            }
        };

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();

        Ok(IsocConsoleUnit {
            unit,
            model,
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
            let _ = tx.send(ConsoleUnitEvent::Disconnected);
        })?;

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
                ConsoleUnitEvent::Shutdown => break,
                ConsoleUnitEvent::Disconnected => break,
                ConsoleUnitEvent::BusReset(generation) => {
                    println!("IEEE 1394 bus is updated: {}", generation);
                }
            }
        }
    }
}
