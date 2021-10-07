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

const MIXER_RETURN_SOURCE_OFFSET: usize = 0x0b2c; // TODO: read-only.
const  MIXER_RETURN_SOURCE_MASK: u32 = 0x000000ff;
const MIXER_RETURN_ENABLE_OFFSET: usize = 0x0c18;

/// The structure for state of mixer return.
#[derive(Default)]
pub struct RegisterDspMixerReturnState {
    pub source: TargetPort,
    pub enable: bool,
}

/// The trait for operation of mixer return.
pub trait RegisterDspMixerReturnOperation {
    const RETURN_SOURCES: &'static [TargetPort];

    fn read_mixer_return_state(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut RegisterDspMixerReturnState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        read_quad(req, node, MIXER_RETURN_ENABLE_OFFSET as u32, timeout_ms).map(|val| {
            state.enable = val > 0;
        })?;

        Ok(())
    }

    fn write_mixer_return_source(
        req: &mut FwReq,
        node: &mut FwNode,
        source: TargetPort,
        state: &mut RegisterDspMixerReturnState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let idx = Self::RETURN_SOURCES
            .iter()
            .position(|s| source.eq(s))
            .ok_or_else(||{
                let msg = format!("Invalid source of mix return");
                Error::new(FileError::Inval, &msg)
            })?;
        let mut val = read_quad(req, node, MIXER_RETURN_SOURCE_OFFSET as u32, timeout_ms)?;
        val &= !MIXER_RETURN_SOURCE_MASK;
        val |= idx as u32;
        write_quad(req, node, MIXER_RETURN_SOURCE_OFFSET as u32, val, timeout_ms).map(|_| {
            state.source = source;
        })
    }

    fn write_mixer_return_enable(
        req: &mut FwReq,
        node: &mut FwNode,
        enable: bool,
        state: &mut RegisterDspMixerReturnState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_quad(req, node, MIXER_RETURN_ENABLE_OFFSET as u32, enable as u32, timeout_ms).map(|_| {
            state.enable = enable;
        })
    }
}

/// The structure for state of sources in mixer entiry.
#[derive(Default, Clone)]
pub struct RegisterDspMixerMonauralSourceEntry {
    pub gain: Vec<u8>,
    pub pan: Vec<u8>,
    pub mute: Vec<bool>,
    pub solo: Vec<bool>,
}

/// The structure for state of mixer sources.
#[derive(Default)]
pub struct RegisterDspMixerMonauralSourceState(pub [RegisterDspMixerMonauralSourceEntry; MIXER_COUNT]);

const MIXER_SOURCE_OFFSETS: [usize; MIXER_COUNT] = [0x4000, 0x4100, 0x4200, 0x4300];
const   MIXER_SOURCE_PAN_CHANGE_FLAG: u32 = 0x80000000;
const   MIXER_SOURCE_GAIN_CHANGE_FLAG: u32 = 0x40000000;
const   MIXER_SOURCE_MUTE_FLAG: u32 = 0x00010000;
const   MIXER_SOURCE_SOLO_FLAG: u32 = 0x00020000;
const   MIXER_SOURCE_PAN_MASK: u32 = 0x0000ff00;
const   MIXER_SOURCE_GAIN_MASK: u32 = 0x000000ff;

/// The trait for operation of mixer sources.
pub trait RegisterDspMixerMonauralSourceOperation {
    const MIXER_SOURCES: &'static [TargetPort];

    const MIXER_COUNT: usize = MIXER_COUNT;

    const SOURCE_GAIN_MIN: u8 = 0x00;
    const SOURCE_GAIN_MAX: u8 = 0x80;
    const SOURCE_GAIN_STEP: u8 = 0x01;

    const SOURCE_PAN_MIN: u8 = 0x00;
    const SOURCE_PAN_MAX: u8 = 0x80;
    const SOURCE_PAN_STEP: u8 = 0x01;

    fn create_mixer_monaural_source_state() -> RegisterDspMixerMonauralSourceState {
        let mut state = RegisterDspMixerMonauralSourceState::default();
        state.0
            .iter_mut()
            .for_each(|entry| {
                entry.gain = vec![Default::default(); Self::MIXER_SOURCES.len()];
                entry.pan = vec![Default::default(); Self::MIXER_SOURCES.len()];
                entry.mute = vec![Default::default(); Self::MIXER_SOURCES.len()];
                entry.solo = vec![Default::default(); Self::MIXER_SOURCES.len()];
            });
        state
    }

    fn read_mixer_monaural_source_state(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut RegisterDspMixerMonauralSourceState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        state.0
            .iter_mut()
            .zip(MIXER_SOURCE_OFFSETS.iter())
            .try_for_each(|(entry, &offset)| {
                (0..Self::MIXER_SOURCES.len())
                    .try_for_each(|i| {
                        read_quad(req, node, (offset + i * 4) as u32, timeout_ms).map(|val| {
                            entry.gain[i] = (val & MIXER_SOURCE_GAIN_MASK) as u8;
                            entry.pan[i] = ((val & MIXER_SOURCE_PAN_MASK) >> 8) as u8;
                            entry.mute[i] = val & MIXER_SOURCE_MUTE_FLAG > 0;
                            entry.solo[i] = val & MIXER_SOURCE_SOLO_FLAG > 0;
                        })
                    })
            })
    }

    fn write_mixer_monaural_source_gain(
        req: &mut FwReq,
        node: &mut FwNode,
        mixer: usize,
        gain: &[u8],
        state: &mut RegisterDspMixerMonauralSourceState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(mixer < MIXER_COUNT);
        assert_eq!(gain.len(), Self::MIXER_SOURCES.len());

        let offset = MIXER_SOURCE_OFFSETS[mixer];

        state.0[mixer].gain.iter_mut()
            .zip(gain.iter())
            .enumerate()
            .filter(|(_, (old, new))| !old.eq(new))
            .try_for_each(|(i, (old, new))| {
                let mut val = read_quad(req, node, (offset + i * 4) as u32, timeout_ms)?;
                val &= !MIXER_SOURCE_GAIN_MASK;
                val |= *new as u32;
                val |= MIXER_SOURCE_GAIN_CHANGE_FLAG;
                write_quad(req, node, (offset + i * 4) as u32, val, timeout_ms).map(|_| *old = *new)
            })
    }

    fn write_mixer_monaural_source_pan(
        req: &mut FwReq,
        node: &mut FwNode,
        mixer: usize,
        pan: &[u8],
        state: &mut RegisterDspMixerMonauralSourceState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(mixer < MIXER_COUNT);
        assert_eq!(pan.len(), Self::MIXER_SOURCES.len());

        let offset = MIXER_SOURCE_OFFSETS[mixer];

        state.0[mixer].pan.iter_mut()
            .zip(pan.iter())
            .enumerate()
            .filter(|(_, (old, new))| !old.eq(new))
            .try_for_each(|(i, (old, new))| {
                let mut val = read_quad(req, node, (offset + i * 4) as u32, timeout_ms)?;
                val &= !MIXER_SOURCE_PAN_MASK;
                val |= (*new as u32) << 8;
                val |= MIXER_SOURCE_PAN_CHANGE_FLAG;
                write_quad(req, node, (offset + i * 4) as u32, val, timeout_ms).map(|_| *old = *new)
            })
    }

    fn write_mixer_monaural_source_mute(
        req: &mut FwReq,
        node: &mut FwNode,
        mixer: usize,
        mute: &[bool],
        state: &mut RegisterDspMixerMonauralSourceState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(mixer < MIXER_COUNT);
        assert_eq!(mute.len(), Self::MIXER_SOURCES.len());

        let offset = MIXER_SOURCE_OFFSETS[mixer];

        state.0[mixer].mute.iter_mut()
            .zip(mute.iter())
            .enumerate()
            .filter(|(_, (old, new))| !old.eq(new))
            .try_for_each(|(i, (old, new))| {
                let mut val = read_quad(req, node, (offset + i * 4) as u32, timeout_ms)?;
                val &= !MIXER_SOURCE_MUTE_FLAG;
                if *new {
                    val |= MIXER_SOURCE_MUTE_FLAG;
                }
                write_quad(req, node, (offset + i * 4) as u32, val, timeout_ms).map(|_| *old = *new)
            })
    }

    fn write_mixer_monaural_source_solo(
        req: &mut FwReq,
        node: &mut FwNode,
        mixer: usize,
        solo: &[bool],
        state: &mut RegisterDspMixerMonauralSourceState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(mixer < MIXER_COUNT);
        assert_eq!(solo.len(), Self::MIXER_SOURCES.len());

        let offset = MIXER_SOURCE_OFFSETS[mixer];

        state.0[mixer].solo.iter_mut()
            .zip(solo.iter())
            .enumerate()
            .filter(|(_, (old, new))| !old.eq(new))
            .try_for_each(|(i, (old, new))| {
                let mut val = read_quad(req, node, (offset + i * 4) as u32, timeout_ms)?;
                val &= !MIXER_SOURCE_SOLO_FLAG;
                if *new {
                    val |= MIXER_SOURCE_SOLO_FLAG;
                }
                write_quad(req, node, (offset + i * 4) as u32, val, timeout_ms).map(|_| *old = *new)
            })
    }
}

const MIXER_STEREO_SOURCE_COUNT: usize = 6;
const MIXER_STEREO_SOURCE_PAIR_COUNT: usize = MIXER_STEREO_SOURCE_COUNT / 2;

/// The structure for state of sources in mixer entiry.
#[derive(Default, Clone)]
pub struct RegisterDspMixerStereoSourceEntry {
    pub gain: [u8; MIXER_STEREO_SOURCE_COUNT],
    pub mute: [bool; MIXER_STEREO_SOURCE_COUNT],
    pub solo: [bool; MIXER_STEREO_SOURCE_COUNT],
}

/// The structure for state of mixer sources.
#[derive(Default)]
pub struct RegisterDspMixerStereoSourceState {
    pub source_paired: [bool; MIXER_STEREO_SOURCE_PAIR_COUNT],
    pub mixer_sources: [RegisterDspMixerStereoSourceEntry; MIXER_COUNT],
}

const MIXER_SOURCE_PAIRED_OFFSET: u32 = 0x0c84;
const  MIXER_SOURCE_PAIRED_FLAG: u8 = 0x00000001;
const  MIXER_SOURCE_PAIRED_CHANGE: u8 = 0x00000080;

// TODO: Audio Express and 4 pre have independent configurations for the below:
//const MIXER_SOURCE_STEREO_WIDTH_FLAG: u32 = 0x00400000;
//const MIXER_SOURCE_STEREO_BALANCE_FLAG: u32 = 0x00800000;

/// The trait for operation of mixer sources.
pub trait RegisterDspMixerStereoSourceOperation {
    const MIXER_SOURCES: [TargetPort; MIXER_STEREO_SOURCE_COUNT] = [
        TargetPort::Analog0,
        TargetPort::Analog1,
        TargetPort::Analog2,
        TargetPort::Analog3,
        TargetPort::Spdif0,
        TargetPort::Spdif1,
    ];
    const MIXER_SOURCE_PAIR_COUNT: usize = Self::MIXER_SOURCES.len() / 2;

    const MIXER_COUNT: usize = MIXER_COUNT;

    const SOURCE_GAIN_MIN: u8 = 0x00;
    const SOURCE_GAIN_MAX: u8 = 0x80;
    const SOURCE_GAIN_STEP: u8 = 0x01;

    const SOURCE_PAN_MIN: u8 = 0x00;
    const SOURCE_PAN_MAX: u8 = 0x80;
    const SOURCE_PAN_STEP: u8 = 0x01;

    fn create_mixer_stereo_source_state() -> RegisterDspMixerStereoSourceState {
        Default::default()
    }

    fn compute_mixer_source_offset(base_offset: usize, src_idx: usize) -> usize {
        base_offset + 4 * if src_idx < 4 { src_idx } else { src_idx + 4 }
    }

    fn read_mixer_stereo_source_state(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut RegisterDspMixerStereoSourceState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        read_quad(req, node, MIXER_SOURCE_PAIRED_OFFSET as u32, timeout_ms).map(|val| {
            state.source_paired
                .iter_mut()
                .enumerate()
                .for_each(|(i, paired)| {
                    let flags = ((val >> (i * 8)) & 0x000000ff) as u8;
                    *paired = (flags & MIXER_SOURCE_PAIRED_FLAG) > 0;
                });
        })?;

        state.mixer_sources
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, entry)| {
                let base_offset = MIXER_SOURCE_OFFSETS[i];
                (0..Self::MIXER_SOURCES.len())
                    .try_for_each(|j| {
                        let offset = Self::compute_mixer_source_offset(base_offset, j);
                        read_quad(req, node, offset as u32, timeout_ms).map(|val| {
                            entry.gain[j] = (val & MIXER_SOURCE_GAIN_MASK) as u8;
                            entry.mute[j] = ((val & MIXER_SOURCE_MUTE_FLAG) >> 8) > 0;
                            entry.solo[j] = ((val & MIXER_SOURCE_SOLO_FLAG) >> 16) > 0;
                        })
                    })
            })?;

        Ok(())
    }

    fn write_mixer_stereo_source_paired(
        req: &mut FwReq,
        node: &mut FwNode,
        paired: &[bool],
        state: &mut RegisterDspMixerStereoSourceState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(paired.len(), MIXER_STEREO_SOURCE_PAIR_COUNT);

        let mut val = 0u32;

        state.source_paired
            .iter_mut()
            .zip(paired.iter())
            .enumerate()
            .filter(|(_, (old, new))| !old.eq(new))
            .for_each(|(i, (_, new))| {
                let mut v = MIXER_SOURCE_PAIRED_CHANGE;
                if *new {
                    v |= MIXER_SOURCE_PAIRED_FLAG;
                }
                val |= (v as u32) << (i * 8);
            });

        write_quad(req, node, MIXER_SOURCE_PAIRED_OFFSET, val, timeout_ms).map(|_| {
            state.source_paired.copy_from_slice(paired);
        })
    }

    fn write_mixer_stereo_source_gain(
        req: &mut FwReq,
        node: &mut FwNode,
        mixer: usize,
        gain: &[u8],
        state: &mut RegisterDspMixerStereoSourceState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(mixer < MIXER_COUNT);
        assert_eq!(gain.len(), Self::MIXER_SOURCES.len());

        let base_offset = MIXER_SOURCE_OFFSETS[mixer];

        state.mixer_sources[mixer].gain
            .iter_mut()
            .zip(gain.iter())
            .enumerate()
            .filter(|(_, (old, new))| !old.eq(new))
            .try_for_each(|(i, (old, new))| {
                let offset = Self::compute_mixer_source_offset(base_offset, i);
                let mut val = read_quad(req, node, offset as u32, timeout_ms)?;
                val &= !MIXER_SOURCE_GAIN_MASK;
                val |= *new as u32;
                write_quad(req, node, offset as u32, val, timeout_ms).map(|_| *old = *new)
            })
    }

    fn write_mixer_stereo_source_mute(
        req: &mut FwReq,
        node: &mut FwNode,
        mixer: usize,
        mute: &[bool],
        state: &mut RegisterDspMixerStereoSourceState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(mixer < MIXER_COUNT);
        assert_eq!(mute.len(), Self::MIXER_SOURCES.len());

        let base_offset = MIXER_SOURCE_OFFSETS[mixer];

        state.mixer_sources[mixer].mute
            .iter_mut()
            .zip(mute.iter())
            .enumerate()
            .filter(|(_, (old, new))| !old.eq(new))
            .try_for_each(|(i, (old, new))| {
                let offset = Self::compute_mixer_source_offset(base_offset, i);
                let mut val = read_quad(req, node, offset as u32, timeout_ms)?;
                val &= !MIXER_SOURCE_MUTE_FLAG;
                if *new {
                    val |= MIXER_SOURCE_MUTE_FLAG;
                }
                write_quad(req, node, offset as u32, val, timeout_ms).map(|_| *old = *new)
            })
    }

    fn write_mixer_stereo_source_solo(
        req: &mut FwReq,
        node: &mut FwNode,
        mixer: usize,
        solo: &[bool],
        state: &mut RegisterDspMixerStereoSourceState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(mixer < MIXER_COUNT);
        assert_eq!(solo.len(), Self::MIXER_SOURCES.len());

        let base_offset = MIXER_SOURCE_OFFSETS[mixer];

        state.mixer_sources[mixer].solo
            .iter_mut()
            .zip(solo.iter())
            .enumerate()
            .filter(|(_, (old, new))| !old.eq(new))
            .try_for_each(|(i, (old, new))| {
                let offset = Self::compute_mixer_source_offset(base_offset, i);
                let mut val = read_quad(req, node, offset as u32, timeout_ms)?;
                val &= !MIXER_SOURCE_SOLO_FLAG;
                if *new {
                    val |= MIXER_SOURCE_SOLO_FLAG;
                }
                write_quad(req, node, offset as u32, val, timeout_ms).map(|_| *old = *new)
            })
    }
}
