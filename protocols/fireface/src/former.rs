// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for former models of Firewire series.

pub mod ff400;
pub mod ff800;

use super::*;

/// State of hardware meter.
///
/// Each value of 32 bit integer is between 0x00000000 and 0x7fffff00 to represent -90.03 and
/// 0.00 dB. When reaching saturation, 1 byte in LSB side represent ratio of overload.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FormerMeterState {
    pub analog_inputs: Vec<i32>,
    pub spdif_inputs: Vec<i32>,
    pub adat_inputs: Vec<i32>,
    pub stream_inputs: Vec<i32>,
    pub analog_outputs: Vec<i32>,
    pub spdif_outputs: Vec<i32>,
    pub adat_outputs: Vec<i32>,
}

/// Meter protocol of Fireface 400.
pub trait RmeFfFormerMeterOperation {
    const METER_OFFSET: usize;

    const ANALOG_INPUT_COUNT: usize;
    const SPDIF_INPUT_COUNT: usize;
    const ADAT_INPUT_COUNT: usize;
    const STREAM_INPUT_COUNT: usize;

    const ANALOG_OUTPUT_COUNT: usize;
    const SPDIF_OUTPUT_COUNT: usize;
    const ADAT_OUTPUT_COUNT: usize;

    const PHYS_INPUT_COUNT: usize =
        Self::ANALOG_INPUT_COUNT + Self::SPDIF_INPUT_COUNT + Self::ADAT_INPUT_COUNT;
    const PHYS_OUTPUT_COUNT: usize =
        Self::ANALOG_OUTPUT_COUNT + Self::SPDIF_OUTPUT_COUNT + Self::ADAT_OUTPUT_COUNT;

    const LEVEL_MIN: i32 = 0x00000000;
    const LEVEL_MAX: i32 = 0x7fffff00;
    const LEVEL_STEP: i32 = 0x100;

    fn create_meter_state() -> FormerMeterState {
        FormerMeterState {
            analog_inputs: vec![0; Self::ANALOG_INPUT_COUNT],
            spdif_inputs: vec![0; Self::SPDIF_INPUT_COUNT],
            adat_inputs: vec![0; Self::ADAT_INPUT_COUNT],
            stream_inputs: vec![0; Self::STREAM_INPUT_COUNT],
            analog_outputs: vec![0; Self::ANALOG_OUTPUT_COUNT],
            spdif_outputs: vec![0; Self::SPDIF_OUTPUT_COUNT],
            adat_outputs: vec![0; Self::ADAT_OUTPUT_COUNT],
        }
    }

    fn read_meter(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut FormerMeterState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        // NOTE:
        // Each of the first octuples is for level of corresponding source to mixer.
        // Each of the following octuples is for level of corresponding output from mixer (pre-fader).
        // Each of the following octuples is for level of corresponding output from mixer (post-fader).
        // Each of the following quadlets is for level of corresponding physical input.
        // Each of the following quadlets is for level of corresponding stream input.
        // Each of the following quadlets is for level of corresponding physical output.
        let length = 8 * (Self::PHYS_INPUT_COUNT + Self::PHYS_OUTPUT_COUNT * 2)
            + 4 * (Self::PHYS_INPUT_COUNT + Self::STREAM_INPUT_COUNT + Self::PHYS_OUTPUT_COUNT);
        let mut raw = vec![0; length];
        req.transaction_sync(
            node,
            FwTcode::ReadBlockRequest,
            Self::METER_OFFSET as u64,
            raw.len(),
            &mut raw,
            timeout_ms,
        )
        .map(|_| {
            // TODO: pick up overload.
            let mut quadlet = [0; 4];
            let mut offset = 8 * (Self::PHYS_INPUT_COUNT + Self::PHYS_OUTPUT_COUNT * 2);
            [
                &mut state.analog_inputs[..],
                &mut state.spdif_inputs[..],
                &mut state.adat_inputs[..],
                &mut state.stream_inputs[..],
                &mut state.analog_outputs[..],
                &mut state.spdif_outputs[..],
                &mut state.adat_outputs[..],
            ]
            .iter_mut()
            .for_each(|meters| {
                meters.iter_mut().for_each(|v| {
                    quadlet.copy_from_slice(&raw[offset..(offset + 4)]);
                    *v = i32::from_le_bytes(quadlet) & 0x7fffff00;
                    offset += 4;
                });
            });
        })
    }
}

/// State of output volumes.
///
/// The value for volume is between 0x00000000 and 0x00010000 through 0x00000001 and 0x00080000 to
/// represent the range from negative infinite to 6.00 dB through -90.30 dB and 0.00 dB.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FormerOutputVolumeState(pub Vec<i32>);

/// Output protocol specific to former models of RME Fireface.
pub trait RmeFormerOutputOperation {
    const ANALOG_OUTPUT_COUNT: usize;
    const SPDIF_OUTPUT_COUNT: usize;
    const ADAT_OUTPUT_COUNT: usize;

    const PHYS_OUTPUT_COUNT: usize =
        Self::ANALOG_OUTPUT_COUNT + Self::SPDIF_OUTPUT_COUNT + Self::ADAT_OUTPUT_COUNT;

    const VOL_MIN: i32 = 0x00000000;
    const VOL_ZERO: i32 = 0x00008000;
    const VOL_MAX: i32 = 0x00010000;
    const VOL_STEP: i32 = 1;

    fn create_output_volume_state() -> FormerOutputVolumeState {
        FormerOutputVolumeState(vec![0; Self::PHYS_OUTPUT_COUNT])
    }

    fn write_output_vol(
        req: &mut FwReq,
        node: &mut FwNode,
        ch: usize,
        vol: i32,
        timeout_ms: u32,
    ) -> Result<(), Error>;

    fn init_output_vols(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &FormerOutputVolumeState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        state
            .0
            .iter()
            .enumerate()
            .try_for_each(|(i, vol)| Self::write_output_vol(req, node, i, *vol, timeout_ms))
    }

    fn write_output_vols(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut FormerOutputVolumeState,
        vols: &[i32],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        state
            .0
            .iter_mut()
            .zip(vols)
            .enumerate()
            .filter(|(_, (o, n))| !o.eq(n))
            .try_for_each(|(i, (o, n))| {
                Self::write_output_vol(req, node, i, *n, timeout_ms).map(|_| *o = *n)
            })
    }
}

/// Sources of mixer specific to former models of RME Fireface.
///
/// The value is between 0x00000000 and 0x00010000 through 0x00008000 to represent -90.30 and 6.02 dB
/// through 0x00008000.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FormerMixerSrc {
    pub analog_gains: Vec<i32>,
    pub spdif_gains: Vec<i32>,
    pub adat_gains: Vec<i32>,
    pub stream_gains: Vec<i32>,
}

/// State of mixer.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FormerMixerState(pub Vec<FormerMixerSrc>);

/// Mixer protocol specific to former models of RME Fireface.
pub trait RmeFormerMixerOperation {
    const ANALOG_INPUT_COUNT: usize;
    const SPDIF_INPUT_COUNT: usize;
    const ADAT_INPUT_COUNT: usize;
    const STREAM_INPUT_COUNT: usize;

    const ANALOG_OUTPUT_COUNT: usize;
    const SPDIF_OUTPUT_COUNT: usize;
    const ADAT_OUTPUT_COUNT: usize;

    const MIXER_OFFSET: usize;
    const AVAIL_COUNT: usize;

    const DST_COUNT: usize =
        Self::ANALOG_OUTPUT_COUNT + Self::SPDIF_OUTPUT_COUNT + Self::ADAT_OUTPUT_COUNT;

    const GAIN_MIN: i32 = 0x00000000;
    const GAIN_ZERO: i32 = 0x00008000;
    const GAIN_MAX: i32 = 0x00010000;
    const GAIN_STEP: i32 = 1;

    fn create_mixer_state() -> FormerMixerState {
        FormerMixerState(vec![
            FormerMixerSrc {
                analog_gains: vec![0; Self::ANALOG_INPUT_COUNT],
                spdif_gains: vec![0; Self::SPDIF_INPUT_COUNT],
                adat_gains: vec![0; Self::ADAT_INPUT_COUNT],
                stream_gains: vec![0; Self::STREAM_INPUT_COUNT],
            };
            Self::DST_COUNT
        ])
    }
    fn write_mixer_src_gains(
        req: &mut FwReq,
        node: &mut FwNode,
        mixer: usize,
        src_offset: usize,
        gains: &[i32],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0; gains.len() * 4];
        gains.iter().enumerate().for_each(|(i, gain)| {
            let pos = i * 4;
            raw[pos..(pos + 4)].copy_from_slice(&gain.to_le_bytes());
        });

        let offset = ((Self::AVAIL_COUNT * mixer * 2) + src_offset) * 4;
        req.transaction_sync(
            node,
            FwTcode::WriteBlockRequest,
            (Self::MIXER_OFFSET + offset) as u64,
            raw.len(),
            &mut raw,
            timeout_ms,
        )
    }

    fn init_mixer_src_gains(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut FormerMixerState,
        mixer: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        [
            (&state.0[mixer].analog_gains, 0),
            (&state.0[mixer].spdif_gains, Self::ANALOG_INPUT_COUNT),
            (
                &state.0[mixer].adat_gains,
                Self::ANALOG_INPUT_COUNT + Self::SPDIF_INPUT_COUNT,
            ),
            (&state.0[mixer].stream_gains, Self::AVAIL_COUNT),
        ]
        .iter()
        .try_for_each(|(gains, src_offset)| {
            Self::write_mixer_src_gains(req, node, mixer, *src_offset, gains, timeout_ms)
        })
    }

    fn write_mixer_analog_gains(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut FormerMixerState,
        mixer: usize,
        gains: &[i32],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::write_mixer_src_gains(req, node, mixer, 0, gains, timeout_ms)
            .map(|_| state.0[mixer].analog_gains.copy_from_slice(&gains))
    }

    fn write_mixer_spdif_gains(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut FormerMixerState,
        mixer: usize,
        gains: &[i32],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::write_mixer_src_gains(
            req,
            node,
            mixer,
            Self::ANALOG_INPUT_COUNT,
            gains,
            timeout_ms,
        )
        .map(|_| state.0[mixer].spdif_gains.copy_from_slice(&gains))
    }

    fn write_mixer_adat_gains(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut FormerMixerState,
        mixer: usize,
        gains: &[i32],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::write_mixer_src_gains(
            req,
            node,
            mixer,
            Self::ANALOG_INPUT_COUNT + Self::SPDIF_INPUT_COUNT,
            gains,
            timeout_ms,
        )
        .map(|_| state.0[mixer].adat_gains.copy_from_slice(&gains))
    }

    fn write_mixer_stream_gains(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut FormerMixerState,
        mixer: usize,
        gains: &[i32],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::write_mixer_src_gains(req, node, mixer, Self::AVAIL_COUNT, gains, timeout_ms)
            .map(|_| state.0[mixer].stream_gains.copy_from_slice(&gains))
    }
}

const FORMER_CONFIG_SIZE: usize = 12;

fn write_config<T: RmeFfParamsSerialize<U, u8>, U>(
    req: &mut FwReq,
    node: &mut FwNode,
    offset: u64,
    config: &U,
    timeout_ms: u32,
) -> Result<(), Error> {
    let mut raw = T::serialize(config);

    req.transaction_sync(
        node,
        FwTcode::WriteBlockRequest,
        offset,
        raw.len(),
        &mut raw,
        timeout_ms,
    )
}

const FORMER_STATUS_SIZE: usize = 8;

fn read_status<T: RmeFfParamsDeserialize<U, u8>, U>(
    req: &mut FwReq,
    node: &mut FwNode,
    offset: u64,
    status: &mut U,
    timeout_ms: u32,
) -> Result<(), Error> {
    let mut raw = [0; FORMER_STATUS_SIZE];
    req.transaction_sync(
        node,
        FwTcode::ReadBlockRequest,
        offset,
        raw.len(),
        &mut raw,
        timeout_ms,
    )
    .map(|_| T::deserialize(status, &raw))
}

/// Configuration of S/PDIF output.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct FormerSpdifOutput {
    /// The format of S/PDIF signal.
    pub format: SpdifFormat,
    /// Whether to boost signal.
    pub emphasis: bool,
    /// Whether to transfer non-audio bit in preemble.
    pub non_audio: bool,
}

/// Nominal level of line inputs.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FormerLineInNominalLevel {
    Low,
    /// -10 dBV.
    Consumer,
    /// +4 dBu.
    Professional,
}

impl Default for FormerLineInNominalLevel {
    fn default() -> Self {
        Self::Low
    }
}
