// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use std::sync::mpsc;
use std::time::Duration;

use nix::sys::signal;

use glib::Error;
use glib::source;

use hinawa::FwNodeExt;
use hinawa::{SndTscm, SndUnitExt};

use alsactl::{CardExt, CardExtManual};
use alsactl::{ElemEventMask, ElemId, ElemIfaceType, ElemValue, ElemValueExtManual};

use core::dispatcher::*;
use core::card_cntr::*;

use crate::fw1804_model::*;

enum RackUnitEvent {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Elem((ElemId, ElemEventMask)),
    Timer,
}

pub struct IsochRackRuntime {
    unit: SndTscm,
    model: Fw1804Model,
    card_cntr: CardCntr,
    rx: mpsc::Receiver<RackUnitEvent>,
    tx: mpsc::SyncSender<RackUnitEvent>,
    dispatchers: Vec<Dispatcher>,
    timer: Option<Dispatcher>,
    measure_elems: Vec<ElemId>,
}

impl Drop for IsochRackRuntime {
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

impl IsochRackRuntime {
    const NODE_DISPATCHER_NAME: &'static str = "node event dispatcher";
    const SYSTEM_DISPATCHER_NAME: &'static str = "system event dispatcher";
    const TIMER_DISPATCHER_NAME: &'static str = "interval timer dispatcher";

    const TIMER_NAME: &'static str = "meter";
    const TIMER_INTERVAL: Duration = Duration::from_millis(50);

    pub fn new(unit: SndTscm, _: &str, sysnum: u32) -> Result<Self, Error> {
        let model = Fw1804Model::default();

        let card_cntr = CardCntr::new();
        card_cntr.card.open(sysnum, 0)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();

        Ok(IsochRackRuntime {
            unit,
            model,
            card_cntr,
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
            let _ = tx.send(RackUnitEvent::Disconnected);
        })?;

        let tx = self.tx.clone();
        dispatcher.attach_fw_node(&self.unit.get_node(), move |_| {
            let _ = tx.send(RackUnitEvent::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.get_node().connect_bus_update(move |node| {
            let generation = node.get_property_generation();
            let _ = tx.send(RackUnitEvent::BusReset(generation));
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    fn launch_system_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = Self::SYSTEM_DISPATCHER_NAME.to_string();
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

    pub fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.launch_system_event_dispatcher()?;

        self.model.load(&mut self.unit, &mut self.card_cntr)?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            Self::TIMER_NAME,
            0,
        );
        let _ = self.card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        self.model.get_measure_elem_list(&mut self.measure_elems);

        Ok(())
    }

    fn start_interval_timer(&mut self) -> Result<(), Error> {
        let mut dispatcher = Dispatcher::run(Self::TIMER_DISPATCHER_NAME.to_string())?;
        let tx = self.tx.clone();
        dispatcher.attach_interval_handler(Self::TIMER_INTERVAL, move || {
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

    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            let ev = match self.rx.recv() {
                Ok(ev) => ev,
                Err(_) => continue,
            };

            match ev {
                RackUnitEvent::Shutdown => break,
                RackUnitEvent::Disconnected => break,
                RackUnitEvent::BusReset(generation) => {
                    println!("IEEE 1394 bus is updated: {}", generation);
                }
                RackUnitEvent::Elem((elem_id, events)) => {
                    if elem_id.get_name() != Self::TIMER_NAME {
                        let _ = self.card_cntr.dispatch_elem_event(&mut self.unit, &elem_id, &events,
                                                                   &mut self.model);
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
                RackUnitEvent::Timer => {
                    let _ = self.card_cntr.measure_elems(&mut self.unit, &self.measure_elems,
                                                         &mut self.model);
                }
            }
        }

        Ok(())
    }
}
