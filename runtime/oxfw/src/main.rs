// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
mod apogee_model;
mod common_model;
mod griffin_model;
mod lacie_model;
mod loud_model;
mod model;
mod tascam_model;

mod common_ctl;

use {
    alsactl::{prelude::*, *},
    clap::Parser,
    common_ctl::*,
    firewire_oxfw_protocols as protocols,
    glib::{source, Error, FileError},
    hinawa::{
        prelude::{FwNodeExt, FwNodeExtManual},
        FwNode, FwReq,
    },
    hitaki::{prelude::*, *},
    ieee1212_config_rom::*,
    model::*,
    nix::sys::signal,
    protocols::*,
    runtime_core::{card_cntr::*, cmdline::*, dispatcher::*, LogLevel, *},
    std::{convert::TryFrom, fmt::Debug, sync::mpsc},
    ta1394_avc_general::config_rom::*,
    tracing::{debug, debug_span, Level},
};

enum Event {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Elem((ElemId, ElemEventMask)),
    Timer,
    StreamLock(bool),
}

struct OxfwRuntime {
    unit: (SndUnit, FwNode),
    model: OxfwModel,
    card_cntr: CardCntr,
    rx: mpsc::Receiver<Event>,
    tx: mpsc::SyncSender<Event>,
    dispatchers: Vec<Dispatcher>,
    timer: Option<Dispatcher>,
}

impl Drop for OxfwRuntime {
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

impl RuntimeOperation<u32> for OxfwRuntime {
    fn new(card_id: u32, log_level: Option<LogLevel>) -> Result<Self, Error> {
        if let Some(level) = log_level {
            let fmt_level = match level {
                LogLevel::Debug => Level::DEBUG,
            };
            tracing_subscriber::fmt().with_max_level(fmt_level).init();
        }

        let cdev = format!("/dev/snd/hwC{}D0", card_id);
        let unit = SndUnit::new();
        unit.open(&cdev, 0)?;

        if unit.unit_type() != AlsaFirewireType::Oxfw {
            let label = "ALSA oxfw driver is not bound to the unit.";
            return Err(Error::new(FileError::Inval, label));
        }

        let cdev = format!("/dev/{}", unit.node_device().unwrap());
        let node = FwNode::new();
        node.open(&cdev)?;

        let raw = node.config_rom()?;
        let config_rom = ConfigRom::try_from(raw).map_err(|e| {
            let label = format!("Malformed configuration ROM detected: {}", e);
            Error::new(FileError::Nxio, &label)
        })?;

        let (vendor, model) = config_rom
            .get_vendor()
            .and_then(|vendor| config_rom.get_model().map(|model| (vendor, model)))
            .ok_or(Error::new(
                FileError::Nxio,
                "Configuration ROM is not for 1394TA standard",
            ))?;

        let model = OxfwModel::new(vendor.vendor_id, model.model_id)?;

        let card_cntr = CardCntr::default();
        card_cntr.card.open(card_id, 0)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        Ok(OxfwRuntime {
            unit: (unit, node),
            model,
            card_cntr,
            rx,
            tx,
            dispatchers: Vec::new(),
            timer: None,
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

        if self.model.measure_elem_list.len() > 0 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::TIMER_NAME, 0);
            let _ = self.card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;
        }
        enter.exit();

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
                Event::Shutdown => break,
                Event::Disconnected => break,
                Event::BusReset(generation) => {
                    debug!("IEEE 1394 bus is updated: {}", generation);
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

                    if elem_id.name() != Self::TIMER_NAME {
                        let _ = self.model.dispatch_elem_event(
                            &mut self.unit,
                            &mut self.card_cntr,
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
                Event::Timer => {
                    let _enter = debug_span!("timer").entered();
                    let _ = self
                        .model
                        .measure_elems(&mut self.unit, &mut self.card_cntr);
                }
                Event::StreamLock(locked) => {
                    let _enter = debug_span!("stream-lock").entered();
                    let _ = self.model.dispatch_notification(
                        &mut self.unit,
                        &mut self.card_cntr,
                        locked,
                    );
                }
            }
        }

        enter.exit();

        Ok(())
    }
}

impl OxfwRuntime {
    const NODE_DISPATCHER_NAME: &'static str = "node event dispatcher";
    const SYSTEM_DISPATCHER_NAME: &'static str = "system event dispatcher";
    const TIMER_DISPATCHER_NAME: &'static str = "interval timer dispatcher";

    const TIMER_NAME: &'static str = "metering";
    const TIMER_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);

    fn launch_node_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = Self::NODE_DISPATCHER_NAME.to_string();
        let mut dispatcher = Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_alsa_firewire(&self.unit.0, move |_| {
            let _ = tx.send(Event::Disconnected);
        })?;

        let tx = self.tx.clone();
        dispatcher.attach_fw_node(&self.unit.1, move |_| {
            let _ = tx.send(Event::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.1.connect_bus_update(move |node| {
            let _ = tx.send(Event::BusReset(node.generation()));
        });

        let tx = self.tx.clone();
        self.unit.0.connect_is_locked_notify(move |unit| {
            let locked = unit.is_locked();
            let t = tx.clone();
            let _ = std::thread::spawn(move || {
                // The notification of stream lock is not strictly corresponding to actual
                // packet streaming. Here, wait for 500 msec to catch the actual packet
                // streaming.
                std::thread::sleep(std::time::Duration::from_millis(500));
                let _ = t.send(Event::StreamLock(locked));
            });
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    fn launch_system_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = Self::SYSTEM_DISPATCHER_NAME.to_string();
        let mut dispatcher = Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_signal_handler(signal::Signal::SIGINT, move || {
            let _ = tx.send(Event::Shutdown);
            source::Continue(false)
        });

        let tx = self.tx.clone();
        dispatcher.attach_snd_card(&self.card_cntr.card, |_| {})?;
        self.card_cntr
            .card
            .connect_handle_elem_event(move |_, elem_id, events| {
                let _ = tx.send(Event::Elem((elem_id.clone(), events)));
            });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    fn start_interval_timer(&mut self) -> Result<(), Error> {
        let mut dispatcher = Dispatcher::run(Self::TIMER_DISPATCHER_NAME.to_string())?;
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

struct OxfwServiceCmd;

#[derive(Parser, Default)]
#[clap(name = "snd-oxfw-ctl-service")]
struct Arguments {
    /// The numeric identifier of sound card in Linux sound subsystem.
    card_id: u32,

    /// The level to debug runtime, disabled as a default.
    #[clap(long, short, arg_enum)]
    log_level: Option<LogLevel>,
}

impl ServiceCmd<Arguments, u32, OxfwRuntime> for OxfwServiceCmd {
    fn params(args: &Arguments) -> (u32, Option<LogLevel>) {
        (args.card_id, args.log_level)
    }
}

fn main() {
    OxfwServiceCmd::run()
}
