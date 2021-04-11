// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;
use glib::source;
use nix::sys::signal;
use std::sync::mpsc;

use hinawa::{FwNodeExt, FwNodeExtManual, SndUnitExt, SndEfwExt};

use alsactl::{CardExt, CardExtManual, ElemValueExtManual};

use core::dispatcher;
use core::card_cntr;
use card_cntr::{CtlModel, MeasureModel};
use core::RuntimeOperation;

use super::model;

enum Event {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Timer,
    Elem((alsactl::ElemId, alsactl::ElemEventMask)),
}

pub struct EfwRuntime {
    unit: hinawa::SndEfw,
    model: model::EfwModel,
    card_cntr: card_cntr::CardCntr,
    rx: mpsc::Receiver<Event>,
    tx: mpsc::SyncSender<Event>,
    dispatchers: Vec<dispatcher::Dispatcher>,
    timer: Option<dispatcher::Dispatcher>,
    measure_elems: Vec<alsactl::ElemId>,
}

impl Drop for EfwRuntime {
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

impl RuntimeOperation<u32> for EfwRuntime {
    fn new(card_id: u32) -> Result<Self, Error> {
        let unit = hinawa::SndEfw::new();
        unit.open(&format!("/dev/snd/hwC{}D0", card_id))?;

        let node = unit.get_node();
        let data = node.get_config_rom()?;
        let model = model::EfwModel::new(&data)?;

        let card_cntr = card_cntr::CardCntr::new();
        card_cntr.card.open(card_id, 0)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();
        let timer = None;
        let measure_elems = Vec::new();

        Ok(EfwRuntime {
            unit,
            model,
            card_cntr,
            rx,
            tx,
            dispatchers,
            timer,
            measure_elems,
        })
    }

    fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.launch_system_event_dispatcher()?;

        self.model.load(&self.unit, &mut self.card_cntr)?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0,
                                                   Self::TIMER_NAME, 0);
        let _ = self.card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        self.model.get_measure_elem_list(&mut self.measure_elems);

        Ok(())
    }

    fn run(&mut self) -> Result<(), Error> {
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
                Event::Timer => {
                    let _ = self.card_cntr.measure_elems(&self.unit, &self.measure_elems,
                                                         &mut self.model);
                }
                Event::Elem((elem_id, events)) => {
                    if elem_id.get_name() != Self::TIMER_NAME {
                        let _ = self.card_cntr.dispatch_elem_event(
                            &self.unit,
                            &elem_id,
                            &events,
                            &mut self.model,
                        );
                    } else {
                        let mut elem_value = alsactl::ElemValue::new();
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
            }
        }
        Ok(())
    }
}

impl<'a> EfwRuntime {
    const NODE_DISPATCHER_NAME: &'a str = "node event dispatcher";
    const SYSTEM_DISPATCHER_NAME: &'a str = "system event dispatcher";
    const TIMER_DISPATCHER_NAME: &'a str = "interval timer dispatcher";

    const TIMER_NAME: &'a str = "metering";
    const TIMER_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);

    fn launch_node_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = Self::NODE_DISPATCHER_NAME.to_string();
        let mut dispatcher = dispatcher::Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_snd_unit(&self.unit, move |_| {
            let _ = tx.send(Event::Disconnected);
        })?;

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
        let name = Self::SYSTEM_DISPATCHER_NAME.to_string();
        let mut dispatcher = dispatcher::Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_signal_handler(signal::Signal::SIGINT, move || {
            let _ = tx.send(Event::Shutdown);
            source::Continue(false)
        });

        dispatcher.attach_snd_card(&self.card_cntr.card, |_| {})?;
        let tx = self.tx.clone();
        self.card_cntr
            .card
            .connect_handle_elem_event(move |_, elem_id, events| {
                let elem_id: alsactl::ElemId = elem_id.clone();
                let _ = tx.send(Event::Elem((elem_id, events)));
            });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    fn start_interval_timer(&mut self) -> Result<(), Error> {
        let mut dispatcher = dispatcher::Dispatcher::run(Self::TIMER_DISPATCHER_NAME.to_string())?;
        let tx = self.tx.clone();
        dispatcher.attach_interval_handler(Self::TIMER_INTERVAL, move || {
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
}
