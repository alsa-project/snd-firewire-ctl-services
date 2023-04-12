// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use {
    super::{fw1804_model::*, *},
    alsactl::{prelude::*, *},
    core::dispatcher::*,
    nix::sys::signal,
    std::{sync::mpsc, time::Duration},
};

pub type Fw1804Runtime = IsochRackRuntime<Fw1804Model>;

pub struct IsochRackRuntime<T>
where
    T: Default + CtlModel<(SndTascam, FwNode)> + MeasureModel<(SndTascam, FwNode)>,
{
    unit: (SndTascam, FwNode),
    model: T,
    card_cntr: CardCntr,
    rx: mpsc::Receiver<RackUnitEvent>,
    tx: mpsc::SyncSender<RackUnitEvent>,
    dispatchers: Vec<Dispatcher>,
    timer: Option<Dispatcher>,
    measure_elems: Vec<ElemId>,
}

impl<T> Drop for IsochRackRuntime<T>
where
    T: Default + CtlModel<(SndTascam, FwNode)> + MeasureModel<(SndTascam, FwNode)>,
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

enum RackUnitEvent {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Elem((ElemId, ElemEventMask)),
    Timer,
}

const NODE_DISPATCHER_NAME: &str = "node event dispatcher";
const SYSTEM_DISPATCHER_NAME: &str = "system event dispatcher";
const TIMER_DISPATCHER_NAME: &str = "interval timer dispatcher";

const TIMER_NAME: &str = "meter";
const TIMER_INTERVAL: Duration = Duration::from_millis(50);

impl<T> IsochRackRuntime<T>
where
    T: Default + CtlModel<(SndTascam, FwNode)> + MeasureModel<(SndTascam, FwNode)>,
{
    pub fn new(unit: SndTascam, node: FwNode, _: &str, sysnum: u32) -> Result<Self, Error> {
        let card_cntr = CardCntr::default();
        card_cntr.card.open(sysnum, 0)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        Ok(Self {
            unit: (unit, node),
            model: Default::default(),
            card_cntr,
            tx,
            rx,
            dispatchers: Default::default(),
            timer: Default::default(),
            measure_elems: Default::default(),
        })
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.launch_system_event_dispatcher()?;

        let enter = debug_span!("cache").entered();
        self.model.cache(&mut self.unit)?;
        enter.exit();

        let enter = debug_span!("load").entered();
        self.model.load(&mut self.unit, &mut self.card_cntr)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, TIMER_NAME, 0);
        let _ = self.card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;
        self.model.get_measure_elem_list(&mut self.measure_elems);
        enter.exit();

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Error> {
        let enter = debug_span!("event").entered();
        loop {
            let ev = match self.rx.recv() {
                Ok(ev) => ev,
                Err(_) => continue,
            };

            match ev {
                RackUnitEvent::Shutdown => break,
                RackUnitEvent::Disconnected => break,
                RackUnitEvent::BusReset(generation) => {
                    debug!("IEEE 1394 bus is updated: {}", generation);
                }
                RackUnitEvent::Elem((elem_id, events)) => {
                    let _enter = debug_span!("element").entered();

                    debug!(
                        numid = elem_id.numid(),
                        name = elem_id.name().as_str(),
                        iface = ?elem_id.iface(),
                        device_id = elem_id.device_id(),
                        subdevice_id = elem_id.subdevice_id(),
                        index = elem_id.index(),
                    );

                    if elem_id.name() != TIMER_NAME {
                        let _ = self.card_cntr.dispatch_elem_event(
                            &mut self.unit,
                            &elem_id,
                            &events,
                            &mut self.model,
                        );
                    } else {
                        let mut elem_value = ElemValue::new();
                        if self
                            .card_cntr
                            .card
                            .read_elem_value(&elem_id, &mut elem_value)
                            .is_ok()
                        {
                            let val = elem_value.boolean()[0];
                            if val {
                                let _ = self.start_interval_timer();
                            } else {
                                self.stop_interval_timer();
                            }
                        }
                    }
                }
                RackUnitEvent::Timer => {
                    let _enter = debug_span!("timer").entered();
                    let _ = self.card_cntr.measure_elems(
                        &mut self.unit,
                        &self.measure_elems,
                        &mut self.model,
                    );
                }
            }
        }

        enter.exit();

        Ok(())
    }

    fn launch_node_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = NODE_DISPATCHER_NAME.to_string();
        let mut dispatcher = Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_alsa_firewire(&self.unit.0, move |_| {
            let _ = tx.send(RackUnitEvent::Disconnected);
        })?;

        let tx = self.tx.clone();
        dispatcher.attach_fw_node(&self.unit.1, move |_| {
            let _ = tx.send(RackUnitEvent::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.1.connect_bus_update(move |node| {
            let generation = node.generation();
            let _ = tx.send(RackUnitEvent::BusReset(generation));
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    fn launch_system_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = SYSTEM_DISPATCHER_NAME.to_string();
        let mut dispatcher = Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_signal_handler(signal::Signal::SIGINT, move || {
            let _ = tx.send(RackUnitEvent::Shutdown);
            source::Continue(false)
        });

        let tx = self.tx.clone();
        dispatcher.attach_snd_card(&self.card_cntr.card, |_| {})?;
        self.card_cntr
            .card
            .connect_handle_elem_event(move |_, elem_id, events| {
                let _ = tx.send(RackUnitEvent::Elem((elem_id.clone(), events)));
            });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    fn start_interval_timer(&mut self) -> Result<(), Error> {
        let mut dispatcher = Dispatcher::run(TIMER_DISPATCHER_NAME.to_string())?;
        let tx = self.tx.clone();
        dispatcher.attach_interval_handler(TIMER_INTERVAL, move || {
            let _ = tx.send(RackUnitEvent::Timer);
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
}
