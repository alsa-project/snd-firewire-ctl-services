// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::{fw1082_model::*, fw1884_model::*, seq_cntr::*, *},
    alsactl::*,
    alsaseq::{EventType, *},
    core::dispatcher::*,
    nix::sys::signal,
    std::{marker::PhantomData, sync::mpsc, time::Duration},
    tascam_protocols::isoch::{fw1082::*, fw1884::*},
};

pub type Fw1884Runtime = IsochConsoleRuntime<Fw1884Model, Fw1884Protocol, Fw1884SurfaceState>;
pub type Fw1082Runtime = IsochConsoleRuntime<Fw1082Model, Fw1082Protocol, Fw1082SurfaceState>;

pub struct IsochConsoleRuntime<S, T, U>
where
    S: CtlModel<(SndTascam, FwNode)>
        + MeasureModel<(SndTascam, FwNode)>
        + SequencerCtlOperation<T, U>
        + Default,
    T: MachineStateOperation + SurfaceImageOperation<U>,
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
    _phantom0: PhantomData<T>,
    _phantom1: PhantomData<U>,
}

impl<S, T, U> Drop for IsochConsoleRuntime<S, T, U>
where
    S: CtlModel<(SndTascam, FwNode)>
        + MeasureModel<(SndTascam, FwNode)>
        + SequencerCtlOperation<T, U>
        + Default,
    T: MachineStateOperation + SurfaceImageOperation<U>,
{
    fn drop(&mut self) {
        let _ = self.model.finalize_sequencer(&mut self.unit.1);
        self.dispatchers.clear();
    }
}

enum ConsoleUnitEvent {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Elem((ElemId, ElemEventMask)),
    Interval,
    SeqAppl(EventDataCtl),
    Surface((u32, u32, u32)),
}

const NODE_DISPATCHER_NAME: &str = "node event dispatcher";
const SYSTEM_DISPATCHER_NAME: &str = "system event dispatcher";
const TIMER_DISPATCHER_NAME: &str = "interval timer dispatcher";

const TIMER_NAME: &str = "metering";
const TIMER_INTERVAL: Duration = Duration::from_millis(50);

impl<S, T, U> IsochConsoleRuntime<S, T, U>
where
    S: CtlModel<(SndTascam, FwNode)>
        + MeasureModel<(SndTascam, FwNode)>
        + SequencerCtlOperation<T, U>
        + Default,
    T: MachineStateOperation + SurfaceImageOperation<U>,
{
    pub fn new(unit: SndTascam, node: FwNode, name: &str, sysnum: u32) -> Result<Self, Error> {
        let card_cntr = CardCntr::new();
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
            _phantom0: Default::default(),
            _phantom1: Default::default(),
        })
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.launch_system_event_dispatcher()?;

        self.seq_cntr.open_port()?;
        self.model.initialize_sequencer(&mut self.unit.1)?;
        self.model.load(&mut self.unit, &mut self.card_cntr)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, TIMER_NAME, 0);
        let _ = self.card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;
        self.model.get_measure_elem_list(&mut self.measure_elems);

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Error> {
        let mut image = vec![0u32; 64];

        loop {
            let ev = match self.rx.recv() {
                Ok(ev) => ev,
                Err(_) => continue,
            };

            match ev {
                ConsoleUnitEvent::Shutdown => break,
                ConsoleUnitEvent::Disconnected => break,
                ConsoleUnitEvent::BusReset(generation) => {
                    println!("IEEE 1394 bus is updated: {}", generation);
                }
                ConsoleUnitEvent::Elem((elem_id, events)) => {
                    if elem_id.get_name() != TIMER_NAME {
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
                            let mut vals = [false];
                            elem_value.get_bool(&mut vals);
                            if vals[0] {
                                let _ = self.start_interval_timer();
                            } else {
                                self.stop_interval_timer();
                            }
                        }
                    }
                }
                ConsoleUnitEvent::Interval => {
                    let _ = self.card_cntr.measure_elems(
                        &mut self.unit,
                        &self.measure_elems,
                        &mut self.model,
                    );
                }
                ConsoleUnitEvent::SeqAppl(data) => {
                    let _ =
                        self.model
                            .dispatch_appl_event(&mut self.unit.1, &mut self.seq_cntr, &data);
                }
                ConsoleUnitEvent::Surface((index, before, after)) => {
                    self.unit.0.read_state(&mut image)?;
                    let _ = self.model.dispatch_surface_event(
                        &mut self.unit.1,
                        &mut self.seq_cntr,
                        &image,
                        index,
                        before,
                        after,
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
            let generation = node.get_property_generation();
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
                let _ = (0..ev_cntr.count_events())
                    .filter(|&i| {
                        // At present, controller event is handled.
                        ev_cntr.get_event_type(i).unwrap_or(EventType::None)
                            == EventType::Controller
                    })
                    .for_each(|i| {
                        if let Ok(ctl_data) = ev_cntr.get_ctl_data(i) {
                            let data = ConsoleUnitEvent::SeqAppl(ctl_data);
                            let _ = tx.send(data);
                        }
                    });
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
