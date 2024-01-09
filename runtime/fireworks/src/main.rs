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

mod onyx1200f_model;
mod onyx400f_model;

mod audiofire12_former_model;
mod audiofire12_later_model;
mod audiofire2_model;
mod audiofire4_model;
mod audiofire8_model;
mod audiofire9_model;

mod rip_model;

use {
    self::{
        clk_ctl::*, guitar_ctl::*, iec60958_ctl::*, input_ctl::*, meter_ctl::*, mixer_ctl::*,
        model::*, output_ctl::*, port_ctl::*,
    },
    alsactl::{prelude::*, *},
    clap::Parser,
    firewire_fireworks_protocols as protocols,
    glib::{source, Error, FileError},
    hinawa::{
        prelude::{FwNodeExt, FwNodeExtManual},
        FwNode,
    },
    hitaki::{prelude::*, SndEfw},
    nix::sys::signal,
    protocols::{hw_ctl::*, hw_info::*, *},
    runtime_core::{card_cntr::*, cmdline::*, dispatcher::*, LogLevel, *},
    std::{marker::PhantomData, sync::mpsc, thread, time},
    tracing::{debug, debug_span, Level},
};

enum Event {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Timer,
    Elem((ElemId, ElemEventMask)),
    StreamLock(bool),
}

struct EfwRuntime {
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
    fn new(card_id: u32, log_level: Option<LogLevel>) -> Result<Self, Error> {
        if let Some(level) = log_level {
            let fmt_level = match level {
                LogLevel::Debug => Level::DEBUG,
            };
            tracing_subscriber::fmt().with_max_level(fmt_level).init();
        }

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

        let enter = debug_span!("cache").entered();
        self.model.cache(&mut self.unit)?;
        enter.exit();

        let enter = debug_span!("load").entered();
        self.model.load(&mut self.card_cntr)?;
        enter.exit();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, TIMER_NAME, 0);
        let _ = self.card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        self.model
            .get_measured_elem_id_list(&mut self.measured_elem_id_list);

        self.model
            .get_notified_elem_id_list(&mut self.notified_elem_id_list);

        Ok(())
    }

    fn run(&mut self) -> Result<(), Error> {
        let enter = debug_span!("event").entered();
        loop {
            let ev = match self.rx.recv() {
                Ok(ev) => ev,
                Err(_) => continue,
            };

            match ev {
                Event::Shutdown | Event::Disconnected => break,
                Event::BusReset(generation) => {
                    debug!("IEEE 1394 bus is updated: {}", generation);
                }
                Event::Timer => {
                    let _enter = debug_span!("stream-lock").entered();
                    let _ = self.model.measure_elems(
                        &mut self.card_cntr,
                        &mut self.unit,
                        &self.measured_elem_id_list,
                    );
                }
                Event::Elem((elem_id, events)) => {
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
                    let _enter = debug_span!("stream-lock").entered();
                    let _ = self.model.dispatch_notification(
                        &mut self.card_cntr,
                        &mut self.unit,
                        locked,
                        &self.notified_elem_id_list,
                    );
                }
            }
        }

        enter.exit();

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
        self.timer = None;
    }
}

struct EfwServiceCmd;

#[derive(Parser, Default)]
#[clap(name = "snd-fireworks-ctl-service")]
struct Arguments {
    /// The numeric identifier of sound card in Linux sound subsystem.
    card_id: u32,

    /// The level to debug runtime, disabled as a default.
    #[clap(long, short, arg_enum)]
    log_level: Option<LogLevel>,
}

impl ServiceCmd<Arguments, u32, EfwRuntime> for EfwServiceCmd {
    fn params(args: &Arguments) -> (u32, Option<LogLevel>) {
        (args.card_id, args.log_level)
    }
}

fn main() {
    EfwServiceCmd::run()
}
