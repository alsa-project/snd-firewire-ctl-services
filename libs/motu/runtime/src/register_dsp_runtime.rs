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
        + CtlModel<(SndMotu, FwNode)>
        + NotifyModel<(SndMotu, FwNode), u32>
        + NotifyModel<(SndMotu, FwNode), bool>
        + NotifyModel<(SndMotu, FwNode), Vec<RegisterDspEvent>>
        + MeasureModel<(SndMotu, FwNode)>,
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
    timer: Option<Dispatcher>,
    measured_elem_id_list: Vec<ElemId>,
}

impl<T> Drop for RegisterDspRuntime<T>
where
    T: Default
        + CtlModel<(SndMotu, FwNode)>
        + NotifyModel<(SndMotu, FwNode), u32>
        + NotifyModel<(SndMotu, FwNode), bool>
        + NotifyModel<(SndMotu, FwNode), Vec<RegisterDspEvent>>
        + MeasureModel<(SndMotu, FwNode)>,
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
    Timer,
}

const NODE_DISPATCHER_NAME: &str = "node event dispatcher";
const SYSTEM_DISPATCHER_NAME: &str = "system event dispatcher";
const TIMER_DISPATCHER_NAME: &str = "interval timer dispatcher";

const TIMER_NAME: &str = "metering";
const TIMER_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

impl<T> RegisterDspRuntime<T>
where
    T: Default
        + CtlModel<(SndMotu, FwNode)>
        + NotifyModel<(SndMotu, FwNode), u32>
        + NotifyModel<(SndMotu, FwNode), bool>
        + NotifyModel<(SndMotu, FwNode), Vec<RegisterDspEvent>>
        + MeasureModel<(SndMotu, FwNode)>,
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
            timer: Default::default(),
            measured_elem_id_list: Default::default(),
        })
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.launch_system_event_dispatcher()?;

        self.model.load(&mut self.unit, &mut self.card_cntr)?;

        NotifyModel::<(SndMotu, FwNode), u32>::get_notified_elem_list(
            &mut self.model,
            &mut self.notified_elem_id_list,
        );

        NotifyModel::<(SndMotu, FwNode), bool>::get_notified_elem_list(
            &mut self.model,
            &mut self.notified_elem_id_list,
        );

        self.model
            .get_measure_elem_list(&mut self.measured_elem_id_list);
        if self.measured_elem_id_list.len() > 0 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, TIMER_NAME, 0);
            let _ = self.card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;
        }

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
                    if elem_id.get_name() != TIMER_NAME {
                        let _ = self.card_cntr.dispatch_elem_event(
                            &mut self.unit,
                            &elem_id,
                            &events,
                            &mut self.model,
                        );
                    } else {
                        let mut elem_value = ElemValue::new();
                        let _ = self
                            .card_cntr
                            .card
                            .read_elem_value(&elem_id, &mut elem_value)
                            .map(|_| {
                                let val = elem_value.get_bool()[0];
                                if val {
                                    let _ = self.start_interval_timer();
                                } else {
                                    self.stop_interval_timer();
                                }
                            });
                    }
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
                Event::Timer => {
                    let _ = self.card_cntr.measure_elems(
                        &mut self.unit,
                        &self.measured_elem_id_list,
                        &mut self.model,
                    );
                }
            }
        }
        Ok(())
    }

    fn start_interval_timer(&mut self) -> Result<(), Error> {
        let mut dispatcher = Dispatcher::run(TIMER_DISPATCHER_NAME.to_string())?;
        let tx = self.tx.clone();
        dispatcher.attach_interval_handler(TIMER_INTERVAL, move || {
            let _ = tx.send(Event::Timer);
            source::Continue(true)
        });

        self.timer = Some(dispatcher);

        Ok(())
    }

    fn stop_interval_timer(&mut self) {
        if let Some(dispatcher) = &self.timer {
            drop(dispatcher);
            self.timer = None;
        }
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
            let _ = tx.send(Event::MessageNotify(msg));
        });

        let tx = self.tx.clone();
        dispatcher.attach_fw_node(&self.unit.1, move |_| {
            let _ = tx.send(Event::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.1.connect_bus_update(move |node| {
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
        self.unit.0.connect_property_is_locked_notify(move |unit| {
            let is_locked = unit.get_property_is_locked();
            let _ = tx.send(Event::LockNotify(is_locked));
        });

        let tx = self.tx.clone();
        self.unit.0.connect_changed(move |_, events| {
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
