// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use std::sync::{mpsc, Arc, Mutex};
use std::marker::PhantomData;

use nix::sys::signal::Signal;

use glib::{Error, FileError};
use glib::source;

use hinawa::{FwNode, FwNodeExt, FwRcode, FwResp, FwRespExt, FwRespExtManual};

use alsaseq::{UserClientExt, EventCntrExt, EventCntrExtManual, EventDataCtl, EventType};

use core::dispatcher::*;

use tascam_protocols::asynch::{fe8::*, *};

use crate::{fe8_model::*, seq_cntr::*, *};

pub type Fe8Runtime = AsynchRuntime<Fe8Model, Fe8Protocol, Fe8SurfaceState>;

pub struct AsynchRuntime<S, T, U>
where
    S: AsynchCtlOperation + SequencerCtlOperation<FwNode, T, U> + Default,
    T: MachineStateOperation + SurfaceImageOperation<U>,
{
    node: FwNode,
    model: S,
    resp: FwResp,
    seq_cntr: SeqCntr,
    rx: mpsc::Receiver<AsyncUnitEvent>,
    tx: mpsc::SyncSender<AsyncUnitEvent>,
    dispatchers: Vec<Dispatcher>,
    state_cntr: Arc<Mutex<AsynchSurfaceImage>>,
    _phantom0: PhantomData<T>,
    _phantom1: PhantomData<U>,
}

impl<S, T, U> Drop for AsynchRuntime<S, T, U>
where
    S: AsynchCtlOperation + SequencerCtlOperation<FwNode, T, U> + Default,
    T: MachineStateOperation + SurfaceImageOperation<U>,
{
    fn drop(&mut self) {
        let _ = self.model.finalize_surface(&mut self.node);
        self.resp.release();

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

enum AsyncUnitEvent {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Surface((u32, u32, u32)),
    SeqAppl(EventDataCtl),
}

const NODE_DISPATCHER_NAME: &str = "node event dispatcher";

impl<S, T, U> AsynchRuntime<S, T, U>
where
    S: AsynchCtlOperation + SequencerCtlOperation<FwNode, T, U> + Default,
    T: MachineStateOperation + SurfaceImageOperation<U>,
{
    pub fn new(node: FwNode, name: String) -> Result<Self, Error> {
        let seq_cntr = SeqCntr::new(&name)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        Ok(Self{
            node,
            model: Default::default(),
            resp: Default::default(),
            seq_cntr,
            tx,
            rx,
            dispatchers: Default::default(),
            state_cntr: Arc::new(Mutex::new(Default::default())),
            _phantom0: Default::default(),
            _phantom1: Default::default(),
        })
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.register_address_space()?;

        self.seq_cntr.open_port()?;

        self.model.initialize_sequencer(&mut self.node)?;

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Error> {
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
                AsyncUnitEvent::Surface((index, before, after)) => {
                    // Handle error of mutex lock as unrecoverable one.
                    let image = self.state_cntr.lock().map_err(|_| {
                        Error::new(FileError::Failed, "Unrecoverable error at mutex lock")
                    }).map(|s| s.0.to_vec())?;
                    let _ = self.model.dispatch_surface_event(
                        &mut self.node,
                        &mut self.seq_cntr,
                        &image,
                        index,
                        before,
                        after,
                    );
                }
                AsyncUnitEvent::SeqAppl(data) => {
                    let _ = self.model.dispatch_appl_event(
                        &mut self.node,
                        &mut self.seq_cntr,
                        &data,
                    );
                }
            }
        }

        Ok(())
    }

    fn launch_node_event_dispatcher(&mut self) -> Result<(), Error> {
        // Use a dispatcher.
        let name = NODE_DISPATCHER_NAME.to_string();
        let mut dispatcher = Dispatcher::run(name)?;

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
        dispatcher.attach_signal_handler(Signal::SIGINT, move || {
            let _ = tx.send(AsyncUnitEvent::Shutdown);
            source::Continue(false)
        });

        let tx = self.tx.clone();
        dispatcher.attach_snd_seq(&self.seq_cntr.client)?;
        self.seq_cntr
            .client
            .connect_handle_event(move |_, ev_cntr| {
                (0..ev_cntr.count_events()).filter_map(|i| {
                    ev_cntr.get_event_type(i).ok().filter(|ev_type| {
                        EventType::Controller.eq(ev_type)
                    }).and_then(|_| ev_cntr.get_ctl_data(i).ok())
                }).for_each(|ctl_data| {
                    let data = AsyncUnitEvent::SeqAppl(ctl_data);
                    let _ = tx.send(data);
                });
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

        let tx = self.tx.clone();
        let state_cntr = self.state_cntr.clone();
        let node_id = self.node.get_property_node_id();
        self.resp.connect_requested2(move |_, tcode, _, src, _, _, _, frame| {
            if src != node_id {
                FwRcode::AddressError
            } else {
                if let Ok(s) = &mut state_cntr.lock() {
                    let mut events = Vec::new();
                    let tcode = s.parse_notification(&mut events, tcode, frame);
                    events.iter().for_each(|&ev| {
                        let _ = tx.send(AsyncUnitEvent::Surface(ev));
                    });
                    tcode
                } else {
                    FwRcode::DataError
                }
            }
        });
        // Register the address to the unit.
        addr |= (self.node.get_property_local_node_id() as u64) << 48;
        self.model.register_notification_address(&mut self.node, addr)?;

        Ok(())
    }
}

pub trait AsynchCtlOperation {
    fn register_notification_address(
        &mut self,
        node: &mut FwNode,
        addr: u64,
    ) -> Result<(), Error>;
}
