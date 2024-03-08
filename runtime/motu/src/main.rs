// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
mod command_dsp_runtime;
mod register_dsp_runtime;
mod v1_runtime;

mod f828_model;
mod f896_model;

mod f828mk2_model;
mod f896hd_model;
mod f8pre_model;
mod traveler_model;
mod ultralite_model;

mod audioexpress_model;
mod f828mk3_hybrid_model;
mod f828mk3_model;
mod f896mk3_hybrid_model;
mod f896mk3_model;
mod h4pre_model;
mod track16_model;
mod traveler_mk3_model;
mod ultralite_mk3_hybrid_model;
mod ultralite_mk3_model;

mod command_dsp_ctls;
mod common_ctls;
mod register_dsp_ctls;
mod v1_ctls;
mod v2_ctls;
mod v3_ctls;

use {
    self::{command_dsp_runtime::*, register_dsp_runtime::*, v1_runtime::*},
    clap::Parser,
    firewire_motu_protocols as protocols,
    glib::{Error, FileError},
    hinawa::{
        prelude::{FwNodeExt, FwNodeExtManual},
        FwNode,
    },
    hitaki::{prelude::*, *},
    ieee1212_config_rom::*,
    protocols::{config_rom::*, *},
    runtime_core::{cmdline::*, LogLevel, *},
    std::{convert::TryFrom, marker::PhantomData},
    tracing::{debug, debug_span, Level},
};

enum MotuRuntime {
    F828(F828Runtime),
    F896(F896Runtime),
    F828mk2(F828mk2Runtime),
    F896hd(F896hdRuntime),
    Traveler(TravelerRuntime),
    Ultralite(UltraliteRuntime),
    F8pre(F8preRuntime),
    Ultralitemk3(UltraliteMk3Runtime),
    TravelerMk3(TravelerMk3Runtime),
    Ultralitemk3Hybrid(UltraliteMk3HybridRuntime),
    AudioExpress(AudioExpressRuntime),
    F828mk3(F828mk3Runtime),
    F828mk3Hybrid(F828mk3HybridRuntime),
    F896mk3(F896mk3Runtime),
    F896mk3Hybrid(F896mk3HybridRuntime),
    Track16(Track16Runtime),
    H4pre(H4preRuntime),
}

impl RuntimeOperation<u32> for MotuRuntime {
    fn new(card_id: u32, log_level: Option<LogLevel>) -> Result<Self, Error> {
        if let Some(level) = log_level {
            let fmt_level = match level {
                LogLevel::Debug => Level::DEBUG,
            };
            tracing_subscriber::fmt().with_max_level(fmt_level).init();
        }

        let cdev = format!("/dev/snd/hwC{}D0", card_id);
        let unit = SndMotu::new();
        unit.open(&cdev, 0)?;

        let cdev = format!("/dev/{}", unit.node_device().unwrap());
        let node = FwNode::new();
        node.open(&cdev, 0)?;

        let data = node.config_rom()?;
        let unit_data = ConfigRom::try_from(data)
            .map_err(|e| {
                let msg = format!("Malformed configuration ROM detected: {}", e);
                Error::new(FileError::Nxio, &msg)
            })
            .and_then(|config_rom| {
                config_rom.get_unit_data().ok_or_else(|| {
                    Error::new(FileError::Nxio, "Unexpected content of configuration ROM.")
                })
            })?;

        let version = unit_data.version;

        match unit_data.model_id {
            0x000001 => Ok(Self::F828(F828Runtime::new(unit, node, card_id, version)?)),
            0x000002 => Ok(Self::F896(F896Runtime::new(unit, node, card_id, version)?)),
            0x000003 => Ok(Self::F828mk2(F828mk2Runtime::new(
                unit, node, card_id, version,
            )?)),
            0x000005 => Ok(Self::F896hd(F896hdRuntime::new(
                unit, node, card_id, version,
            )?)),
            0x000009 => Ok(Self::Traveler(TravelerRuntime::new(
                unit, node, card_id, version,
            )?)),
            0x00000d => Ok(Self::Ultralite(UltraliteRuntime::new(
                unit, node, card_id, version,
            )?)),
            0x00000f => Ok(Self::F8pre(F8preRuntime::new(
                unit, node, card_id, version,
            )?)),
            0x000015 => Ok(Self::F828mk3(F828mk3Runtime::new(
                unit, node, card_id, version,
            )?)),
            0x000017 => Ok(Self::F896mk3(F896mk3Runtime::new(
                unit, node, card_id, version,
            )?)),
            0x000019 => Ok(Self::Ultralitemk3(UltraliteMk3Runtime::new(
                unit, node, card_id, version,
            )?)),
            0x00001b => Ok(Self::TravelerMk3(TravelerMk3Runtime::new(
                unit, node, card_id, version,
            )?)),
            0x000030 => Ok(Self::Ultralitemk3Hybrid(UltraliteMk3HybridRuntime::new(
                unit, node, card_id, version,
            )?)),
            0x000033 => Ok(Self::AudioExpress(AudioExpressRuntime::new(
                unit, node, card_id, version,
            )?)),
            0x000035 => Ok(Self::F828mk3Hybrid(F828mk3HybridRuntime::new(
                unit, node, card_id, version,
            )?)),
            0x000037 => Ok(Self::F896mk3Hybrid(F896mk3HybridRuntime::new(
                unit, node, card_id, version,
            )?)),
            0x000039 => Ok(Self::Track16(Track16Runtime::new(
                unit, node, card_id, version,
            )?)),
            0x000045 => Ok(Self::H4pre(H4preRuntime::new(
                unit, node, card_id, version,
            )?)),
            _ => {
                let label = format!("Unsupported model ID: 0x{:06x}", unit_data.model_id);
                Err(Error::new(FileError::Noent, &label))
            }
        }
    }

    fn listen(&mut self) -> Result<(), Error> {
        match self {
            Self::F828(runtime) => runtime.listen(),
            Self::F896(runtime) => runtime.listen(),
            Self::F828mk2(runtime) => runtime.listen(),
            Self::F896hd(runtime) => runtime.listen(),
            Self::Traveler(runtime) => runtime.listen(),
            Self::Ultralite(runtime) => runtime.listen(),
            Self::F8pre(runtime) => runtime.listen(),
            Self::F828mk3(runtime) => runtime.listen(),
            Self::F896mk3(runtime) => runtime.listen(),
            Self::Ultralitemk3(runtime) => runtime.listen(),
            Self::TravelerMk3(runtime) => runtime.listen(),
            Self::Ultralitemk3Hybrid(runtime) => runtime.listen(),
            Self::AudioExpress(runtime) => runtime.listen(),
            Self::F828mk3Hybrid(runtime) => runtime.listen(),
            Self::F896mk3Hybrid(runtime) => runtime.listen(),
            Self::Track16(runtime) => runtime.listen(),
            Self::H4pre(runtime) => runtime.listen(),
        }
    }

    fn run(&mut self) -> Result<(), Error> {
        match self {
            Self::F828(runtime) => runtime.run(),
            Self::F896(runtime) => runtime.run(),
            Self::F828mk2(runtime) => runtime.run(),
            Self::F896hd(runtime) => runtime.run(),
            Self::Traveler(runtime) => runtime.run(),
            Self::Ultralite(runtime) => runtime.run(),
            Self::F8pre(runtime) => runtime.run(),
            Self::F828mk3(runtime) => runtime.run(),
            Self::F896mk3(runtime) => runtime.run(),
            Self::Ultralitemk3(runtime) => runtime.run(),
            Self::TravelerMk3(runtime) => runtime.run(),
            Self::Ultralitemk3Hybrid(runtime) => runtime.run(),
            Self::AudioExpress(runtime) => runtime.run(),
            Self::F828mk3Hybrid(runtime) => runtime.run(),
            Self::F896mk3Hybrid(runtime) => runtime.run(),
            Self::Track16(runtime) => runtime.run(),
            Self::H4pre(runtime) => runtime.run(),
        }
    }
}

pub(crate) fn clk_rate_to_str(rate: &ClkRate) -> &'static str {
    match rate {
        ClkRate::R44100 => "44100",
        ClkRate::R48000 => "48000",
        ClkRate::R88200 => "88200",
        ClkRate::R96000 => "96000",
        ClkRate::R176400 => "176400",
        ClkRate::R192000 => "192000",
    }
}

pub(crate) fn target_port_to_string(port: &TargetPort) -> String {
    match port {
        TargetPort::Disabled => "Disabled".to_string(),
        TargetPort::AnalogPair(ch) => format!("Analog-{}/{}", *ch * 2 + 1, *ch * 2 + 2),
        TargetPort::AesEbuPair => "AES/EBU-1/2".to_string(),
        TargetPort::PhonePair => "Phone-1/2".to_string(),
        TargetPort::MainPair => "Main-1/2".to_string(),
        TargetPort::SpdifPair => "SPDIF-1/2".to_string(),
        TargetPort::AdatPair(ch) => format!("ADAT-{}/{}", *ch * 2 + 1, *ch * 2 + 2),
        TargetPort::Analog6Pairs => "Analog-1/2/3/4/5/6".to_string(),
        TargetPort::Analog8Pairs => "Analog-1/2/3/4/5/6/7/8".to_string(),
        TargetPort::OpticalAPair(ch) => format!("Optical-A-{}/{}", *ch + 1, *ch + 2),
        TargetPort::OpticalBPair(ch) => format!("Optical-B-{}/{}", *ch + 1, *ch + 2),
        TargetPort::Analog(ch) => format!("Analog-{}", *ch + 1),
        TargetPort::AesEbu(ch) => format!("AES/EBU-{}", *ch + 1),
        TargetPort::Phone(ch) => format!("Phone-{}", *ch + 1),
        TargetPort::Main(ch) => format!("Main-{}", *ch + 1),
        TargetPort::Spdif(ch) => format!("S/PDIF-{}", *ch + 1),
        TargetPort::Adat(ch) => format!("ADAT-{}", *ch + 1),
        TargetPort::OpticalA(ch) => format!("Optical-A-{}", *ch + 1),
        TargetPort::OpticalB(ch) => format!("Optical-B-{}", *ch + 1),
    }
}

pub(crate) fn nominal_signal_level_to_str(level: &NominalSignalLevel) -> &'static str {
    match level {
        NominalSignalLevel::Consumer => "-10dBu",
        NominalSignalLevel::Professional => "+4dBV",
    }
}

struct MotuServiceCmd;

#[derive(Parser, Default)]
#[clap(name = "snd-firewire-motu-ctl-service")]
struct Arguments {
    /// The numeric identifier of sound card in Linux sound subsystem.
    card_id: u32,

    /// The level to debug runtime, disabled as a default.
    #[clap(long, short, value_enum)]
    log_level: Option<LogLevel>,
}

impl ServiceCmd<Arguments, u32, MotuRuntime> for MotuServiceCmd {
    fn params(args: &Arguments) -> (u32, Option<LogLevel>) {
        (args.card_id, args.log_level)
    }
}

fn main() {
    MotuServiceCmd::run()
}
