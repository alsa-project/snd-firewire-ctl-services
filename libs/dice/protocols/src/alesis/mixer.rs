// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Mixer protocol specific to Alesis iO FireWire series.
//!
//! The module includes structure, enumeration, and trait and its implementation for mixer
//! protocol defined by Alesis for iO FireWire series.

use glib::*;
use hinawa::FwNode;

use super::*;
use crate::*;

/// The maximum number of analog inputs for mixer.
pub const MAX_ANALOG_INPUT_COUNT: usize = 8;

/// The number of digital A inputs for mixer.
pub const DIGITAL_A_INPUT_COUNT: usize = 8;

/// The maximum number of digital B inputs for mixer.
pub const MAX_DIGITAL_B_INPUT_COUNT: usize = 8;

/// The number of stream inputs for mixer.
pub const STREAM_INPUT_COUNT: usize = 8;

/// The structure to represent parameters of mixer. 0x00000000..0x007fffff (-60.0..0.0 dB).
#[derive(Debug, Clone)]
pub struct IoMixerGain{
    pub analog_inputs: Vec<i32>,
    pub stream_inputs: [i32;STREAM_INPUT_COUNT],
    pub digital_a_inputs: [i32;DIGITAL_A_INPUT_COUNT],
    pub digital_b_inputs: Vec<i32>,
}

impl IoMixerGain {
    fn build(&self, raw: &mut [u8]) {
        assert_eq!(raw.len(),
                   4 * (MAX_ANALOG_INPUT_COUNT + STREAM_INPUT_COUNT + DIGITAL_A_INPUT_COUNT + MAX_DIGITAL_B_INPUT_COUNT),
                   "Programming error...");
        let analog_input_count = self.analog_inputs.len();
        assert!(analog_input_count <= MAX_ANALOG_INPUT_COUNT);
        let digital_b_input_count = self.digital_b_inputs.len();
        assert!(digital_b_input_count <= MAX_DIGITAL_B_INPUT_COUNT);

        let mut pos = 0;
        self.analog_inputs.build_quadlet_block(&mut raw[pos..(pos + 4 * analog_input_count)]);

        pos += 4 * MAX_ANALOG_INPUT_COUNT;
        self.stream_inputs.build_quadlet_block(&mut raw[pos..(pos + 4 * STREAM_INPUT_COUNT)]);

        pos += 4 * STREAM_INPUT_COUNT;
        self.digital_a_inputs.build_quadlet_block(&mut raw[pos..(pos + 4 * DIGITAL_A_INPUT_COUNT)]);

        pos += 4 * DIGITAL_A_INPUT_COUNT;
        pos += 4 * (MAX_DIGITAL_B_INPUT_COUNT - digital_b_input_count);
        self.digital_b_inputs.build_quadlet_block(&mut raw[pos..(pos + 4 * digital_b_input_count)]);
    }

    fn parse(&mut self, raw: &[u8]) {
        assert_eq!(raw.len(),
                   4 * (MAX_ANALOG_INPUT_COUNT + STREAM_INPUT_COUNT + DIGITAL_A_INPUT_COUNT + MAX_DIGITAL_B_INPUT_COUNT),
                   "Programming error...");
        let analog_input_count = self.analog_inputs.len();
        assert!(analog_input_count <= MAX_ANALOG_INPUT_COUNT);
        let digital_b_input_count = self.digital_b_inputs.len();
        assert!(digital_b_input_count <= MAX_DIGITAL_B_INPUT_COUNT);

        let mut pos = 0;
        self.analog_inputs.parse_quadlet_block(&raw[pos..(pos + 4 * analog_input_count)]);

        pos += 4 * MAX_ANALOG_INPUT_COUNT;
        self.stream_inputs.parse_quadlet_block(&raw[pos..(pos + 4 * STREAM_INPUT_COUNT)]);

        pos += 4 * STREAM_INPUT_COUNT;
        self.digital_a_inputs.parse_quadlet_block(&raw[pos..(pos + 4 * DIGITAL_A_INPUT_COUNT)]);

        pos += 4 * DIGITAL_A_INPUT_COUNT;
        pos += 4 * (MAX_DIGITAL_B_INPUT_COUNT - digital_b_input_count);
        self.digital_b_inputs.parse_quadlet_block(&raw[pos..(pos + 4 * digital_b_input_count)]);
    }
}

/// The number of mixer.
pub const IO_MIXER_COUNT: usize = 8;

/// The structure to represent mute state of mixer input pairs.
#[derive(Debug, Clone)]
pub struct IoMixerMute{
    pub analog_inputs: Vec<bool>,
    pub digital_a_inputs: [bool;DIGITAL_A_INPUT_COUNT],
    pub digital_b_inputs: Vec<bool>,
}

impl IoMixerMute {
    fn build(&self, raw: &mut [u8]) {
        assert_eq!(raw.len(), 4, "Programming error...");
        assert!(self.analog_inputs.len() <= MAX_ANALOG_INPUT_COUNT, "Programming error...");
        assert!(self.digital_b_inputs.len() <= MAX_DIGITAL_B_INPUT_COUNT, "Programming error...");

        let mut val = 0u32;

        let mut shift = 0;
        self.analog_inputs.iter()
            .take(MAX_ANALOG_INPUT_COUNT)
            .enumerate()
            .filter(|(_, &v)| v)
            .for_each(|(i, _)| val |= 1 << (shift + i));

        shift += MAX_ANALOG_INPUT_COUNT;
        self.digital_a_inputs.iter()
            .enumerate()
            .filter(|(_, &v)| v)
            .for_each(|(i, _)| val |= 1 << (shift + i));

        shift += DIGITAL_A_INPUT_COUNT;
        shift += MAX_DIGITAL_B_INPUT_COUNT - self.digital_b_inputs.len();
        self.digital_b_inputs.iter()
            .take(MAX_DIGITAL_B_INPUT_COUNT)
            .enumerate()
            .filter(|(_, &v)| v)
            .for_each(|(i, _)| val |= 1 << (shift + i));

        val.build_quadlet(raw);
    }

    fn parse(&mut self, raw: &[u8]) {
        assert_eq!(raw.len(), 4, "Programming error...");
        assert!(self.analog_inputs.len() <= MAX_ANALOG_INPUT_COUNT, "Programming error...");
        assert!(self.digital_b_inputs.len() <= MAX_DIGITAL_B_INPUT_COUNT, "Programming error...");

        let mut val = 0u32;
        val.parse_quadlet(raw);

        let mut shift = 0;
        self.analog_inputs.iter_mut()
            .take(MAX_ANALOG_INPUT_COUNT)
            .enumerate()
            .for_each(|(i, v)| *v = val & (1 << (shift + i)) > 0);

        shift += MAX_ANALOG_INPUT_COUNT;
        self.digital_a_inputs.iter_mut()
            .enumerate()
            .for_each(|(i, v)| *v = val & (1 << (shift + i)) > 0);

        shift += DIGITAL_A_INPUT_COUNT;
        shift += MAX_DIGITAL_B_INPUT_COUNT - self.digital_b_inputs.len();
        self.digital_b_inputs.iter_mut()
            .take(MAX_DIGITAL_B_INPUT_COUNT)
            .enumerate()
            .for_each(|(i, v)| *v = val & (1 << (shift + i)) > 0);
    }
}

/// The structure to represent state of knobs.
#[derive(Default, Debug, Copy, Clone)]
pub struct IoKnobState{
    /// The ratio to mix monitored inputs and stream inputs. 0x0000..0x0100.
    pub mix_blend: u32,
    /// The volume of main level. 0x0000..0x0100.
    pub main_level: u32,
}

impl IoKnobState {
    const SIZE: usize = 8;

    fn parse(&mut self, raw: &[u8]) {
        assert_eq!(raw.len(), Self::SIZE);

        self.mix_blend.parse_quadlet(&raw[..4]);
        self.main_level.parse_quadlet(&raw[4..8]);
    }
}

/// The structure to represent state of mixer.
#[derive(Debug)]
pub struct IoMixerState{
    pub gains: Vec<IoMixerGain>,
    pub mutes: Vec<IoMixerMute>,
    pub out_vols: [i32;IO_MIXER_COUNT],
    pub out_mutes: [bool;IO_MIXER_COUNT],
    pub knobs: IoKnobState,
}

pub trait IoMixerSpec {
    const ANALOG_INPUT_COUNT: usize;
    const DIGITAL_B_INPUT_COUNT: usize;

    fn create_mixer_state() -> IoMixerState {
        IoMixerState{
            gains: vec![IoMixerGain{
                analog_inputs: vec![0;Self::ANALOG_INPUT_COUNT],
                stream_inputs: [0;STREAM_INPUT_COUNT],
                digital_a_inputs: [0;DIGITAL_A_INPUT_COUNT],
                digital_b_inputs: vec![0;Self::DIGITAL_B_INPUT_COUNT],
            };IO_MIXER_COUNT],
            mutes: vec![IoMixerMute{
                analog_inputs: vec![false;Self::ANALOG_INPUT_COUNT],
                digital_a_inputs: [false;DIGITAL_A_INPUT_COUNT],
                digital_b_inputs: vec![false;Self::DIGITAL_B_INPUT_COUNT],
            };IO_MIXER_COUNT / 2],
            out_vols: [0;IO_MIXER_COUNT],
            out_mutes: [false;IO_MIXER_COUNT],
            knobs: Default::default(),
        }
    }
}

/// The structure to represent mixer of iO 14.
#[derive(Debug)]
pub struct Io14MixerState(IoMixerState);

impl IoMixerSpec for Io14MixerState {
    const ANALOG_INPUT_COUNT: usize = 4;
    const DIGITAL_B_INPUT_COUNT: usize = 2;
}

impl AsRef<IoMixerState> for Io14MixerState {
    fn as_ref(&self) -> &IoMixerState {
        &self.0
    }
}

impl AsMut<IoMixerState> for Io14MixerState {
    fn as_mut(&mut self) -> &mut IoMixerState {
        &mut self.0
    }
}

impl Default for Io14MixerState {
    fn default() -> Self {
        Io14MixerState(Self::create_mixer_state())
    }
}

/// The structure to represent mixer of iO 26.
#[derive(Debug)]
pub struct Io26MixerState(IoMixerState);

impl IoMixerSpec for Io26MixerState {
    const ANALOG_INPUT_COUNT: usize = 8;
    const DIGITAL_B_INPUT_COUNT: usize = 8;
}

impl AsRef<IoMixerState> for Io26MixerState {
    fn as_ref(&self) -> &IoMixerState {
        &self.0
    }
}

impl AsMut<IoMixerState> for Io26MixerState {
    fn as_mut(&mut self) -> &mut IoMixerState {
        &mut self.0
    }
}

impl Default for Io26MixerState {
    fn default() -> Self {
        Io26MixerState(Self::create_mixer_state())
    }
}

pub trait IoMixerProtocol<T, U> : AlesisIoProtocol<T>
    where T: AsRef<FwNode>,
          U: AsMut<IoMixerState> + AsRef<IoMixerState>,
{
    const MONITOR_SRC_GAIN_OFFSET: usize = 0x0038;
    const MONITOR_OUT_VOL_OFFSET: usize = 0x0438;
    const MONITOR_SRC_MUTE_OFFSET: usize = 0x0458;
    //const MIXER_OUT_SELECT_OFFSET: usize = 0x0460;
    // NOTE: This has no actual side-effect except for assist to software.
    //const MONITOR_SRC_LINK_OFFSET: usize = 0x047c;
    const MONITOR_OUT_MUTE_OFFSET: usize = 0x0468;
    const MIXER_SELECT_OFFSET: usize = 0x0560;
    const KNOB_STATE_OFFSET: usize = 0x0574;

    const MAX_MIXER_SRC_COUNT: usize = MAX_ANALOG_INPUT_COUNT + DIGITAL_A_INPUT_COUNT +
                                       MAX_DIGITAL_B_INPUT_COUNT + STREAM_INPUT_COUNT;

    fn read_mixer_src_gains(&self, node: &T, state: &mut U, timeout_ms: u32) -> Result<(), Error> {
        state.as_mut().gains.iter_mut()
            .enumerate()
            .try_for_each(|(i, m)| {
                let offset = i * 4 * Self::MAX_MIXER_SRC_COUNT;
                let mut raw = vec![0;4 * Self::MAX_MIXER_SRC_COUNT];
                self.read_block(node, Self::MONITOR_SRC_GAIN_OFFSET + offset, &mut raw, timeout_ms)
                    .map(|_| m.parse(&raw))
            })
    }

    fn write_mixer_src_gains(&self, node: &T, state: &mut U, mixer: usize, gains: &IoMixerGain, timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut raw = [0;4];
        ((mixer / 2) as u32).build_quadlet(&mut raw);
        self.write_block(node, Self::MIXER_SELECT_OFFSET, &mut raw, timeout_ms)?;

        let mut new = vec![0;4 * Self::MAX_MIXER_SRC_COUNT];
        gains.build(&mut new);
        let mut old = vec![0;4 * Self::MAX_MIXER_SRC_COUNT];
        state.as_ref().gains[mixer].build(&mut old);

        let offset = 4 * mixer * Self::MAX_MIXER_SRC_COUNT;

        (0..Self::MAX_MIXER_SRC_COUNT)
            .try_for_each(|i| {
                let pos = 4 * i;
                if &new[pos..(pos + 4)] != &old[pos..(pos + 4)] {
                    self.write_block(node, Self::MONITOR_SRC_GAIN_OFFSET + offset + pos, &mut new[pos..(pos + 4)],
                                     timeout_ms)
                } else {
                    Ok(())
                }
            })
            .map(|_| state.as_mut().gains[mixer].parse(&new))
    }

    fn read_mixer_src_mutes(&self, node: &T, state: &mut U, timeout_ms: u32) -> Result<(), Error> {
        state.as_mut().mutes.iter_mut()
            .enumerate()
            .try_for_each(|(i, m)| {
                let mut raw = [0;4];
                let offset = Self::MONITOR_SRC_MUTE_OFFSET + i * 4;
                self.read_block(node, offset, &mut raw, timeout_ms)
                    .map(|_| m.parse(&raw))
            })
    }

    fn write_mixer_src_mutes(&self, node: &T, state: &mut U, mixer_pair: usize, mutes: &IoMixerMute,
                             timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut raw = [0;4];
        (mixer_pair as u32).build_quadlet(&mut raw);
        self.write_block(node, Self::MIXER_SELECT_OFFSET, &mut raw, timeout_ms)?;

        mutes.build(&mut raw);
        self.write_block(node, Self::MONITOR_SRC_MUTE_OFFSET + 4 * mixer_pair, &mut raw, timeout_ms)
            .map(|_| state.as_mut().mutes[mixer_pair].parse(&raw))
    }

    fn read_mixer_out_vols(&self, node: &T, state: &mut U, timeout_ms: u32) -> Result<(), Error> {
        let mut raw = [0;4 * IO_MIXER_COUNT];
        self.read_block(node, Self::MONITOR_OUT_VOL_OFFSET, &mut raw, timeout_ms)
            .map(|_| state.as_mut().out_vols.parse_quadlet_block(&raw))
    }

    fn write_mixer_out_vols(&self, node: &T, state: &mut U, vols: &[i32], timeout_ms: u32)
        -> Result<(), Error>
    {
        assert_eq!(state.as_ref().out_vols.len(), vols.len());

        vols.iter()
            .zip(state.as_mut().out_vols.iter_mut())
            .enumerate()
            .filter(|(_, (n, o))| !n.eq(o))
            .try_for_each(|(i, (n, o))| {
                let mut raw = [0;4];
                n.build_quadlet(&mut raw);
                let offset = Self::MONITOR_OUT_VOL_OFFSET + 4 * i;
                self.write_block(node, offset, &mut raw, timeout_ms)
                    .map(|_| *o = *n)
            })
    }

    fn read_mixer_out_mutes(&self, node: &T, state: &mut U, timeout_ms: u32) -> Result<(), Error> {
        self.read_flags(node, Self::MONITOR_OUT_MUTE_OFFSET, &mut state.as_mut().out_mutes, timeout_ms)
    }

    fn write_mixer_out_mutes(&self, node: &T, state: &mut U, mutes: &[bool], timeout_ms: u32)
        -> Result<(), Error>
    {
        assert_eq!(state.as_ref().out_mutes.len(), mutes.len());

        self.write_flags(node, Self::MONITOR_OUT_MUTE_OFFSET, mutes, timeout_ms)
            .map(|_| state.as_mut().out_mutes.copy_from_slice(mutes))
    }

    fn read_knob_state(&self, node: &T, state: &mut U, timeout_ms: u32) -> Result<(), Error> {
        let mut raw = [0;8];
        self.read_block(node, Self::KNOB_STATE_OFFSET, &mut raw, timeout_ms)
            .map(|_| state.as_mut().knobs.parse(&raw))
    }
}

impl<O, T, U> IoMixerProtocol<T, U> for O
    where O: AlesisIoProtocol<T>,
          T: AsRef<FwNode>,
          U: AsMut<IoMixerState> + AsRef<IoMixerState>,
{}
