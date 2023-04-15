// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
mod ff400_model;
mod ff800_model;

mod ff802_model;
mod ucx_model;

mod former_ctls;
mod latter_ctls;

mod ff400_runtime;
mod ff800_runtime;
mod latter_runtime;

use {
    alsactl::{prelude::*, *},
    clap::Parser,
    core::{card_cntr::*, cmdline::*, dispatcher::*, LogLevel, *},
    ff400_runtime::*,
    ff800_runtime::*,
    firewire_fireface_protocols as protocols,
    glib::{source, Error, FileError},
    hinawa::{
        prelude::{FwNodeExt, FwNodeExtManual},
        FwNode, FwReq,
    },
    hitaki::{prelude::*, *},
    ieee1212_config_rom::*,
    latter_runtime::*,
    nix::sys::signal,
    protocols::{former::*, latter::*, *},
    std::convert::TryFrom,
    tracing::{debug, debug_span, Level},
};

enum FfRuntime {
    Ff800(Ff800Runtime),
    Ff400(Ff400Runtime),
    FfUcx(FfUcxRuntime),
    Ff802(Ff802Runtime),
}

impl RuntimeOperation<u32> for FfRuntime {
    fn new(card_id: u32, log_level: Option<LogLevel>) -> Result<Self, Error> {
        if let Some(level) = log_level {
            let fmt_level = match level {
                LogLevel::Debug => Level::DEBUG,
            };
            tracing_subscriber::fmt().with_max_level(fmt_level).init();
        }

        let unit = SndFireface::new();
        let path = format!("/dev/snd/hwC{}D0", card_id);
        unit.open(&path, 0)?;

        let cdev = format!("/dev/{}", unit.node_device().unwrap());
        let node = FwNode::new();
        node.open(&cdev)?;

        let card_cntr = CardCntr::default();
        card_cntr.card.open(card_id, 0)?;

        let raw = node.config_rom()?;
        let model_id = ConfigRom::try_from(&raw[..])
            .map_err(|e| {
                let msg = format!("Malformed configuration ROM detected: {}", e);
                Error::new(FileError::Nxio, &msg)
            })
            .and_then(|config_rom| {
                config_rom.get_model_id().ok_or_else(|| {
                    Error::new(FileError::Nxio, "Unexpected format of configuration ROM")
                })
            })?;

        let runtime = match model_id {
            0x00000001 => Self::Ff800(Ff800Runtime::new(unit, node, card_cntr)?),
            0x00000002 => Self::Ff400(Ff400Runtime::new(unit, node, card_cntr)?),
            0x00000004 => Self::FfUcx(FfUcxRuntime::new(unit, node, card_cntr)?),
            0x00000005 => Self::Ff802(Ff802Runtime::new(unit, node, card_cntr)?),
            _ => Err(Error::new(FileError::Nxio, "Not supported."))?,
        };

        Ok(runtime)
    }

    fn listen(&mut self) -> Result<(), Error> {
        match self {
            Self::Ff800(model) => model.listen(),
            Self::Ff400(model) => model.listen(),
            Self::Ff802(model) => model.listen(),
            Self::FfUcx(model) => model.listen(),
        }
    }

    fn run(&mut self) -> Result<(), Error> {
        match self {
            Self::Ff800(model) => model.run(),
            Self::Ff400(model) => model.run(),
            Self::Ff802(model) => model.run(),
            Self::FfUcx(model) => model.run(),
        }
    }
}

const NODE_DISPATCHER_NAME: &str = "node event dispatcher";
const SYSTEM_DISPATCHER_NAME: &str = "system event dispatcher";
const TIMER_DISPATCHER_NAME: &str = "interval timer dispatcher";

const TIMER_NAME: &str = "metering";
const TIMER_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);

fn spdif_iface_to_string(iface: &SpdifIface) -> String {
    match iface {
        SpdifIface::Coaxial => "Coaxial",
        SpdifIface::Optical => "Optical",
    }
    .to_string()
}

fn spdif_format_to_string(fmt: &SpdifFormat) -> String {
    match fmt {
        SpdifFormat::Consumer => "Consumer",
        SpdifFormat::Professional => "Professional",
    }
    .to_string()
}

fn optical_output_signal_to_string(sig: &OpticalOutputSignal) -> String {
    match sig {
        OpticalOutputSignal::Adat => "ADAT",
        OpticalOutputSignal::Spdif => "S/PDIF",
    }
    .to_string()
}

fn former_line_in_nominal_level_to_string(level: &FormerLineInNominalLevel) -> String {
    match level {
        FormerLineInNominalLevel::Low => "Low",
        FormerLineInNominalLevel::Consumer => "-10dBV",
        FormerLineInNominalLevel::Professional => "+4dBu",
    }
    .to_string()
}

fn line_out_nominal_level_to_str(level: &LineOutNominalLevel) -> &str {
    match level {
        LineOutNominalLevel::High => "High",
        LineOutNominalLevel::Consumer => "-10dBV",
        LineOutNominalLevel::Professional => "+4dBu",
    }
}

fn clk_nominal_rate_to_string(rate: &ClkNominalRate) -> String {
    match rate {
        ClkNominalRate::R32000 => "32000",
        ClkNominalRate::R44100 => "44100",
        ClkNominalRate::R48000 => "48000",
        ClkNominalRate::R64000 => "64000",
        ClkNominalRate::R88200 => "88200",
        ClkNominalRate::R96000 => "96000",
        ClkNominalRate::R128000 => "128000",
        ClkNominalRate::R176400 => "176400",
        ClkNominalRate::R192000 => "192000",
    }
    .to_string()
}

fn optional_clk_nominal_rate_to_string(rate: &Option<ClkNominalRate>) -> String {
    if let Some(r) = rate {
        clk_nominal_rate_to_string(r)
    } else {
        "not-detected".to_string()
    }
}

struct FfServiceCmd;

#[derive(Parser, Default)]
#[clap(name = "snd-fireface-ctl-service")]
struct Arguments {
    /// The numeric identifier of sound card in Linux sound subsystem.
    card_id: u32,

    /// The level to debug runtime, disabled as a default.
    #[clap(long, short, arg_enum)]
    log_level: Option<LogLevel>,
}

impl ServiceCmd<Arguments, u32, FfRuntime> for FfServiceCmd {
    fn params(args: &Arguments) -> (u32, Option<LogLevel>) {
        (args.card_id, args.log_level)
    }
}

fn main() {
    FfServiceCmd::run()
}
