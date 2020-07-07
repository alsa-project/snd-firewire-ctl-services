// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;
use glib::source;

use nix::sys::signal;
use std::sync::mpsc;

use hinawa::{FwNodeExt, SndUnitExt};

use alsactl::{CardExt, CardExtManual, ElemValueExtManual};

use crate::dispatcher;
use crate::card_cntr;
use card_cntr::{CtlModel, MonitorModel};

use super::fw1804_model::Fw1804Model;

enum RackUnitEvent {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Elem((alsactl::ElemId, alsactl::ElemEventMask)),
    Monitor,
}

pub struct IsocRackUnit<'a> {
    unit: hinawa::SndTscm,
    model: Fw1804Model<'a>,
    card_cntr: card_cntr::CardCntr,
    rx: mpsc::Receiver<RackUnitEvent>,
    tx: mpsc::SyncSender<RackUnitEvent>,
    dispatchers: Vec<dispatcher::Dispatcher>,
    monitor: Option<dispatcher::Dispatcher>,
    monitored_elems: Vec<alsactl::ElemId>,
}

impl<'a> Drop for IsocRackUnit<'a> {
    fn drop(&mut self) {
        self.dispatchers.clear();
    }
}

impl<'a> IsocRackUnit<'a> {
    const NODE_DISPATCHER_NAME: &'a str = "node event dispatcher";
    const SYSTEM_DISPATCHER_NAME: &'a str = "system event dispatcher";
    const MONITOR_DISPATCHER_NAME: &'a str = "interval monitor dispatcher";

    const MONITOR_NAME: &'a str = "monitor";
    const MONITOR_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);

    pub fn new(unit: hinawa::SndTscm, _: String, sysnum: u32) -> Result<Self, Error> {
        let model = Fw1804Model::new();

        let card_cntr = card_cntr::CardCntr::new();
        card_cntr.card.open(sysnum, 0)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();

        Ok(IsocRackUnit {
            unit,
            model,
            card_cntr,
            tx,
            rx,
            dispatchers,
            monitor: None,
            monitored_elems: Vec::new(),
        })
    }

    fn launch_node_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = Self::NODE_DISPATCHER_NAME.to_string();
        let mut dispatcher = dispatcher::Dispatcher::run(name)?;

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
        let mut dispatcher = dispatcher::Dispatcher::run(name)?;

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

        self.model.load(&self.unit, &mut self.card_cntr)?;

        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::MONITOR_NAME,
            0,
        );
        let _ = self.card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        self.model.get_monitored_elems(&mut self.monitored_elems);

        Ok(())
    }

    fn start_interval_monitor(&mut self) -> Result<(), Error> {
        let mut dispatcher = dispatcher::Dispatcher::run(Self::MONITOR_DISPATCHER_NAME.to_string())?;
        let tx = self.tx.clone();
        dispatcher.attach_interval_handler(Self::MONITOR_INTERVAL, move || {
            let _ = tx.send(RackUnitEvent::Monitor);
            source::Continue(true)
        });

        self.monitor = Some(dispatcher);

        Ok(())
    }

    fn stop_interval_monitor(&mut self) {
        if let Some(dispatcher) = &self.monitor {
            drop(dispatcher);
            self.monitor = None;
        }
    }
    pub fn run(&mut self) {
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
                    if elem_id.get_name() != Self::MONITOR_NAME {
                        let _ = self.card_cntr.dispatch_elem_event(&self.unit, &elem_id, &events,
                                                                   &mut self.model);
                    } else {
                        let mut elem_value = alsactl::ElemValue::new();
                        if self.card_cntr.card.read_elem_value(&elem_id, &mut elem_value).is_ok() {
                            let mut vals = [false];
                            elem_value.get_bool(&mut vals);
                            if vals[0] {
                                let _ = self.start_interval_monitor();
                            } else {
                                self.stop_interval_monitor();
                            }
                        }
                    }
                }
                RackUnitEvent::Monitor => {
                    let _ = self.model.monitor_unit(&self.unit);
                    let _ = self.card_cntr.monitor_elems(&self.unit, &self.monitored_elems,
                                                         &mut self.model);
                }
            }
        }
    }
}
