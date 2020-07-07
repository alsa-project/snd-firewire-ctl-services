// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};
use glib::source;

use nix::sys::signal;
use std::sync::mpsc;

use hinawa::{FwNodeExt, FwRespExt};

use crate::dispatcher;

use super::protocol::ExpanderProtocol;

enum AsyncUnitEvent {
    Shutdown,
    Disconnected,
    BusReset(u32),
}

pub struct AsyncUnit {
    node: hinawa::FwNode,
    resp: hinawa::FwResp,
    rx: mpsc::Receiver<AsyncUnitEvent>,
    tx: mpsc::SyncSender<AsyncUnitEvent>,
    dispatchers: Vec<dispatcher::Dispatcher>,
    req: hinawa::FwReq,
}

impl Drop for AsyncUnit {
    fn drop(&mut self) {
        let _ = self.req.enable_notification(&self.node, false);
        self.resp.release();
        self.dispatchers.clear();
    }
}

impl<'a> AsyncUnit {
    const NODE_DISPATCHER_NAME: &'a str = "node event dispatcher";

    pub fn new(node: hinawa::FwNode, _: String) -> Result<Self, Error> {
        let resp = hinawa::FwResp::new();

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();

        Ok(AsyncUnit {
            node,
            resp,
            tx,
            rx,
            dispatchers,
            req: hinawa::FwReq::new(),
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

    fn register_address_space(&mut self) -> Result<(), Error> {
        // Reserve local address to receive async messages from the
        // unit within private space.
        let mut addr = 0x0000ffffe0000000 as u64;
        while addr < 0x0000fffff0000000 {
            if let Err(_) = self.resp.reserve(&self.node, addr, 0x80) {
                addr += 0x80;
                continue;
            }

            break;
        }
        if !self.resp.get_property_is_reserved() {
            let label = "Fail to reserve address space";
            return Err(Error::new(FileError::Nospc, label));
        }

        // Register the address to the unit.
        addr |= (self.node.get_property_local_node_id() as u64) << 48;
        self.req.register_notification_addr(&self.node, addr)?;

        self.req.enable_notification(&self.node, true)?;

        Ok(())
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.register_address_space()?;

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
