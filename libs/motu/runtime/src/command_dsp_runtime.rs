// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use nix::sys::signal::Signal;

use glib::source;
use glib::{Error, FileError};

use hinawa::{FwNodeExt, FwRcode, FwResp, FwTcode};
use hinawa::{SndMotu, SndMotuExt, SndUnitExt};

use alsactl::{CardExt, ElemEventMask, ElemId};

use core::{card_cntr::*, dispatcher::*};

use motu_protocols::command_dsp::*;

use crate::{f828mk3::*, f828mk3_hybrid::*, ultralite_mk3::*, ultralite_mk3_hybrid::*};

pub type UltraliteMk3Runtime = Version3Runtime<UltraLiteMk3>;
pub type UltraliteMk3HybridRuntime = Version3Runtime<UltraliteMk3Hybrid>;
pub type F828mk3Runtime = Version3Runtime<F828mk3>;
pub type F828mk3HybridRuntime = Version3Runtime<F828mk3Hybrid>;

pub struct Version3Runtime<T>
where
    for<'a> T: Default
        + CtlModel<SndMotu>
        + NotifyModel<SndMotu, u32>
        + NotifyModel<SndMotu, &'a [DspCmd]>
        + CommandDspModel<'a>,
{
    unit: SndMotu,
    model: T,
    card_cntr: CardCntr,
    rx: mpsc::Receiver<Event>,
    tx: mpsc::SyncSender<Event>,
    dispatchers: Vec<Dispatcher>,
    #[allow(dead_code)]
    version: u32,
    notified_elem_id_list: Vec<ElemId>,
    msg_handler: Arc<Mutex<CommandDspMessageHandler>>,
    cmd_notified_elem_id_list: Vec<ElemId>,
}

impl<T> Drop for Version3Runtime<T>
where
    for<'a> T: Default
        + CtlModel<SndMotu>
        + NotifyModel<SndMotu, u32>
        + NotifyModel<SndMotu, &'a [DspCmd]>
        + CommandDspModel<'a>,
{
    fn drop(&mut self) {
        let _ = self.model.release_message_handler(&mut self.unit);

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

enum Event {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Elem((ElemId, ElemEventMask)),
    Notify(u32),
    DspMsg,
}

const NODE_DISPATCHER_NAME: &str = "node event dispatcher";
const SYSTEM_DISPATCHER_NAME: &str = "system event dispatcher";

impl<T> Version3Runtime<T>
where
    for<'a> T: Default
        + CtlModel<SndMotu>
        + NotifyModel<SndMotu, u32>
        + NotifyModel<SndMotu, &'a [DspCmd]>
        + CommandDspModel<'a>,
{
    pub fn new(unit: SndMotu, card_id: u32, version: u32) -> Result<Self, Error> {
        let card_cntr = CardCntr::new();
        card_cntr.card.open(card_id, 0)?;

        // Use uni-directional channel for communication to child threads. Use large number of
        // queue to avoid task blocking in node message handling.
        let (tx, rx) = mpsc::sync_channel(256);

        Ok(Self {
            unit,
            model: Default::default(),
            card_cntr,
            rx,
            tx,
            dispatchers: Default::default(),
            version,
            notified_elem_id_list: Default::default(),
            msg_handler: Default::default(),
            cmd_notified_elem_id_list: Default::default(),
        })
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.launch_system_event_dispatcher()?;

        let node = self.unit.get_node();
        let tx = self.tx.clone();
        let handler = self.msg_handler.clone();
        // TODO: bus reset can cause change of node ID by updating bus topology.
        let peer_node_id = node.get_property_node_id();
        self.model.prepare_message_handler(
            &mut self.unit,
            move |_, tcode, _, src, _, _, _, frame| {
                if src != peer_node_id {
                    FwRcode::AddressError
                } else if tcode != FwTcode::WriteQuadletRequest
                    && tcode != FwTcode::WriteBlockRequest
                {
                    FwRcode::TypeError
                } else {
                    let notify = if let Ok(handler) = &mut handler.lock() {
                        handler.cache_dsp_messages(frame);
                        if handler.has_dsp_message() {
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    // Full queue block the task, thus it is better to emit the event outside of
                    // critical section.
                    if notify {
                        let _ = tx.send(Event::DspMsg);
                    }
                    FwRcode::Complete
                }
            },
        )?;
        self.model.begin_messaging(&mut self.unit)?;

        // Queue Event::DspMsg at first so that initial state of control is cached.
        let mut count = 0;
        while count < 10 {
            thread::sleep(Duration::from_millis(200));

            if let Ok(handler) = &self.msg_handler.lock() {
                if handler.has_dsp_message() {
                    break;
                }
            }
            count += 1;
        }
        if count == 10 {
            Err(Error::new(FileError::Io, "No message for state arrived."))?;
        }

        self.model.load(&mut self.unit, &mut self.card_cntr)?;
        NotifyModel::<SndMotu, u32>::get_notified_elem_list(
            &mut self.model,
            &mut self.notified_elem_id_list,
        );
        NotifyModel::<SndMotu, &[DspCmd]>::get_notified_elem_list(
            &mut self.model,
            &mut self.cmd_notified_elem_id_list,
        );

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Error> {
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
                        &mut self.unit,
                        &elem_id,
                        &events,
                        &mut self.model,
                    );
                }
                Event::Notify(msg) => {
                    let _ = self.card_cntr.dispatch_notification(
                        &mut self.unit,
                        &msg,
                        &self.notified_elem_id_list,
                        &mut self.model,
                    );
                }
                Event::DspMsg => {
                    let cmds = if let Ok(handler) = &mut self.msg_handler.lock() {
                        if handler.has_dsp_message() {
                            handler.decode_messages()
                        } else {
                            Default::default()
                        }
                    } else {
                        Default::default()
                    };
                    let _ = self.card_cntr.dispatch_notification(
                        &mut self.unit,
                        &&cmds[..],
                        &self.cmd_notified_elem_id_list,
                        &mut self.model,
                    );
                }
            }
        }
        Ok(())
    }

    fn launch_node_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = NODE_DISPATCHER_NAME.to_string();
        let mut dispatcher = Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_snd_unit(&self.unit, move |_| {
            let _ = tx.send(Event::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.connect_notified(move |_, msg| {
            let _ = tx.send(Event::Notify(msg));
        });

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
        let name = SYSTEM_DISPATCHER_NAME.to_string();
        let mut dispatcher = Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_signal_handler(Signal::SIGINT, move || {
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

        self.dispatchers.push(dispatcher);

        Ok(())
    }
}

pub trait CommandDspModel<'a>: NotifyModel<SndMotu, &'a [DspCmd]> {
    fn prepare_message_handler<F>(&mut self, unit: &mut SndMotu, handler: F) -> Result<(), Error>
    where
        F: Fn(&FwResp, FwTcode, u64, u32, u32, u32, u32, &[u8]) -> FwRcode + 'static;
    fn begin_messaging(&mut self, unit: &mut SndMotu) -> Result<(), Error>;
    fn release_message_handler(&mut self, unit: &mut SndMotu) -> Result<(), Error>;
}
