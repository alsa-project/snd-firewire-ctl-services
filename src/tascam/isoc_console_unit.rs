// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};
use glib::source;

use nix::sys::signal;
use std::sync::mpsc;

use hinawa::{FwNodeExt, SndUnitExt};

use alsactl::{CardExt, CardExtManual, ElemValueExtManual};
use alsaseq::{UserClientExt, EventCntrExt, EventCntrExtManual};

use crate::dispatcher;
use crate::card_cntr;
use card_cntr::{CtlModel, MonitorModel};

use super::fw1884_model::Fw1884Model;
use super::fw1082_model::Fw1082Model;

use super::seq_cntr;

enum ConsoleUnitEvent {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Elem((alsactl::ElemId, alsactl::ElemEventMask)),
    Monitor,
    SeqAppl(alsaseq::EventDataCtl),
}

enum ConsoleModel<'a> {
    Fw1884(Fw1884Model<'a>),
    Fw1082(Fw1082Model<'a>),
}

pub struct IsocConsoleUnit<'a> {
    unit: hinawa::SndTscm,
    model: ConsoleModel<'a>,
    card_cntr: card_cntr::CardCntr,
    seq_cntr: seq_cntr::SeqCntr,
    rx: mpsc::Receiver<ConsoleUnitEvent>,
    tx: mpsc::SyncSender<ConsoleUnitEvent>,
    dispatchers: Vec<dispatcher::Dispatcher>,
    monitor: Option<dispatcher::Dispatcher>,
    monitored_elems: Vec<alsactl::ElemId>,
}

impl<'a> Drop for IsocConsoleUnit<'a> {
    fn drop(&mut self) {
        self.dispatchers.clear();
    }
}

impl<'a> IsocConsoleUnit<'a> {
    const NODE_DISPATCHER_NAME: &'a str = "node event dispatcher";
    const SYSTEM_DISPATCHER_NAME: &'a str = "system event dispatcher";
    const MONITOR_DISPATCHER_NAME: &'a str = "interval monitor dispatcher";

    const MONITOR_NAME: &'a str = "monitor";
    const MONITOR_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);

    pub fn new(unit: hinawa::SndTscm, name: String, sysnum: u32) -> Result<Self, Error> {
        let model = match name.as_str() {
            "FW-1884" => ConsoleModel::Fw1884(Fw1884Model::new()),
            "FW-1082" => ConsoleModel::Fw1082(Fw1082Model::new()),
            _ => {
                let label = format!("Unsupported model name: {}", name);
                return Err(Error::new(FileError::Nxio, &label));
            }
        };

        let card_cntr = card_cntr::CardCntr::new();
        card_cntr.card.open(sysnum, 0)?;

        let seq_cntr = seq_cntr::SeqCntr::new(name)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();

        Ok(IsocConsoleUnit {
            unit,
            model,
            card_cntr,
            seq_cntr,
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
            let _ = tx.send(ConsoleUnitEvent::Disconnected);
        })?;

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
        let mut dispatcher = dispatcher::Dispatcher::run(name)?;

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
                        ev_cntr.get_event_type(i).unwrap_or(alsaseq::EventType::None) == alsaseq::EventType::Controller
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
            ConsoleModel::Fw1884(m) => m.load(&self.unit, &mut self.card_cntr)?,
            ConsoleModel::Fw1082(m) => m.load(&self.unit, &mut self.card_cntr)?,
        }

        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::MONITOR_NAME,
            0,
        );
        let _ = self.card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        match &mut self.model {
            ConsoleModel::Fw1884(m) => m.get_monitored_elems(&mut self.monitored_elems),
            ConsoleModel::Fw1082(m) => m.get_monitored_elems(&mut self.monitored_elems),
        }

        Ok(())
    }

    fn start_interval_monitor(&mut self) -> Result<(), Error> {
        let mut dispatcher = dispatcher::Dispatcher::run(Self::MONITOR_DISPATCHER_NAME.to_string())?;
        let tx = self.tx.clone();
        dispatcher.attach_interval_handler(Self::MONITOR_INTERVAL, move || {
            let _ = tx.send(ConsoleUnitEvent::Monitor);
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
                ConsoleUnitEvent::Shutdown => break,
                ConsoleUnitEvent::Disconnected => break,
                ConsoleUnitEvent::BusReset(generation) => {
                    println!("IEEE 1394 bus is updated: {}", generation);
                }
                ConsoleUnitEvent::Elem((elem_id, events)) => {
                    if elem_id.get_name() != Self::MONITOR_NAME {
                        let _ = match &mut self.model {
                            ConsoleModel::Fw1884(m) =>
                                self.card_cntr.dispatch_elem_event(&self.unit, &elem_id, &events, m),
                            ConsoleModel::Fw1082(m) =>
                                self.card_cntr.dispatch_elem_event(&self.unit, &elem_id, &events, m),
                        };
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
                ConsoleUnitEvent::Monitor => {
                    match &mut self.model {
                        ConsoleModel::Fw1884(m) =>{
                            let _ = m.monitor_unit(&self.unit);
                            let _ = self.card_cntr.monitor_elems(&self.unit, &self.monitored_elems, m);
                        }
                        ConsoleModel::Fw1082(m) => {
                            let _ = m.monitor_unit(&self.unit);
                            let _ = self.card_cntr.monitor_elems(&self.unit, &self.monitored_elems, m);
                        }
                    };
                }
                ConsoleUnitEvent::SeqAppl(_) => (),
            }
        }
    }
}
