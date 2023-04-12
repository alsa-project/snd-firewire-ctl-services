// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol for hardware mixer function expressed in registers.
//!
//! The module includes structure, enumeration, and trait for hardware mixer function expressed
//! in registers.
//!
//! The hardware transfers isochronous packets in which its state, PCM frames, and MIDI messages
//! are multiplexed. ALSA firewire-motu driver caches the state, and allow userspace application
//! to read the cache from kernel space as `SndMotuRegisterDspParameter` structure. Additionally,
//! when changing the cache, the driver generates notification to the application.
//! `RegisterDspEvent` is available to parse the notification.

use {super::*, hitaki::SndMotuRegisterDspParameter};

const EV_TYPE_MIXER_SRC_GAIN: u8 = 0x02;
const EV_TYPE_MIXER_SRC_PAN: u8 = 0x03;
const EV_TYPE_MIXER_SRC_FLAG: u8 = 0x04;
const EV_TYPE_MIXER_OUTPUT_PAIRED_VOLUME: u8 = 0x05;
const EV_TYPE_MIXER_OUTPUT_PAIRED_FLAG: u8 = 0x06;
const EV_TYPE_MAIN_OUTPUT_PAIRED_VOLUME: u8 = 0x07;
const EV_TYPE_HP_OUTPUT_PAIRED_VOLUME: u8 = 0x08;
const EV_TYPE_LINE_INPUT_BOOST: u8 = 0x0d;
const EV_TYPE_LINE_INPUT_NOMINAL_LEVEL: u8 = 0x0e;
const EV_TYPE_INPUT_GAIN_AND_INVERT: u8 = 0x15;
const EV_TYPE_INPUT_FLAG: u8 = 0x16;
const EV_TYPE_MIXER_SRC_PAIRED_BALANCE: u8 = 0x17;
const EV_TYPE_MIXER_SRC_PAIRED_WIDTH: u8 = 0x18;

/// The event emitted from ALSA firewire-motu driver for register DSP.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct RegisterDspEvent {
    /// The numeric type of event.
    pub ev_type: u8,
    /// The first identifier specific to the event.
    pub identifier0: u8,
    /// The second identifier specific to the event.
    pub identifier1: u8,
    /// The value of event.
    pub value: u8,
}

impl From<u32> for RegisterDspEvent {
    fn from(val: u32) -> Self {
        Self {
            ev_type: ((val & 0xff000000) >> 24) as u8,
            identifier0: ((val & 0x00ff0000) >> 16) as u8,
            identifier1: ((val & 0x0000ff00) >> 8) as u8,
            value: (val & 0x000000ff) as u8,
        }
    }
}

/// The specification for register DSP.
pub trait MotuRegisterDspSpecification {
    /// The destinations of mixer outputs.
    const MIXER_OUTPUT_DESTINATIONS: &'static [TargetPort];

    /// The number of mixers.
    const MIXER_COUNT: usize = 4;

    /// The minimum value of mixer output volume.
    const MIXER_OUTPUT_VOLUME_MIN: u8 = 0x00;
    /// The maximum value of mixer output volume.
    const MIXER_OUTPUT_VOLUME_MAX: u8 = 0x80;
    /// The step value of mixer output volume.
    const MIXER_OUTPUT_VOLUME_STEP: u8 = 0x01;

    /// The minimum value of physical output volume.
    const OUTPUT_VOLUME_MIN: u8 = 0x00;
    /// The maximum value of physical output volume.
    const OUTPUT_VOLUME_MAX: u8 = 0x80;
    /// The step value of physical output volume.
    const OUTPUT_VOLUME_STEP: u8 = 0x01;
}

/// The trait for DSP image operations.
pub trait MotuRegisterDspImageOperation<T, U> {
    /// Parse image transferred in the series of isochronous packets.
    fn parse_image(params: &mut T, image: &U);
}

/// The trait for DSP event operation.
pub trait MotuRegisterDspEventOperation<T> {
    /// Parse event.
    fn parse_event(params: &mut T, event: &RegisterDspEvent) -> bool;
}

const MIXER_COUNT: usize = 4;

const MIXER_OUTPUT_OFFSETS: [usize; MIXER_COUNT] = [0x0c20, 0x0c24, 0x0c28, 0x0c2c];
const MIXER_OUTPUT_MUTE_FLAG: u32 = 0x00001000;
const MIXER_OUTPUT_DESTINATION_MASK: u32 = 0x00000f00;
const MIXER_OUTPUT_VOLUME_MASK: u32 = 0x000000ff;

/// The parameters of mixer return.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RegisterDspMixerReturnParameters(pub bool);

impl<O> MotuWhollyCacheableParamsOperation<RegisterDspMixerReturnParameters> for O
where
    O: MotuRegisterDspSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut RegisterDspMixerReturnParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        read_quad(req, node, MIXER_RETURN_ENABLE_OFFSET as u32, timeout_ms)
            .map(|val| params.0 = val > 0)
    }
}

impl<O> MotuWhollyUpdatableParamsOperation<RegisterDspMixerReturnParameters> for O
where
    O: MotuRegisterDspSpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &RegisterDspMixerReturnParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_quad(
            req,
            node,
            MIXER_RETURN_ENABLE_OFFSET as u32,
            params.0 as u32,
            timeout_ms,
        )
    }
}

/// State of mixer output.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RegisterDspMixerOutputState {
    pub volume: [u8; MIXER_COUNT],
    pub mute: [bool; MIXER_COUNT],
    pub destination: [TargetPort; MIXER_COUNT],
}

impl<O> MotuWhollyCacheableParamsOperation<RegisterDspMixerOutputState> for O
where
    O: MotuRegisterDspSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut RegisterDspMixerOutputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        MIXER_OUTPUT_OFFSETS
            .iter()
            .enumerate()
            .try_for_each(|(i, &offset)| {
                read_quad(req, node, offset as u32, timeout_ms).map(|val| {
                    params.volume[i] = (val & MIXER_OUTPUT_VOLUME_MASK) as u8;
                    params.mute[i] = (val & MIXER_OUTPUT_MUTE_FLAG) > 0;

                    let src = ((val & MIXER_OUTPUT_DESTINATION_MASK) >> 8) as usize;
                    params.destination[i] = Self::MIXER_OUTPUT_DESTINATIONS
                        .iter()
                        .nth(src)
                        .map(|&p| p)
                        .unwrap_or_default();
                })
            })
    }
}

impl<O> MotuPartiallyUpdatableParamsOperation<RegisterDspMixerOutputState> for O
where
    O: MotuRegisterDspSpecification,
{
    fn update_partially(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut RegisterDspMixerOutputState,
        updates: RegisterDspMixerOutputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        MIXER_OUTPUT_OFFSETS
            .iter()
            .enumerate()
            .try_for_each(|(i, &offset)| {
                if params.volume[i] != updates.volume[i]
                    || params.mute[i] != updates.mute[i]
                    || params.destination[i] != updates.destination[i]
                {
                    let quad = read_quad(req, node, offset as u32, timeout_ms)?;
                    let mut change = quad;
                    if params.volume[i] != updates.volume[i] {
                        change &= !MIXER_OUTPUT_VOLUME_MASK;
                        change |= updates.volume[i] as u32;
                    }
                    if params.mute[i] != updates.mute[i] {
                        change &= !MIXER_OUTPUT_MUTE_FLAG;
                        if updates.mute[i] {
                            change |= MIXER_OUTPUT_MUTE_FLAG;
                        }
                    }
                    if params.destination[i] != updates.destination[i] {
                        let pos = Self::MIXER_OUTPUT_DESTINATIONS
                            .iter()
                            .position(|d| updates.destination[i].eq(d))
                            .unwrap_or_default();
                        change &= !MIXER_OUTPUT_DESTINATION_MASK;
                        change |= (pos as u32) << 8;
                    }
                    if quad != change {
                        write_quad(req, node, offset as u32, change, timeout_ms)
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            })
    }
}

impl<O> MotuRegisterDspImageOperation<RegisterDspMixerOutputState, SndMotuRegisterDspParameter>
    for O
where
    O: MotuRegisterDspSpecification,
{
    fn parse_image(params: &mut RegisterDspMixerOutputState, image: &SndMotuRegisterDspParameter) {
        let vols = image.mixer_output_paired_volume();
        params.volume.copy_from_slice(vols);

        let flags = image.mixer_output_paired_flag();
        params.mute.iter_mut().zip(flags).for_each(|(mute, &flag)| {
            let val = (flag as u32) << 8;
            *mute = val & MIXER_OUTPUT_MUTE_FLAG > 0;
        });
        params
            .destination
            .iter_mut()
            .zip(flags)
            .for_each(|(dest, &flag)| {
                let val = (flag as u32) << 8;
                let idx = ((val & MIXER_OUTPUT_DESTINATION_MASK) >> 8) as usize;
                *dest = Self::MIXER_OUTPUT_DESTINATIONS
                    .iter()
                    .nth(idx)
                    .map(|&port| port)
                    .unwrap_or_default();
            });
    }
}

impl<O> MotuRegisterDspEventOperation<RegisterDspMixerOutputState> for O
where
    O: MotuRegisterDspSpecification,
{
    fn parse_event(params: &mut RegisterDspMixerOutputState, event: &RegisterDspEvent) -> bool {
        match event.ev_type {
            EV_TYPE_MIXER_OUTPUT_PAIRED_VOLUME => {
                let mixer = event.identifier0 as usize;
                if mixer < MIXER_COUNT {
                    params.volume[mixer] = event.value;
                    true
                } else {
                    false
                }
            }
            EV_TYPE_MIXER_OUTPUT_PAIRED_FLAG => {
                let mixer = event.identifier0 as usize;
                if mixer < MIXER_COUNT {
                    let val = (event.value as u32) << 8;

                    params.mute[mixer] = val & MIXER_OUTPUT_MUTE_FLAG > 0;

                    let assign = ((val & MIXER_OUTPUT_DESTINATION_MASK) >> 8) as usize;
                    params.destination[mixer] = Self::MIXER_OUTPUT_DESTINATIONS
                        .iter()
                        .nth(assign)
                        .map(|&port| port)
                        .unwrap_or_default();

                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

const MIXER_RETURN_ENABLE_OFFSET: usize = 0x0c18;

/// State of sources in mixer entiry.
#[derive(Default, Debug, Clone)]
pub struct RegisterDspMixerMonauralSourceEntry {
    pub gain: Vec<u8>,
    pub pan: Vec<u8>,
    pub mute: Vec<bool>,
    pub solo: Vec<bool>,
}

/// State of mixer sources.
#[derive(Default, Debug)]
pub struct RegisterDspMixerMonauralSourceState(
    pub [RegisterDspMixerMonauralSourceEntry; MIXER_COUNT],
);

const MIXER_SOURCE_OFFSETS: [usize; MIXER_COUNT] = [0x4000, 0x4100, 0x4200, 0x4300];
const MIXER_SOURCE_PAN_CHANGE_FLAG: u32 = 0x80000000;
const MIXER_SOURCE_GAIN_CHANGE_FLAG: u32 = 0x40000000;
const MIXER_SOURCE_MUTE_FLAG: u32 = 0x00010000;
const MIXER_SOURCE_SOLO_FLAG: u32 = 0x00020000;
const MIXER_SOURCE_PAN_MASK: u32 = 0x0000ff00;
const MIXER_SOURCE_GAIN_MASK: u32 = 0x000000ff;

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
        state.0.iter_mut().for_each(|entry| {
            entry.gain = vec![Default::default(); Self::MIXER_SOURCES.len()];
            entry.pan = vec![Default::default(); Self::MIXER_SOURCES.len()];
            entry.mute = vec![Default::default(); Self::MIXER_SOURCES.len()];
            entry.solo = vec![Default::default(); Self::MIXER_SOURCES.len()];
        });
        state
    }

    fn parse_dsp_parameter(
        state: &mut RegisterDspMixerMonauralSourceState,
        param: &SndMotuRegisterDspParameter,
    ) {
        state.0.iter_mut().enumerate().for_each(|(i, src)| {
            let gains = param.mixer_source_gain(i);
            src.gain
                .iter_mut()
                .zip(gains)
                .for_each(|(dst, src)| *dst = *src);

            let pans = param.mixer_source_pan(i);
            src.pan
                .iter_mut()
                .zip(pans)
                .for_each(|(dst, src)| *dst = *src);

            let flags: Vec<u32> = param
                .mixer_source_flag(i)
                .iter()
                .map(|&flag| (flag as u32) << 16)
                .collect();

            src.mute
                .iter_mut()
                .zip(&flags)
                .for_each(|(mute, flag)| *mute = flag & MIXER_SOURCE_MUTE_FLAG > 0);
            src.solo
                .iter_mut()
                .zip(&flags)
                .for_each(|(solo, flag)| *solo = flag & MIXER_SOURCE_SOLO_FLAG > 0);
        });
    }

    fn parse_dsp_event(
        state: &mut RegisterDspMixerMonauralSourceState,
        event: &RegisterDspEvent,
    ) -> bool {
        match event.ev_type {
            EV_TYPE_MIXER_SRC_GAIN => {
                let mixer = event.identifier0 as usize;
                let src = event.identifier1 as usize;
                let val = event.value;

                if mixer < MIXER_COUNT && src < Self::MIXER_SOURCES.len() {
                    state.0[mixer].gain[src] = val;
                    true
                } else {
                    false
                }
            }
            EV_TYPE_MIXER_SRC_PAN => {
                let mixer = event.identifier0 as usize;
                let src = event.identifier1 as usize;
                let val = event.value;

                if mixer < MIXER_COUNT && src < Self::MIXER_SOURCES.len() {
                    state.0[mixer].pan[src] = val;
                    true
                } else {
                    false
                }
            }
            EV_TYPE_MIXER_SRC_FLAG => {
                let mixer = event.identifier0 as usize;
                let src = event.identifier1 as usize;
                let val = (event.value as u32) << 16;

                if mixer < MIXER_COUNT && src < Self::MIXER_SOURCES.len() {
                    state.0[mixer].mute[src] = val & MIXER_SOURCE_MUTE_FLAG > 0;
                    state.0[mixer].solo[src] = val & MIXER_SOURCE_SOLO_FLAG > 0;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    fn read_mixer_monaural_source_state(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut RegisterDspMixerMonauralSourceState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        state
            .0
            .iter_mut()
            .zip(MIXER_SOURCE_OFFSETS)
            .try_for_each(|(entry, offset)| {
                (0..Self::MIXER_SOURCES.len()).try_for_each(|i| {
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

        state.0[mixer]
            .gain
            .iter_mut()
            .zip(gain)
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

        state.0[mixer]
            .pan
            .iter_mut()
            .zip(pan)
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

        state.0[mixer]
            .mute
            .iter_mut()
            .zip(mute)
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

        state.0[mixer]
            .solo
            .iter_mut()
            .zip(solo)
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

/// State of sources in mixer entiry.
#[derive(Default, Debug, Clone)]
pub struct RegisterDspMixerStereoSourceEntry {
    pub gain: [u8; MIXER_STEREO_SOURCE_COUNT],
    pub pan: [u8; MIXER_STEREO_SOURCE_COUNT],
    pub mute: [bool; MIXER_STEREO_SOURCE_COUNT],
    pub solo: [bool; MIXER_STEREO_SOURCE_COUNT],
    pub balance: [u8; MIXER_STEREO_SOURCE_PAIR_COUNT],
    pub width: [u8; MIXER_STEREO_SOURCE_PAIR_COUNT],
}

/// State of mixer sources.
#[derive(Default, Debug)]
pub struct RegisterDspMixerStereoSourceState(pub [RegisterDspMixerStereoSourceEntry; MIXER_COUNT]);

const MIXER_SOURCE_PAIRED_WIDTH_FLAG: u32 = 0x00400000;
const MIXER_SOURCE_PAIRED_BALANCE_FLAG: u32 = 0x00800000;

const EV_MIXER_SOURCE_PAIRED_CH_MAP: [usize; MIXER_STEREO_SOURCE_COUNT] = [0, 1, 2, 3, 8, 9];

/// The trait for operation of mixer sources.
pub trait RegisterDspMixerStereoSourceOperation {
    const MIXER_SOURCES: [TargetPort; MIXER_STEREO_SOURCE_COUNT] = [
        TargetPort::Analog(0),
        TargetPort::Analog(1),
        TargetPort::Analog(2),
        TargetPort::Analog(3),
        TargetPort::Spdif(0),
        TargetPort::Spdif(1),
    ];
    const MIXER_SOURCE_PAIR_COUNT: usize = Self::MIXER_SOURCES.len() / 2;

    const MIXER_COUNT: usize = MIXER_COUNT;

    const SOURCE_GAIN_MIN: u8 = 0x00;
    const SOURCE_GAIN_MAX: u8 = 0x80;
    const SOURCE_GAIN_STEP: u8 = 0x01;

    const SOURCE_PAN_MIN: u8 = 0x00;
    const SOURCE_PAN_MAX: u8 = 0x80;
    const SOURCE_PAN_STEP: u8 = 0x01;

    const SOURCE_STEREO_BALANCE_MIN: u8 = 0x00;
    const SOURCE_STEREO_BALANCE_MAX: u8 = 0x80;
    const SOURCE_STEREO_BALANCE_STEP: u8 = 0x01;

    const SOURCE_STEREO_WIDTH_MIN: u8 = 0x00;
    const SOURCE_STEREO_WIDTH_MAX: u8 = 0x80;
    const SOURCE_STEREO_WIDTH_STEP: u8 = 0x01;

    fn create_mixer_stereo_source_state() -> RegisterDspMixerStereoSourceState {
        Default::default()
    }

    fn parse_dsp_parameter(
        state: &mut RegisterDspMixerStereoSourceState,
        param: &SndMotuRegisterDspParameter,
    ) {
        state.0.iter_mut().enumerate().for_each(|(i, src)| {
            let gains = param.mixer_source_gain(i);
            src.gain
                .iter_mut()
                .zip(gains)
                .for_each(|(dst, src)| *dst = *src);

            let pans = param.mixer_source_pan(i);
            src.pan
                .iter_mut()
                .zip(pans)
                .for_each(|(dst, src)| *dst = *src);

            let flags: Vec<u32> = param
                .mixer_source_flag(i)
                .iter()
                .map(|&flag| (flag as u32) << 16)
                .collect();

            src.mute
                .iter_mut()
                .zip(&flags)
                .for_each(|(mute, flag)| *mute = flag & MIXER_SOURCE_MUTE_FLAG > 0);
            src.solo
                .iter_mut()
                .zip(&flags)
                .for_each(|(solo, flag)| *solo = flag & MIXER_SOURCE_SOLO_FLAG > 0);
        });
    }

    fn parse_dsp_event(
        state: &mut RegisterDspMixerStereoSourceState,
        event: &RegisterDspEvent,
    ) -> bool {
        match event.ev_type {
            EV_TYPE_MIXER_SRC_GAIN => {
                let mixer = event.identifier0 as usize;
                let src = event.identifier1 as usize;
                let val = event.value;

                if mixer < MIXER_COUNT && src < Self::MIXER_SOURCES.len() {
                    state.0[mixer].gain[src] = val;
                    true
                } else {
                    false
                }
            }
            EV_TYPE_MIXER_SRC_PAN => {
                let mixer = event.identifier0 as usize;
                let src = event.identifier1 as usize;
                let val = event.value;

                if mixer < MIXER_COUNT && src < Self::MIXER_SOURCES.len() {
                    state.0[mixer].pan[src] = val;
                    true
                } else {
                    false
                }
            }
            EV_TYPE_MIXER_SRC_FLAG => {
                let mixer = event.identifier0 as usize;
                let src = event.identifier1 as usize;
                let val = (event.value as u32) << 16;

                if mixer < MIXER_COUNT && src < Self::MIXER_SOURCES.len() {
                    state.0[mixer].mute[src] = val & MIXER_SOURCE_MUTE_FLAG > 0;
                    state.0[mixer].solo[src] = val & MIXER_SOURCE_SOLO_FLAG > 0;
                    true
                } else {
                    false
                }
            }
            EV_TYPE_MIXER_SRC_PAIRED_BALANCE => {
                let mixer = event.identifier0 as usize;
                let src = event.identifier1 as usize;
                let val = event.value;

                if let Some(idx) = EV_MIXER_SOURCE_PAIRED_CH_MAP.iter().position(|&p| p == src) {
                    state.0[mixer].balance[idx / 2] = val;
                    true
                } else {
                    false
                }
            }
            EV_TYPE_MIXER_SRC_PAIRED_WIDTH => {
                let mixer = event.identifier0 as usize;
                let src = event.identifier1 as usize;
                let val = event.value;

                if let Some(idx) = EV_MIXER_SOURCE_PAIRED_CH_MAP.iter().position(|&p| p == src) {
                    state.0[mixer].width[idx / 2] = val;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
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
        state.0.iter_mut().enumerate().try_for_each(|(i, entry)| {
            let base_offset = MIXER_SOURCE_OFFSETS[i];
            (0..Self::MIXER_SOURCES.len()).try_for_each(|j| {
                let offset = Self::compute_mixer_source_offset(base_offset, j);
                read_quad(req, node, offset as u32, timeout_ms).map(|val| {
                    entry.gain[j] = (val & MIXER_SOURCE_GAIN_MASK) as u8;
                    entry.mute[j] = ((val & MIXER_SOURCE_MUTE_FLAG) >> 8) > 0;
                    entry.solo[j] = ((val & MIXER_SOURCE_SOLO_FLAG) >> 16) > 0;
                })
            })
        })?;

        // MEMO: no register to read balance and width of mixer source.

        Ok(())
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

        state.0[mixer]
            .gain
            .iter_mut()
            .zip(gain)
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

    fn write_mixer_stereo_source_pan(
        req: &mut FwReq,
        node: &mut FwNode,
        mixer: usize,
        pan: &[u8],
        state: &mut RegisterDspMixerStereoSourceState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(mixer < MIXER_COUNT);
        assert_eq!(pan.len(), Self::MIXER_SOURCES.len());

        let base_offset = MIXER_SOURCE_OFFSETS[mixer];

        state.0[mixer]
            .pan
            .iter_mut()
            .zip(pan)
            .enumerate()
            .filter(|(_, (old, new))| !old.eq(new))
            .try_for_each(|(i, (old, new))| {
                let offset = Self::compute_mixer_source_offset(base_offset, i);
                let mut val = read_quad(req, node, offset as u32, timeout_ms)?;
                val &= !MIXER_SOURCE_PAN_MASK;
                val |= (*new as u32) << 8;
                val |= MIXER_SOURCE_PAN_CHANGE_FLAG;
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

        state.0[mixer]
            .mute
            .iter_mut()
            .zip(mute)
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

        state.0[mixer]
            .solo
            .iter_mut()
            .zip(solo)
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

    fn write_mixer_stereo_source_balance(
        req: &mut FwReq,
        node: &mut FwNode,
        mixer: usize,
        balance: &[u8],
        state: &mut RegisterDspMixerStereoSourceState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(mixer < MIXER_COUNT);
        assert_eq!(balance.len(), Self::MIXER_SOURCE_PAIR_COUNT);

        let base_offset = MIXER_SOURCE_OFFSETS[mixer];

        state.0[mixer]
            .balance
            .iter_mut()
            .zip(balance)
            .enumerate()
            .filter(|(_, (old, new))| !old.eq(new))
            .try_for_each(|(i, (old, new))| {
                let offset = Self::compute_mixer_source_offset(base_offset, i * 2);
                let val = MIXER_SOURCE_PAN_CHANGE_FLAG
                    | MIXER_SOURCE_PAIRED_BALANCE_FLAG
                    | ((*new as u32) << 8);
                write_quad(req, node, offset as u32, val, timeout_ms).map(|_| *old = *new)
            })
    }

    fn write_mixer_stereo_source_width(
        req: &mut FwReq,
        node: &mut FwNode,
        mixer: usize,
        width: &[u8],
        state: &mut RegisterDspMixerStereoSourceState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(mixer < MIXER_COUNT);
        assert_eq!(width.len(), Self::MIXER_SOURCE_PAIR_COUNT);

        let base_offset = MIXER_SOURCE_OFFSETS[mixer];

        state.0[mixer]
            .width
            .iter_mut()
            .zip(width)
            .enumerate()
            .filter(|(_, (old, new))| !old.eq(new))
            .try_for_each(|(i, (old, new))| {
                let offset = Self::compute_mixer_source_offset(base_offset, i * 2);
                let val = MIXER_SOURCE_PAN_CHANGE_FLAG
                    | MIXER_SOURCE_PAIRED_WIDTH_FLAG
                    | ((*new as u32) << 8);
                write_quad(req, node, offset as u32, val, timeout_ms).map(|_| *old = *new)
            })
    }
}

const MASTER_VOLUME_OFFSET: usize = 0x0c0c;
const PHONE_VOLUME_OFFSET: usize = 0x0c10;

/// State of output.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RegisterDspOutputState {
    /// The volume of master output.
    pub master_volume: u8,
    /// The volume of headphone output.
    pub phone_volume: u8,
}

impl<O> MotuWhollyCacheableParamsOperation<RegisterDspOutputState> for O
where
    O: MotuRegisterDspSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut RegisterDspOutputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        read_quad(req, node, MASTER_VOLUME_OFFSET as u32, timeout_ms).map(|val| {
            params.master_volume = val as u8;
        })?;
        read_quad(req, node, PHONE_VOLUME_OFFSET as u32, timeout_ms).map(|val| {
            params.phone_volume = val as u8;
        })?;
        Ok(())
    }
}

impl<O> MotuPartiallyUpdatableParamsOperation<RegisterDspOutputState> for O
where
    O: MotuRegisterDspSpecification,
{
    fn update_partially(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut RegisterDspOutputState,
        updates: RegisterDspOutputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if params.master_volume != updates.master_volume {
            write_quad(
                req,
                node,
                MASTER_VOLUME_OFFSET as u32,
                updates.master_volume as u32,
                timeout_ms,
            )?;
            params.master_volume = updates.master_volume;
        }

        if params.phone_volume != updates.phone_volume {
            write_quad(
                req,
                node,
                PHONE_VOLUME_OFFSET as u32,
                updates.phone_volume as u32,
                timeout_ms,
            )?;
            params.phone_volume = updates.phone_volume;
        }

        Ok(())
    }
}

impl<O> MotuRegisterDspImageOperation<RegisterDspOutputState, SndMotuRegisterDspParameter> for O
where
    O: MotuRegisterDspSpecification,
{
    fn parse_image(params: &mut RegisterDspOutputState, image: &SndMotuRegisterDspParameter) {
        params.master_volume = image.main_output_paired_volume();
        params.phone_volume = image.headphone_output_paired_volume();
    }
}

impl<O> MotuRegisterDspEventOperation<RegisterDspOutputState> for O
where
    O: MotuRegisterDspSpecification,
{
    fn parse_event(params: &mut RegisterDspOutputState, event: &RegisterDspEvent) -> bool {
        match event.ev_type {
            EV_TYPE_MAIN_OUTPUT_PAIRED_VOLUME => {
                params.master_volume = event.value;
                true
            }
            EV_TYPE_HP_OUTPUT_PAIRED_VOLUME => {
                params.phone_volume = event.value;
                true
            }
            _ => false,
        }
    }
}

/// The trait for operation of output.
pub trait RegisterDspOutputOperation {
    const VOLUME_MIN: u8 = 0x00;
    const VOLUME_MAX: u8 = 0x80;
    const VOLUME_STEP: u8 = 0x01;

    fn parse_dsp_parameter(
        state: &mut RegisterDspOutputState,
        param: &SndMotuRegisterDspParameter,
    ) {
        state.master_volume = param.main_output_paired_volume();
        state.phone_volume = param.headphone_output_paired_volume();
    }

    fn parse_dsp_event(state: &mut RegisterDspOutputState, event: &RegisterDspEvent) -> bool {
        match event.ev_type {
            EV_TYPE_MAIN_OUTPUT_PAIRED_VOLUME => {
                state.master_volume = event.value;
                true
            }
            EV_TYPE_HP_OUTPUT_PAIRED_VOLUME => {
                state.phone_volume = event.value;
                true
            }
            _ => false,
        }
    }

    fn read_output_state(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut RegisterDspOutputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        read_quad(req, node, MASTER_VOLUME_OFFSET as u32, timeout_ms).map(|val| {
            state.master_volume = val as u8;
        })?;
        read_quad(req, node, PHONE_VOLUME_OFFSET as u32, timeout_ms).map(|val| {
            state.phone_volume = val as u8;
        })?;
        Ok(())
    }

    fn write_output_master_volume(
        req: &mut FwReq,
        node: &mut FwNode,
        vol: u8,
        state: &mut RegisterDspOutputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_quad(
            req,
            node,
            MASTER_VOLUME_OFFSET as u32,
            vol as u32,
            timeout_ms,
        )
        .map(|_| {
            state.master_volume = vol;
        })
    }

    fn write_output_phone_volume(
        req: &mut FwReq,
        node: &mut FwNode,
        vol: u8,
        state: &mut RegisterDspOutputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_quad(
            req,
            node,
            PHONE_VOLUME_OFFSET as u32,
            vol as u32,
            timeout_ms,
        )
        .map(|_| {
            state.phone_volume = vol;
        })
    }
}

/// State of inputs in 828mkII.
#[derive(Default, Debug)]
pub struct RegisterDspLineInputState {
    pub level: Vec<NominalSignalLevel>,
    /// + 6dB.
    pub boost: Vec<bool>,
}

const LINE_INPUT_LEVEL_OFFSET: usize = 0x0c08;
const LINE_INPUT_BOOST_OFFSET: usize = 0x0c14;

/// The trait for operation of line input in Traveler and 828mk2.
pub trait Traveler828mk2LineInputOperation {
    const LINE_INPUT_COUNT: usize;
    const CH_OFFSET: usize;

    fn create_line_input_state() -> RegisterDspLineInputState {
        RegisterDspLineInputState {
            level: vec![Default::default(); Self::LINE_INPUT_COUNT],
            boost: vec![Default::default(); Self::LINE_INPUT_COUNT],
        }
    }

    fn parse_dsp_parameter(
        state: &mut RegisterDspLineInputState,
        param: &SndMotuRegisterDspParameter,
    ) {
        let flags = param.line_input_nominal_level_flag();
        state.level.iter_mut().enumerate().for_each(|(i, level)| {
            let shift = i + Self::CH_OFFSET;
            *level = if flags & (1 << shift) > 0 {
                NominalSignalLevel::Professional
            } else {
                NominalSignalLevel::Consumer
            };
        });

        let flags = param.line_input_boost_flag();
        state.boost.iter_mut().enumerate().for_each(|(i, boost)| {
            let shift = i + Self::CH_OFFSET;
            *boost = flags & (1 << shift) > 0;
        });
    }

    fn parse_dsp_event(state: &mut RegisterDspLineInputState, event: &RegisterDspEvent) -> bool {
        match event.ev_type {
            EV_TYPE_LINE_INPUT_NOMINAL_LEVEL => {
                state.level.iter_mut().enumerate().for_each(|(i, level)| {
                    let shift = i + Self::CH_OFFSET;
                    *level = if event.value & (1 << shift) > 0 {
                        NominalSignalLevel::Professional
                    } else {
                        NominalSignalLevel::Consumer
                    };
                });
                true
            }
            EV_TYPE_LINE_INPUT_BOOST => {
                state.boost.iter_mut().enumerate().for_each(|(i, boost)| {
                    let shift = i + Self::CH_OFFSET;
                    *boost = event.value & (1 << shift) > 0;
                });
                true
            }
            _ => false,
        }
    }

    fn read_line_input_state(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut RegisterDspLineInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        read_quad(req, node, LINE_INPUT_LEVEL_OFFSET as u32, timeout_ms).map(|val| {
            state
                .level
                .iter_mut()
                .enumerate()
                .for_each(|(mut i, level)| {
                    i += Self::CH_OFFSET;
                    *level = if val & (1 << i) > 0 {
                        NominalSignalLevel::Professional
                    } else {
                        NominalSignalLevel::Consumer
                    };
                });
        })?;

        read_quad(req, node, LINE_INPUT_BOOST_OFFSET as u32, timeout_ms).map(|val| {
            state
                .boost
                .iter_mut()
                .enumerate()
                .for_each(|(mut i, boost)| {
                    i += Self::CH_OFFSET;
                    *boost = val & (1 << i) > 0
                });
        })?;

        Ok(())
    }

    fn write_line_input_level(
        req: &mut FwReq,
        node: &mut FwNode,
        level: &[NominalSignalLevel],
        state: &mut RegisterDspLineInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = level.iter().enumerate().fold(0u32, |mut val, (i, l)| {
            if NominalSignalLevel::Professional.eq(l) {
                val |= 1 << (i + Self::CH_OFFSET);
            }
            val
        });

        write_quad(req, node, LINE_INPUT_LEVEL_OFFSET as u32, val, timeout_ms).map(|_| {
            state.level.copy_from_slice(level);
        })
    }

    fn write_line_input_boost(
        req: &mut FwReq,
        node: &mut FwNode,
        boost: &[bool],
        state: &mut RegisterDspLineInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = boost.iter().enumerate().fold(0u32, |mut val, (mut i, b)| {
            i += Self::CH_OFFSET;
            if *b {
                val |= 1 << i;
            }
            val
        });

        write_quad(req, node, LINE_INPUT_BOOST_OFFSET as u32, val, timeout_ms).map(|_| {
            state.boost.copy_from_slice(boost);
        })
    }
}

const MONAURAL_INPUT_COUNT: usize = 10;

/// State of input in Ultralite.
#[derive(Default, Debug)]
pub struct RegisterDspMonauralInputState {
    pub gain: [u8; MONAURAL_INPUT_COUNT],
    pub invert: [bool; MONAURAL_INPUT_COUNT],
}

const STEREO_INPUT_COUNT: usize = 6;

/// State of input in Audio Express, and 4 pre.
#[derive(Default, Debug)]
pub struct RegisterDspStereoInputState {
    pub gain: [u8; STEREO_INPUT_COUNT],
    pub invert: [bool; STEREO_INPUT_COUNT],
    pub paired: [bool; STEREO_INPUT_COUNT / 2],
    pub phantom: Vec<bool>,
    pub pad: Vec<bool>,
    pub jack: Vec<bool>,
}

const INPUT_GAIN_INVERT_OFFSET: usize = 0x0c70;
const MONAURAL_INPUT_GAIN_MASK: u8 = 0x1f;
const MONAURAL_INPUT_INVERT_FLAG: u8 = 0x20;
const STEREO_INPUT_GAIN_MASK: u8 = 0x3f;
const STEREO_INPUT_INVERT_FLAG: u8 = 0x40;
const INPUT_CHANGE_FLAG: u8 = 0x80;
const MIC_PARAM_OFFSET: usize = 0x0c80;
const MIC_PARAM_PAD_FLAG: u8 = 0x02;
const MIC_PARAM_PHANTOM_FLAG: u8 = 0x01;
const MIC_PARAM_CHANGE_FLAG: u8 = 0x80;
const INPUT_PAIRED_OFFSET: usize = 0x0c84;
const INPUT_PAIRED_FLAG: u8 = 0x01;
const INPUT_PAIRED_CHANGE_FLAG: u8 = 0x80;
const INPUT_PAIRED_CH_MAP: [usize; STEREO_INPUT_COUNT / 2] = [0, 1, 3];

const EV_INPUT_PAIRED_FLAG: u8 = 0x01;
const EV_MIC_PHANTOM_FLAG: u8 = 0x02;
const EV_MIC_PAD_FLAG: u8 = 0x04;
const EV_INPUT_JACK_FLAG: u8 = 0x08;
const EV_INPUT_PAIRED_CH_MAP: [usize; STEREO_INPUT_COUNT] = [0, 1, 2, 3, 8, 9];

/// The trait for operation of input in Ultralite.
pub trait RegisterDspMonauralInputOperation {
    const INPUT_COUNT: usize = MONAURAL_INPUT_COUNT;

    const INPUT_GAIN_MIN: u8 = 0x00;
    const INPUT_MIC_GAIN_MAX: u8 = 0x18;
    const INPUT_LINE_GAIN_MAX: u8 = 0x12;
    const INPUT_SPDIF_GAIN_MAX: u8 = 0x0c;
    const INPUT_GAIN_STEP: u8 = 0x01;

    fn create_monaural_input_state() -> RegisterDspMonauralInputState {
        RegisterDspMonauralInputState {
            gain: Default::default(),
            invert: Default::default(),
        }
    }

    fn parse_dsp_parameter(
        state: &mut RegisterDspMonauralInputState,
        param: &SndMotuRegisterDspParameter,
    ) {
        let vals = param.input_gain_and_invert();
        state
            .gain
            .iter_mut()
            .zip(vals)
            .for_each(|(gain, val)| *gain = val & MONAURAL_INPUT_GAIN_MASK);
        state
            .invert
            .iter_mut()
            .zip(vals)
            .for_each(|(invert, val)| *invert = val & MONAURAL_INPUT_INVERT_FLAG > 0);
    }

    fn parse_dsp_event(
        state: &mut RegisterDspMonauralInputState,
        event: &RegisterDspEvent,
    ) -> bool {
        match event.ev_type {
            EV_TYPE_INPUT_GAIN_AND_INVERT => {
                let ch = event.identifier0 as usize;
                if ch < MONAURAL_INPUT_COUNT {
                    state.gain[ch] = event.value & MONAURAL_INPUT_GAIN_MASK;
                    state.invert[ch] = event.value & MONAURAL_INPUT_INVERT_FLAG > 0;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn read_monaural_input_state(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut RegisterDspMonauralInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quads = vec![0; (Self::INPUT_COUNT + 3) / 4];
        quads.iter_mut().enumerate().try_for_each(|(i, quad)| {
            let offset = INPUT_GAIN_INVERT_OFFSET + i * 4;
            read_quad(req, node, offset as u32, timeout_ms).map(|val| *quad = val)
        })?;

        (0..Self::INPUT_COUNT).for_each(|i| {
            let pos = i / 4;
            let shift = (i % 4) * 8;
            let val = ((quads[pos] >> shift) as u8) & 0xff;

            state.gain[i] = val & MONAURAL_INPUT_GAIN_MASK;
            state.invert[i] = val & MONAURAL_INPUT_INVERT_FLAG > 0;
        });

        Ok(())
    }

    fn write_monaural_input_gain(
        req: &mut FwReq,
        node: &mut FwNode,
        gain: &[u8],
        state: &mut RegisterDspMonauralInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(gain.len(), Self::INPUT_COUNT);

        let prev_gains = &state.gain;
        let inverts = &state.invert;

        gain.iter()
            .enumerate()
            .filter(|&(i, val)| !prev_gains[i].eq(val))
            .try_for_each(|(i, &val)| {
                let pos = i / 4;
                let shift = (i % 4) * 8;
                let mut byte = val | INPUT_CHANGE_FLAG;
                if inverts[i] {
                    byte |= MONAURAL_INPUT_INVERT_FLAG;
                }
                let quad = (byte as u32) << shift;
                let offset = INPUT_GAIN_INVERT_OFFSET + pos * 4;
                write_quad(req, node, offset as u32, quad, timeout_ms)
            })
            .map(|_| state.gain.copy_from_slice(gain))
    }

    fn write_monaural_input_invert(
        req: &mut FwReq,
        node: &mut FwNode,
        invert: &[bool],
        state: &mut RegisterDspMonauralInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(invert.len(), Self::INPUT_COUNT);

        let gains = &state.gain;
        let prev_inverts = &state.invert;

        invert
            .iter()
            .enumerate()
            .filter(|&(i, val)| !prev_inverts[i].eq(val))
            .try_for_each(|(i, &val)| {
                let pos = i / 4;
                let shift = (i % 4) * 8;
                let mut byte = gains[i] | INPUT_CHANGE_FLAG;
                if val {
                    byte |= MONAURAL_INPUT_INVERT_FLAG;
                }
                let quad = (byte as u32) << shift;
                let offset = INPUT_GAIN_INVERT_OFFSET + pos * 4;
                write_quad(req, node, offset as u32, quad, timeout_ms)
            })
            .map(|_| state.invert.copy_from_slice(invert))
    }
}

/// The trait for operation of input in Audio Express and 4 pre.
pub trait RegisterDspStereoInputOperation {
    const INPUT_COUNT: usize = STEREO_INPUT_COUNT;
    const INPUT_PAIR_COUNT: usize = STEREO_INPUT_COUNT / 2;
    const MIC_COUNT: usize;

    const INPUT_GAIN_MIN: u8 = 0x00;
    const INPUT_MIC_GAIN_MAX: u8 = 0x3c;
    const INPUT_LINE_GAIN_MAX: u8 = 0x16;
    const INPUT_SPDIF_GAIN_MAX: u8 = 0x0c;
    const INPUT_GAIN_STEP: u8 = 0x01;

    fn create_stereo_input_state() -> RegisterDspStereoInputState {
        RegisterDspStereoInputState {
            gain: Default::default(),
            invert: Default::default(),
            paired: Default::default(),
            phantom: vec![false; Self::MIC_COUNT],
            pad: vec![false; Self::MIC_COUNT],
            jack: vec![false; Self::MIC_COUNT],
        }
    }

    fn parse_dsp_parameter(
        state: &mut RegisterDspStereoInputState,
        param: &SndMotuRegisterDspParameter,
    ) {
        let vals = param.input_gain_and_invert();
        state
            .gain
            .iter_mut()
            .zip(vals)
            .for_each(|(gain, val)| *gain = val & STEREO_INPUT_GAIN_MASK);
        state
            .invert
            .iter_mut()
            .zip(vals)
            .for_each(|(invert, val)| *invert = val & STEREO_INPUT_INVERT_FLAG > 0);

        let flags = param.input_flag();
        state
            .phantom
            .iter_mut()
            .zip(flags)
            .for_each(|(phantom, val)| *phantom = val & EV_MIC_PHANTOM_FLAG > 0);
        state
            .pad
            .iter_mut()
            .zip(flags)
            .for_each(|(pad, val)| *pad = val & EV_MIC_PAD_FLAG > 0);
        state
            .jack
            .iter_mut()
            .zip(flags)
            .for_each(|(jack, val)| *jack = val & EV_INPUT_JACK_FLAG > 0);
        state.paired.iter_mut().enumerate().for_each(|(i, paired)| {
            let pos = EV_INPUT_PAIRED_CH_MAP[i * 2];
            *paired = flags[pos] & EV_INPUT_PAIRED_FLAG > 0;
        });
    }

    fn parse_dsp_event(state: &mut RegisterDspStereoInputState, event: &RegisterDspEvent) -> bool {
        match event.ev_type {
            EV_TYPE_INPUT_GAIN_AND_INVERT => {
                let ch = event.identifier0 as usize;
                if ch < STEREO_INPUT_COUNT {
                    state.gain[ch] = event.value & STEREO_INPUT_GAIN_MASK;
                    state.invert[ch] = event.value & STEREO_INPUT_INVERT_FLAG > 0;
                    true
                } else {
                    false
                }
            }
            EV_TYPE_INPUT_FLAG => {
                let ch = event.identifier0 as usize;
                if let Some(pos) = EV_INPUT_PAIRED_CH_MAP.iter().position(|&p| p == ch) {
                    if pos % 2 == 0 {
                        state.paired[pos / 2] = event.value & EV_INPUT_PAIRED_FLAG > 0;
                    }
                    if pos < Self::MIC_COUNT {
                        state.phantom[ch] = event.value & EV_MIC_PHANTOM_FLAG > 0;
                        state.pad[ch] = event.value & EV_MIC_PAD_FLAG > 0;
                        state.jack[ch] = event.value & EV_INPUT_JACK_FLAG > 0;
                    }
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn read_stereo_input_state(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut RegisterDspStereoInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quads = vec![0; (Self::INPUT_COUNT + 3) / 4];
        quads.iter_mut().enumerate().try_for_each(|(i, quad)| {
            let offset = INPUT_GAIN_INVERT_OFFSET + i * 4;
            read_quad(req, node, offset as u32, timeout_ms).map(|val| *quad = val)
        })?;

        (0..Self::INPUT_COUNT).for_each(|i| {
            let pos = i / 4;
            let shift = (i % 4) * 8;
            let val = ((quads[pos] >> shift) as u8) & 0xff;

            state.gain[i] = val & STEREO_INPUT_GAIN_MASK;
            state.invert[i] = val & STEREO_INPUT_INVERT_FLAG > 0;
        });

        read_quad(req, node, MIC_PARAM_OFFSET as u32, timeout_ms).map(|quad| {
            (0..Self::MIC_COUNT).for_each(|i| {
                let val = (quad >> (i * 8)) as u8;
                state.phantom[i] = val & MIC_PARAM_PHANTOM_FLAG > 0;
                state.pad[i] = val & MIC_PARAM_PAD_FLAG > 0;
            });
        })?;

        read_quad(req, node, INPUT_PAIRED_OFFSET as u32, timeout_ms).map(|quad| {
            // MEMO: The flag is put from LSB to MSB in its order.
            state.paired.iter_mut().enumerate().for_each(|(i, paired)| {
                *paired = ((quad >> (i * 8)) as u8) & INPUT_PAIRED_FLAG > 0
            });
        })?;

        Ok(())
    }

    fn write_stereo_input_gain(
        req: &mut FwReq,
        node: &mut FwNode,
        gain: &[u8],
        state: &mut RegisterDspStereoInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(gain.len(), Self::INPUT_COUNT);

        let prev_gains = &state.gain;
        let inverts = &state.invert;

        gain.iter()
            .enumerate()
            .filter(|&(i, val)| !prev_gains[i].eq(val))
            .try_for_each(|(i, &val)| {
                let pos = i / 4;
                let shift = (i % 4) * 8;
                let mut byte = val | INPUT_CHANGE_FLAG;
                if inverts[i] {
                    byte |= STEREO_INPUT_INVERT_FLAG;
                }
                let quad = (byte as u32) << shift;
                let offset = INPUT_GAIN_INVERT_OFFSET + pos * 4;
                write_quad(req, node, offset as u32, quad, timeout_ms)
            })
            .map(|_| state.gain.copy_from_slice(gain))
    }

    fn write_stereo_input_invert(
        req: &mut FwReq,
        node: &mut FwNode,
        invert: &[bool],
        state: &mut RegisterDspStereoInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(invert.len(), Self::INPUT_COUNT);

        let gains = &state.gain;
        let prev_inverts = &state.invert;

        invert
            .iter()
            .enumerate()
            .filter(|&(i, val)| !prev_inverts[i].eq(val))
            .try_for_each(|(i, &val)| {
                let pos = i / 4;
                let shift = (i % 4) * 8;
                let mut byte = gains[i] | INPUT_CHANGE_FLAG;
                if val {
                    byte |= STEREO_INPUT_INVERT_FLAG;
                }
                let quad = (byte as u32) << shift;
                let offset = INPUT_GAIN_INVERT_OFFSET + pos * 4;
                write_quad(req, node, offset as u32, quad, timeout_ms)
            })
            .map(|_| state.invert.copy_from_slice(invert))
    }

    fn write_stereo_input_paired(
        req: &mut FwReq,
        node: &mut FwNode,
        paired: &[bool],
        state: &mut RegisterDspStereoInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(paired.len(), STEREO_INPUT_COUNT / 2);

        paired
            .iter()
            .enumerate()
            .filter(|&(i, val)| !state.paired[i].eq(val))
            .try_for_each(|(i, &paired)| {
                // MEMO: 0x00ff0000 mask is absent.
                let shift = INPUT_PAIRED_CH_MAP[i] * 8;
                let mut val = INPUT_PAIRED_CHANGE_FLAG;
                if paired {
                    val |= INPUT_PAIRED_FLAG;
                }
                let quad = (val as u32) << shift;
                write_quad(req, node, INPUT_PAIRED_OFFSET as u32, quad, timeout_ms)
            })
            .map(|_| state.paired.copy_from_slice(paired))
    }

    fn write_mic_phantom(
        req: &mut FwReq,
        node: &mut FwNode,
        phantom: &[bool],
        state: &mut RegisterDspStereoInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(phantom.len(), Self::MIC_COUNT);

        let val = phantom
            .iter()
            .enumerate()
            .filter(|&(i, p)| !state.phantom[i].eq(p))
            .fold(0u32, |val, (i, &p)| {
                let mut v = MIC_PARAM_CHANGE_FLAG;
                if p {
                    v |= MIC_PARAM_PHANTOM_FLAG;
                }
                if state.pad[i] {
                    v |= MIC_PARAM_PAD_FLAG;
                }
                val | ((v as u32) << (i * 8))
            });

        write_quad(req, node, MIC_PARAM_OFFSET as u32, val, timeout_ms).map(|_| {
            state.phantom.copy_from_slice(phantom);
        })
    }

    fn write_mic_pad(
        req: &mut FwReq,
        node: &mut FwNode,
        pad: &[bool],
        state: &mut RegisterDspStereoInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(pad.len(), Self::MIC_COUNT);

        let val = pad
            .iter()
            .enumerate()
            .filter(|&(i, p)| !state.pad[i].eq(p))
            .fold(0u32, |val, (i, &p)| {
                let mut v = MIC_PARAM_CHANGE_FLAG;
                if state.phantom[i] {
                    v |= MIC_PARAM_PHANTOM_FLAG;
                }
                if p {
                    v |= MIC_PARAM_PAD_FLAG;
                }
                val | ((v as u32) << (i * 8))
            });

        write_quad(req, node, MIC_PARAM_OFFSET as u32, val, timeout_ms).map(|_| {
            state.pad.copy_from_slice(pad);
        })
    }
}

/// Information of meter.
#[derive(Default, Debug)]
pub struct RegisterDspMeterState {
    pub inputs: Vec<u8>,
    pub outputs: Vec<u8>,
    pub selected: Option<usize>,
}

// Read-only.
const METER_OUTPUT_SELECT_OFFSET: usize = 0x0b2c;
const METER_OUTPUT_SELECT_TARGET_MASK: u32 = 0x000000ff;
const METER_OUTPUT_SELECT_CHANGE_FLAG: u32 = 0x00000b00;

// Assertion from UAPI of ALSA firewire stack.
const MAX_METER_INPUT_COUNT: usize = 24;
const MAX_METER_OUTPUT_COUNT: usize = 48;

/// The trait for meter operation.
pub trait RegisterDspMeterOperation {
    const SELECTABLE: bool;
    const INPUT_PORTS: &'static [TargetPort];
    const OUTPUT_PORT_PAIRS: &'static [(TargetPort, [usize; 2])];
    const OUTPUT_PORT_COUNT: usize = Self::OUTPUT_PORT_PAIRS.len() * 2;

    const LEVEL_MIN: u8 = 0;
    const LEVEL_MAX: u8 = 0x7f;
    const LEVEL_STEP: u8 = 1;

    fn create_meter_state() -> RegisterDspMeterState {
        // Assertion from UAPI of ALSA firewire stack.
        assert!(Self::INPUT_PORTS.len() <= MAX_METER_INPUT_COUNT);
        assert!(Self::OUTPUT_PORT_PAIRS.len() <= MAX_METER_OUTPUT_COUNT);

        RegisterDspMeterState {
            inputs: vec![0; Self::INPUT_PORTS.len()],
            outputs: vec![0; Self::OUTPUT_PORT_COUNT],
            selected: if Self::SELECTABLE { Some(0) } else { None },
        }
    }

    fn select_output(
        req: &mut FwReq,
        node: &mut FwNode,
        target: usize,
        meter: &mut RegisterDspMeterState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(target < Self::OUTPUT_PORT_COUNT);

        if let Some(t) = &mut meter.selected {
            let mut quad = ((target + 1) as u32) & METER_OUTPUT_SELECT_TARGET_MASK;
            quad |= METER_OUTPUT_SELECT_CHANGE_FLAG;
            write_quad(
                req,
                node,
                METER_OUTPUT_SELECT_OFFSET as u32,
                quad,
                timeout_ms,
            )
            .map(|_| *t = target)
        } else {
            Err(Error::new(FileError::Nxio, "Not supported"))
        }
    }

    fn parse_dsp_meter(state: &mut RegisterDspMeterState, data: &[u8]) {
        state
            .inputs
            .copy_from_slice(&data[..Self::INPUT_PORTS.len()]);

        if let Some(selected) = state.selected {
            state.outputs.iter_mut().for_each(|meter| *meter = 0);

            let pair = &Self::OUTPUT_PORT_PAIRS[selected].1;
            state.outputs[selected * 2] = data[MAX_METER_INPUT_COUNT + pair[0]];
            state.outputs[selected * 2 + 1] = data[MAX_METER_INPUT_COUNT + pair[1]];
        } else {
            Self::OUTPUT_PORT_PAIRS
                .iter()
                .flat_map(|(_, pair)| pair)
                .zip(&mut state.outputs)
                .for_each(|(pos, m)| *m = data[MAX_METER_INPUT_COUNT + pos]);
        }
    }
}
