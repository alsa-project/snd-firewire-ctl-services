// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
mod model;

mod common_ctl;
mod io_fw_model;
mod ionix_model;
mod minimal_model;
mod presonus;
mod tcelectronic;

mod blackbird_model;
mod extension_model;
mod focusrite;
mod mbox3_model;
mod pfire_model;
mod tcd22xx_ctl;
mod weiss;

use {
    alsa_ctl_tlv_codec::DbInterval,
    alsactl::{prelude::*, *},
    clap::Parser,
    common_ctl::*,
    firewire_dice_protocols as protocols,
    glib::{source, Error, FileError},
    hinawa::{
        prelude::{FwNodeExt, FwNodeExtManual},
        FwNode, FwReq,
    },
    hitaki::{prelude::*, *},
    model::*,
    nix::sys::signal,
    protocols::tcat::{global_section::*, *},
    runtime_core::{card_cntr::*, cmdline::*, dispatcher::*, LogLevel, *},
    std::{fmt::Debug, sync::mpsc},
    tracing::{debug, debug_span, Level},
};

enum Event {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Elem(ElemId, ElemEventMask),
    Notify(u32),
    Timer,
}

struct DiceRuntime {
    unit: (SndDice, FwNode),
    model: DiceModel,
    card_cntr: CardCntr,
    rx: mpsc::Receiver<Event>,
    tx: mpsc::SyncSender<Event>,
    dispatchers: Vec<Dispatcher>,
    timer: Option<Dispatcher>,
}

impl RuntimeOperation<u32> for DiceRuntime {
    fn new(card_id: u32, log_level: Option<LogLevel>) -> Result<Self, Error> {
        if let Some(level) = log_level {
            let fmt_level = match level {
                LogLevel::Debug => Level::DEBUG,
            };
            tracing_subscriber::fmt().with_max_level(fmt_level).init();
        }

        let unit = SndDice::new();
        let path = format!("/dev/snd/hwC{}D0", card_id);
        unit.open(&path, 0)?;

        let path = format!("/dev/{}", unit.node_device().unwrap());
        let node = FwNode::new();
        node.open(&path, 0)?;

        let model = DiceModel::new(&node)?;

        let card_cntr = CardCntr::default();
        card_cntr.card.open(card_id, 0)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();

        let timer = None;

        Ok(DiceRuntime {
            unit: (unit, node),
            model,
            card_cntr,
            rx,
            tx,
            dispatchers,
            timer,
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

        if self.model.measured_elem_list.len() > 0 {
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
                Event::Shutdown => {
                    let _enter = debug_span!("shutdown").entered();
                    self.model.store_configuration(&mut self.unit.1)?;
                    break;
                }
                Event::Disconnected => break,
                Event::BusReset(generation) => {
                    debug!("IEEE 1394 bus is updated: {}", generation);
                }
                Event::Elem(elem_id, events) => {
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
                        let _ = self
                            .card_cntr
                            .card
                            .read_elem_value(&elem_id, &mut elem_value)
                            .map(|_| {
                                let val = elem_value.boolean()[0];
                                if val {
                                    let _ = self.start_interval_timer();
                                } else {
                                    self.stop_interval_timer();
                                }
                            });
                    }
                }
                Event::Notify(msg) => {
                    let _enter = debug_span!("notify").entered();
                    debug!("msg = 0x{:08x}", msg);
                    let _ = self
                        .model
                        .dispatch_msg(&mut self.unit, &mut self.card_cntr, msg);
                }
                Event::Timer => {
                    let _enter = debug_span!("timer").entered();
                    let _ = self
                        .model
                        .measure_elems(&mut self.unit, &mut self.card_cntr);
                }
            }
        }

        enter.exit();

        Ok(())
    }
}

impl Drop for DiceRuntime {
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

impl DiceRuntime {
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
        self.unit.0.connect_notified(move |_, msg| {
            let _ = tx.send(Event::Notify(msg));
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
                let elem_id: ElemId = elem_id.clone();
                let _ = tx.send(Event::Elem(elem_id, events));
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
        self.timer = None;
    }
}

fn clock_rate_to_string(rate: &ClockRate) -> String {
    match rate {
        ClockRate::R32000 => "32000".to_string(),
        ClockRate::R44100 => "44100".to_string(),
        ClockRate::R48000 => "48000".to_string(),
        ClockRate::R88200 => "88200".to_string(),
        ClockRate::R96000 => "96000".to_string(),
        ClockRate::R176400 => "176400".to_string(),
        ClockRate::R192000 => "192000".to_string(),
        ClockRate::AnyLow => "Any-low".to_string(),
        ClockRate::AnyMid => "Any-mid".to_string(),
        ClockRate::AnyHigh => "Any-high".to_string(),
        ClockRate::None => "None".to_string(),
        ClockRate::Reserved(val) => format!("Reserved({})", val),
    }
}

fn clock_source_to_string(source: &ClockSource) -> String {
    match source {
        ClockSource::Aes1 => "AES1".to_string(),
        ClockSource::Aes2 => "AES2".to_string(),
        ClockSource::Aes3 => "AES3".to_string(),
        ClockSource::Aes4 => "AES4".to_string(),
        ClockSource::AesAny => "AES-ANY".to_string(),
        ClockSource::Adat => "ADAT".to_string(),
        ClockSource::Tdif => "TDIF".to_string(),
        ClockSource::WordClock => "Word-Clock".to_string(),
        ClockSource::Arx1 => "AVS-Audio-Rx1".to_string(),
        ClockSource::Arx2 => "AVS-Audio-Rx2".to_string(),
        ClockSource::Arx3 => "AVS-Audio-Rx3".to_string(),
        ClockSource::Arx4 => "AVS-Audio-Rx4".to_string(),
        ClockSource::Internal => "Internal".to_string(),
        ClockSource::Reserved(val) => format!("Reserved({})", val),
    }
}

struct DiceServiceCmd;

#[derive(Parser, Default)]
#[clap(name = "snd-dice-ctl-service")]
struct Arguments {
    /// The numeric identifier of sound card in Linux sound subsystem.
    card_id: u32,

    /// The level to debug runtime, disabled as a default.
    #[clap(long, short, arg_enum)]
    log_level: Option<LogLevel>,
}

impl ServiceCmd<Arguments, u32, DiceRuntime> for DiceServiceCmd {
    fn params(args: &Arguments) -> (u32, Option<LogLevel>) {
        (args.card_id, args.log_level)
    }
}

fn main() {
    DiceServiceCmd::run()
}
