// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use std::sync::mpsc;
use std::time::Duration;

use nix::sys::signal;

use glib::{Error, FileError};
use glib::source;

use hinawa::FwNodeExt;
use hinawa::{SndTscm, SndTscmExt, SndTscmExtManual, SndUnitExt};

use alsactl::{CardExt, CardExtManual};
use alsactl::{ElemId, ElemIfaceType, ElemEventMask, ElemValue, ElemValueExtManual};
use alsaseq::{EventCntrExt, EventCntrExtManual, EventDataCtl, EventType, UserClientExt};

use core::dispatcher::*;
use core::card_cntr::*;

use crate::{fw1082_model::*, fw1884_model::*, seq_cntr::*, *};

enum ConsoleUnitEvent {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Elem((ElemId, ElemEventMask)),
    Interval,
    SeqAppl(EventDataCtl),
    Surface((u32, u32, u32)),
}

enum ConsoleModel {
    Fw1884(Fw1884Model),
    Fw1082(Fw1082Model),
}

pub struct IsochConsoleRuntime {
    unit: SndTscm,
    model: ConsoleModel,
    card_cntr: CardCntr,
    seq_cntr: SeqCntr,
    rx: mpsc::Receiver<ConsoleUnitEvent>,
    tx: mpsc::SyncSender<ConsoleUnitEvent>,
    dispatchers: Vec<Dispatcher>,
    timer: Option<Dispatcher>,
    measure_elems: Vec<ElemId>,
}

impl Drop for IsochConsoleRuntime {
    fn drop(&mut self) {
        let _ = match &mut self.model {
            ConsoleModel::Fw1884(m) => m.finalize_sequencer(&mut self.unit),
            ConsoleModel::Fw1082(m) => m.finalize_sequencer(&mut self.unit),
        };

        self.dispatchers.clear();
    }
}

impl IsochConsoleRuntime {
    const NODE_DISPATCHER_NAME: &'static str = "node event dispatcher";
    const SYSTEM_DISPATCHER_NAME: &'static str = "system event dispatcher";
    const TIMER_DISPATCHER_NAME: &'static str = "interval timer dispatcher";

    const TIMER_NAME: &'static str = "metering";
    const TIMER_INTERVAL: Duration = Duration::from_millis(50);

    pub fn new(unit: SndTscm, name: &str, sysnum: u32) -> Result<Self, Error> {
        let model = match name {
            "FW-1884" => ConsoleModel::Fw1884(Default::default()),
            "FW-1082" => ConsoleModel::Fw1082(Default::default()),
            _ => {
                let label = format!("Unsupported model name: {}", name);
                return Err(Error::new(FileError::Nxio, &label));
            }
        };

        let card_cntr = CardCntr::new();
        card_cntr.card.open(sysnum, 0)?;

        let seq_cntr = SeqCntr::new(name)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();

        Ok(IsochConsoleRuntime {
            unit,
            model,
            card_cntr,
            seq_cntr,
            tx,
            rx,
            dispatchers,
            timer: None,
            measure_elems: Vec::new(),
        })
    }

    fn launch_node_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = Self::NODE_DISPATCHER_NAME.to_string();
        let mut dispatcher = Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_snd_unit(&self.unit, move |_| {
            let _ = tx.send(ConsoleUnitEvent::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.connect_control(move |_, index, before, after| {
            let _ = tx.send(ConsoleUnitEvent::Surface((index, before, after)));
        });

        let tx = self.tx.clone();
        dispatcher.attach_fw_node(&self.unit.get_node(), move |_| {
            let _ = tx.send(ConsoleUnitEvent::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.get_node().connect_bus_update(move |node| {
            let generation = node.get_property_generation();
            let _ = tx.send(ConsoleUnitEvent::BusReset(generation));
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    fn launch_system_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = Self::SYSTEM_DISPATCHER_NAME.to_string();
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
                        ev_cntr.get_event_type(i).unwrap_or(EventType::None) == EventType::Controller
                    }).for_each(|i| {
                        if let Ok(ctl_data) = ev_cntr.get_ctl_data(i) {
                            let data = ConsoleUnitEvent::SeqAppl(ctl_data);
                            let _ = tx.send(data);
                        }
                    });
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.launch_system_event_dispatcher()?;

        self.seq_cntr.open_port()?;
        match &mut self.model {
            ConsoleModel::Fw1884(m) => m.initialize_sequencer(&mut self.unit),
            ConsoleModel::Fw1082(m) => m.initialize_sequencer(&mut self.unit),
        }?;

        match &mut self.model {
            ConsoleModel::Fw1884(m) => m.load(&mut self.unit, &mut self.card_cntr)?,
            ConsoleModel::Fw1082(m) => m.load(&mut self.unit, &mut self.card_cntr)?,
        }

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::TIMER_NAME, 0);
        let _ = self.card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        match &mut self.model {
            ConsoleModel::Fw1884(m) => m.get_measure_elem_list(&mut self.measure_elems),
            ConsoleModel::Fw1082(m) => m.get_measure_elem_list(&mut self.measure_elems),
        }

        Ok(())
    }

    fn start_interval_timer(&mut self) -> Result<(), Error> {
        let mut dispatcher = Dispatcher::run(Self::TIMER_DISPATCHER_NAME.to_string())?;
        let tx = self.tx.clone();
        dispatcher.attach_interval_handler(Self::TIMER_INTERVAL, move || {
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

    pub fn run(&mut self) -> Result<(), Error> {
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
                    if elem_id.get_name() != Self::TIMER_NAME {
                        let _ = match &mut self.model {
                            ConsoleModel::Fw1884(m) =>
                                self.card_cntr.dispatch_elem_event(&mut self.unit, &elem_id, &events, m),
                            ConsoleModel::Fw1082(m) =>
                                self.card_cntr.dispatch_elem_event(&mut self.unit, &elem_id, &events, m),
                        };
                    } else {
                        let mut elem_value = ElemValue::new();
                        if self.card_cntr.card.read_elem_value(&elem_id, &mut elem_value).is_ok() {
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
                    match &mut self.model {
                        ConsoleModel::Fw1884(m) =>{
                            let _ = self.card_cntr.measure_elems(&mut self.unit, &self.measure_elems, m);
                        }
                        ConsoleModel::Fw1082(m) => {
                            let _ = self.card_cntr.measure_elems(&mut self.unit, &self.measure_elems, m);
                        }
                    };
                }
                ConsoleUnitEvent::SeqAppl(data) => {
                    let _ = match &mut self.model {
                        ConsoleModel::Fw1884(m) => m.dispatch_appl_event(
                            &mut self.unit,
                            &mut self.seq_cntr,
                            &data,
                        ),
                        ConsoleModel::Fw1082(m) => m.dispatch_appl_event(
                            &mut self.unit,
                            &mut self.seq_cntr,
                            &data,
                        ),
                    };
                }
                ConsoleUnitEvent::Surface((index, before, after)) => {
                    let image = self.unit.get_state().map(|s| s.to_vec())?;

                    let _ = match &mut self.model {
                        ConsoleModel::Fw1884(m) => m.dispatch_surface_event(
                            &mut self.unit,
                            &mut self.seq_cntr,
                            &image,
                            index,
                            before,
                            after,
                        ),
                        ConsoleModel::Fw1082(m) => m.dispatch_surface_event(
                            &mut self.unit,
                            &mut self.seq_cntr,
                            &image,
                            index,
                            before,
                            after,
                        ),
                    };
                }
            }
        }

        Ok(())
    }
}
