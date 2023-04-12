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

/// State of sources in mixer entiry which can be operated as monaural channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterDspMixerMonauralSourceEntry {
    /// The gain of source. The value is between 0x00 and 0x80.
    pub gain: Vec<u8>,
    /// The left and right balance of source to stereo mixer. The value is between 0x00 and 0x80.
    pub pan: Vec<u8>,
    /// Whether to mute the source.
    pub mute: Vec<bool>,
    /// Whether to mute the other sources.
    pub solo: Vec<bool>,
}

/// State of mixer sources which can be operated as monaural channel.
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// The trait for specification of mixer sources which can be operated as monaural channel.
pub trait MotuRegisterDspMixerMonauralSourceSpecification: MotuRegisterDspSpecification {
    /// The port of mixer sources.
    const MIXER_SOURCES: &'static [TargetPort];

    /// The minimum value of gain for mixer source.
    const SOURCE_GAIN_MIN: u8 = 0x00;
    /// The maximum value of gain for mixer source.
    const SOURCE_GAIN_MAX: u8 = 0x80;
    /// The step value of gain for mixer source.
    const SOURCE_GAIN_STEP: u8 = 0x01;

    /// The minimum value of left and right balance for mixer source.
    const SOURCE_PAN_MIN: u8 = 0x00;
    /// The maximum value of left and right balance for mixer source.
    const SOURCE_PAN_MAX: u8 = 0x80;
    /// The step value of left and right balance for mixer source.
    const SOURCE_PAN_STEP: u8 = 0x01;

    fn create_mixer_monaural_source_state() -> RegisterDspMixerMonauralSourceState {
        let entry = RegisterDspMixerMonauralSourceEntry {
            gain: vec![Default::default(); Self::MIXER_SOURCES.len()],
            pan: vec![Default::default(); Self::MIXER_SOURCES.len()],
            mute: vec![Default::default(); Self::MIXER_SOURCES.len()],
            solo: vec![Default::default(); Self::MIXER_SOURCES.len()],
        };
        RegisterDspMixerMonauralSourceState([
            entry.clone(),
            entry.clone(),
            entry.clone(),
            entry.clone(),
        ])
    }
}

impl<O> MotuWhollyCacheableParamsOperation<RegisterDspMixerMonauralSourceState> for O
where
    O: MotuRegisterDspMixerMonauralSourceSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut RegisterDspMixerMonauralSourceState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        params
            .0
            .iter_mut()
            .zip(MIXER_SOURCE_OFFSETS)
            .try_for_each(|(entry, offset)| {
                (0..Self::MIXER_SOURCES.len()).try_for_each(|i| {
                    read_quad(req, node, (offset + i * 4) as u32, timeout_ms).map(|quad| {
                        entry.gain[i] = (quad & MIXER_SOURCE_GAIN_MASK) as u8;
                        entry.pan[i] = ((quad & MIXER_SOURCE_PAN_MASK) >> 8) as u8;
                        entry.mute[i] = quad & MIXER_SOURCE_MUTE_FLAG > 0;
                        entry.solo[i] = quad & MIXER_SOURCE_SOLO_FLAG > 0;
                    })
                })
            })
    }
}

impl<O> MotuPartiallyUpdatableParamsOperation<RegisterDspMixerMonauralSourceState> for O
where
    O: MotuRegisterDspMixerMonauralSourceSpecification,
{
    fn update_partially(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut RegisterDspMixerMonauralSourceState,
        updates: RegisterDspMixerMonauralSourceState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        params
            .0
            .iter_mut()
            .zip(&updates.0)
            .zip(MIXER_SOURCE_OFFSETS)
            .try_for_each(|((current, update), mixer_offset)| {
                (0..Self::MIXER_SOURCES.len()).try_for_each(|i| {
                    if current.gain[i] != update.gain[i]
                        || current.pan[i] != update.pan[i]
                        || current.mute[i] != update.mute[i]
                        || current.solo[i] != update.solo[i]
                    {
                        let offset = (mixer_offset + i * 4) as u32;
                        let mut quad = read_quad(req, node, offset, timeout_ms)?;
                        if current.gain[i] != update.gain[i] {
                            quad &= !MIXER_SOURCE_GAIN_MASK;
                            quad |= update.gain[i] as u32;
                            quad |= MIXER_SOURCE_GAIN_CHANGE_FLAG;
                        }
                        if current.pan[i] != update.pan[i] {
                            quad &= !MIXER_SOURCE_PAN_MASK;
                            quad |= (update.pan[i] as u32) << 8;
                            quad |= MIXER_SOURCE_PAN_CHANGE_FLAG;
                        }
                        if current.mute[i] != update.mute[i] {
                            quad &= !MIXER_SOURCE_MUTE_FLAG;
                            if update.mute[i] {
                                quad |= MIXER_SOURCE_MUTE_FLAG;
                            }
                        }
                        if current.solo[i] != update.solo[i] {
                            quad &= !MIXER_SOURCE_SOLO_FLAG;
                            if update.solo[i] {
                                quad |= MIXER_SOURCE_SOLO_FLAG;
                            }
                        }
                        write_quad(req, node, offset, quad, timeout_ms).map(|_| {
                            current.gain[i] = update.gain[i];
                            current.pan[i] = update.pan[i];
                            current.mute[i] = update.mute[i];
                            current.solo[i] = update.solo[i];
                        })
                    } else {
                        Ok(())
                    }
                })
            })
    }
}

impl<O>
    MotuRegisterDspImageOperation<RegisterDspMixerMonauralSourceState, SndMotuRegisterDspParameter>
    for O
where
    O: MotuRegisterDspMixerMonauralSourceSpecification,
{
    fn parse_image(
        params: &mut RegisterDspMixerMonauralSourceState,
        image: &SndMotuRegisterDspParameter,
    ) {
        params.0.iter_mut().enumerate().for_each(|(i, src)| {
            let gains = image.mixer_source_gain(i);
            src.gain
                .iter_mut()
                .zip(gains)
                .for_each(|(dst, src)| *dst = *src);

            let pans = image.mixer_source_pan(i);
            src.pan
                .iter_mut()
                .zip(pans)
                .for_each(|(dst, src)| *dst = *src);

            let flags: Vec<u32> = image
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
}

impl<O> MotuRegisterDspEventOperation<RegisterDspMixerMonauralSourceState> for O
where
    O: MotuRegisterDspMixerMonauralSourceSpecification,
{
    fn parse_event(
        params: &mut RegisterDspMixerMonauralSourceState,
        event: &RegisterDspEvent,
    ) -> bool {
        match event.ev_type {
            EV_TYPE_MIXER_SRC_GAIN => {
                let mixer = event.identifier0 as usize;
                let src = event.identifier1 as usize;
                let val = event.value;

                if mixer < MIXER_COUNT && src < Self::MIXER_SOURCES.len() {
                    params.0[mixer].gain[src] = val;
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
                    params.0[mixer].pan[src] = val;
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
                    params.0[mixer].mute[src] = val & MIXER_SOURCE_MUTE_FLAG > 0;
                    params.0[mixer].solo[src] = val & MIXER_SOURCE_SOLO_FLAG > 0;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

const MIXER_STEREO_SOURCE_COUNT: usize = 6;
const MIXER_STEREO_SOURCE_PAIR_COUNT: usize = MIXER_STEREO_SOURCE_COUNT / 2;

/// State of sources in mixer entiry.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RegisterDspMixerStereoSourceEntry {
    /// The gain of mixer sources.
    pub gain: [u8; MIXER_STEREO_SOURCE_COUNT],
    /// The left and right panning of mixer sources.
    pub pan: [u8; MIXER_STEREO_SOURCE_COUNT],
    /// Whether to mute mixer sources.
    pub mute: [bool; MIXER_STEREO_SOURCE_COUNT],
    /// Whether to mute the other mixer sources.
    pub solo: [bool; MIXER_STEREO_SOURCE_COUNT],
    /// The left and right balance of paired mixer sources.
    pub balance: [u8; MIXER_STEREO_SOURCE_PAIR_COUNT],
    /// The left and right width of paired mixer sources.
    pub width: [u8; MIXER_STEREO_SOURCE_PAIR_COUNT],
}

/// State of mixer sources.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RegisterDspMixerStereoSourceState(pub [RegisterDspMixerStereoSourceEntry; MIXER_COUNT]);

const MIXER_SOURCE_PAIRED_WIDTH_FLAG: u32 = 0x00400000;
const MIXER_SOURCE_PAIRED_BALANCE_FLAG: u32 = 0x00800000;

const EV_MIXER_SOURCE_PAIRED_CH_MAP: [usize; MIXER_STEREO_SOURCE_COUNT] = [0, 1, 2, 3, 8, 9];

/// The trait for specification of mixer sources.
pub trait MotuRegisterDspMixerStereoSourceSpecification: MotuRegisterDspSpecification {
    /// The ports of mixer sources.
    const MIXER_SOURCES: [TargetPort; MIXER_STEREO_SOURCE_COUNT] = [
        TargetPort::Analog(0),
        TargetPort::Analog(1),
        TargetPort::Analog(2),
        TargetPort::Analog(3),
        TargetPort::Spdif(0),
        TargetPort::Spdif(1),
    ];

    /// The number of paired mixer sources.
    const MIXER_SOURCE_PAIR_COUNT: usize = Self::MIXER_SOURCES.len() / 2;

    /// The minimum value of gain for mixer source.
    const SOURCE_GAIN_MIN: u8 = 0x00;
    /// The maximum value of gain for mixer source.
    const SOURCE_GAIN_MAX: u8 = 0x80;
    /// The step value of gain for mixer source.
    const SOURCE_GAIN_STEP: u8 = 0x01;

    /// The minimum value of left and right balance for mixer source.
    const SOURCE_PAN_MIN: u8 = 0x00;
    /// The maximum value of left and right balance for mixer source.
    const SOURCE_PAN_MAX: u8 = 0x80;
    /// The step value of left and right balance for mixer source.
    const SOURCE_PAN_STEP: u8 = 0x01;

    /// The minimum value of left and right balance for paired mixer source.
    const SOURCE_STEREO_BALANCE_MIN: u8 = 0x00;
    /// The maximum value of left and right balance for paired mixer source.
    const SOURCE_STEREO_BALANCE_MAX: u8 = 0x80;
    /// The step value of left and right balance for paired mixer source.
    const SOURCE_STEREO_BALANCE_STEP: u8 = 0x01;

    /// The minimum value of left and right width for paired mixer source.
    const SOURCE_STEREO_WIDTH_MIN: u8 = 0x00;
    /// The maximum value of left and right width for paired mixer source.
    const SOURCE_STEREO_WIDTH_MAX: u8 = 0x80;
    /// The step value of left and right width for paired mixer source.
    const SOURCE_STEREO_WIDTH_STEP: u8 = 0x01;
}

fn compute_mixer_stereo_source_offset(base_offset: usize, src_idx: usize) -> usize {
    base_offset + 4 * if src_idx < 4 { src_idx } else { src_idx + 4 }
}

// MEMO: no register to read balance and width of mixer source. They are just expressed in DSP
// events.
impl<O> MotuWhollyCacheableParamsOperation<RegisterDspMixerStereoSourceState> for O
where
    O: MotuRegisterDspMixerStereoSourceSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut RegisterDspMixerStereoSourceState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        params.0.iter_mut().enumerate().try_for_each(|(i, entry)| {
            let base_offset = MIXER_SOURCE_OFFSETS[i];
            (0..Self::MIXER_SOURCES.len()).try_for_each(|j| {
                let offset = compute_mixer_stereo_source_offset(base_offset, j) as u32;
                read_quad(req, node, offset, timeout_ms).map(|val| {
                    entry.gain[j] = (val & MIXER_SOURCE_GAIN_MASK) as u8;
                    entry.mute[j] = ((val & MIXER_SOURCE_MUTE_FLAG) >> 8) > 0;
                    entry.solo[j] = ((val & MIXER_SOURCE_SOLO_FLAG) >> 16) > 0;
                })
            })
        })?;

        Ok(())
    }
}

impl<O> MotuPartiallyUpdatableParamsOperation<RegisterDspMixerStereoSourceState> for O
where
    O: MotuRegisterDspMixerStereoSourceSpecification,
{
    fn update_partially(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut RegisterDspMixerStereoSourceState,
        updates: RegisterDspMixerStereoSourceState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        params
            .0
            .iter_mut()
            .zip(&updates.0)
            .zip(MIXER_SOURCE_OFFSETS)
            .try_for_each(|((current, update), base_offset)| {
                (0..Self::MIXER_SOURCES.len()).try_for_each(|i| {
                    if current.gain[i] != update.gain[i]
                        || current.pan[i] != update.pan[i]
                        || current.mute[i] != update.mute[i]
                        || current.solo[i] != update.solo[i]
                    {
                        let offset = compute_mixer_stereo_source_offset(base_offset, i) as u32;
                        let mut quad = read_quad(req, node, offset, timeout_ms)?;

                        if current.gain[i] != update.gain[i] {
                            quad &= !MIXER_SOURCE_GAIN_MASK;
                            quad |= update.gain[i] as u32;
                        }

                        if current.pan[i] != update.pan[i] {
                            quad &= !MIXER_SOURCE_PAN_MASK;
                            quad |= (update.pan[i] as u32) << 8;
                            quad |= MIXER_SOURCE_PAN_CHANGE_FLAG;
                        }

                        if current.mute[i] != update.mute[i] {
                            quad &= !MIXER_SOURCE_MUTE_FLAG;
                            if update.mute[i] {
                                quad |= MIXER_SOURCE_MUTE_FLAG;
                            }
                        }

                        if current.solo[i] != update.solo[i] {
                            quad &= !MIXER_SOURCE_SOLO_FLAG;
                            if update.solo[i] {
                                quad |= MIXER_SOURCE_SOLO_FLAG;
                            }
                        }

                        write_quad(req, node, offset, quad, timeout_ms).map(|_| {
                            current.gain[i] = update.gain[i];
                            current.pan[i] = update.pan[i];
                            current.mute[i] = update.mute[i];
                            current.solo[i] = update.solo[i];
                        })
                    } else {
                        Ok(())
                    }
                })?;

                (0..Self::MIXER_SOURCE_PAIR_COUNT).try_for_each(|i| {
                    if current.balance[i] != update.balance[i]
                        || current.width[i] != update.width[i]
                    {
                        let offset = compute_mixer_stereo_source_offset(base_offset, i) as u32;
                        let mut quad = 0;

                        if current.balance[i] != update.balance[i] {
                            quad |= MIXER_SOURCE_PAN_CHANGE_FLAG;
                            quad |= MIXER_SOURCE_PAIRED_WIDTH_FLAG;
                            quad |= (update.balance[i] as u32) << 8;
                        }

                        if current.width[i] != update.width[i] {
                            quad |= MIXER_SOURCE_PAN_CHANGE_FLAG;
                            quad |= MIXER_SOURCE_PAIRED_BALANCE_FLAG;
                            quad |= (update.width[i] as u32) << 8;
                        }

                        write_quad(req, node, offset, quad, timeout_ms).map(|_| {
                            current.balance[i] = update.balance[i];
                            current.width[i] = update.width[i];
                        })
                    } else {
                        Ok(())
                    }
                })
            })
    }
}

impl<O>
    MotuRegisterDspImageOperation<RegisterDspMixerStereoSourceState, SndMotuRegisterDspParameter>
    for O
where
    O: MotuRegisterDspMixerStereoSourceSpecification,
{
    fn parse_image(
        params: &mut RegisterDspMixerStereoSourceState,
        image: &SndMotuRegisterDspParameter,
    ) {
        params.0.iter_mut().enumerate().for_each(|(i, src)| {
            let gains = image.mixer_source_gain(i);
            src.gain
                .iter_mut()
                .zip(gains)
                .for_each(|(dst, src)| *dst = *src);

            let pans = image.mixer_source_pan(i);
            src.pan
                .iter_mut()
                .zip(pans)
                .for_each(|(dst, src)| *dst = *src);

            let flags: Vec<u32> = image
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
}

impl<O> MotuRegisterDspEventOperation<RegisterDspMixerStereoSourceState> for O
where
    O: MotuRegisterDspMixerStereoSourceSpecification,
{
    fn parse_event(
        params: &mut RegisterDspMixerStereoSourceState,
        event: &RegisterDspEvent,
    ) -> bool {
        match event.ev_type {
            EV_TYPE_MIXER_SRC_GAIN => {
                let mixer = event.identifier0 as usize;
                let src = event.identifier1 as usize;
                let val = event.value;

                if mixer < MIXER_COUNT && src < Self::MIXER_SOURCES.len() {
                    params.0[mixer].gain[src] = val;
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
                    params.0[mixer].pan[src] = val;
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
                    params.0[mixer].mute[src] = val & MIXER_SOURCE_MUTE_FLAG > 0;
                    params.0[mixer].solo[src] = val & MIXER_SOURCE_SOLO_FLAG > 0;
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
                    params.0[mixer].balance[idx / 2] = val;
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
                    params.0[mixer].width[idx / 2] = val;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
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

/// State of inputs in 828mkII and Traveler.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct RegisterDspLineInputState {
    /// The nominal level of input signal.
    pub level: Vec<NominalSignalLevel>,
    /// Apply +6dB.
    pub boost: Vec<bool>,
}

const LINE_INPUT_LEVEL_OFFSET: usize = 0x0c08;
const LINE_INPUT_BOOST_OFFSET: usize = 0x0c14;

/// The trait for specification of line input.
pub trait MotuRegisterDspLineInputSpecification: MotuRegisterDspSpecification {
    const LINE_INPUT_COUNT: usize;
    const CH_OFFSET: usize;

    fn create_line_input_state() -> RegisterDspLineInputState {
        RegisterDspLineInputState {
            level: vec![Default::default(); Self::LINE_INPUT_COUNT],
            boost: vec![Default::default(); Self::LINE_INPUT_COUNT],
        }
    }
}

impl<O> MotuWhollyCacheableParamsOperation<RegisterDspLineInputState> for O
where
    O: MotuRegisterDspLineInputSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut RegisterDspLineInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        read_quad(req, node, LINE_INPUT_LEVEL_OFFSET as u32, timeout_ms).map(|val| {
            params
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
            params
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
}

impl<O> MotuPartiallyUpdatableParamsOperation<RegisterDspLineInputState> for O
where
    O: MotuRegisterDspLineInputSpecification,
{
    fn update_partially(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut RegisterDspLineInputState,
        updates: RegisterDspLineInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if params.level != updates.level {
            let quad = updates
                .level
                .iter()
                .enumerate()
                .fold(0u32, |mut quad, (i, l)| {
                    if NominalSignalLevel::Professional.eq(l) {
                        quad |= 1 << (i + Self::CH_OFFSET);
                    }
                    quad
                });
            write_quad(req, node, LINE_INPUT_LEVEL_OFFSET as u32, quad, timeout_ms)?;
            params.level.copy_from_slice(&updates.level);
        }

        if params.boost != updates.boost {
            let quad = updates
                .boost
                .iter()
                .enumerate()
                .fold(0u32, |mut quad, (mut i, b)| {
                    i += Self::CH_OFFSET;
                    if *b {
                        quad |= 1 << i;
                    }
                    quad
                });

            write_quad(req, node, LINE_INPUT_BOOST_OFFSET as u32, quad, timeout_ms)?;
            params.boost.copy_from_slice(&updates.boost);
        }

        Ok(())
    }
}

impl<O> MotuRegisterDspImageOperation<RegisterDspLineInputState, SndMotuRegisterDspParameter> for O
where
    O: MotuRegisterDspLineInputSpecification,
{
    fn parse_image(params: &mut RegisterDspLineInputState, image: &SndMotuRegisterDspParameter) {
        let flags = image.line_input_nominal_level_flag();
        params.level.iter_mut().enumerate().for_each(|(i, level)| {
            let shift = i + Self::CH_OFFSET;
            *level = if flags & (1 << shift) > 0 {
                NominalSignalLevel::Professional
            } else {
                NominalSignalLevel::Consumer
            };
        });

        let flags = image.line_input_boost_flag();
        params.boost.iter_mut().enumerate().for_each(|(i, boost)| {
            let shift = i + Self::CH_OFFSET;
            *boost = flags & (1 << shift) > 0;
        });
    }
}

impl<O> MotuRegisterDspEventOperation<RegisterDspLineInputState> for O
where
    O: MotuRegisterDspLineInputSpecification,
{
    fn parse_event(params: &mut RegisterDspLineInputState, event: &RegisterDspEvent) -> bool {
        match event.ev_type {
            EV_TYPE_LINE_INPUT_NOMINAL_LEVEL => {
                params.level.iter_mut().enumerate().for_each(|(i, level)| {
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
                params.boost.iter_mut().enumerate().for_each(|(i, boost)| {
                    let shift = i + Self::CH_OFFSET;
                    *boost = event.value & (1 << shift) > 0;
                });
                true
            }
            _ => false,
        }
    }
}

const MONAURAL_INPUT_COUNT: usize = 10;

/// State of input in Ultralite.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RegisterDspMonauralInputState {
    /// The gain of inputs.
    pub gain: [u8; MONAURAL_INPUT_COUNT],
    /// Whether to invert signal of inputs.
    pub invert: [bool; MONAURAL_INPUT_COUNT],
}

const STEREO_INPUT_COUNT: usize = 6;

/// State of input in Audio Express, and 4 pre.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct RegisterDspStereoInputState {
    /// The gain of inputs.
    pub gain: [u8; STEREO_INPUT_COUNT],
    /// Whether to invert signal of inputs.
    pub invert: [bool; STEREO_INPUT_COUNT],
    /// Whether the stereo channels are paired.
    pub paired: [bool; STEREO_INPUT_COUNT / 2],
    /// Whether to enable phantom powering.
    pub phantom: Vec<bool>,
    /// Whether to attenate inputs.
    pub pad: Vec<bool>,
    /// Whether jack is inserted.
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

/// The trait for specification of monaural inputs.
pub trait MotuRegisterDspMonauralInputSpecification {
    /// The number of inputs.
    const INPUT_COUNT: usize = MONAURAL_INPUT_COUNT;

    /// The minimum value of gain.
    const INPUT_GAIN_MIN: u8 = 0x00;
    /// The maximum value of gain for microphone inputs.
    const INPUT_MIC_GAIN_MAX: u8 = 0x18;
    /// The maximum value of gain for line inputs.
    const INPUT_LINE_GAIN_MAX: u8 = 0x12;
    /// The maximum value of gain for S/PDIF inputs.
    const INPUT_SPDIF_GAIN_MAX: u8 = 0x0c;
    /// The step value of gain.
    const INPUT_GAIN_STEP: u8 = 0x01;
}

impl<O> MotuWhollyCacheableParamsOperation<RegisterDspMonauralInputState> for O
where
    O: MotuRegisterDspMonauralInputSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut RegisterDspMonauralInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quads = [0; (MONAURAL_INPUT_COUNT + 3) / 4];
        quads.iter_mut().enumerate().try_for_each(|(i, quad)| {
            let offset = INPUT_GAIN_INVERT_OFFSET + i * 4;
            read_quad(req, node, offset as u32, timeout_ms).map(|val| *quad = val)
        })?;

        (0..MONAURAL_INPUT_COUNT).for_each(|i| {
            let pos = i / 4;
            let shift = (i % 4) * 8;
            let val = ((quads[pos] >> shift) as u8) & 0xff;

            params.gain[i] = val & MONAURAL_INPUT_GAIN_MASK;
            params.invert[i] = val & MONAURAL_INPUT_INVERT_FLAG > 0;
        });

        Ok(())
    }
}

impl<O> MotuPartiallyUpdatableParamsOperation<RegisterDspMonauralInputState> for O
where
    O: MotuRegisterDspMonauralInputSpecification,
{
    fn update_partially(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut RegisterDspMonauralInputState,
        updates: RegisterDspMonauralInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        (0..MONAURAL_INPUT_COUNT).try_for_each(|i| {
            if params.gain[i] != updates.gain[i] || params.invert[i] != updates.invert[i] {
                let pos = i / 4;
                let shift = (i % 4) * 8;
                let mut byte = INPUT_CHANGE_FLAG;
                if params.gain[i] != updates.gain[i] {
                    byte |= updates.gain[i];
                }
                if params.invert[i] != updates.invert[i] {
                    if updates.invert[i] {
                        byte |= MONAURAL_INPUT_INVERT_FLAG;
                    }
                }
                let quad = (byte as u32) << shift;
                let offset = (INPUT_GAIN_INVERT_OFFSET + pos * 4) as u32;
                write_quad(req, node, offset, quad, timeout_ms).map(|_| {
                    params.gain[i] = updates.gain[i];
                    params.invert[i] = updates.invert[i];
                })
            } else {
                Ok(())
            }
        })
    }
}

impl<O> MotuRegisterDspImageOperation<RegisterDspMonauralInputState, SndMotuRegisterDspParameter>
    for O
where
    O: MotuRegisterDspMonauralInputSpecification,
{
    fn parse_image(
        params: &mut RegisterDspMonauralInputState,
        image: &SndMotuRegisterDspParameter,
    ) {
        let vals = image.input_gain_and_invert();
        params
            .gain
            .iter_mut()
            .zip(vals)
            .for_each(|(gain, val)| *gain = val & MONAURAL_INPUT_GAIN_MASK);
        params
            .invert
            .iter_mut()
            .zip(vals)
            .for_each(|(invert, val)| *invert = val & MONAURAL_INPUT_INVERT_FLAG > 0);
    }
}

impl<O> MotuRegisterDspEventOperation<RegisterDspMonauralInputState> for O
where
    O: MotuRegisterDspMonauralInputSpecification,
{
    fn parse_event(params: &mut RegisterDspMonauralInputState, event: &RegisterDspEvent) -> bool {
        match event.ev_type {
            EV_TYPE_INPUT_GAIN_AND_INVERT => {
                let ch = event.identifier0 as usize;
                if ch < MONAURAL_INPUT_COUNT {
                    params.gain[ch] = event.value & MONAURAL_INPUT_GAIN_MASK;
                    params.invert[ch] = event.value & MONAURAL_INPUT_INVERT_FLAG > 0;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

/// The trait for specification of stereo inputs in Audio Express and 4 pre.
pub trait MotuRegisterDspStereoInputSpecification: MotuRegisterDspSpecification {
    /// The number of inputs.
    const INPUT_COUNT: usize = STEREO_INPUT_COUNT;

    /// The number of input pairs.
    const INPUT_PAIR_COUNT: usize = STEREO_INPUT_COUNT / 2;

    /// The number of microphone inputs.
    const MIC_COUNT: usize;

    /// The minimum value of gain.
    const INPUT_GAIN_MIN: u8 = 0x00;
    /// The maximum value of gain for microphone inputs.
    const INPUT_MIC_GAIN_MAX: u8 = 0x3c;
    /// The maximum value of gain for line inputs.
    const INPUT_LINE_GAIN_MAX: u8 = 0x16;
    /// The maximum value of gain for S/PDIF inputs.
    const INPUT_SPDIF_GAIN_MAX: u8 = 0x0c;
    /// The step value of gain.
    const INPUT_GAIN_STEP: u8 = 0x01;

    fn create_stereo_input_state() -> RegisterDspStereoInputState {
        RegisterDspStereoInputState {
            gain: Default::default(),
            invert: Default::default(),
            paired: Default::default(),
            phantom: vec![Default::default(); Self::MIC_COUNT],
            pad: vec![Default::default(); Self::MIC_COUNT],
            jack: vec![Default::default(); Self::MIC_COUNT],
        }
    }
}

impl<O> MotuWhollyCacheableParamsOperation<RegisterDspStereoInputState> for O
where
    O: MotuRegisterDspStereoInputSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut RegisterDspStereoInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        (0..Self::INPUT_COUNT).step_by(4).try_for_each(|i| {
            let offset = (INPUT_GAIN_INVERT_OFFSET + i) as u32;
            read_quad(req, node, offset, timeout_ms).map(|quad| {
                params.gain[i..]
                    .iter_mut()
                    .take(4)
                    .enumerate()
                    .for_each(|(j, gain)| {
                        let shift = j * 8;
                        let val = ((quad >> shift) as u8) & 0xff;
                        *gain = val & STEREO_INPUT_GAIN_MASK;
                    });
                params.invert[i..]
                    .iter_mut()
                    .take(4)
                    .enumerate()
                    .for_each(|(j, invert)| {
                        let shift = j * 8;
                        let val = ((quad >> shift) as u8) & 0xff;
                        *invert = val & STEREO_INPUT_INVERT_FLAG > 0;
                    });
            })
        })?;

        read_quad(req, node, MIC_PARAM_OFFSET as u32, timeout_ms).map(|quad| {
            (0..Self::MIC_COUNT).for_each(|i| {
                let val = (quad >> (i * 8)) as u8;
                params.phantom[i] = val & MIC_PARAM_PHANTOM_FLAG > 0;
                params.pad[i] = val & MIC_PARAM_PAD_FLAG > 0;
            });
        })?;

        read_quad(req, node, INPUT_PAIRED_OFFSET as u32, timeout_ms).map(|quad| {
            // MEMO: The flag is put from LSB to MSB in its order.
            params
                .paired
                .iter_mut()
                .enumerate()
                .for_each(|(i, paired)| {
                    *paired = ((quad >> (i * 8)) as u8) & INPUT_PAIRED_FLAG > 0
                });
        })?;

        Ok(())
    }
}

impl<O> MotuPartiallyUpdatableParamsOperation<RegisterDspStereoInputState> for O
where
    O: MotuRegisterDspStereoInputSpecification,
{
    fn update_partially(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut RegisterDspStereoInputState,
        updates: RegisterDspStereoInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        (0..Self::INPUT_COUNT).try_for_each(|i| {
            if params.gain[i] != updates.gain[i] || params.invert[i] != updates.invert[i] {
                let pos = i / 4;
                let shift = (i % 4) * 8;

                let mut byte = 0;
                if params.gain[i] != updates.gain[i] {
                    byte |= updates.gain[i] | INPUT_CHANGE_FLAG;
                }

                if params.invert[i] != updates.invert[i] {
                    if updates.invert[i] {
                        byte |= STEREO_INPUT_INVERT_FLAG;
                    }
                }

                let quad = (byte as u32) << shift;
                let offset = (INPUT_GAIN_INVERT_OFFSET + pos * 4) as u32;
                write_quad(req, node, offset, quad, timeout_ms)
            } else {
                Ok(())
            }
        })?;

        if params.paired != updates.paired {
            let quad = params
                .paired
                .iter_mut()
                .zip(&updates.paired)
                .zip(INPUT_PAIRED_CH_MAP)
                .filter(|((o, n), _)| !o.eq(n))
                .fold(0, |quad, ((_, &paired), ch_map)| {
                    // MEMO: 0x00ff0000 mask is absent.
                    let shift = ch_map * 8;
                    let mut val = INPUT_PAIRED_CHANGE_FLAG;
                    if paired {
                        val |= INPUT_PAIRED_FLAG;
                    }
                    quad | ((val as u32) << shift)
                });
            write_quad(req, node, INPUT_PAIRED_OFFSET as u32, quad, timeout_ms)
                .map(|_| params.paired.copy_from_slice(&updates.paired))?;
        }

        if params.phantom != updates.phantom || params.pad != updates.pad {
            let quad = (0..Self::MIC_COUNT).fold(0, |quad, i| {
                let shift = i * 8;
                let mut val = MIC_PARAM_CHANGE_FLAG;

                if updates.phantom[i] {
                    val |= MIC_PARAM_PHANTOM_FLAG;
                }

                if updates.pad[i] {
                    val |= MIC_PARAM_PAD_FLAG;
                }

                quad | ((val as u32) << shift)
            });

            write_quad(req, node, MIC_PARAM_OFFSET as u32, quad, timeout_ms).map(|_| {
                params.phantom.copy_from_slice(&updates.phantom);
                params.pad.copy_from_slice(&updates.pad);
            })?;
        }

        Ok(())
    }
}

impl<O> MotuRegisterDspImageOperation<RegisterDspStereoInputState, SndMotuRegisterDspParameter>
    for O
where
    O: MotuRegisterDspStereoInputSpecification,
{
    fn parse_image(params: &mut RegisterDspStereoInputState, image: &SndMotuRegisterDspParameter) {
        let vals = image.input_gain_and_invert();
        params
            .gain
            .iter_mut()
            .zip(vals)
            .for_each(|(gain, val)| *gain = val & STEREO_INPUT_GAIN_MASK);
        params
            .invert
            .iter_mut()
            .zip(vals)
            .for_each(|(invert, val)| *invert = val & STEREO_INPUT_INVERT_FLAG > 0);

        let flags = image.input_flag();
        params
            .phantom
            .iter_mut()
            .zip(flags)
            .for_each(|(phantom, val)| *phantom = val & EV_MIC_PHANTOM_FLAG > 0);
        params
            .pad
            .iter_mut()
            .zip(flags)
            .for_each(|(pad, val)| *pad = val & EV_MIC_PAD_FLAG > 0);
        params
            .jack
            .iter_mut()
            .zip(flags)
            .for_each(|(jack, val)| *jack = val & EV_INPUT_JACK_FLAG > 0);
        params
            .paired
            .iter_mut()
            .enumerate()
            .for_each(|(i, paired)| {
                let pos = EV_INPUT_PAIRED_CH_MAP[i * 2];
                *paired = flags[pos] & EV_INPUT_PAIRED_FLAG > 0;
            });
    }
}

impl<O> MotuRegisterDspEventOperation<RegisterDspStereoInputState> for O
where
    O: MotuRegisterDspStereoInputSpecification,
{
    fn parse_event(params: &mut RegisterDspStereoInputState, event: &RegisterDspEvent) -> bool {
        match event.ev_type {
            EV_TYPE_INPUT_GAIN_AND_INVERT => {
                let ch = event.identifier0 as usize;
                if ch < STEREO_INPUT_COUNT {
                    params.gain[ch] = event.value & STEREO_INPUT_GAIN_MASK;
                    params.invert[ch] = event.value & STEREO_INPUT_INVERT_FLAG > 0;
                    true
                } else {
                    false
                }
            }
            EV_TYPE_INPUT_FLAG => {
                let ch = event.identifier0 as usize;
                if let Some(pos) = EV_INPUT_PAIRED_CH_MAP.iter().position(|&p| p == ch) {
                    if pos % 2 == 0 {
                        params.paired[pos / 2] = event.value & EV_INPUT_PAIRED_FLAG > 0;
                    }
                    if pos < Self::MIC_COUNT {
                        params.phantom[ch] = event.value & EV_MIC_PHANTOM_FLAG > 0;
                        params.pad[ch] = event.value & EV_MIC_PAD_FLAG > 0;
                        params.jack[ch] = event.value & EV_INPUT_JACK_FLAG > 0;
                    }
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

/// Information of meter.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct RegisterDspMeterState {
    /// The detected level of signal in inputs.
    pub inputs: Vec<u8>,
    /// The detected level of signal in outputs.
    pub outputs: Vec<u8>,
    /// The selected port for output metering.
    pub selected: Option<usize>,
}

// Read-only.
const METER_OUTPUT_SELECT_OFFSET: usize = 0x0b2c;
const METER_OUTPUT_SELECT_TARGET_MASK: u32 = 0x000000ff;
const METER_OUTPUT_SELECT_CHANGE_FLAG: u32 = 0x00000b00;

// Assertion from UAPI of ALSA firewire stack.
const MAX_METER_INPUT_COUNT: usize = 24;
const MAX_METER_OUTPUT_COUNT: usize = 48;

const METER_IMAGE_SIZE: usize = 48;

/// The trait for specification of hardware metering.
pub trait MotuRegisterDspMeterSpecification: MotuRegisterDspSpecification {
    /// Whether to select output port for metering.
    const SELECTABLE: bool;
    /// The input ports.
    const INPUT_PORTS: &'static [TargetPort];
    /// The output pairs.
    const OUTPUT_PORT_PAIRS: &'static [TargetPort];
    const OUTPUT_PORT_PAIR_POS: &'static [[usize; 2]];
    /// The number of outputs.
    const OUTPUT_PORT_COUNT: usize = Self::OUTPUT_PORT_PAIRS.len() * 2;

    /// The minimum value of detected signal level.
    const LEVEL_MIN: u8 = 0;
    /// The maximum value of detected signal level.
    const LEVEL_MAX: u8 = 0x7f;
    /// The step value of detected signal level.
    const LEVEL_STEP: u8 = 1;

    /// The size of image.
    const METER_IMAGE_SIZE: usize = METER_IMAGE_SIZE;

    fn create_meter_state() -> RegisterDspMeterState {
        // Assertion from UAPI of ALSA firewire stack.
        assert!(Self::INPUT_PORTS.len() <= MAX_METER_INPUT_COUNT);
        assert!(Self::OUTPUT_PORT_PAIRS.len() <= MAX_METER_OUTPUT_COUNT);
        assert_eq!(
            Self::OUTPUT_PORT_PAIRS.len(),
            Self::OUTPUT_PORT_PAIR_POS.len()
        );

        RegisterDspMeterState {
            inputs: vec![0; Self::INPUT_PORTS.len()],
            outputs: vec![0; Self::OUTPUT_PORT_COUNT],
            selected: if Self::SELECTABLE { Some(0) } else { None },
        }
    }
}

impl<O> MotuWhollyUpdatableParamsOperation<RegisterDspMeterState> for O
where
    O: MotuRegisterDspMeterSpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &RegisterDspMeterState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if !Self::SELECTABLE || params.selected == None {
            Err(Error::new(FileError::Nxio, "Not supported"))?;
        }

        if let Some(target) = params.selected {
            if target >= Self::OUTPUT_PORT_PAIRS.len() {
                Err(Error::new(
                    FileError::Inval,
                    "Invalid argument for output metering target",
                ))?;
            } else {
                let mut quad = ((target + 1) as u32) & METER_OUTPUT_SELECT_TARGET_MASK;
                quad |= METER_OUTPUT_SELECT_CHANGE_FLAG;
                write_quad(
                    req,
                    node,
                    METER_OUTPUT_SELECT_OFFSET as u32,
                    quad,
                    timeout_ms,
                )?;
            }
        }

        Ok(())
    }
}

impl<O> MotuRegisterDspImageOperation<RegisterDspMeterState, [u8; METER_IMAGE_SIZE]> for O
where
    O: MotuRegisterDspMeterSpecification,
{
    fn parse_image(params: &mut RegisterDspMeterState, image: &[u8; METER_IMAGE_SIZE]) {
        params
            .inputs
            .copy_from_slice(&image[..Self::INPUT_PORTS.len()]);

        if let Some(selected) = params.selected {
            params.outputs.iter_mut().for_each(|meter| *meter = 0);

            let pair = &Self::OUTPUT_PORT_PAIR_POS[selected];
            params.outputs[selected * 2] = image[MAX_METER_INPUT_COUNT + pair[0]];
            params.outputs[selected * 2 + 1] = image[MAX_METER_INPUT_COUNT + pair[1]];
        } else {
            Self::OUTPUT_PORT_PAIR_POS
                .iter()
                .flatten()
                .zip(&mut params.outputs)
                .for_each(|(pos, m)| *m = image[MAX_METER_INPUT_COUNT + pos]);
        }
    }
}

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
