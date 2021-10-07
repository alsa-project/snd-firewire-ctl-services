// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol for hardware mixer function expressed in registers.
//!
//! The module includes structure, enumeration, and trait for hardware mixer function expressed
//! in registers.

use hinawa::{FwReq, FwNode};

use super::*;

const MIXER_COUNT: usize = 4;

const MIXER_OUTPUT_OFFSETS: [usize; MIXER_COUNT] = [0x0c20, 0x0c24, 0x0c28, 0x0c2c];
const   MIXER_OUTPUT_MUTE_FLAG: u32 = 0x00001000;
const   MIXER_OUTPUT_DESTINATION_MASK: u32 = 0x00000f00;
const   MIXER_OUTPUT_VOLUME_MASK: u32 = 0x000000ff;

/// The structure for state of mixer output.
#[derive(Default)]
pub struct RegisterDspMixerOutputState {
    pub volume: [u8; MIXER_COUNT],
    pub mute: [bool; MIXER_COUNT],
    pub destination: [TargetPort; MIXER_COUNT],
}

/// The trait for operations of mixer output.
pub trait RegisterDspMixerOutputOperation {
    const OUTPUT_DESTINATIONS: &'static [TargetPort];

    const MIXER_COUNT: usize = 4;

    const MIXER_OUTPUT_VOLUME_MIN: u8 = 0x00;
    const MIXER_OUTPUT_VOLUME_MAX: u8 = 0x80;
    const MIXER_OUTPUT_VOLUME_STEP: u8 = 0x01;

    fn read_mixer_output_state(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut RegisterDspMixerOutputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        MIXER_OUTPUT_OFFSETS
            .iter()
            .enumerate()
            .try_for_each(|(i, &offset)| {
                read_quad(req, node, offset as u32, timeout_ms).map(|val| {
                    state.volume[i] = (val & MIXER_OUTPUT_VOLUME_MASK) as u8;
                    state.mute[i] = (val & MIXER_OUTPUT_MUTE_FLAG) > 0;

                    let src = ((val & MIXER_OUTPUT_DESTINATION_MASK) >> 8) as usize;
                    state.destination[i] = Self::OUTPUT_DESTINATIONS
                        .iter()
                        .nth(src)
                        .map(|&p| p)
                        .unwrap_or_default();
                })
            })
    }

    fn write_mixer_output_volume(
        req: &mut FwReq,
        node: &mut FwNode,
        volume: &[u8],
        state: &mut RegisterDspMixerOutputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        state.volume
            .iter_mut()
            .zip(volume.iter())
            .zip(MIXER_OUTPUT_OFFSETS.iter())
            .filter(|((old, new), _)| !new.eq(old))
            .try_for_each(|((old, new), &offset)| {
                let mut val = read_quad(req, node, offset as u32, timeout_ms)?;
                val &= !MIXER_OUTPUT_VOLUME_MASK;
                val |= *new as u32;
                write_quad(req, node, offset as u32, val, timeout_ms).map(|_| *old = *new)
            })
    }

    fn write_mixer_output_mute(
        req: &mut FwReq,
        node: &mut FwNode,
        mute: &[bool],
        state: &mut RegisterDspMixerOutputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        state.mute
            .iter_mut()
            .zip(mute.iter())
            .zip(MIXER_OUTPUT_OFFSETS.iter())
            .filter(|((old, new), _)| !new.eq(old))
            .try_for_each(|((old, new), &offset)| {
                let mut val = read_quad(req, node, offset as u32, timeout_ms)?;
                if *new {
                    val |= MIXER_OUTPUT_MUTE_FLAG;
                } else {
                    val &= !MIXER_OUTPUT_MUTE_FLAG;
                }
                write_quad(req, node, offset as u32, val, timeout_ms).map(|_| *old = *new)
            })
    }

    fn write_mixer_output_destination(
        req: &mut FwReq,
        node: &mut FwNode,
        destination: &[TargetPort],
        state: &mut RegisterDspMixerOutputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        state.destination
            .iter_mut()
            .zip(destination.iter())
            .zip(MIXER_OUTPUT_OFFSETS.iter())
            .filter(|((old, new), _)| !new.eq(old))
            .try_for_each(|((old, new), &offset)| {
                let mut val = read_quad(req, node, offset as u32, timeout_ms)?;
                let pos = Self::OUTPUT_DESTINATIONS
                    .iter()
                    .position(|s| new.eq(s))
                    .ok_or_else(|| {
                        let msg = "Invalid source of mixer output";
                        Error::new(FileError::Inval, &msg)
                    })?;
                val &= !MIXER_OUTPUT_DESTINATION_MASK;
                val |= (pos as u32) << 8;
                write_quad(req, node, offset as u32, val, timeout_ms).map(|_| *old = *new)
            })
    }
}
