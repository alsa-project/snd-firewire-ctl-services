// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
mod v1_runtime;
mod register_dsp_runtime;
mod command_dsp_runtime;

mod f828;
mod f896;

mod f828mk2;
mod f896hd;
mod f8pre;
mod traveler;
mod ultralite;

mod audioexpress;
mod f828mk3;
mod f828mk3_hybrid;
mod h4pre;
mod ultralite_mk3;
mod ultralite_mk3_hybrid;

mod common_ctls;
mod v1_ctls;
mod v2_ctls;
mod v3_ctls;
mod register_dsp_ctls;
mod command_dsp_ctls;

use glib::{Error, FileError};
use std::convert::TryFrom;

use hinawa::{FwNodeExtManual, SndUnitExt, SndMotuExt};

use core::RuntimeOperation;

use ieee1212_config_rom::*;
use motu_protocols::{config_rom::*, *};

use crate::{v1_runtime::*, register_dsp_runtime::*, command_dsp_runtime::*};

pub enum MotuRuntime {
         F828(F828Runtime),
         F896(F896Runtime),
         F828mk2(F828mk2Runtime),
         F896hd(F896hdRuntime),
         Traveler(TravelerRuntime),
         Ultralite(UltraliteRuntime),
         F8pre(F8preRuntime),
         Ultralitemk3(UltraliteMk3Runtime),
         Ultralitemk3Hybrid(UltraliteMk3HybridRuntime),
         AudioExpress(AudioExpressRuntime),
         F828mk3(F828mk3Runtime),
         F828mk3Hybrid(F828mk3HybridRuntime),
         H4pre(H4preRuntime),
}

impl RuntimeOperation<u32> for MotuRuntime {
    fn new(card_id: u32) -> Result<Self, Error> {
        let unit = hinawa::SndMotu::new();
        unit.open(&format!("/dev/snd/hwC{}D0", card_id))?;

        let node = unit.get_node();
        let data = node.get_config_rom()?;
        let unit_data = ConfigRom::try_from(data)
            .map_err(|e| {
                let msg = format!("Malformed configuration ROM detected: {}", e);
                Error::new(FileError::Nxio, &msg)
            })
            .and_then(|config_rom| {
                config_rom.get_unit_data()
                    .ok_or_else(|| {
                        Error::new(FileError::Nxio, "Unexpected content of configuration ROM.")
                    })
            })?;

        let version = unit_data.version;

        match unit_data.model_id {
            0x000001 => Ok(Self::F828(F828Runtime::new(unit, card_id, version)?)),
            0x000002 => Ok(Self::F896(F896Runtime::new(unit, card_id, version)?)),
            0x000003 => Ok(Self::F828mk2(F828mk2Runtime::new(unit, card_id, version)?)),
            0x000005 => Ok(Self::F896hd(F896hdRuntime::new(unit, card_id, version)?)),
            0x000009 => Ok(Self::Traveler(TravelerRuntime::new(unit, card_id, version)?)),
            0x00000d => Ok(Self::Ultralite(UltraliteRuntime::new(unit, card_id, version)?)),
            0x00000f => Ok(Self::F8pre(F8preRuntime::new(unit, card_id, version)?)),
            0x000015 => Ok(Self::F828mk3(F828mk3Runtime::new(unit, card_id, version)?)),
            0x000019 => Ok(Self::Ultralitemk3(UltraliteMk3Runtime::new(unit, card_id, version)?)),
            0x000030 => Ok(Self::Ultralitemk3Hybrid(UltraliteMk3HybridRuntime::new(unit, card_id, version)?)),
            0x000033 => Ok(Self::AudioExpress(AudioExpressRuntime::new(unit, card_id, version)?)),
            0x000035 => Ok(Self::F828mk3Hybrid(F828mk3HybridRuntime::new(unit, card_id, version)?)),
            0x000045 => Ok(Self::H4pre(H4preRuntime::new(unit, card_id, version)?)),
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
            Self::Ultralitemk3(runtime) => runtime.listen(),
            Self::Ultralitemk3Hybrid(runtime) => runtime.listen(),
            Self::AudioExpress(runtime) => runtime.listen(),
            Self::F828mk3Hybrid(runtime) => runtime.listen(),
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
            Self::Ultralitemk3(runtime) => runtime.run(),
            Self::Ultralitemk3Hybrid(runtime) => runtime.run(),
            Self::AudioExpress(runtime) => runtime.run(),
            Self::F828mk3Hybrid(runtime) => runtime.run(),
            Self::H4pre(runtime) => runtime.run(),
        }
    }
}

pub fn clk_rate_to_str(rate: &ClkRate) -> &'static str {
    match rate {
        ClkRate::R44100 => "44100",
        ClkRate::R48000 => "48000",
        ClkRate::R88200 => "88200",
        ClkRate::R96000 => "96000",
        ClkRate::R176400 => "176400",
        ClkRate::R192000 => "192000",
    }
}

pub fn target_port_to_str(port: &TargetPort) -> &'static str {
    match port {
        TargetPort::Disabled => "Disabled",
        TargetPort::AnalogPair0 => "Analog-1/2",
        TargetPort::AnalogPair1 => "Analog-3/4",
        TargetPort::AnalogPair2 => "Analog-5/6",
        TargetPort::AnalogPair3 => "Analog-7/8",
        TargetPort::AesEbuPair0 => "AES/EBU-1/2",
        TargetPort::PhonePair0 => "Phone-1/2",
        TargetPort::MainPair0 => "Main-1/2",
        TargetPort::SpdifPair0 => "SPDIF-1/2",
        TargetPort::AdatPair0 => "ADAT-1/2",
        TargetPort::AdatPair1 => "ADAT-3/4",
        TargetPort::AdatPair2 => "ADAT-5/6",
        TargetPort::AdatPair3 => "ADAT-7/8",
        TargetPort::Analog0 => "Analog-1",
        TargetPort::Analog1 => "Analog-2",
        TargetPort::Analog2 => "Analog-3",
        TargetPort::Analog3 => "Analog-4",
        TargetPort::Analog4 => "Analog-5",
        TargetPort::Analog5 => "Analog-6",
        TargetPort::Analog6 => "Analog-7",
        TargetPort::Analog7 => "Analog-8",
        TargetPort::AesEbu0 => "AES/EBU-1",
        TargetPort::AesEbu1 => "AES/EBU-2",
        TargetPort::Analog6Pairs => "Analog-1/2/3/4/5/6",
        TargetPort::Analog8Pairs => "Analog-1/2/3/4/5/6/7/8",
        TargetPort::OpticalAPair0 => "Optical-A-1/2",
        TargetPort::OpticalAPair1 => "Optical-A-3/4",
        TargetPort::OpticalAPair2 => "Optical-A-5/6",
        TargetPort::OpticalAPair3 => "Optical-A-7/8",
        TargetPort::OpticalBPair0 => "Optical-B-1/2",
        TargetPort::OpticalBPair1 => "Optical-B-3/4",
        TargetPort::OpticalBPair2 => "Optical-B-5/6",
        TargetPort::OpticalBPair3 => "Optical-B-7/8",
        TargetPort::Mic0 => "Mic-1",
        TargetPort::Mic1 => "Mic-2",
        TargetPort::Spdif0 => "S/PDIF-1",
        TargetPort::Spdif1 => "S/PDIF-2",
        TargetPort::Adat0 => "ADAT-1",
        TargetPort::Adat1 => "ADAT-2",
        TargetPort::Adat2 => "ADAT-3",
        TargetPort::Adat3 => "ADAT-4",
        TargetPort::Adat4 => "ADAT-5",
        TargetPort::Adat5 => "ADAT-6",
        TargetPort::Adat6 => "ADAT-7",
        TargetPort::Adat7 => "ADAT-8",
        TargetPort::OpticalA0 => "Optical-A-0",
        TargetPort::OpticalA1 => "Optical-A-1",
        TargetPort::OpticalA2 => "Optical-A-2",
        TargetPort::OpticalA3 => "Optical-A-3",
        TargetPort::OpticalA4 => "Optical-A-4",
        TargetPort::OpticalA5 => "Optical-A-5",
        TargetPort::OpticalA6 => "Optical-A-6",
        TargetPort::OpticalA7 => "Optical-A-7",
        TargetPort::OpticalB0 => "Optical-B-0",
        TargetPort::OpticalB1 => "Optical-B-1",
        TargetPort::OpticalB2 => "Optical-B-2",
        TargetPort::OpticalB3 => "Optical-B-3",
        TargetPort::OpticalB4 => "Optical-B-4",
        TargetPort::OpticalB5 => "Optical-B-5",
        TargetPort::OpticalB6 => "Optical-B-6",
        TargetPort::OpticalB7 => "Optical-B-7",
    }
}

pub fn nominal_signal_level_to_str(level: &NominalSignalLevel) -> &'static str {
    match level {
        NominalSignalLevel::Consumer => "-10dBu",
        NominalSignalLevel::Professional => "+4dBV",
    }
}
