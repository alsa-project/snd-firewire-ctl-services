// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Output protocol specific to Alesis iO FireWire series.
//!
//! The module includes structure, enumeration, and trait and its implementation for output
//! protocol defined by Alesis for iO FireWire series.

use glib::Error;
use hinawa::FwNode;

use super::*;

use crate::*;

/// The enumeration to represent nominal level of signal.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NominalSignalLevel{
    /// -10dBV.
    Consumer,
    /// +4dBu.
    Professional,
}

impl Default for NominalSignalLevel {
    fn default() -> Self {
        NominalSignalLevel::Consumer
    }
}

impl From<u32> for NominalSignalLevel {
    fn from(val: u32) -> Self {
        if val > 0 {
            Self::Professional
        } else {
            Self::Consumer
        }
    }
}

impl From<NominalSignalLevel> for u32 {
    fn from(level: NominalSignalLevel) -> Self {
        match level {
            NominalSignalLevel::Consumer => 0,
            NominalSignalLevel::Professional => 1,
        }
    }
}

/// The enumeration to represent source of 6/7 channels of digital B input.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DigitalB67Src{
    Spdif12,
    Adat67,
}

impl Default for DigitalB67Src {
    fn default() -> Self {
        Self::Spdif12
    }
}

impl From<u32> for DigitalB67Src {
    fn from(val: u32) -> Self {
        if val > 0 {
            Self::Adat67
        } else {
            Self::Spdif12
        }
    }
}

impl From<DigitalB67Src> for u32 {
    fn from(src: DigitalB67Src) -> Self {
        match src {
            DigitalB67Src::Spdif12 => 0,
            DigitalB67Src::Adat67 => 1,
        }
    }
}

/// The enumeration to represent pair of mixer output.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MixerOutPair{
    Mixer01,
    Mixer23,
    Mixer45,
    Mixer67,
}

impl Default for MixerOutPair {
    fn default() -> Self {
        Self::Mixer01
    }
}

impl From<u32> for MixerOutPair {
    fn from(val: u32) -> Self {
        match val {
            3 => Self::Mixer67,
            2 => Self::Mixer45,
            1 => Self::Mixer23,
            _ => Self::Mixer01,
        }
    }
}

impl From<MixerOutPair> for u32 {
    fn from(pair: MixerOutPair) -> Self {
        match pair {
            MixerOutPair::Mixer01 => 0,
            MixerOutPair::Mixer23 => 1,
            MixerOutPair::Mixer45 => 2,
            MixerOutPair::Mixer67 => 3,
        }
    }
}

/// The trait to represent output protocol for iO FireWire series.
pub trait IoOutputProtocol: AlesisIoProtocol {
    const OUT_LEVEL_OFFSET: usize = 0x0564;
    const MIXER_DIGITAL_B_67_SRC_OFFSET: usize = 0x0568;
    const SPDIF_OUT_SRC_OFFSET: usize = 0x056c;
    const HP34_SRC_OFFSET: usize = 0x0570;

    fn read_out_levels(
        &self,
        node: &mut FwNode,
        levels: &mut [NominalSignalLevel],
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut flags = vec![false;levels.len()];
        self.read_flags(node, Self::OUT_LEVEL_OFFSET, &mut flags[..], timeout_ms)
            .map(|_| {
                levels.iter_mut()
                    .zip(flags.iter())
                    .for_each(|(l, &f)| *l = NominalSignalLevel::from(f as u32))
            })
    }

    fn write_out_levels(
        &self,
        node: &mut FwNode,
        levels: &[NominalSignalLevel],
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut flags: Vec<bool> = levels.iter()
            .map(|l| u32::from(*l) > 0)
            .collect();
        self.write_flags(node, Self::OUT_LEVEL_OFFSET, &mut flags, timeout_ms)
    }

    fn read_mixer_digital_b_67_src(
        &self,
        node: &mut FwNode,
        src: &mut DigitalB67Src,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut raw = [0;4];
        self.read_block(node, Self::MIXER_DIGITAL_B_67_SRC_OFFSET, &mut raw, timeout_ms)
            .map(|_| src.parse_quadlet(&raw))
    }

    fn write_mixer_digital_b_67_src(
        &self,
        node: &mut FwNode,
        src: &DigitalB67Src,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut raw = [0;4];
        src.build_quadlet(&mut raw);
        self.write_block(node, Self::MIXER_DIGITAL_B_67_SRC_OFFSET, &mut raw, timeout_ms)
    }

    fn read_spdif_out_src(
        &self,
        node: &mut FwNode,
        src: &mut MixerOutPair,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut raw = [0;4];
        self.read_block(node, Self::SPDIF_OUT_SRC_OFFSET, &mut raw, timeout_ms)
            .map(|_| src.parse_quadlet(&raw))
    }

    fn write_spdif_out_src(
        &self,
        node: &mut FwNode,
        src: &MixerOutPair,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut raw = [0;4];
        src.build_quadlet(&mut raw);
        self.write_block(node, Self::SPDIF_OUT_SRC_OFFSET, &mut raw, timeout_ms)
    }

    fn read_hp23_out_src(
        &self,
        node: &mut FwNode,
        src: &mut MixerOutPair,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut raw = [0;4];
        self.read_block(node, Self::HP34_SRC_OFFSET, &mut raw, timeout_ms)
            .map(|_| src.parse_quadlet(&raw))
    }

    fn write_hp23_out_src(
        &self,
        node: &mut FwNode,
        src: &MixerOutPair,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut raw = [0;4];
        src.build_quadlet(&mut raw);
        self.write_block(node, Self::HP34_SRC_OFFSET, &mut raw, timeout_ms)
    }
}

impl<O> IoOutputProtocol for O
    where O: AlesisIoProtocol,
{}
