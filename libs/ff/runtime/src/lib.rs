// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;
use glib::source;

use nix::sys::signal;

use std::sync::mpsc;

use core::RuntimeOperation;
use core::dispatcher;

enum Event {
    Shutdown,
}

pub struct FfRuntime{
    rx: mpsc::Receiver<Event>,
    tx: mpsc::SyncSender<Event>,
    dispatchers: Vec<dispatcher::Dispatcher>,
}

impl RuntimeOperation<u32> for FfRuntime {
    fn new(_: u32) -> Result<Self, Error> {
        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();

        Ok(FfRuntime{rx, tx, dispatchers})
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
                }
            }
        }
        Ok(())
    }
}

impl Drop for FfRuntime {
    fn drop(&mut self) {
        // Finish I/O threads.
        self.dispatchers.clear();
    }
}

impl<'a> FfRuntime {
    const NODE_DISPATCHER_NAME: &'a str = "node event dispatcher";
    const SYSTEM_DISPATCHER_NAME: &'a str = "system event dispatcher";

    fn launch_node_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = Self::NODE_DISPATCHER_NAME.to_string();
        let dispatcher = dispatcher::Dispatcher::run(name)?;

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
