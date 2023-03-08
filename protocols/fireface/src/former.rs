// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for former models of Firewire series.

pub mod ff400;
pub mod ff800;

use super::*;

/// The specification of former model.
pub trait RmeFfFormerSpecification {
    /// The number of analog (line and microphone) inputs.
    const ANALOG_INPUT_COUNT: usize;
    /// The number of S/PDIF inputs.
    const SPDIF_INPUT_COUNT: usize;
    /// The number of ADAT inputs.
    const ADAT_INPUT_COUNT: usize;
    /// The number of stream inputs.
    const STREAM_INPUT_COUNT: usize;

    /// The number of analog outputs.
    const ANALOG_OUTPUT_COUNT: usize;
    /// The number of S/PDIF outputs.
    const SPDIF_OUTPUT_COUNT: usize;
    /// The number of ADAT outputs.
    const ADAT_OUTPUT_COUNT: usize;

    /// The number of physical inputs (line, microphone, S/PDIF, and ADAT).
    const PHYS_INPUT_COUNT: usize =
        Self::ANALOG_INPUT_COUNT + Self::SPDIF_INPUT_COUNT + Self::ADAT_INPUT_COUNT;
    /// The number of physical outputs (line, S/PDIF, and ADAT).
    const PHYS_OUTPUT_COUNT: usize =
        Self::ANALOG_OUTPUT_COUNT + Self::SPDIF_OUTPUT_COUNT + Self::ADAT_OUTPUT_COUNT;
}

/// State of hardware meter.
///
/// Each value of 32 bit integer is between 0x00000000 and 0x7fffff00 to represent -90.03 and
/// 0.00 dB. When reaching saturation, 1 byte in LSB side represent ratio of overload.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FormerMeterState {
    /// The detected levels for analog (line and microphone) inputs.
    pub analog_inputs: Vec<i32>,
    /// The detected levels for S/PDIF inputs.
    pub spdif_inputs: Vec<i32>,
    /// The detected levels for ADAT inputs.
    pub adat_inputs: Vec<i32>,
    /// The detected levels for stream inputs.
    pub stream_inputs: Vec<i32>,
    /// The detected levels for analog outputs.
    pub analog_outputs: Vec<i32>,
    /// The detected levels for S/PDIF outputs.
    pub spdif_outputs: Vec<i32>,
    /// The detected levels for ADAT outputs.
    pub adat_outputs: Vec<i32>,
}

/// The specification of hardware metering in former models.
pub trait RmeFfFormerMeterSpecification: RmeFfFormerSpecification {
    const METER_OFFSET: u64;

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
}

const METER_LEVEL_MASK: i32 = 0x7fffff00;

// MEMO: The content of meter appears to consist of three regions. The purpose of each region is
// still unclear. Let us use the last region.
fn serialize_meter(
    params: &FormerMeterState,
    phys_input_count: usize,
    stream_input_count: usize,
    phys_output_count: usize,
) -> Vec<u8> {
    let offset = 4 * (phys_input_count + stream_input_count + phys_output_count) * 2;
    let mut raw = vec![0; offset];
    raw.reserve(
        params.analog_inputs.len()
            + params.spdif_inputs.len()
            + params.stream_inputs.len()
            + params.analog_outputs.len()
            + params.spdif_outputs.len()
            + params.adat_outputs.len(),
    );
    params
        .analog_inputs
        .iter()
        .chain(params.spdif_inputs.iter())
        .chain(params.adat_inputs.iter())
        .chain(params.stream_inputs.iter())
        .chain(params.analog_outputs.iter())
        .chain(params.spdif_outputs.iter())
        .chain(params.adat_outputs.iter())
        .for_each(|meter| {
            raw.extend_from_slice(&(meter & METER_LEVEL_MASK).to_le_bytes());
        });
    raw
}

fn deserialize_meter(
    params: &mut FormerMeterState,
    raw: &[u8],
    phys_input_count: usize,
    stream_input_count: usize,
    phys_output_count: usize,
) {
    // TODO: pick up overload.
    let mut quadlet = [0; 4];
    let offset = 4 * (phys_input_count + stream_input_count + phys_output_count) * 2;
    params
        .analog_inputs
        .iter_mut()
        .chain(params.spdif_inputs.iter_mut())
        .chain(params.adat_inputs.iter_mut())
        .chain(params.stream_inputs.iter_mut())
        .chain(params.analog_outputs.iter_mut())
        .chain(params.spdif_outputs.iter_mut())
        .chain(params.adat_outputs.iter_mut())
        .enumerate()
        .for_each(|(i, meter)| {
            let pos = offset + i * 4;
            quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
            *meter = i32::from_le_bytes(quadlet) & METER_LEVEL_MASK;
        });
}

impl<O: RmeFfFormerSpecification> RmeFfParamsSerialize<FormerMeterState, u8> for O {
    fn serialize(params: &FormerMeterState) -> Vec<u8> {
        serialize_meter(
            params,
            Self::PHYS_INPUT_COUNT,
            Self::STREAM_INPUT_COUNT,
            Self::PHYS_OUTPUT_COUNT,
        )
    }
}

impl<O: RmeFfFormerMeterSpecification> RmeFfParamsDeserialize<FormerMeterState, u8> for O {
    fn deserialize(params: &mut FormerMeterState, raw: &[u8]) {
        deserialize_meter(
            params,
            raw,
            Self::PHYS_INPUT_COUNT,
            Self::STREAM_INPUT_COUNT,
            Self::PHYS_OUTPUT_COUNT,
        );
    }
}

impl<O: RmeFfFormerMeterSpecification + RmeFfParamsDeserialize<FormerMeterState, u8>>
    RmeFfCacheableParamsOperation<FormerMeterState> for O
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut FormerMeterState,
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
            Self::METER_OFFSET,
            raw.len(),
            &mut raw,
            timeout_ms,
        )
        .map(|_| Self::deserialize(params, &raw))
    }
}

/// State of output volumes.
///
/// The value for volume is between 0x00000000 and 0x00010000 through 0x00000001 and 0x00080000 to
/// represent the range from negative infinite to 6.00 dB through -90.30 dB and 0.00 dB.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct FormerOutputVolumeState(pub Vec<i32>);

/// Output protocol specific to former models of RME Fireface.
pub trait RmeFormerOutputOperation: RmeFfFormerSpecification {
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
pub trait RmeFormerMixerOperation: RmeFfFormerSpecification {
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

fn calculate_mixer_length(avail_count: usize) -> usize {
    avail_count * 2 * 4
}

fn calculate_mixer_total_length(dst_count: usize, avail_count: usize) -> usize {
    calculate_mixer_length(avail_count) * dst_count
}

fn serialize_mixer(params: &FormerMixerState, dst_count: usize, avail_count: usize) -> Vec<u8> {
    let mut raw = vec![0; calculate_mixer_total_length(dst_count, avail_count)];

    params.0.iter().enumerate().for_each(|(i, mixer)| {
        mixer
            .analog_gains
            .iter()
            .chain(mixer.spdif_gains.iter())
            .chain(mixer.adat_gains.iter())
            .enumerate()
            .for_each(|(j, gain)| {
                let pos = ((avail_count * 2) * i + j) * 4;
                raw[pos..(pos + 4)].copy_from_slice(&gain.to_le_bytes());
            });

        mixer.stream_gains.iter().enumerate().for_each(|(j, gain)| {
            let pos = ((avail_count * 2) * i + (avail_count + j)) * 4;
            raw[pos..(pos + 4)].copy_from_slice(&gain.to_le_bytes());
        });
    });

    raw
}

fn deserialize_mixer(
    params: &mut FormerMixerState,
    raw: &[u8],
    dst_count: usize,
    avail_count: usize,
) {
    assert!(raw.len() >= calculate_mixer_total_length(dst_count, avail_count));

    let mut quadlet = [0; 4];
    params.0.iter_mut().enumerate().for_each(|(i, mixer)| {
        mixer
            .analog_gains
            .iter_mut()
            .chain(mixer.spdif_gains.iter_mut())
            .chain(mixer.adat_gains.iter_mut())
            .enumerate()
            .for_each(|(j, gain)| {
                let pos = ((avail_count * 2) * i + j) * 4;
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                *gain = i32::from_le_bytes(quadlet)
            });

        mixer
            .stream_gains
            .iter_mut()
            .enumerate()
            .for_each(|(j, gain)| {
                let pos = ((avail_count * 2) * i + (avail_count + j)) * 4;
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                *gain = i32::from_le_bytes(quadlet)
            });
    });
}

impl<O: RmeFormerMixerOperation> RmeFfParamsSerialize<FormerMixerState, u8> for O {
    fn serialize(params: &FormerMixerState) -> Vec<u8> {
        serialize_mixer(params, Self::DST_COUNT, Self::AVAIL_COUNT)
    }
}

impl<O: RmeFormerMixerOperation> RmeFfParamsDeserialize<FormerMixerState, u8> for O {
    fn deserialize(params: &mut FormerMixerState, raw: &[u8]) {
        deserialize_mixer(params, raw, Self::DST_COUNT, Self::AVAIL_COUNT)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn meter_serdes() {
        let mut orig = FormerMeterState {
            analog_inputs: vec![0, 1],
            spdif_inputs: vec![2, 3],
            adat_inputs: vec![4, 5],
            stream_inputs: vec![6, 7],
            analog_outputs: vec![8, 9],
            spdif_outputs: vec![10, 11],
            adat_outputs: vec![12, 13],
        };
        orig.analog_inputs
            .iter_mut()
            .chain(orig.spdif_inputs.iter_mut())
            .chain(orig.adat_inputs.iter_mut())
            .chain(orig.stream_inputs.iter_mut())
            .chain(orig.analog_outputs.iter_mut())
            .chain(orig.spdif_outputs.iter_mut())
            .chain(orig.adat_outputs.iter_mut())
            .for_each(|gain| *gain <<= 8);
        let raw = serialize_meter(&orig, 6, 2, 6);
        let mut target = FormerMeterState {
            analog_inputs: vec![0; 2],
            spdif_inputs: vec![0; 2],
            adat_inputs: vec![0; 2],
            stream_inputs: vec![0; 2],
            analog_outputs: vec![0; 2],
            spdif_outputs: vec![0; 2],
            adat_outputs: vec![0; 2],
        };
        deserialize_meter(&mut target, &raw, 6, 2, 6);

        assert_eq!(target, orig);
    }

    #[test]
    fn mixer_serdes() {
        let orig = FormerMixerState(vec![
            FormerMixerSrc {
                analog_gains: vec![0, 1],
                spdif_gains: vec![2, 3],
                adat_gains: vec![4, 5],
                stream_gains: vec![6, 7],
            },
            FormerMixerSrc {
                analog_gains: vec![8, 9],
                spdif_gains: vec![10, 11],
                adat_gains: vec![12, 13],
                stream_gains: vec![14, 15],
            },
        ]);
        let raw = serialize_mixer(&orig, 6, 8);
        let mut target = FormerMixerState(vec![
            FormerMixerSrc {
                analog_gains: vec![0; 2],
                spdif_gains: vec![0; 2],
                adat_gains: vec![0; 2],
                stream_gains: vec![0; 2],
            },
            FormerMixerSrc {
                analog_gains: vec![0; 2],
                spdif_gains: vec![0; 2],
                adat_gains: vec![0; 2],
                stream_gains: vec![0; 2],
            },
        ]);
        deserialize_mixer(&mut target, &raw, 6, 8);

        assert_eq!(target, orig);
    }
}
