// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub use {
    super::{
        audioexpress::*, common_ctls::*, f828mk2::*, f896hd::*, f8pre::*, h4pre::*,
        register_dsp_ctls::*, traveler::*, ultralite::*, v2_ctls::*, v3_ctls::*, *,
    },
    alsa_ctl_tlv_codec::items::DbInterval,
    alsactl::*,
    core::{card_cntr::*, dispatcher::*, elem_value_accessor::*},
    glib::source,
    hinawa::FwReq,
    hinawa::{SndMotu, SndMotuExt, SndMotuExtManual, SndMotuRegisterDspParameter, SndUnitExt},
    motu_protocols::{register_dsp::*, version_2::*, version_3::*},
    nix::sys::signal::Signal,
    std::sync::mpsc,
};

pub type F828mk2Runtime = RegisterDspRuntime<F828mk2>;
pub type F896hdRuntime = RegisterDspRuntime<F896hd>;
pub type F8preRuntime = RegisterDspRuntime<F8pre>;
pub type TravelerRuntime = RegisterDspRuntime<Traveler>;
pub type UltraliteRuntime = RegisterDspRuntime<UltraLite>;

pub type AudioExpressRuntime = RegisterDspRuntime<AudioExpress>;
pub type H4preRuntime = RegisterDspRuntime<H4pre>;

pub struct RegisterDspRuntime<T>
where
    T: Default
        + CtlModel<SndMotu>
        + NotifyModel<SndMotu, u32>
        + NotifyModel<SndMotu, bool>
        + NotifyModel<SndMotu, Vec<RegisterDspEvent>>,
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
}

impl<T> Drop for RegisterDspRuntime<T>
where
    T: Default
        + CtlModel<SndMotu>
        + NotifyModel<SndMotu, u32>
        + NotifyModel<SndMotu, bool>
        + NotifyModel<SndMotu, Vec<RegisterDspEvent>>,
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
    MessageNotify(u32),
    LockNotify(bool),
    ChangedNotify(Vec<RegisterDspEvent>),
}

const NODE_DISPATCHER_NAME: &str = "node event dispatcher";
const SYSTEM_DISPATCHER_NAME: &str = "system event dispatcher";

impl<T> RegisterDspRuntime<T>
where
    T: Default
        + CtlModel<SndMotu>
        + NotifyModel<SndMotu, u32>
        + NotifyModel<SndMotu, bool>
        + NotifyModel<SndMotu, Vec<RegisterDspEvent>>,
{
    pub fn new(unit: SndMotu, card_id: u32, version: u32) -> Result<Self, Error> {
        let card_cntr = CardCntr::new();
        card_cntr.card.open(card_id, 0)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        Ok(Self {
            unit,
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

        NotifyModel::<SndMotu, u32>::get_notified_elem_list(
            &mut self.model,
            &mut self.notified_elem_id_list,
        );

        NotifyModel::<SndMotu, bool>::get_notified_elem_list(
            &mut self.model,
            &mut self.notified_elem_id_list,
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
                Event::MessageNotify(msg) => {
                    let _ = self.card_cntr.dispatch_notification(
                        &mut self.unit,
                        &msg,
                        &self.notified_elem_id_list,
                        &mut self.model,
                    );
                }
                Event::LockNotify(locked) => {
                    let _ = self.card_cntr.dispatch_notification(
                        &mut self.unit,
                        &locked,
                        &self.notified_elem_id_list,
                        &mut self.model,
                    );
                }
                Event::ChangedNotify(events) => {
                    let _ = self.card_cntr.dispatch_notification(
                        &mut self.unit,
                        &events,
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
        dispatcher.attach_snd_unit(&self.unit, move |_| {
            let _ = tx.send(Event::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.connect_notified(move |_, msg| {
            let _ = tx.send(Event::MessageNotify(msg));
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
        self.unit.connect_lock_status(move |_, locked| {
            let _ = tx.send(Event::LockNotify(locked));
        });

        let tx = self.tx.clone();
        self.unit.connect_register_dsp_changed(move |_, events| {
            let events = events
                .iter()
                .map(|&event| RegisterDspEvent::from(event))
                .collect();
            let _ = tx.send(Event::ChangedNotify(events));
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
