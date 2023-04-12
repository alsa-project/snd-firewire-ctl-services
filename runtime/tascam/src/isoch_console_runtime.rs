// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::{fw1082_model::*, fw1884_model::*, seq_cntr::*, *},
    alsactl::{prelude::*, *},
    alsaseq::{prelude::*, *},
    core::dispatcher::*,
    nix::sys::signal,
    protocols::isoch::{fw1082::*, fw1884::*},
    std::{marker::PhantomData, sync::mpsc, time::Duration},
};

pub type Fw1884Runtime = IsochConsoleRuntime<Fw1884Model, Fw1884Protocol>;
pub type Fw1082Runtime = IsochConsoleRuntime<Fw1082Model, Fw1082Protocol>;

pub struct IsochConsoleRuntime<S, T>
where
    S: Default
        + CtlModel<(SndTascam, FwNode)>
        + MeasureModel<(SndTascam, FwNode)>
        + SequencerCtlOperation<SndTascam, T>,
    T: MachineStateOperation,
{
    unit: (SndTascam, FwNode),
    model: S,
    card_cntr: CardCntr,
    seq_cntr: SeqCntr,
    rx: mpsc::Receiver<ConsoleUnitEvent>,
    tx: mpsc::SyncSender<ConsoleUnitEvent>,
    dispatchers: Vec<Dispatcher>,
    timer: Option<Dispatcher>,
    measure_elems: Vec<ElemId>,
    converter: EventConverter<T>,
    _phantom: PhantomData<T>,
}

impl<S, T> Drop for IsochConsoleRuntime<S, T>
where
    S: Default
        + CtlModel<(SndTascam, FwNode)>
        + MeasureModel<(SndTascam, FwNode)>
        + SequencerCtlOperation<SndTascam, T>,
    T: MachineStateOperation,
{
    fn drop(&mut self) {
        let _ = self.model.fin(&mut self.unit.1);
        self.dispatchers.clear();
    }
}

enum ConsoleUnitEvent {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Elem((ElemId, ElemEventMask)),
    Interval,
    SeqAppl(Vec<Event>),
    Surface((u32, u32, u32)),
}

const NODE_DISPATCHER_NAME: &str = "node event dispatcher";
const SYSTEM_DISPATCHER_NAME: &str = "system event dispatcher";
const TIMER_DISPATCHER_NAME: &str = "interval timer dispatcher";

const TIMER_NAME: &str = "metering";
const TIMER_INTERVAL: Duration = Duration::from_millis(50);

impl<S, T> IsochConsoleRuntime<S, T>
where
    S: Default
        + CtlModel<(SndTascam, FwNode)>
        + MeasureModel<(SndTascam, FwNode)>
        + SequencerCtlOperation<SndTascam, T>,
    T: MachineStateOperation,
{
    pub(crate) fn new(
        unit: SndTascam,
        node: FwNode,
        name: &str,
        sysnum: u32,
    ) -> Result<Self, Error> {
        let card_cntr = CardCntr::default();
        card_cntr.card.open(sysnum, 0)?;

        let seq_cntr = SeqCntr::new(name)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        Ok(Self {
            unit: (unit, node),
            model: Default::default(),
            card_cntr,
            seq_cntr,
            tx,
            rx,
            dispatchers: Default::default(),
            timer: Default::default(),
            measure_elems: Default::default(),
            converter: Default::default(),
            _phantom: Default::default(),
        })
    }

    pub(crate) fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.launch_system_event_dispatcher()?;

        let enter = debug_span!("init").entered();
        self.seq_cntr.open_port()?;
        self.model.initialize_sequencer(&mut self.unit.1)?;
        enter.exit();

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

    pub(crate) fn run(&mut self) -> Result<(), Error> {
        let enter = debug_span!("event").entered();
        loop {
            let ev = match self.rx.recv() {
                Ok(ev) => ev,
                Err(_) => continue,
            };

            match ev {
                ConsoleUnitEvent::Shutdown => break,
                ConsoleUnitEvent::Disconnected => break,
                ConsoleUnitEvent::BusReset(generation) => {
                    debug!("IEEE 1394 bus is updated: {}", generation);
                }
                ConsoleUnitEvent::Elem((elem_id, events)) => {
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
                ConsoleUnitEvent::Interval => {
                    let _enter = debug_span!("timer").entered();
                    let _ = self.card_cntr.measure_elems(
                        &mut self.unit,
                        &self.measure_elems,
                        &mut self.model,
                    );
                }
                ConsoleUnitEvent::SeqAppl(events) => {
                    let _enter = debug_span!("application").entered();
                    let _ = self.model.dispatch_appl_events(
                        &mut self.unit.1,
                        &mut self.seq_cntr,
                        &self.converter,
                        &events,
                    );
                }
                ConsoleUnitEvent::Surface((index, before, after)) => {
                    let _enter = debug_span!("surface").entered();
                    debug!(
                        "index: {}, before: 0x{:08x}, after: 0x{:08x}",
                        index, before, after
                    );
                    let _ = self.model.dispatch_surface_event(
                        &mut self.unit.0,
                        &mut self.unit.1,
                        &mut self.seq_cntr,
                        &self.converter,
                        index,
                        before,
                        after,
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
            let _ = tx.send(ConsoleUnitEvent::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.0.connect_changed(move |_, index, before, after| {
            let _ = tx.send(ConsoleUnitEvent::Surface((index, before, after)));
        });

        let tx = self.tx.clone();
        dispatcher.attach_fw_node(&self.unit.1, move |_| {
            let _ = tx.send(ConsoleUnitEvent::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.1.connect_bus_update(move |node| {
            let generation = node.generation();
            let _ = tx.send(ConsoleUnitEvent::BusReset(generation));
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    fn launch_system_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = SYSTEM_DISPATCHER_NAME.to_string();
        let mut dispatcher = Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_signal_handler(signal::Signal::SIGINT, move || {
            let _ = tx.send(ConsoleUnitEvent::Shutdown);
            source::Continue(false)
        });

        let tx = self.tx.clone();
        dispatcher.attach_snd_card(&self.card_cntr.card, |_| {})?;
        self.card_cntr
            .card
            .connect_handle_elem_event(move |_, elem_id, events| {
                let _ = tx.send(ConsoleUnitEvent::Elem((elem_id.clone(), events)));
            });

        let tx = self.tx.clone();
        dispatcher.attach_snd_seq(&self.seq_cntr.client)?;
        self.seq_cntr
            .client
            .connect_handle_event(move |_, ev_cntr| {
                let events = ev_cntr.deserialize();
                let _ = tx.send(ConsoleUnitEvent::SeqAppl(events));
            });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    fn start_interval_timer(&mut self) -> Result<(), Error> {
        let mut dispatcher = Dispatcher::run(TIMER_DISPATCHER_NAME.to_string())?;
        let tx = self.tx.clone();
        dispatcher.attach_interval_handler(TIMER_INTERVAL, move || {
            let _ = tx.send(ConsoleUnitEvent::Interval);
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
