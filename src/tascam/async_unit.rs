// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;
use glib::source;

use nix::sys::signal;
use std::sync::mpsc;

use hinawa::FwNodeExt;

use crate::dispatcher;

enum AsyncUnitEvent {
    Shutdown,
    Disconnected,
    BusReset(u32),
}

pub struct AsyncUnit {
    node: hinawa::FwNode,
    rx: mpsc::Receiver<AsyncUnitEvent>,
    tx: mpsc::SyncSender<AsyncUnitEvent>,
    dispatchers: Vec<dispatcher::Dispatcher>,
}

impl Drop for AsyncUnit {
    fn drop(&mut self) {
        self.dispatchers.clear();
    }
}

impl<'a> AsyncUnit {
    const NODE_DISPATCHER_NAME: &'a str = "node event dispatcher";

    pub fn new(node: hinawa::FwNode, _: String) -> Result<Self, Error> {
        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();

        Ok(AsyncUnit {
            node,
            tx,
            rx,
            dispatchers,
        })
    }

    fn launch_node_event_dispatcher(&mut self) -> Result<(), Error> {
        // Use a dispatcher.
        let name = Self::NODE_DISPATCHER_NAME.to_string();
        let mut dispatcher = dispatcher::Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_fw_node(&self.node, move |_| {
            let _ = tx.send(AsyncUnitEvent::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.node.connect_bus_update(move |node| {
            let generation = node.get_property_generation();
            let _ = tx.send(AsyncUnitEvent::BusReset(generation));
        });

        let tx = self.tx.clone();
        dispatcher.attach_signal_handler(signal::Signal::SIGINT, move || {
            let _ = tx.send(AsyncUnitEvent::Shutdown);
            source::Continue(false)
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;

        Ok(())
    }

    pub fn run(&mut self) {
        loop {
            let ev = match self.rx.recv() {
                Ok(ev) => ev,
                Err(_) => continue,
            };

            match ev {
                AsyncUnitEvent::Shutdown | AsyncUnitEvent::Disconnected => break,
                AsyncUnitEvent::BusReset(generation) => {
                    println!("IEEE 1394 bus is updated: {}", generation);
                }
            }
        }
    }
}
