// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub use {
    super::{f828::*, f896::*, v1_ctls::*, *},
    alsactl::{prelude::*, *},
    core::{card_cntr::*, dispatcher::*},
    glib::source,
    nix::sys::signal::Signal,
    protocols::version_1::*,
    std::sync::mpsc,
};

pub type F828Runtime = Version1Runtime<F828>;
pub type F896Runtime = Version1Runtime<F896>;

pub struct Version1Runtime<T>
where
    T: CtlModel<(SndMotu, FwNode)> + NotifyModel<(SndMotu, FwNode), u32> + Default,
{
    unit: (SndMotu, FwNode),
    model: T,
    card_cntr: CardCntr,
    rx: mpsc::Receiver<Event>,
    tx: mpsc::SyncSender<Event>,
    dispatchers: Vec<Dispatcher>,
    #[allow(dead_code)]
    version: u32,
    notified_elem_id_list: Vec<ElemId>,
}

impl<T> Drop for Version1Runtime<T>
where
    T: CtlModel<(SndMotu, FwNode)> + NotifyModel<(SndMotu, FwNode), u32> + Default,
{
    fn drop(&mut self) {
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
}

const NODE_DISPATCHER_NAME: &str = "node event dispatcher";
const SYSTEM_DISPATCHER_NAME: &str = "system event dispatcher";

impl<T> Version1Runtime<T>
where
    T: CtlModel<(SndMotu, FwNode)> + NotifyModel<(SndMotu, FwNode), u32> + Default,
{
    pub fn new(unit: SndMotu, node: FwNode, card_id: u32, version: u32) -> Result<Self, Error> {
        let card_cntr = CardCntr::default();
        card_cntr.card.open(card_id, 0)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        Ok(Self {
            unit: (unit, node),
            model: Default::default(),
            card_cntr,
            rx,
            tx,
            dispatchers: Default::default(),
            version,
            notified_elem_id_list: Default::default(),
        })
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.launch_system_event_dispatcher()?;

        self.model.load(&mut self.unit, &mut self.card_cntr)?;
        self.model
            .get_notified_elem_list(&mut self.notified_elem_id_list);

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
            }
        }
        Ok(())
    }

    fn launch_node_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = NODE_DISPATCHER_NAME.to_string();
        let mut dispatcher = Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_alsa_firewire(&self.unit.0, move |_| {
            let _ = tx.send(Event::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.0.connect_notified(move |_, msg| {
            let _ = tx.send(Event::Notify(msg));
        });

        let tx = self.tx.clone();
        dispatcher.attach_fw_node(&self.unit.1, move |_| {
            let _ = tx.send(Event::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.1.connect_bus_update(move |node| {
            let _ = tx.send(Event::BusReset(node.generation()));
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
