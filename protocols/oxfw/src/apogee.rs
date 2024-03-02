// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by Apogee Electronics for Duet FireWire.
//!
//! The module includes protocol implementation defined by Apogee Electronics for Duet FireWire.
//!
//! ## Diagram of internal signal flow for Apogee Duet FireWire
//!
//! ```text
//!
//! xlr-input-1 ----> or ------> analog-input-1 --+-----+---------------> stream-output-1/2
//!                   ^                           |     |
//! xlr-input-2 ------|-> or --> analog-input-2 --|--+--+
//!                   |   ^                       |  |
//! phone-input-1 --- +   |                       |  |
//!                       |                       v  v
//! phone-input-2 --------+                   ++=========++
//!                                           ||  mixer  ||
//! stream-input-1/2 -----------------------> ||         || ------------> analog-output-1/2
//!                                           ||  4 x 2  ||
//!                                           ++=========++
//! ```

use super::*;

/// Protocol implementation for Duet FireWire.
#[derive(Default, Debug)]
pub struct DuetFwProtocol;

impl OxfordOperation for DuetFwProtocol {}

impl OxfwStreamFormatOperation<OxfwAvc> for DuetFwProtocol {}

/// Serializer and deserializer for parameters.
pub trait DuetFwParamsSerdes<T> {
    /// Build commands for AV/C status operation.
    fn default_cmds_for_params(cmds: &mut Vec<VendorCmd>);

    /// Deserialize parameters for AV/C status operation.
    fn cmds_to_params(params: &mut T, cmds: &[VendorCmd]);

    /// Serialize parameters for AV/C control operation.
    fn cmds_from_params(params: &T, cmds: &mut Vec<VendorCmd>);
}

impl<T> OxfwFcpParamsOperation<OxfwAvc, T> for DuetFwProtocol
where
    DuetFwProtocol: DuetFwParamsSerdes<T>,
{
    fn cache(avc: &mut OxfwAvc, params: &mut T, timeout_ms: u32) -> Result<(), Error> {
        let mut cmds = Vec::new();
        Self::default_cmds_for_params(&mut cmds);

        let mut states = Vec::new();
        cmds.into_iter().try_for_each(|cmd| {
            let mut op = ApogeeCmd::new(cmd);
            avc.status(&AvcAddr::Unit, &mut op, timeout_ms)
                .map(|_| states.push(op.cmd))
        })?;

        Self::cmds_to_params(params, &states);

        Ok(())
    }
}

impl<T> OxfwFcpMutableParamsOperation<OxfwAvc, T> for DuetFwProtocol
where
    T: Copy,
    DuetFwProtocol: DuetFwParamsSerdes<T>,
{
    fn update(avc: &mut OxfwAvc, params: &T, prev: &mut T, timeout_ms: u32) -> Result<(), Error> {
        let mut new = Vec::new();
        Self::cmds_from_params(params, &mut new);

        let mut old = Vec::new();
        Self::cmds_from_params(prev, &mut old);

        new.iter()
            .zip(&old)
            .filter(|(n, o)| !n.eq(o))
            .try_for_each(|(cmd, _)| {
                let mut op = ApogeeCmd::new(*cmd);
                avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
            })
            .map(|_| *prev = *params)
    }
}

/// Specification of meter.
pub trait DuetFwMeterSpecification<T> {
    /// Offset for raw meter data.
    const OFFSET: usize;

    /// Size for raw meter data.
    const SIZE: usize;

    /// Deserialize for meter.
    fn deserialize_meter(params: &mut T, raw: &[u8]);
}

/// Operation for meter.
pub trait DuetFwMeterOperation<T>: DuetFwMeterSpecification<T> {
    fn cache_meter(
        req: &FwReq,
        node: &FwNode,
        params: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0u8; Self::SIZE];

        req.transaction(
            node,
            FwTcode::ReadBlockRequest,
            (METER_OFFSET_BASE + Self::OFFSET) as u64,
            raw.len(),
            &mut raw,
            timeout_ms,
        )
        .map(|_| Self::deserialize_meter(params, &raw))
    }
}

impl<O, T> DuetFwMeterOperation<T> for O where O: DuetFwMeterSpecification<T> {}

const APOGEE_OUI: [u8; 3] = [0x00, 0x03, 0xdb];

const METER_OFFSET_BASE: usize = 0xfffff0080000;
const METER_INPUT_OFFSET: usize = 0x0004;
const METER_MIXER_OFFSET: usize = 0x0404;

/// The state of meter for analog input.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DuetFwInputMeterState(pub [i32; 2]);

impl DuetFwMeterSpecification<DuetFwInputMeterState> for DuetFwProtocol {
    const OFFSET: usize = METER_INPUT_OFFSET;
    const SIZE: usize = 8;

    fn deserialize_meter(state: &mut DuetFwInputMeterState, raw: &[u8]) {
        let mut quadlet = [0; 4];
        state.0.iter_mut().enumerate().for_each(|(i, level)| {
            let pos = i * 4;
            quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
            *level = i32::from_be_bytes(quadlet);
        });
    }
}

impl DuetFwProtocol {
    /// The minimum value of detected level in meter state.
    pub const METER_LEVEL_MIN: i32 = 0;
    /// The maximum value of detected level in meter state.
    pub const METER_LEVEL_MAX: i32 = i32::MAX;
    /// The step value of detected level in meter state.
    pub const METER_LEVEL_STEP: i32 = 0x100;
}

/// The state of meter for mixer source/output.
#[derive(Default, Debug)]
pub struct DuetFwMixerMeterState {
    /// Detected level of stream inputs.
    pub stream_inputs: [i32; 2],
    /// Detected level of mixer outputs.
    pub mixer_outputs: [i32; 2],
}

impl DuetFwMeterSpecification<DuetFwMixerMeterState> for DuetFwProtocol {
    const OFFSET: usize = METER_MIXER_OFFSET;
    const SIZE: usize = 16;

    fn deserialize_meter(state: &mut DuetFwMixerMeterState, raw: &[u8]) {
        let mut quadlet = [0; 4];
        state
            .stream_inputs
            .iter_mut()
            .chain(&mut state.mixer_outputs)
            .enumerate()
            .for_each(|(i, meter)| {
                let pos = i * 4;
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                *meter = i32::from_be_bytes(quadlet);
            });
    }
}

/// Target of knob.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DuetFwKnobTarget {
    /// The pair of outputs.
    OutputPair0,
    /// The 1st channel (left) of input pair.
    InputPair0,
    /// The 2nd channel (right) of input pair.
    InputPair1,
}

impl Default for DuetFwKnobTarget {
    fn default() -> Self {
        Self::OutputPair0
    }
}

/// The state of rotary knob.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DuetFwKnobState {
    /// Whether to mute pair of outputs.
    pub output_mute: bool,
    /// Target to control by knob.
    pub target: DuetFwKnobTarget,
    /// The value of output volume.
    pub output_volume: u8,
    /// The value of input gains.
    pub input_gains: [u8; 2],
}

fn default_cmds_for_knob_params(cmds: &mut Vec<VendorCmd>) {
    cmds.push(VendorCmd::HwState(Default::default()));
}

fn cmds_from_knob_params(params: &DuetFwKnobState, cmds: &mut Vec<VendorCmd>) {
    let mut raw = [0; 11];

    raw[0] = params.output_mute as u8;
    raw[1] = match params.target {
        DuetFwKnobTarget::OutputPair0 => 0,
        DuetFwKnobTarget::InputPair0 => 1,
        DuetFwKnobTarget::InputPair1 => 2,
    };
    raw[3] = DuetFwProtocol::KNOB_OUTPUT_VALUE_MAX - params.output_volume;
    raw[4] = params.input_gains[0];
    raw[5] = params.input_gains[1];

    cmds.push(VendorCmd::HwState(raw));
}

fn cmds_to_knob_params(params: &mut DuetFwKnobState, cmds: &[VendorCmd]) {
    cmds.iter().for_each(|cmd| match cmd {
        VendorCmd::HwState(raw) => {
            params.output_mute = raw[0] > 0;
            params.target = match raw[1] {
                2 => DuetFwKnobTarget::InputPair1,
                1 => DuetFwKnobTarget::InputPair0,
                _ => DuetFwKnobTarget::OutputPair0,
            };
            params.output_volume = DuetFwProtocol::KNOB_OUTPUT_VALUE_MAX - raw[3];
            params.input_gains[0] = raw[4];
            params.input_gains[1] = raw[5];
        }
        _ => (),
    });
}

impl DuetFwParamsSerdes<DuetFwKnobState> for DuetFwProtocol {
    fn default_cmds_for_params(cmds: &mut Vec<VendorCmd>) {
        default_cmds_for_knob_params(cmds);
    }

    fn cmds_to_params(params: &mut DuetFwKnobState, cmds: &[VendorCmd]) {
        cmds_to_knob_params(params, cmds);
    }

    fn cmds_from_params(params: &DuetFwKnobState, cmds: &mut Vec<VendorCmd>) {
        cmds_from_knob_params(params, cmds);
    }
}

impl DuetFwProtocol {
    /// The minimum value of output in knob parameters.
    pub const KNOB_OUTPUT_VALUE_MIN: u8 = 0;
    /// The maximum value of output in knob parameters.
    pub const KNOB_OUTPUT_VALUE_MAX: u8 = 64;

    /// The minimum value of input in knob parameters.
    pub const KNOB_INPUT_VALUE_MIN: u8 = 10;
    /// The maximum value of input in knob parameters.
    pub const KNOB_INPUT_VALUE_MAX: u8 = 75;
}

/// Source of output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DuetFwOutputSource {
    /// The pair of stream inputs.
    StreamInputPair0,
    /// The pair of mixer outputs.
    MixerOutputPair0,
}

impl Default for DuetFwOutputSource {
    fn default() -> Self {
        Self::StreamInputPair0
    }
}

/// Level of output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DuetFwOutputNominalLevel {
    /// Fixed level for external amplifier.
    Instrument,
    /// -10 dBV, adjustable between 0 to 64 (-64 to 0 dB).
    Consumer,
}

impl Default for DuetFwOutputNominalLevel {
    fn default() -> Self {
        Self::Instrument
    }
}

/// Mode of relation between mute and knob.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DuetFwOutputMuteMode {
    /// Never.
    Never,
    /// Muted at knob pushed, unmuted at knob released.
    Normal,
    /// Muted at knob released, unmuted at knob pushed.
    Swapped,
}

impl Default for DuetFwOutputMuteMode {
    fn default() -> Self {
        Self::Never
    }
}

/// Parameters for output.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DuetFwOutputParams {
    /// Whether to mute.
    pub mute: bool,
    /// Volume.
    pub volume: u8,
    /// Source of signal.
    pub source: DuetFwOutputSource,
    /// Nominal level.
    pub nominal_level: DuetFwOutputNominalLevel,
    /// Mode of mute for line output.
    pub line_mute_mode: DuetFwOutputMuteMode,
    /// Mode of mute for headphone output.
    pub hp_mute_mode: DuetFwOutputMuteMode,
}

fn parse_mute_mode(mute: bool, unmute: bool) -> DuetFwOutputMuteMode {
    match (mute, unmute) {
        (true, true) => DuetFwOutputMuteMode::Never,
        (true, false) => DuetFwOutputMuteMode::Swapped,
        (false, true) => DuetFwOutputMuteMode::Normal,
        (false, false) => DuetFwOutputMuteMode::Never,
    }
}

fn build_mute_mode(mode: &DuetFwOutputMuteMode, mute: &mut bool, unmute: &mut bool) {
    match mode {
        DuetFwOutputMuteMode::Never => {
            *mute = true;
            *unmute = true;
        }
        DuetFwOutputMuteMode::Normal => {
            *mute = false;
            *unmute = true;
        }
        DuetFwOutputMuteMode::Swapped => {
            *mute = true;
            *unmute = false;
        }
    }
}

fn default_cmds_for_output_params(cmds: &mut Vec<VendorCmd>) {
    cmds.push(VendorCmd::OutMute(Default::default()));
    cmds.push(VendorCmd::OutVolume(Default::default()));
    cmds.push(VendorCmd::OutSourceIsMixer(Default::default()));
    cmds.push(VendorCmd::OutIsConsumerLevel(Default::default()));
    cmds.push(VendorCmd::MuteForLineOut(Default::default()));
    cmds.push(VendorCmd::UnmuteForLineOut(Default::default()));
    cmds.push(VendorCmd::MuteForHpOut(Default::default()));
    cmds.push(VendorCmd::UnmuteForHpOut(Default::default()));
}

fn cmds_to_output_params(params: &mut DuetFwOutputParams, cmds: &[VendorCmd]) {
    let mut line_out_mute = false;
    let mut line_out_unmute = false;
    let mut hp_out_mute = false;
    let mut hp_out_unmute = false;

    cmds.iter().for_each(|&cmd| match cmd {
        VendorCmd::OutMute(muted) => params.mute = muted,
        VendorCmd::OutVolume(vol) => params.volume = vol,
        VendorCmd::OutSourceIsMixer(is_mixer) => {
            params.source = if is_mixer {
                DuetFwOutputSource::MixerOutputPair0
            } else {
                DuetFwOutputSource::StreamInputPair0
            };
        }
        VendorCmd::OutIsConsumerLevel(is_consumer_level) => {
            params.nominal_level = if is_consumer_level {
                DuetFwOutputNominalLevel::Consumer
            } else {
                DuetFwOutputNominalLevel::Instrument
            };
        }
        VendorCmd::MuteForLineOut(muted) => line_out_mute = muted,
        VendorCmd::UnmuteForLineOut(unmuted) => line_out_unmute = unmuted,
        VendorCmd::MuteForHpOut(muted) => hp_out_mute = muted,
        VendorCmd::UnmuteForHpOut(unmuted) => hp_out_unmute = unmuted,
        _ => (),
    });

    params.line_mute_mode = parse_mute_mode(line_out_mute, line_out_unmute);
    params.hp_mute_mode = parse_mute_mode(hp_out_mute, hp_out_unmute);
}

fn cmds_from_output_params(params: &DuetFwOutputParams, cmds: &mut Vec<VendorCmd>) {
    let mut line_out_mute = false;
    let mut line_out_unmute = false;
    let mut hp_out_mute = false;
    let mut hp_out_unmute = false;

    build_mute_mode(
        &params.line_mute_mode,
        &mut line_out_mute,
        &mut line_out_unmute,
    );
    build_mute_mode(&params.hp_mute_mode, &mut hp_out_mute, &mut hp_out_unmute);

    let is_mixer = params.source == DuetFwOutputSource::MixerOutputPair0;
    let is_consumer_level = params.nominal_level == DuetFwOutputNominalLevel::Consumer;

    cmds.push(VendorCmd::OutMute(params.mute));
    cmds.push(VendorCmd::OutVolume(params.volume));
    cmds.push(VendorCmd::OutSourceIsMixer(is_mixer));
    cmds.push(VendorCmd::OutIsConsumerLevel(is_consumer_level));
    cmds.push(VendorCmd::MuteForLineOut(line_out_mute));
    cmds.push(VendorCmd::UnmuteForLineOut(line_out_unmute));
    cmds.push(VendorCmd::MuteForHpOut(hp_out_mute));
    cmds.push(VendorCmd::UnmuteForHpOut(hp_out_unmute));
}

impl DuetFwParamsSerdes<DuetFwOutputParams> for DuetFwProtocol {
    fn default_cmds_for_params(cmds: &mut Vec<VendorCmd>) {
        default_cmds_for_output_params(cmds);
    }

    fn cmds_to_params(params: &mut DuetFwOutputParams, cmds: &[VendorCmd]) {
        cmds_to_output_params(params, cmds);
    }

    fn cmds_from_params(params: &DuetFwOutputParams, cmds: &mut Vec<VendorCmd>) {
        cmds_from_output_params(params, cmds);
    }
}

impl DuetFwProtocol {
    /// The minimum value of volume in output parameters.
    pub const OUTPUT_VOLUME_MIN: u8 = 0;
    /// The maximum value of volume in output parameters.
    pub const OUTPUT_VOLUME_MAX: u8 = 64;
}

/// Source of input.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DuetFwInputSource {
    /// From XLR plug.
    Xlr,
    /// From Phone plug. The gain is adjustable between 0 and 65 dB.
    Phone,
}

impl Default for DuetFwInputSource {
    fn default() -> Self {
        Self::Xlr
    }
}

/// Nominal level of input.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DuetFwInputXlrNominalLevel {
    /// Adjustable between 10 and 75 dB.
    Microphone,
    /// +4 dBu, with fixed gain.
    Professional,
    /// -10 dBV, with fixed gain.
    Consumer,
}

impl Default for DuetFwInputXlrNominalLevel {
    fn default() -> Self {
        Self::Microphone
    }
}

/// Input parameters.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DuetFwInputParameters {
    /// Gains of inputs.
    pub gains: [u8; 2],
    /// Polarity of XLR inputs.
    pub polarities: [bool; 2],
    /// Nominal level of XLR inputs.
    pub xlr_nominal_levels: [DuetFwInputXlrNominalLevel; 2],
    /// Whether to enable phantom powering for XLR inputs.
    pub phantom_powerings: [bool; 2],
    /// Source of analog inputs.
    pub srcs: [DuetFwInputSource; 2],
    /// Disable click sound for microphone amplifier.
    pub clickless: bool,
}

fn default_cmds_for_input_params(cmds: &mut Vec<VendorCmd>) {
    cmds.push(VendorCmd::InGain(0, Default::default()));
    cmds.push(VendorCmd::InGain(1, Default::default()));
    cmds.push(VendorCmd::MicPolarity(0, Default::default()));
    cmds.push(VendorCmd::MicPolarity(1, Default::default()));
    cmds.push(VendorCmd::XlrIsMicLevel(0, Default::default()));
    cmds.push(VendorCmd::XlrIsMicLevel(1, Default::default()));
    cmds.push(VendorCmd::XlrIsConsumerLevel(0, Default::default()));
    cmds.push(VendorCmd::XlrIsConsumerLevel(1, Default::default()));
    cmds.push(VendorCmd::MicPhantom(0, Default::default()));
    cmds.push(VendorCmd::MicPhantom(1, Default::default()));
    cmds.push(VendorCmd::InputSourceIsPhone(0, Default::default()));
    cmds.push(VendorCmd::InputSourceIsPhone(1, Default::default()));
    cmds.push(VendorCmd::InClickless(Default::default()));
}

fn cmds_to_input_params(params: &mut DuetFwInputParameters, cmds: &[VendorCmd]) {
    let mut is_mic_levels = [false; 2];
    let mut is_consumer_levels = [false; 2];

    cmds.iter().for_each(|&cmd| match cmd {
        VendorCmd::InGain(i, gain) => {
            if i < params.gains.len() {
                params.gains[i] = gain;
            }
        }
        VendorCmd::MicPolarity(i, polarity) => {
            if i < params.polarities.len() {
                params.polarities[i] = polarity;
            }
        }
        VendorCmd::XlrIsMicLevel(i, is_mic_level) => {
            if i < is_mic_levels.len() {
                is_mic_levels[i] = is_mic_level;
            }
        }
        VendorCmd::XlrIsConsumerLevel(i, is_consumer_level) => {
            if i < is_consumer_levels.len() {
                is_consumer_levels[i] = is_consumer_level;
            }
        }
        VendorCmd::MicPhantom(i, enabled) => {
            if i < params.phantom_powerings.len() {
                params.phantom_powerings[i] = enabled;
            }
        }
        VendorCmd::InputSourceIsPhone(i, is_phone) => {
            if i < params.srcs.len() {
                params.srcs[i] = if is_phone {
                    DuetFwInputSource::Phone
                } else {
                    DuetFwInputSource::Xlr
                };
            }
        }
        VendorCmd::InClickless(enabled) => {
            params.clickless = enabled;
        }
        _ => (),
    });

    params
        .xlr_nominal_levels
        .iter_mut()
        .enumerate()
        .for_each(|(i, xlr_nominal_level)| {
            *xlr_nominal_level = if is_mic_levels[i] {
                DuetFwInputXlrNominalLevel::Microphone
            } else if is_consumer_levels[i] {
                DuetFwInputXlrNominalLevel::Consumer
            } else {
                DuetFwInputXlrNominalLevel::Professional
            };
        });
}

fn cmds_from_input_params(params: &DuetFwInputParameters, cmds: &mut Vec<VendorCmd>) {
    params.gains.iter().enumerate().for_each(|(i, &gain)| {
        cmds.push(VendorCmd::InGain(i, gain));
    });
    params
        .polarities
        .iter()
        .enumerate()
        .for_each(|(i, &polarity)| {
            cmds.push(VendorCmd::MicPolarity(i, polarity));
        });
    params
        .phantom_powerings
        .iter()
        .enumerate()
        .for_each(|(i, &enabled)| {
            cmds.push(VendorCmd::MicPhantom(i, enabled));
        });
    params.srcs.iter().enumerate().for_each(|(i, &src)| {
        let enabled = src == DuetFwInputSource::Phone;
        cmds.push(VendorCmd::InputSourceIsPhone(i, enabled));
    });
    cmds.push(VendorCmd::InClickless(params.clickless));

    let mut is_mic_levels = [false; 2];
    let mut is_consumer_levels = [false; 2];

    params
        .xlr_nominal_levels
        .iter()
        .enumerate()
        .for_each(|(i, xlr_nominal_level)| match xlr_nominal_level {
            DuetFwInputXlrNominalLevel::Microphone => is_mic_levels[i] = true,
            DuetFwInputXlrNominalLevel::Consumer => is_consumer_levels[i] = true,
            _ => (),
        });

    is_mic_levels.iter().enumerate().for_each(|(i, &enabled)| {
        cmds.push(VendorCmd::XlrIsMicLevel(i, enabled));
    });

    is_consumer_levels
        .iter()
        .enumerate()
        .for_each(|(i, &enabled)| {
            cmds.push(VendorCmd::XlrIsConsumerLevel(i, enabled));
        });
}

impl DuetFwParamsSerdes<DuetFwInputParameters> for DuetFwProtocol {
    fn default_cmds_for_params(cmds: &mut Vec<VendorCmd>) {
        default_cmds_for_input_params(cmds);
    }

    fn cmds_to_params(params: &mut DuetFwInputParameters, cmds: &[VendorCmd]) {
        cmds_to_input_params(params, cmds);
    }

    fn cmds_from_params(params: &DuetFwInputParameters, cmds: &mut Vec<VendorCmd>) {
        cmds_from_input_params(params, cmds);
    }
}

impl DuetFwProtocol {
    /// The minimum value of gain in input parameters.
    pub const INPUT_GAIN_MIN: u8 = 10;
    /// The minimum value of gain in input parameters.
    pub const INPUT_GAIN_MAX: u8 = 75;
}

/// Parameters of mixer coefficients.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DuetFwMixerCoefficients {
    /// Coefficients of analog inputs.
    pub analog_inputs: [u16; 2],
    /// Coefficients of stream inputs.
    pub stream_inputs: [u16; 2],
}

/// Parameters of stereo mixer.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DuetFwMixerParams(pub [DuetFwMixerCoefficients; 2]);

fn default_cmds_for_mixer_params(cmds: &mut Vec<VendorCmd>) {
    cmds.push(VendorCmd::MixerSrc(0, 0, Default::default()));
    cmds.push(VendorCmd::MixerSrc(1, 0, Default::default()));
    cmds.push(VendorCmd::MixerSrc(2, 0, Default::default()));
    cmds.push(VendorCmd::MixerSrc(3, 0, Default::default()));
    cmds.push(VendorCmd::MixerSrc(0, 1, Default::default()));
    cmds.push(VendorCmd::MixerSrc(1, 1, Default::default()));
    cmds.push(VendorCmd::MixerSrc(2, 1, Default::default()));
    cmds.push(VendorCmd::MixerSrc(3, 1, Default::default()));
}

fn cmds_to_mixer_params(params: &mut DuetFwMixerParams, cmds: &[VendorCmd]) {
    cmds.iter().for_each(|&cmd| match cmd {
        VendorCmd::MixerSrc(src, dst, coef) => {
            if src < 2 {
                params.0[dst].analog_inputs[src] = coef;
            } else if src < 4 {
                params.0[dst].stream_inputs[src - 2] = coef;
            }
        }
        _ => (),
    });
}

fn cmds_from_mixer_params(params: &DuetFwMixerParams, cmds: &mut Vec<VendorCmd>) {
    params.0.iter().enumerate().for_each(|(dst, coefs)| {
        coefs
            .analog_inputs
            .iter()
            .chain(&coefs.stream_inputs)
            .enumerate()
            .for_each(|(src, &coef)| {
                cmds.push(VendorCmd::MixerSrc(src, dst, coef));
            });
    })
}

impl DuetFwParamsSerdes<DuetFwMixerParams> for DuetFwProtocol {
    fn default_cmds_for_params(cmds: &mut Vec<VendorCmd>) {
        default_cmds_for_mixer_params(cmds);
    }

    fn cmds_to_params(params: &mut DuetFwMixerParams, cmds: &[VendorCmd]) {
        cmds_to_mixer_params(params, cmds);
    }

    fn cmds_from_params(params: &DuetFwMixerParams, cmds: &mut Vec<VendorCmd>) {
        cmds_from_mixer_params(params, cmds);
    }
}

impl DuetFwProtocol {
    /// The minimum value of source gain in mixer parameters.
    pub const MIXER_SOURCE_GAIN_MIN: u16 = 0;
    /// The maximum value of source gain in mixer parameters.
    pub const MIXER_SOURCE_GAIN_MAX: u16 = 0x3fff;
}

/// Target of display.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DuetFwDisplayTarget {
    /// For output.
    Output,
    /// For input.
    Input,
}

impl Default for DuetFwDisplayTarget {
    fn default() -> Self {
        Self::Output
    }
}

/// Mode of display.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DuetFwDisplayMode {
    /// Independent.
    Independent,
    /// Following to knob target.
    FollowingToKnobTarget,
}

impl Default for DuetFwDisplayMode {
    fn default() -> Self {
        Self::Independent
    }
}

/// Overhold of display.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DuetFwDisplayOverhold {
    /// Infinite.
    Infinite,
    /// Keep during two seconds.
    TwoSeconds,
}

impl Default for DuetFwDisplayOverhold {
    fn default() -> Self {
        Self::Infinite
    }
}

/// Parameters of LED display.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DuetFwDisplayParams {
    /// Target to display.
    pub target: DuetFwDisplayTarget,
    /// Mode to display.
    pub mode: DuetFwDisplayMode,
    /// Mode of overhold.
    pub overhold: DuetFwDisplayOverhold,
}

fn default_cmds_for_display_params(cmds: &mut Vec<VendorCmd>) {
    cmds.push(VendorCmd::DisplayIsInput(Default::default()));
    cmds.push(VendorCmd::DisplayFollowToKnob(Default::default()));
    cmds.push(VendorCmd::DisplayOverholdTwoSec(Default::default()));
}

fn cmds_to_display_params(params: &mut DuetFwDisplayParams, cmds: &[VendorCmd]) {
    cmds.iter().for_each(|&cmd| match cmd {
        VendorCmd::DisplayIsInput(enabled) => {
            params.target = if enabled {
                DuetFwDisplayTarget::Input
            } else {
                DuetFwDisplayTarget::Output
            };
        }
        VendorCmd::DisplayFollowToKnob(enabled) => {
            params.mode = if enabled {
                DuetFwDisplayMode::FollowingToKnobTarget
            } else {
                DuetFwDisplayMode::Independent
            };
        }
        VendorCmd::DisplayOverholdTwoSec(enabled) => {
            params.overhold = if enabled {
                DuetFwDisplayOverhold::TwoSeconds
            } else {
                DuetFwDisplayOverhold::Infinite
            };
        }
        _ => (),
    })
}

fn cmds_from_display_params(params: &DuetFwDisplayParams, cmds: &mut Vec<VendorCmd>) {
    let enabled = params.target == DuetFwDisplayTarget::Input;
    cmds.push(VendorCmd::DisplayIsInput(enabled));

    let enabled = params.mode == DuetFwDisplayMode::FollowingToKnobTarget;
    cmds.push(VendorCmd::DisplayFollowToKnob(enabled));

    let enabled = params.overhold == DuetFwDisplayOverhold::TwoSeconds;
    cmds.push(VendorCmd::DisplayOverholdTwoSec(enabled));
}

impl DuetFwParamsSerdes<DuetFwDisplayParams> for DuetFwProtocol {
    fn default_cmds_for_params(cmds: &mut Vec<VendorCmd>) {
        default_cmds_for_display_params(cmds);
    }

    fn cmds_to_params(params: &mut DuetFwDisplayParams, cmds: &[VendorCmd]) {
        cmds_to_display_params(params, cmds);
    }

    fn cmds_from_params(params: &DuetFwDisplayParams, cmds: &mut Vec<VendorCmd>) {
        cmds_from_display_params(params, cmds);
    }
}

/// Type of command for Apogee Duet FireWire.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VendorCmd {
    MicPolarity(usize, bool),
    XlrIsMicLevel(usize, bool),
    XlrIsConsumerLevel(usize, bool),
    MicPhantom(usize, bool),
    OutIsConsumerLevel(bool),
    InGain(usize, u8),
    HwState([u8; 11]),
    OutMute(bool),
    InputSourceIsPhone(usize, bool),
    MixerSrc(usize, usize, u16),
    OutSourceIsMixer(bool),
    DisplayOverholdTwoSec(bool),
    DisplayClear,
    OutVolume(u8),
    MuteForLineOut(bool),
    MuteForHpOut(bool),
    UnmuteForLineOut(bool),
    UnmuteForHpOut(bool),
    DisplayIsInput(bool),
    InClickless(bool),
    DisplayFollowToKnob(bool),
}

impl VendorCmd {
    const APOGEE_PREFIX: [u8; 3] = [0x50, 0x43, 0x4d]; // 'P', 'C', 'M'

    const MIC_POLARITY: u8 = 0x00;
    const XLR_IS_MIC_LEVEL: u8 = 0x01;
    const XLR_IS_CONSUMER_LEVEL: u8 = 0x02;
    const MIC_PHANTOM: u8 = 0x03;
    const OUT_IS_CONSUMER_LEVEL: u8 = 0x04;
    const IN_GAIN: u8 = 0x05;
    const HW_STATE: u8 = 0x07;
    const OUT_MUTE: u8 = 0x09;
    const INPUT_SOURCE_IS_PHONE: u8 = 0x0c;
    const MIXER_SRC: u8 = 0x10;
    const OUT_SOURCE_IS_MIXER: u8 = 0x11;
    const DISPLAY_OVERHOLD_TWO_SEC: u8 = 0x13;
    const DISPLAY_CLEAR: u8 = 0x14;
    const OUT_VOLUME: u8 = 0x15;
    const MUTE_FOR_LINE_OUT: u8 = 0x16;
    const MUTE_FOR_HP_OUT: u8 = 0x17;
    const UNMUTE_FOR_LINE_OUT: u8 = 0x18;
    const UNMUTE_FOR_HP_OUT: u8 = 0x19;
    const DISPLAY_IS_INPUT: u8 = 0x1b;
    const IN_CLICKLESS: u8 = 0x1e;
    const DISPLAY_FOLLOW_TO_KNOB: u8 = 0x22;

    const ON: u8 = 0x70;
    const OFF: u8 = 0x60;

    fn build_args(&self) -> Vec<u8> {
        let mut args = Vec::with_capacity(6);
        args.extend_from_slice(&Self::APOGEE_PREFIX);
        args.extend_from_slice(&[0xff; 3]);

        match self {
            Self::MicPolarity(ch, _) => {
                args[3] = Self::MIC_POLARITY;
                args[4] = 0x80;
                args[5] = *ch as u8;
            }
            Self::XlrIsMicLevel(ch, _) => {
                args[3] = Self::XLR_IS_MIC_LEVEL;
                args[4] = 0x80;
                args[5] = *ch as u8;
            }
            Self::XlrIsConsumerLevel(ch, _) => {
                args[3] = Self::XLR_IS_CONSUMER_LEVEL;
                args[4] = 0x80;
                args[5] = *ch as u8;
            }
            Self::MicPhantom(ch, _) => {
                args[3] = Self::MIC_PHANTOM;
                args[4] = 0x80;
                args[5] = *ch as u8;
            }
            Self::OutIsConsumerLevel(_) => {
                args[3] = Self::OUT_IS_CONSUMER_LEVEL;
                args[4] = 0x80;
            }
            Self::InGain(ch, _) => {
                args[3] = Self::IN_GAIN;
                args[4] = 0x80;
                args[5] = *ch as u8;
            }
            Self::HwState(_) => args[3] = Self::HW_STATE,
            Self::OutMute(_) => {
                args[3] = Self::OUT_MUTE;
                args[4] = 0x80;
            }
            Self::InputSourceIsPhone(ch, _) => {
                args[3] = Self::INPUT_SOURCE_IS_PHONE;
                args[4] = 0x80;
                args[5] = *ch as u8;
            }
            Self::MixerSrc(src, dst, _) => {
                args[3] = Self::MIXER_SRC;
                args[4] = (((*src / 2) << 4) | (*src % 2)) as u8;
                args[5] = *dst as u8;
            }
            Self::OutSourceIsMixer(_) => args[3] = Self::OUT_SOURCE_IS_MIXER,
            Self::DisplayOverholdTwoSec(_) => args[3] = Self::DISPLAY_OVERHOLD_TWO_SEC,
            Self::DisplayClear => args[3] = Self::DISPLAY_CLEAR,
            Self::OutVolume(_) => {
                args[3] = Self::OUT_VOLUME;
                args[4] = 0x80;
            }
            Self::MuteForLineOut(_) => {
                args[3] = Self::MUTE_FOR_LINE_OUT;
                args[4] = 0x80;
            }
            Self::MuteForHpOut(_) => {
                args[3] = Self::MUTE_FOR_HP_OUT;
                args[4] = 0x80;
            }
            Self::UnmuteForLineOut(_) => {
                args[3] = Self::UNMUTE_FOR_LINE_OUT;
                args[4] = 0x80;
            }
            Self::UnmuteForHpOut(_) => {
                args[3] = Self::UNMUTE_FOR_HP_OUT;
                args[4] = 0x80;
            }
            Self::DisplayIsInput(_) => args[3] = Self::DISPLAY_IS_INPUT,
            Self::InClickless(_) => args[3] = Self::IN_CLICKLESS,
            Self::DisplayFollowToKnob(_) => args[3] = Self::DISPLAY_FOLLOW_TO_KNOB,
        }

        args
    }

    fn append_bool(data: &mut Vec<u8>, val: bool) {
        data.push(if val { Self::ON } else { Self::OFF });
    }

    fn append_variable(&self, data: &mut Vec<u8>) {
        match self {
            Self::MicPolarity(_, enabled) => Self::append_bool(data, *enabled),
            Self::XlrIsMicLevel(_, enabled) => Self::append_bool(data, *enabled),
            Self::XlrIsConsumerLevel(_, enabled) => Self::append_bool(data, *enabled),
            Self::MicPhantom(_, enabled) => Self::append_bool(data, *enabled),
            Self::OutIsConsumerLevel(enabled) => Self::append_bool(data, *enabled),
            Self::InGain(_, gain) => data.push(*gain),
            Self::OutMute(enabled) => Self::append_bool(data, *enabled),
            Self::InputSourceIsPhone(_, enabled) => Self::append_bool(data, *enabled),
            Self::MixerSrc(_, _, gain) => data.extend_from_slice(&gain.to_be_bytes()),
            Self::OutSourceIsMixer(enabled) => Self::append_bool(data, *enabled),
            Self::DisplayOverholdTwoSec(enabled) => Self::append_bool(data, *enabled),
            Self::OutVolume(vol) => data.push(*vol),
            Self::MuteForLineOut(enabled) => Self::append_bool(data, *enabled),
            Self::MuteForHpOut(enabled) => Self::append_bool(data, *enabled),
            Self::UnmuteForLineOut(enabled) => Self::append_bool(data, *enabled),
            Self::UnmuteForHpOut(enabled) => Self::append_bool(data, *enabled),
            Self::DisplayIsInput(enabled) => Self::append_bool(data, *enabled),
            Self::InClickless(enabled) => Self::append_bool(data, *enabled),
            Self::DisplayFollowToKnob(enabled) => Self::append_bool(data, *enabled),
            _ => (),
        }
    }

    fn parse_bool(val: &mut bool, data: &[u8], code: u8) -> Result<(), Error> {
        if data[3] != code {
            let msg = format!("code {} expected but {}", code, data[3]);
            Err(Error::new(FileError::Io, &msg))
        } else if data.len() < 7 {
            let msg = format!("Insufficient length of data {}", data.len());
            Err(Error::new(FileError::Io, &msg))
        } else {
            *val = data[6] == Self::ON;
            Ok(())
        }
    }

    fn parse_idx_and_bool(val: &mut bool, data: &[u8], code: u8, idx: usize) -> Result<(), Error> {
        if data[3] != code {
            let msg = format!("code {} expected but {}", code, data[3]);
            Err(Error::new(FileError::Io, &msg))
        } else if data[5] != idx as u8 {
            let msg = format!("index {} expected but {}", idx, data[5]);
            Err(Error::new(FileError::Io, &msg))
        } else if data.len() < 7 {
            let msg = format!("Insufficient length of data {}", data.len());
            Err(Error::new(FileError::Io, &msg))
        } else {
            *val = data[6] == Self::ON;
            Ok(())
        }
    }

    fn parse_variable(&mut self, data: &[u8]) -> Result<(), Error> {
        if &data[..3] != &Self::APOGEE_PREFIX {
            let msg = format!(
                "Unexpected prefix: 0x{:02x}{:02x}{:02x}",
                data[0], data[1], data[2]
            );
            Err(Error::new(FileError::Io, &msg))?;
        } else if data.len() < 7 {
            let msg = format!("Unexpected length of response: {}", data.len());
            Err(Error::new(FileError::Io, &msg))?;
        }

        match self {
            Self::MicPolarity(idx, enabled) => {
                Self::parse_idx_and_bool(enabled, data, Self::MIC_POLARITY, *idx)
            }
            Self::XlrIsMicLevel(idx, enabled) => {
                Self::parse_idx_and_bool(enabled, data, Self::XLR_IS_MIC_LEVEL, *idx)
            }
            Self::XlrIsConsumerLevel(idx, enabled) => {
                Self::parse_idx_and_bool(enabled, data, Self::XLR_IS_CONSUMER_LEVEL, *idx)
            }
            Self::MicPhantom(idx, enabled) => {
                Self::parse_idx_and_bool(enabled, data, Self::MIC_PHANTOM, *idx)
            }
            Self::OutIsConsumerLevel(enabled) => {
                Self::parse_bool(enabled, data, Self::OUT_IS_CONSUMER_LEVEL)
            }
            Self::InGain(idx, gain) => {
                if data[3] != Self::IN_GAIN {
                    let msg = format!("Unexpected cmd code: {}", data[3]);
                    Err(Error::new(FileError::Io, &msg))
                } else if data[5] != *idx as u8 {
                    let msg = format!("Unexpected index of input: {}", data[5]);
                    Err(Error::new(FileError::Io, &msg))
                } else {
                    *gain = data[6];
                    Ok(())
                }
            }
            Self::OutMute(enabled) => Self::parse_bool(enabled, data, Self::OUT_MUTE),
            Self::InputSourceIsPhone(idx, enabled) => {
                Self::parse_idx_and_bool(enabled, data, Self::INPUT_SOURCE_IS_PHONE, *idx)
            }
            Self::MixerSrc(src, dst, gain) => {
                if data[3] != Self::MIXER_SRC {
                    let msg = format!("Unexpected cmd code: {}", data[3]);
                    Err(Error::new(FileError::Io, &msg))
                } else {
                    if data[4] != (((*src / 2) << 4) | (*src % 2)) as u8 {
                        let msg = format!("Unexpected mixer source: {}", data[4]);
                        Err(Error::new(FileError::Io, &msg))
                    } else if data[5] != *dst as u8 {
                        let msg = format!("Unexpected mixer destination: {}", data[5]);
                        Err(Error::new(FileError::Io, &msg))
                    } else {
                        let mut doublet = [0; 2];
                        doublet.copy_from_slice(&data[6..8]);
                        *gain = u16::from_be_bytes(doublet);
                        Ok(())
                    }
                }
            }
            Self::HwState(raw) => {
                if data[3] != Self::HW_STATE {
                    let msg = format!("Unexpected cmd code: {}", data[3]);
                    Err(Error::new(FileError::Io, &msg))
                } else {
                    raw.copy_from_slice(&data[6..17]);
                    Ok(())
                }
            }
            Self::OutSourceIsMixer(enabled) => {
                Self::parse_bool(enabled, data, Self::OUT_SOURCE_IS_MIXER)
            }
            Self::DisplayOverholdTwoSec(enabled) => {
                Self::parse_bool(enabled, data, Self::DISPLAY_OVERHOLD_TWO_SEC)
            }
            Self::OutVolume(vol) => {
                if data[3] != Self::OUT_VOLUME {
                    let msg = format!("Unexpected cmd code: {}", data[3]);
                    Err(Error::new(FileError::Io, &msg))
                } else {
                    *vol = data[6];
                    Ok(())
                }
            }
            Self::MuteForLineOut(enabled) => {
                Self::parse_bool(enabled, data, Self::MUTE_FOR_LINE_OUT)
            }
            Self::MuteForHpOut(enabled) => Self::parse_bool(enabled, data, Self::MUTE_FOR_HP_OUT),
            Self::UnmuteForLineOut(enabled) => {
                Self::parse_bool(enabled, data, Self::UNMUTE_FOR_LINE_OUT)
            }
            Self::UnmuteForHpOut(enabled) => {
                Self::parse_bool(enabled, data, Self::UNMUTE_FOR_HP_OUT)
            }
            Self::DisplayIsInput(enabled) => {
                Self::parse_bool(enabled, data, Self::DISPLAY_IS_INPUT)
            }
            Self::InClickless(enabled) => Self::parse_bool(enabled, data, Self::IN_CLICKLESS),
            Self::DisplayFollowToKnob(enabled) => {
                Self::parse_bool(enabled, data, Self::DISPLAY_FOLLOW_TO_KNOB)
            }
            _ => Ok(()),
        }
    }
}

/// AV/C vendor-dependent command specific to Apogee Duet FireWire.
struct ApogeeCmd {
    cmd: VendorCmd,
    op: VendorDependent,
}

impl ApogeeCmd {
    fn new(cmd: VendorCmd) -> Self {
        ApogeeCmd {
            cmd,
            op: VendorDependent::new(&APOGEE_OUI),
        }
    }
}

impl AvcOp for ApogeeCmd {
    const OPCODE: u8 = VendorDependent::OPCODE;
}

impl AvcControl for ApogeeCmd {
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        let mut data = self.cmd.build_args();
        self.cmd.append_variable(&mut data);
        self.op.data = data;
        AvcControl::build_operands(&mut self.op, addr)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AvcControl::parse_operands(&mut self.op, addr, operands)
    }
}

impl AvcStatus for ApogeeCmd {
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        self.op.data = self.cmd.build_args();
        AvcStatus::build_operands(&mut self.op, addr)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AvcStatus::parse_operands(&mut self.op, addr, operands)?;
        self.cmd
            .parse_variable(&self.op.data)
            .map_err(|_| AvcRespParseError::UnexpectedOperands(4))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn knob_params_serdes() {
        let params = DuetFwKnobState {
            output_mute: true,
            target: DuetFwKnobTarget::InputPair0,
            output_volume: 0x3f,
            input_gains: [0x4e, 0x1c],
        };
        let mut cmds = Vec::new();
        cmds_from_knob_params(&params, &mut cmds);

        let mut p = DuetFwKnobState::default();
        cmds_to_knob_params(&mut p, &cmds);

        assert_eq!(params, p);

        let mut defaults = Vec::new();
        default_cmds_for_knob_params(&mut defaults);

        assert_eq!(cmds.len(), defaults.len());
    }

    #[test]
    fn output_params_serdes() {
        let params = DuetFwOutputParams {
            mute: true,
            volume: 0x9a,
            source: DuetFwOutputSource::MixerOutputPair0,
            nominal_level: DuetFwOutputNominalLevel::Consumer,
            line_mute_mode: DuetFwOutputMuteMode::Swapped,
            hp_mute_mode: DuetFwOutputMuteMode::Normal,
        };

        let mut cmds = Vec::new();
        cmds_from_output_params(&params, &mut cmds);

        let mut p = DuetFwOutputParams::default();
        cmds_to_output_params(&mut p, &cmds);

        assert_eq!(params, p);

        let mut defaults = Vec::new();
        default_cmds_for_output_params(&mut defaults);

        assert_eq!(cmds.len(), defaults.len());
    }

    #[test]
    fn input_params_serdes() {
        let params = DuetFwInputParameters {
            gains: [0x0f, 0xef],
            polarities: [true, false],
            xlr_nominal_levels: [
                DuetFwInputXlrNominalLevel::Consumer,
                DuetFwInputXlrNominalLevel::Microphone,
            ],
            phantom_powerings: [false, true],
            srcs: [DuetFwInputSource::Phone, DuetFwInputSource::Xlr],
            clickless: true,
        };

        let mut cmds = Vec::new();
        cmds_from_input_params(&params, &mut cmds);

        let mut p = DuetFwInputParameters::default();
        cmds_to_input_params(&mut p, &cmds);

        assert_eq!(params, p);

        let mut defaults = Vec::new();
        default_cmds_for_input_params(&mut defaults);

        assert_eq!(cmds.len(), defaults.len());
    }

    #[test]
    fn mixer_params_serdes() {
        let params = DuetFwMixerParams([
            DuetFwMixerCoefficients {
                analog_inputs: [0x7b, 0x1d],
                stream_inputs: [0x25, 0x83],
            },
            DuetFwMixerCoefficients {
                analog_inputs: [0x01, 0x96],
                stream_inputs: [0xfa, 0xbc],
            },
        ]);

        let mut cmds = Vec::new();
        cmds_from_mixer_params(&params, &mut cmds);

        let mut p = DuetFwMixerParams::default();
        cmds_to_mixer_params(&mut p, &cmds);

        assert_eq!(params, p);

        let mut defaults = Vec::new();
        default_cmds_for_mixer_params(&mut defaults);

        assert_eq!(cmds.len(), defaults.len());
    }

    #[test]
    fn display_params_serdes() {
        let params = DuetFwDisplayParams {
            target: DuetFwDisplayTarget::Input,
            mode: DuetFwDisplayMode::FollowingToKnobTarget,
            overhold: DuetFwDisplayOverhold::TwoSeconds,
        };

        let mut cmds = Vec::new();
        cmds_from_display_params(&params, &mut cmds);

        let mut p = DuetFwDisplayParams::default();
        cmds_to_display_params(&mut p, &cmds);

        assert_eq!(params, p);

        let mut defaults = Vec::new();
        default_cmds_for_display_params(&mut defaults);

        assert_eq!(cmds.len(), defaults.len());
    }

    #[test]
    fn apogee_cmd_proto_operands() {
        // No argument command.
        let mut op = ApogeeCmd::new(VendorCmd::OutSourceIsMixer(Default::default()));
        let operands = [0x00, 0x03, 0xdb, 0x50, 0x43, 0x4d, 0x11, 0xff, 0xff, 0x70];
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        if let VendorCmd::OutSourceIsMixer(enabled) = &op.cmd {
            assert_eq!(*enabled, true);
        } else {
            unreachable!();
        }

        let o = AvcStatus::build_operands(&mut op, &AvcAddr::Unit).unwrap();
        assert_eq!(o, operands[..9]);

        let mut op = ApogeeCmd::new(VendorCmd::OutSourceIsMixer(Default::default()));
        let operands = [0x00, 0x03, 0xdb, 0x50, 0x43, 0x4d, 0x11, 0xff, 0xff, 0x60];
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();

        let o = AvcControl::build_operands(&mut op, &AvcAddr::Unit).unwrap();
        assert_eq!(o, operands);

        // One argument command.
        let mut op = ApogeeCmd::new(VendorCmd::XlrIsConsumerLevel(1, true));
        let operands = [0x00, 0x03, 0xdb, 0x50, 0x43, 0x4d, 0x02, 0x80, 0x01, 0x70];
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        if let VendorCmd::XlrIsConsumerLevel(idx, enabled) = &op.cmd {
            assert_eq!(*idx, 1);
            assert_eq!(*enabled, true);
        } else {
            unreachable!();
        }

        let o = AvcStatus::build_operands(&mut op, &AvcAddr::Unit).unwrap();
        assert_eq!(o, operands[..9]);

        let mut op = ApogeeCmd::new(VendorCmd::XlrIsConsumerLevel(1, true));
        let operands = [0x00, 0x03, 0xdb, 0x50, 0x43, 0x4d, 0x02, 0x80, 0x01, 0x70];
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        if let VendorCmd::XlrIsConsumerLevel(idx, enabled) = &op.cmd {
            assert_eq!(*idx, 1);
            assert_eq!(*enabled, true);
        } else {
            unreachable!();
        }

        let o = AvcControl::build_operands(&mut op, &AvcAddr::Unit).unwrap();
        assert_eq!(o, operands);

        // Two arguments command.
        let mut op = ApogeeCmd::new(VendorCmd::MixerSrc(1, 0, Default::default()));
        let operands = [
            0x00, 0x03, 0xdb, 0x50, 0x43, 0x4d, 0x10, 0x01, 0x00, 0xde, 0x00, 0xad, 0x01, 0xbe,
            0x02, 0xef,
        ];
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        if let VendorCmd::MixerSrc(src, dst, gain) = &op.cmd {
            assert_eq!(*src, 1);
            assert_eq!(*dst, 0);
            assert_eq!(*gain, 0xde00);
        } else {
            unreachable!();
        }

        let o = AvcStatus::build_operands(&mut op, &AvcAddr::Unit).unwrap();
        assert_eq!(o, operands[..9]);

        let mut op = ApogeeCmd::new(VendorCmd::MixerSrc(1, 0, 0xde00));
        let operands = [
            0x00, 0x03, 0xdb, 0x50, 0x43, 0x4d, 0x10, 0x01, 0x00, 0xde, 0x00, 0xad, 0x01, 0xbe,
            0x02, 0xef,
        ];
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();

        let o = AvcControl::build_operands(&mut op, &AvcAddr::Unit).unwrap();
        assert_eq!(o, operands[..11]);

        // Command for block request.
        let mut op = ApogeeCmd::new(VendorCmd::HwState(Default::default()));
        let operands = [
            0x00, 0x03, 0xdb, 0x50, 0x43, 0x4d, 0x07, 0xff, 0xff, 0xde, 0x00, 0xad, 0x01, 0xbe,
            0x02, 0xef, 0xde, 0xad, 0xbe, 0xef,
        ];
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        if let VendorCmd::HwState(raw) = &op.cmd {
            assert_eq!(
                raw,
                &[0xde, 0x00, 0xad, 0x01, 0xbe, 0x02, 0xef, 0xde, 0xad, 0xbe, 0xef]
            );
        } else {
            unreachable!();
        }

        let o = AvcStatus::build_operands(&mut op, &AvcAddr::Unit).unwrap();
        assert_eq!(o, operands[..9]);
    }
}
