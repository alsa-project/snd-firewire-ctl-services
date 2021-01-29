// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use hinawa::{FwNode, FwReq};

use crate::tcat::*;
use crate::*;

#[derive(Default, Debug)]
pub struct FStudioProto(FwReq);

impl AsRef<FwReq> for FStudioProto {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

/// The trait to represent protocol specific to FireStudio.
pub trait PresonusFStudioProto<T> : GeneralProtocol<T>
    where T: AsRef<FwNode>,
{
    const OFFSET: usize = 0x00700000;

    fn read(&self, node: &T, offset: usize, raw: &mut [u8], timeout_ms: u32) -> Result<(), Error> {
        GeneralProtocol::read(self, node, Self::OFFSET + offset, raw, timeout_ms)
    }

    fn write(&self, node: &T, offset: usize, raw: &mut [u8], timeout_ms: u32) -> Result<(), Error> {
        GeneralProtocol::write(self, node, Self::OFFSET + offset, raw, timeout_ms)
    }
}

impl<T: AsRef<FwNode>> PresonusFStudioProto<T> for FStudioProto {}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct FStudioMeter{
    pub analog_inputs: [u8;8],
    pub stream_inputs: [u8;18],
    pub mixer_outputs: [u8;18],
}

impl FStudioMeter {
    const SIZE: usize = 0x40;
}

pub trait PresonusFStudioMeterProtocol<T> : PresonusFStudioProto<T>
    where T: AsRef<FwNode>,
{
    const METER_OFFSET: usize = 0x13e8;

    fn read_meter(&self, node: &T, meter: &mut FStudioMeter, timeout_ms: u32) -> Result<(), Error> {
        let mut raw = vec![0;FStudioMeter::SIZE];
        PresonusFStudioProto::read(self, node, Self::METER_OFFSET, &mut raw, timeout_ms)
            .map(|_| {
                let mut quadlet = [0;4];
                (0..(FStudioMeter::SIZE / 4))
                    .for_each(|i| {
                        let pos = i * 4;
                        quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                        let val = u32::from_be_bytes(quadlet);
                        raw[pos..(pos + 4)].copy_from_slice(&val.to_le_bytes());
                    });
                meter.analog_inputs.copy_from_slice(&raw[8..16]);
                meter.stream_inputs.copy_from_slice(&raw[16..34]);
                meter.mixer_outputs.copy_from_slice(&raw[40..58]);
            })
    }
}

impl<T: AsRef<FwNode>> PresonusFStudioMeterProtocol<T> for FStudioProto {}

/// The enumeration to represent source of output.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OutputSrc{
    Analog(usize),
    Adat0(usize),
    Spdif(usize),
    Stream(usize),
    StreamAdat1(usize),
    MixerOut(usize),
    Reserved(usize),
}

impl From<u32> for OutputSrc {
    fn from(val: u32) -> Self {
        let v = val as usize;
        match v {
            0x00..=0x07 => Self::Analog(v),
            0x08..=0x0f => Self::Adat0(v - 0x08),
            0x10..=0x11 => Self::Spdif(v - 0x10),
            0x12..=0x1b => Self::Stream(v - 0x12),
            0x1c..=0x23 => Self::StreamAdat1(v - 0x1c),
            0x24..=0x35 => Self::MixerOut(v - 0x24),
            _ => Self::Reserved(v),
        }
    }
}

impl From<OutputSrc> for u32 {
    fn from(src: OutputSrc) -> Self {
        (match src {
            OutputSrc::Analog(val) => val,
            OutputSrc::Adat0(val) => val + 0x08,
            OutputSrc::Spdif(val) => val + 0x10,
            OutputSrc::Stream(val) => val + 0x12,
            OutputSrc::StreamAdat1(val) => val + 0x1c,
            OutputSrc::MixerOut(val) => val + 0x24,
            OutputSrc::Reserved(val) => val,
        }) as u32
    }
}

impl Default for OutputSrc {
    fn default() -> Self {
        Self::Reserved(0xff)
    }
}

/// The structure to represent state of outputs for FireStudio.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct OutputState{
    pub vols: [u8;18],
    pub mutes: [bool;18],
    pub srcs: [OutputSrc;18],
    pub links: [bool;9],
}

/// The trait to represent output protocol for FireStudio.
pub trait PresonusFStudioOutputProtocol<T>  : PresonusFStudioProto<T>
    where T: AsRef<FwNode>,
{
    const PARAMS_OFFSET: usize = 0x0f68;
    const SRC_OFFSET: usize = 0x10ac;
    const LINK_OFFSET: usize = 0x1150;
    const BNC_TERMINATE_OFFSET: usize = 0x1118;

    fn read_output_states(&self, node: &T, states: &mut OutputState, timeout_ms: u32) -> Result<(), Error> {
        let mut raw = vec![0;4 * states.vols.len() * 3];
        PresonusFStudioProto::read(self, node, Self::PARAMS_OFFSET, &mut raw, timeout_ms)?;
        let mut quadlet = [0;4];
        let quads: Vec<u32> = (0..(states.vols.len() * 3))
            .map(|i| {
                let pos = i * 4;
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                u32::from_be_bytes(quadlet)
            })
            .collect();
        states.vols.iter_mut()
            .enumerate()
            .for_each(|(i, vol)| {
                let pos = i * 3;
                *vol = quads[pos] as u8;
            });
        states.mutes.iter_mut()
            .enumerate()
            .for_each(|(i, mute)| {
                let pos = 2 + i * 3;
                *mute = quads[pos] > 0;
            });

        let mut raw = vec![0;4 * states.srcs.len()];
        PresonusFStudioProto::read(self, node, Self::SRC_OFFSET, &mut raw, timeout_ms)
            .map(|_| states.srcs.parse_quadlet_block(&raw))?;

        let mut raw = [0;4];
        PresonusFStudioProto::read(self, node, Self::LINK_OFFSET, &mut raw, timeout_ms)?;
        let val = u32::from_be_bytes(raw);
        states.links.iter_mut()
            .enumerate()
            .for_each(|(i, link)| *link = val & (1 << i) > 0);

        Ok(())
    }

    fn write_output_vols(&self, node: &T, states: &mut OutputState, vols: &[u8], timeout_ms: u32)
        -> Result<(), Error>
    {
        assert_eq!(vols.len(), states.vols.len());

        let mut raw = [0;4];
        states.vols.iter_mut()
            .zip(vols.iter())
            .enumerate()
            .filter(|(_, (old, new))| !new.eq(old))
            .try_for_each(|(i, (old, new))| {
                let pos = i * 3 * 4;
                raw.copy_from_slice(&(*new as u32).to_be_bytes());
                PresonusFStudioProto::write(self, node, Self::PARAMS_OFFSET + pos, &mut raw, timeout_ms)
                    .map(|_| *old = *new)
            })
    }

    fn write_output_mute(&self, node: &T, states: &mut OutputState, mutes: &[bool], timeout_ms: u32)
        -> Result<(), Error>
    {
        assert_eq!(mutes.len(), states.mutes.len());

        let mut raw = [0;4];
        states.mutes.iter_mut()
            .zip(mutes.iter())
            .enumerate()
            .filter(|(_, (old, new))| !new.eq(old))
            .try_for_each(|(i, (old, new))| {
                let pos = (2 + i * 3) * 4;
                raw.copy_from_slice(&(*new as u32).to_be_bytes());
                PresonusFStudioProto::write(self, node, Self::PARAMS_OFFSET + pos, &mut raw, timeout_ms)
                    .map(|_| *old = *new)
            })
    }

    fn write_output_src(&self, node: &T, states: &mut OutputState, srcs: &[OutputSrc], timeout_ms: u32)
        -> Result<(), Error>
    {
        assert_eq!(srcs.len(), states.srcs.len());

        let mut raw = [0;4];
        states.srcs.iter_mut()
            .zip(srcs.iter())
            .enumerate()
            .filter(|(_, (old, new))| !new.eq(old))
            .try_for_each(|(i, (old, new))| {
                let pos = i * 4;
                raw.copy_from_slice(&u32::from(*new).to_be_bytes());
                PresonusFStudioProto::write(self, node, Self::SRC_OFFSET + pos, &mut raw, timeout_ms)
                    .map(|_| *old = *new)
            })
    }

    fn write_output_link(&self, node: &T, states: &mut OutputState, links: &[bool], timeout_ms: u32)
        -> Result<(), Error>
    {
        assert_eq!(links.len(), states.links.len());

        let val: u32 = links.iter()
            .enumerate()
            .filter(|(_, &link)| link)
            .fold(0u32, |val, (i, _)| val | (1 << i));

        let mut raw = [0;4];
        val.build_quadlet(&mut raw);
        PresonusFStudioProto::write(self, node, Self::LINK_OFFSET, &mut raw, timeout_ms)
            .map(|_| states.links.copy_from_slice(links))
    }

    fn read_bnc_terminate(&self, node: &T, timeout_ms: u32) -> Result<bool, Error> {
        let mut raw = [0;4];
        PresonusFStudioProto::read(self, node, Self::BNC_TERMINATE_OFFSET, &mut raw, timeout_ms)
            .map(|_| u32::from_be_bytes(raw) > 0)
    }

    fn write_bnc_terminalte(&self, node: &T, terminate: bool, timeout_ms: u32) -> Result<(), Error> {
        let mut raw = [0;4];
        terminate.build_quadlet(&mut raw);
        PresonusFStudioProto::write(self, node, Self::BNC_TERMINATE_OFFSET, &mut raw, timeout_ms)
    }
}

impl<T: AsRef<FwNode>> PresonusFStudioOutputProtocol<T> for FStudioProto {}

/// The enumeration to represent target of output assignment.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AssignTarget {
    Analog01,
    Analog23,
    Analog56,
    Analog78,
    AdatA01,
    AdatA23,
    AdatA45,
    AdatA67,
    Spdif01,
    Reserved(u32),
}

impl From<u32> for AssignTarget {
    fn from(val: u32) -> Self {
        match val {
            0x00 => Self::Analog01,
            0x02 => Self::Analog23,
            0x04 => Self::Analog56,
            0x06 => Self::Analog78,
            0x08 => Self::AdatA01,
            0x0a => Self::AdatA23,
            0x0c => Self::AdatA45,
            0x0e => Self::AdatA67,
            0x10 => Self::Spdif01,
            _ => Self::Reserved(val),
        }
    }
}

impl From<AssignTarget> for u32 {
    fn from(target: AssignTarget) -> Self {
        match target {
            AssignTarget::Analog01 => 0x00,
            AssignTarget::Analog23 => 0x02,
            AssignTarget::Analog56 => 0x04,
            AssignTarget::Analog78 => 0x06,
            AssignTarget::AdatA01 => 0x08,
            AssignTarget::AdatA23 => 0x0a,
            AssignTarget::AdatA45 => 0x0c,
            AssignTarget::AdatA67 => 0x0e,
            AssignTarget::Spdif01 => 0x10,
            AssignTarget::Reserved(val) => val,
        }
    }
}

/// The trait to represent output protocol for FireStudio.
pub trait PresonusFStudioAssignProtocol<T>  : PresonusFStudioProto<T>
    where T: AsRef<FwNode>,
{
    const MAIN_OFFSET: usize = 0x10f4;
    const HP01_OFFSET: usize = 0x10f8;
    const HP23_OFFSET: usize = 0x10fc;
    const HP45_OFFSET: usize = 0x1100;

    fn read_assign_target(&self, node: &T, offset: usize, timeout_ms: u32) -> Result<AssignTarget, Error> {
        let mut raw = [0;4];
        PresonusFStudioProto::read(self, node, offset, &mut raw, timeout_ms)
            .map(|_| AssignTarget::from(u32::from_be_bytes(raw)))
    }

    fn write_assign_target(&self, node: &T, offset: usize, target: AssignTarget, timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut raw = [0;4];
        target.build_quadlet(&mut raw);
        PresonusFStudioProto::write(self, node, offset, &mut raw, timeout_ms)
    }

    fn read_main_assign_target(&self, node: &T, timeout_ms: u32) -> Result<AssignTarget, Error> {
        self.read_assign_target(node, Self::MAIN_OFFSET, timeout_ms)
    }

    fn read_hp_assign_target(&self, node: &T, hp: usize, timeout_ms: u32) -> Result<AssignTarget, Error> {
        let offset = match hp {
            0 => Self::HP01_OFFSET,
            1 => Self::HP23_OFFSET,
            2 => Self::HP45_OFFSET,
            _ => unreachable!(),
        };
        self.read_assign_target(node, offset, timeout_ms)
    }

    fn write_main_assign_target(&self, node: &T, target: AssignTarget, timeout_ms: u32)
        -> Result<(), Error>
    {
        self.write_assign_target(node, Self::MAIN_OFFSET, target, timeout_ms)
    }

    fn write_hp_assign_target(&self, node: &T, hp: usize, target: AssignTarget, timeout_ms: u32)
        -> Result<(), Error>
    {
        let offset = match hp {
            0 => Self::HP01_OFFSET,
            1 => Self::HP23_OFFSET,
            2 => Self::HP45_OFFSET,
            _ => unreachable!(),
        };
        self.write_assign_target(node, offset, target, timeout_ms)
    }
}

impl<T: AsRef<FwNode>> PresonusFStudioAssignProtocol<T> for FStudioProto {}
