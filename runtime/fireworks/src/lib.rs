// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
mod model;

mod clk_ctl;
mod guitar_ctl;
mod iec60958_ctl;
mod input_ctl;
mod meter_ctl;
mod mixer_ctl;
mod output_ctl;
mod port_ctl;

mod onyx1200f;
mod onyx400f;

mod audiofire12_former;
mod audiofire12_later;
mod audiofire2;
mod audiofire4;
mod audiofire8;
mod audiofire9;

mod rip;

use {
    self::{
        clk_ctl::*, guitar_ctl::*, iec60958_ctl::*, input_ctl::*, meter_ctl::*, mixer_ctl::*,
        model::*, output_ctl::*, port_ctl::*,
    },
    alsactl::{prelude::*, *},
    core::{card_cntr::*, dispatcher::*, *},
    firewire_fireworks_protocols as protocols,
    glib::{source, Error, FileError},
    hinawa::{
        prelude::{FwNodeExt, FwNodeExtManual},
        FwNode,
    },
    hitaki::{prelude::*, SndEfw},
    nix::sys::signal,
    protocols::{hw_ctl::*, hw_info::*, *},
    std::{marker::PhantomData, sync::mpsc, thread, time},
};

enum Event {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Timer,
    Elem((ElemId, ElemEventMask)),
    StreamLock(bool),
}

pub struct EfwRuntime {
    node: FwNode,
    unit: SndEfw,
    model: EfwModel,
    card_cntr: CardCntr,
    rx: mpsc::Receiver<Event>,
    tx: mpsc::SyncSender<Event>,
    dispatchers: Vec<Dispatcher>,
    timer: Option<Dispatcher>,
    measured_elem_id_list: Vec<ElemId>,
    notified_elem_id_list: Vec<ElemId>,
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
    fn new(card_id: u32, _: Option<LogLevel>) -> Result<Self, Error> {
        let unit = SndEfw::default();
        unit.open(&format!("/dev/snd/hwC{}D0", card_id), 0)?;

        let node = FwNode::new();
        node.open(&format!("/dev/{}", unit.node_device().unwrap()))?;
        let data = node.config_rom()?;
        let model = EfwModel::new(&data)?;

        let card_cntr = CardCntr::default();
        card_cntr.card.open(card_id, 0)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        Ok(EfwRuntime {
            node,
            unit,
            model,
            card_cntr,
            rx,
            tx,
            dispatchers: Default::default(),
            timer: Default::default(),
            measured_elem_id_list: Default::default(),
            notified_elem_id_list: Default::default(),
        })
    }

    fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.launch_system_event_dispatcher()?;

        self.model.cache(&mut self.unit)?;
        self.model.load(&mut self.unit, &mut self.card_cntr)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, TIMER_NAME, 0);
        let _ = self.card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        self.model
            .get_measured_elem_id_list(&mut self.measured_elem_id_list);

        self.model
            .get_notified_elem_id_list(&mut self.notified_elem_id_list);

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
                    let _ = self.model.measure_elems(
                        &mut self.card_cntr,
                        &mut self.unit,
                        &self.measured_elem_id_list,
                    );
                }
                Event::Elem((elem_id, events)) => {
                    if elem_id.name() != TIMER_NAME {
                        let _ = self.model.dispatch_elem_event(
                            &mut self.card_cntr,
                            &mut self.unit,
                            &elem_id,
                            &events,
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
                Event::StreamLock(locked) => {
                    let _ = self.model.dispatch_notification(
                        &mut self.card_cntr,
                        &mut self.unit,
                        locked,
                        &self.notified_elem_id_list,
                    );
                }
            }
        }
        Ok(())
    }
}

const NODE_DISPATCHER_NAME: &'static str = "node event dispatcher";
const SYSTEM_DISPATCHER_NAME: &'static str = "system event dispatcher";
const TIMER_DISPATCHER_NAME: &'static str = "interval timer dispatcher";

const TIMER_NAME: &'static str = "metering";
const TIMER_INTERVAL: time::Duration = time::Duration::from_millis(50);

impl EfwRuntime {
    fn launch_node_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = NODE_DISPATCHER_NAME.to_string();
        let mut dispatcher = Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_alsa_firewire(&self.unit, move |_| {
            let _ = tx.send(Event::Disconnected);
        })?;

        let tx = self.tx.clone();
        dispatcher.attach_fw_node(&self.node, move |_| {
            let _ = tx.send(Event::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.node.connect_bus_update(move |node| {
            let _ = tx.send(Event::BusReset(node.generation()));
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    fn launch_system_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = SYSTEM_DISPATCHER_NAME.to_string();
        let mut dispatcher = Dispatcher::run(name)?;

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
                let elem_id: ElemId = elem_id.clone();
                let _ = tx.send(Event::Elem((elem_id, events)));
            });

        let tx = self.tx.clone();
        self.unit.connect_is_locked_notify(move |unit| {
            let locked = unit.is_locked();
            let t = tx.clone();
            let _ = thread::spawn(move || {
                // The notification of stream lock is not strictly corresponding to actual
                // packet streaming. Here, wait for 500 msec to catch the actual packet
                // streaming.
                thread::sleep(time::Duration::from_millis(500));
                let _ = t.send(Event::StreamLock(locked));
            });
        });

        self.dispatchers.push(dispatcher);

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
}
