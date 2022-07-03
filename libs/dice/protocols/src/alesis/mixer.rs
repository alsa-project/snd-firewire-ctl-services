// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Mixer protocol specific to Alesis iO FireWire series.
//!
//! The module includes structure, enumeration, and trait and its implementation for mixer
//! protocol defined by Alesis for iO FireWire series.

use super::*;

const MONITOR_SRC_GAIN_OFFSET: usize = 0x0038;
const MONITOR_OUT_VOL_OFFSET: usize = 0x0438;
const MONITOR_SRC_MUTE_OFFSET: usize = 0x0458;
//const MIXER_OUT_SELECT_OFFSET: usize = 0x0460;
// NOTE: This has no actual side-effect except for assist to software.
//const MONITOR_SRC_LINK_OFFSET: usize = 0x047c;
const MONITOR_OUT_MUTE_OFFSET: usize = 0x0468;
const MIXER_SELECT_OFFSET: usize = 0x0560;
const KNOB_STATE_OFFSET: usize = 0x0574;

const MAX_ANALOG_INPUT_COUNT: usize = 8;

const DIGITAL_A_INPUT_COUNT: usize = 8;

const MAX_DIGITAL_B_INPUT_COUNT: usize = 8;

const STREAM_INPUT_COUNT: usize = 8;

const MIXER_COUNT: usize = 8;
const MIXER_PAIR_COUNT: usize = 8 / 4;

/// The structure to represent parameters of mixer. 0x00000000..0x007fffff (-60.0..0.0 dB).
#[derive(Debug, Clone)]
pub struct IofwMixerGain {
    pub analog_inputs: Vec<i32>,
    pub stream_inputs: [i32; STREAM_INPUT_COUNT],
    pub digital_a_inputs: [i32; DIGITAL_A_INPUT_COUNT],
    pub digital_b_inputs: Vec<i32>,
}

impl IofwMixerGain {
    fn build(&self, raw: &mut [u8]) {
        assert_eq!(
            raw.len(),
            4 * (MAX_ANALOG_INPUT_COUNT
                + STREAM_INPUT_COUNT
                + DIGITAL_A_INPUT_COUNT
                + MAX_DIGITAL_B_INPUT_COUNT),
            "Programming error..."
        );
        let analog_input_count = self.analog_inputs.len();
        assert!(analog_input_count <= MAX_ANALOG_INPUT_COUNT);
        let digital_b_input_count = self.digital_b_inputs.len();
        assert!(digital_b_input_count <= MAX_DIGITAL_B_INPUT_COUNT);

        let mut pos = 0;
        self.analog_inputs
            .build_quadlet_block(&mut raw[pos..(pos + 4 * analog_input_count)]);

        pos += 4 * MAX_ANALOG_INPUT_COUNT;
        self.stream_inputs
            .build_quadlet_block(&mut raw[pos..(pos + 4 * STREAM_INPUT_COUNT)]);

        pos += 4 * STREAM_INPUT_COUNT;
        self.digital_a_inputs
            .build_quadlet_block(&mut raw[pos..(pos + 4 * DIGITAL_A_INPUT_COUNT)]);

        pos += 4 * DIGITAL_A_INPUT_COUNT;
        pos += 4 * (MAX_DIGITAL_B_INPUT_COUNT - digital_b_input_count);
        self.digital_b_inputs
            .build_quadlet_block(&mut raw[pos..(pos + 4 * digital_b_input_count)]);
    }

    fn parse(&mut self, raw: &[u8]) {
        assert_eq!(
            raw.len(),
            4 * (MAX_ANALOG_INPUT_COUNT
                + STREAM_INPUT_COUNT
                + DIGITAL_A_INPUT_COUNT
                + MAX_DIGITAL_B_INPUT_COUNT),
            "Programming error..."
        );
        let analog_input_count = self.analog_inputs.len();
        assert!(analog_input_count <= MAX_ANALOG_INPUT_COUNT);
        let digital_b_input_count = self.digital_b_inputs.len();
        assert!(digital_b_input_count <= MAX_DIGITAL_B_INPUT_COUNT);

        let mut pos = 0;
        self.analog_inputs
            .parse_quadlet_block(&raw[pos..(pos + 4 * analog_input_count)]);

        pos += 4 * MAX_ANALOG_INPUT_COUNT;
        self.stream_inputs
            .parse_quadlet_block(&raw[pos..(pos + 4 * STREAM_INPUT_COUNT)]);

        pos += 4 * STREAM_INPUT_COUNT;
        self.digital_a_inputs
            .parse_quadlet_block(&raw[pos..(pos + 4 * DIGITAL_A_INPUT_COUNT)]);

        pos += 4 * DIGITAL_A_INPUT_COUNT;
        pos += 4 * (MAX_DIGITAL_B_INPUT_COUNT - digital_b_input_count);
        self.digital_b_inputs
            .parse_quadlet_block(&raw[pos..(pos + 4 * digital_b_input_count)]);
    }
}

/// The structure to represent mute state of mixer input pairs.
#[derive(Debug, Clone)]
pub struct IofwMixerMute {
    pub analog_inputs: Vec<bool>,
    pub digital_a_inputs: [bool; DIGITAL_A_INPUT_COUNT],
    pub digital_b_inputs: Vec<bool>,
}

impl IofwMixerMute {
    fn build(&self, raw: &mut [u8]) {
        assert_eq!(raw.len(), 4, "Programming error...");
        assert!(
            self.analog_inputs.len() <= MAX_ANALOG_INPUT_COUNT,
            "Programming error..."
        );
        assert!(
            self.digital_b_inputs.len() <= MAX_DIGITAL_B_INPUT_COUNT,
            "Programming error..."
        );

        let mut val = 0u32;

        let mut shift = 0;
        self.analog_inputs
            .iter()
            .take(MAX_ANALOG_INPUT_COUNT)
            .enumerate()
            .filter(|(_, &v)| v)
            .for_each(|(i, _)| val |= 1 << (shift + i));

        shift += MAX_ANALOG_INPUT_COUNT;
        self.digital_a_inputs
            .iter()
            .enumerate()
            .filter(|(_, &v)| v)
            .for_each(|(i, _)| val |= 1 << (shift + i));

        shift += DIGITAL_A_INPUT_COUNT;
        shift += MAX_DIGITAL_B_INPUT_COUNT - self.digital_b_inputs.len();
        self.digital_b_inputs
            .iter()
            .take(MAX_DIGITAL_B_INPUT_COUNT)
            .enumerate()
            .filter(|(_, &v)| v)
            .for_each(|(i, _)| val |= 1 << (shift + i));

        val.build_quadlet(raw);
    }

    fn parse(&mut self, raw: &[u8]) {
        assert_eq!(raw.len(), 4, "Programming error...");
        assert!(
            self.analog_inputs.len() <= MAX_ANALOG_INPUT_COUNT,
            "Programming error..."
        );
        assert!(
            self.digital_b_inputs.len() <= MAX_DIGITAL_B_INPUT_COUNT,
            "Programming error..."
        );

        let mut val = 0u32;
        val.parse_quadlet(raw);

        let mut shift = 0;
        self.analog_inputs
            .iter_mut()
            .take(MAX_ANALOG_INPUT_COUNT)
            .enumerate()
            .for_each(|(i, v)| *v = val & (1 << (shift + i)) > 0);

        shift += MAX_ANALOG_INPUT_COUNT;
        self.digital_a_inputs
            .iter_mut()
            .enumerate()
            .for_each(|(i, v)| *v = val & (1 << (shift + i)) > 0);

        shift += DIGITAL_A_INPUT_COUNT;
        shift += MAX_DIGITAL_B_INPUT_COUNT - self.digital_b_inputs.len();
        self.digital_b_inputs
            .iter_mut()
            .take(MAX_DIGITAL_B_INPUT_COUNT)
            .enumerate()
            .for_each(|(i, v)| *v = val & (1 << (shift + i)) > 0);
    }
}

/// The structure to represent state of knobs.
#[derive(Default, Debug, Copy, Clone)]
pub struct IoKnobState {
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
#[derive(Default, Debug)]
pub struct IofwMixerState {
    pub gains: Vec<IofwMixerGain>,
    pub mutes: Vec<IofwMixerMute>,
    pub out_vols: [i32; MIXER_COUNT],
    pub out_mutes: [bool; MIXER_COUNT],
    pub knobs: IoKnobState,
}

pub trait IofwMixerOperation {
    const ANALOG_INPUT_COUNT: usize;
    const DIGITAL_B_INPUT_COUNT: usize;

    const DIGITAL_A_INPUT_COUNT: usize = DIGITAL_A_INPUT_COUNT;
    const STREAM_INPUT_COUNT: usize = STREAM_INPUT_COUNT;

    const MIXER_COUNT: usize = MIXER_COUNT;
    const MIXER_PAIR_COUNT: usize = MIXER_PAIR_COUNT;

    const MAX_MIXER_SRC_COUNT: usize = MAX_ANALOG_INPUT_COUNT
        + DIGITAL_A_INPUT_COUNT
        + MAX_DIGITAL_B_INPUT_COUNT
        + STREAM_INPUT_COUNT;

    fn create_mixer_state() -> IofwMixerState {
        IofwMixerState {
            gains: vec![
                IofwMixerGain {
                    analog_inputs: vec![0; Self::ANALOG_INPUT_COUNT],
                    stream_inputs: [0; STREAM_INPUT_COUNT],
                    digital_a_inputs: [0; DIGITAL_A_INPUT_COUNT],
                    digital_b_inputs: vec![0; Self::DIGITAL_B_INPUT_COUNT],
                };
                MIXER_COUNT
            ],
            mutes: vec![
                IofwMixerMute {
                    analog_inputs: vec![false; Self::ANALOG_INPUT_COUNT],
                    digital_a_inputs: [false; DIGITAL_A_INPUT_COUNT],
                    digital_b_inputs: vec![false; Self::DIGITAL_B_INPUT_COUNT],
                };
                MIXER_PAIR_COUNT
            ],
            out_vols: [0; MIXER_COUNT],
            out_mutes: [false; MIXER_COUNT],
            knobs: Default::default(),
        }
    }

    fn read_mixer_src_gains(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut IofwMixerState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        state.gains.iter_mut().enumerate().try_for_each(|(i, m)| {
            let offset = i * 4 * Self::MAX_MIXER_SRC_COUNT;
            let mut raw = vec![0; 4 * Self::MAX_MIXER_SRC_COUNT];
            alesis_read_block(
                req,
                node,
                MONITOR_SRC_GAIN_OFFSET + offset,
                &mut raw,
                timeout_ms,
            )
            .map(|_| m.parse(&raw))
        })
    }

    fn write_mixer_src_gains(
        req: &mut FwReq,
        node: &mut FwNode,
        mixer: usize,
        gains: &IofwMixerGain,
        state: &mut IofwMixerState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        ((mixer / 2) as u32).build_quadlet(&mut raw);
        alesis_write_block(req, node, MIXER_SELECT_OFFSET, &mut raw, timeout_ms)?;

        let mut new = vec![0; 4 * Self::MAX_MIXER_SRC_COUNT];
        gains.build(&mut new);
        let mut old = vec![0; 4 * Self::MAX_MIXER_SRC_COUNT];
        state.gains[mixer].build(&mut old);

        let offset = 4 * mixer * Self::MAX_MIXER_SRC_COUNT;

        (0..Self::MAX_MIXER_SRC_COUNT)
            .try_for_each(|i| {
                let pos = 4 * i;
                if &new[pos..(pos + 4)] != &old[pos..(pos + 4)] {
                    alesis_write_block(
                        req,
                        node,
                        MONITOR_SRC_GAIN_OFFSET + offset + pos,
                        &mut new[pos..(pos + 4)],
                        timeout_ms,
                    )
                } else {
                    Ok(())
                }
            })
            .map(|_| state.gains[mixer].parse(&new))
    }

    fn read_mixer_src_mutes(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut IofwMixerState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        state.mutes.iter_mut().enumerate().try_for_each(|(i, m)| {
            let mut raw = [0; 4];
            let offset = MONITOR_SRC_MUTE_OFFSET + i * 4;
            alesis_read_block(req, node, offset, &mut raw, timeout_ms).map(|_| m.parse(&raw))
        })
    }

    fn write_mixer_src_mutes(
        req: &mut FwReq,
        node: &mut FwNode,
        mixer_pair: usize,
        mutes: &IofwMixerMute,
        state: &mut IofwMixerState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        (mixer_pair as u32).build_quadlet(&mut raw);
        alesis_write_block(req, node, MIXER_SELECT_OFFSET, &mut raw, timeout_ms)?;

        mutes.build(&mut raw);
        alesis_write_block(
            req,
            node,
            MONITOR_SRC_MUTE_OFFSET + 4 * mixer_pair,
            &mut raw,
            timeout_ms,
        )
        .map(|_| state.mutes[mixer_pair].parse(&raw))
    }

    fn read_mixer_out_vols(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut IofwMixerState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4 * MIXER_COUNT];
        alesis_read_block(req, node, MONITOR_OUT_VOL_OFFSET, &mut raw, timeout_ms)
            .map(|_| state.out_vols.parse_quadlet_block(&raw))
    }

    fn write_mixer_out_vols(
        req: &mut FwReq,
        node: &mut FwNode,
        vols: &[i32],
        state: &mut IofwMixerState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(state.out_vols.len(), vols.len());

        vols.iter()
            .zip(&mut state.out_vols)
            .enumerate()
            .filter(|(_, (n, o))| !n.eq(o))
            .try_for_each(|(i, (n, o))| {
                let mut raw = [0; 4];
                n.build_quadlet(&mut raw);
                let offset = MONITOR_OUT_VOL_OFFSET + 4 * i;
                alesis_write_block(req, node, offset, &mut raw, timeout_ms).map(|_| *o = *n)
            })
    }

    fn read_mixer_out_mutes(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut IofwMixerState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        alesis_read_flags(
            req,
            node,
            MONITOR_OUT_MUTE_OFFSET,
            &mut state.out_mutes,
            timeout_ms,
        )
    }

    fn write_mixer_out_mutes(
        req: &mut FwReq,
        node: &mut FwNode,
        mutes: &[bool],
        state: &mut IofwMixerState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(state.out_mutes.len(), mutes.len());

        alesis_write_flags(req, node, MONITOR_OUT_MUTE_OFFSET, mutes, timeout_ms)
            .map(|_| state.out_mutes.copy_from_slice(mutes))
    }

    fn read_knob_state(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut IofwMixerState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 8];
        alesis_read_block(req, node, KNOB_STATE_OFFSET, &mut raw, timeout_ms)
            .map(|_| state.knobs.parse(&raw))
    }
}
