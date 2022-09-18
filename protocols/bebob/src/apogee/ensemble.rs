// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Apogee Electronics Ensemble FireWire.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Apogee Electronics Ensemble FireWire.
//!
//! DM1500 ASIC is used for Apogee Ensemble FireWire.
//!
//! ## Diagram of internal signal flow for Apogee Ensemble FireWire
//!
//! ```text
//!                                ++==========++
//! analog-inputs (8 channels) --> ||  18x18   ||
//! spdif-inputs (2 channels) ---> ||  capture || --> stream-outputs (18 channels)
//! adat-inputs (8 channels) ----> ||  router  ||
//!                                ++==========++
//!
//!                                ++==========++
//! analog-inputs (8 channels) --> ||  36x4    ||
//! spdif-inputs (2 channels) ---> ||          || --> mixer-outputs (4 channels)
//! adat-inputs (8 channels) ----> ||  mixer   ||
//! stream-inputs (18 channels) -> ||          ||
//!                                ++==========++
//!
//!                                ++==========++
//! analog-inputs (8 channels) --> ||  40x18   ||
//! spdif-inputs (2 channels) ---> ||          || --> analog-outputs (8 channels)
//! adat-inputs (8 channels) ----> || playback || --> spdif-outputs (2 channels)
//! stream-inputs (18 channels) -> ||          || --> adat-outputs (8 channels)
//! mixer-outputs (4 channels) --> ||  router  ||
//!                                ++==========++
//!
//! (source) ----------------------------> spdif-output-1/2
//!                           ^
//!                           |
//!                 ++================++
//!                 || rate converter || (optional)
//!                 ++================++
//!                           |
//!                           v
//! spdif-input-1/2 ------------------------> (destination)
//!
//! analog-input-1/2 ------------------------> (destination)
//! analog-input-3/4 ------------------------> (destination)
//! analog-input-5/6 ------------------------> (destination)
//! analog-input-7/8 ------------------------> (destination)
//! spdif-input-1/2 -------------------------> (destination)
//!                           ^
//!                           |
//!                ++==================++
//!                || format converter || (optional)
//!                ++==================++
//!                           |
//!                           v
//! (source) ------------------------------> spdif-output-1/2
//! ```
//!
//! The protocol implementation for Apogee Ensemble FireWire was written with firmware version
//! below:
//!
//! ```sh
//! $ cargo run --bin bco-bootloader-info -- /dev/fw1
//! protocol:
//!   version: 3
//! bootloader:
//!   timestamp: 2006-04-07T11:13:17+0000
//!   version: 0.0.0
//! hardware:
//!   GUID: 0x0000f1a50003db05
//!   model ID: 0x000000
//!   revision: 0.0.0
//! software:
//!   timestamp: 2008-11-08T12:36:10+0000
//!   ID: 0x0001eeee
//!   revision: 0.0.5297
//! image:
//!   base address: 0x400c0080
//!   maximum size: 0x156aa8
//! ```

use super::*;

/// The protocol implementation for media and sampling clock of Ensemble FireWire.
#[derive(Default)]
pub struct EnsembleClkProtocol;

impl MediaClockFrequencyOperation for EnsembleClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000, 176400, 192000];
}

impl SamplingClockSourceOperation for EnsembleClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 7,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 7,
        }),
        // S/PDIF-coax
        SignalAddr::Unit(SignalUnitAddr::Ext(4)),
        // Optical
        SignalAddr::Unit(SignalUnitAddr::Ext(5)),
        // Word clock
        SignalAddr::Unit(SignalUnitAddr::Ext(6)),
    ];
}

/// Parameters of sample format converter.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct EnsembleConverterParameters {
    /// The target of sample format converter.
    pub format_target: FormatConvertTarget,
    /// The target of sampling rate converter.
    pub rate_target: RateConvertTarget,
    /// The sampling rate to convert.
    pub converted_rate: RateConvertRate,
    /// The mode of CD (44.1 kHz/16 bit sample).
    pub cd_mode: bool,
}

/// Protocol implementation for converter parameters.
#[derive(Default, Debug)]
pub struct EnsembleConverterProtocol;

impl EnsembleParametersConverter<EnsembleConverterParameters> for EnsembleConverterProtocol {
    fn parse_cmds(params: &mut EnsembleConverterParameters, cmds: &[EnsembleCmd]) {
        cmds.iter().for_each(|cmd| match cmd {
            &EnsembleCmd::FormatConvert(format_target) => {
                params.format_target = format_target;
            }
            &EnsembleCmd::RateConvert(rate_target, converted_rate) => {
                params.rate_target = rate_target;
                params.converted_rate = converted_rate;
            }
            &EnsembleCmd::Hw(HwCmd::CdMode(cd_mode)) => {
                params.cd_mode = cd_mode;
            }
            _ => (),
        });
    }

    fn build_cmds(params: &EnsembleConverterParameters) -> Vec<EnsembleCmd> {
        vec![
            EnsembleCmd::FormatConvert(params.format_target),
            EnsembleCmd::RateConvert(params.rate_target, params.converted_rate),
            EnsembleCmd::Hw(HwCmd::CdMode(params.cd_mode)),
        ]
    }
}

impl From<&EnsembleConverterParameters> for Vec<EnsembleCmd> {
    fn from(params: &EnsembleConverterParameters) -> Self {
        EnsembleConverterProtocol::build_cmds(params)
    }
}

impl EnsembleParameterProtocol<EnsembleConverterParameters> for EnsembleConverterProtocol {}

/// Parameters of display meters.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct EnsembleDisplayParameters {
    /// Whether to enable/disable display.
    pub enabled: bool,
    /// Force to illuminate display.
    pub illuminate: bool,
    /// The target for meter display.
    pub target: DisplayMeterTarget,
    /// Whether to enable/disable overhold of peak detection.
    pub overhold: bool,
}

/// Protocol implementation for display parameters.
#[derive(Default, Debug)]
pub struct EnsembleDisplayProtocol;

impl EnsembleParametersConverter<EnsembleDisplayParameters> for EnsembleDisplayProtocol {
    fn parse_cmds(params: &mut EnsembleDisplayParameters, cmds: &[EnsembleCmd]) {
        cmds.iter().for_each(|cmd| match cmd {
            &EnsembleCmd::Hw(HwCmd::DisplayMode(enabled)) => {
                params.enabled = enabled;
            }
            &EnsembleCmd::Hw(HwCmd::DisplayIlluminate(illuminate)) => {
                params.illuminate = illuminate;
            }
            &EnsembleCmd::Hw(HwCmd::DisplayTarget(target)) => {
                params.target = target;
            }
            &EnsembleCmd::Hw(HwCmd::DisplayOverhold(overhold)) => {
                params.overhold = overhold;
            }
            _ => (),
        })
    }

    fn build_cmds(params: &EnsembleDisplayParameters) -> Vec<EnsembleCmd> {
        vec![
            EnsembleCmd::Hw(HwCmd::DisplayMode(params.enabled)),
            EnsembleCmd::Hw(HwCmd::DisplayIlluminate(params.illuminate)),
            EnsembleCmd::Hw(HwCmd::DisplayTarget(params.target)),
            EnsembleCmd::Hw(HwCmd::DisplayOverhold(params.overhold)),
        ]
    }
}

impl From<&EnsembleDisplayParameters> for Vec<EnsembleCmd> {
    fn from(params: &EnsembleDisplayParameters) -> Self {
        EnsembleDisplayProtocol::build_cmds(params)
    }
}

impl EnsembleParameterProtocol<EnsembleDisplayParameters> for EnsembleDisplayProtocol {}

/// Parameters of analog/digital inputs. The gains, phantoms, and polarities parameters
/// are available when channel 0-3 levels are for mic.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct EnsembleInputParameters {
    /// Whether to enable/disable limitter of analog inputs.
    pub limits: [bool; 8],
    /// The nominal level of analog inputs.
    pub levels: [InputNominalLevel; 8],
    /// The gain of microphone inputs.
    pub gains: [u8; 4],
    /// Whether to enable/disable phantom powering for microphone inputs.
    pub phantoms: [bool; 4],
    /// Whether to invert polarity for microphone inputs.
    pub polarities: [bool; 4],
    /// The mode of optical inputs.
    pub opt_iface_mode: OptIfaceMode,
}

/// Protocol implementation for input parameters.
#[derive(Default, Debug)]
pub struct EnsembleInputProtocol;

impl EnsembleInputProtocol {
    /// The maximum value of gain.
    pub const GAIN_MIN: u8 = 0;

    /// The minimum value of gain.
    pub const GAIN_MAX: u8 = 75;

    /// The step of value of gain.
    pub const GAIN_STEP: u8 = 1;
}

impl EnsembleParametersConverter<EnsembleInputParameters> for EnsembleInputProtocol {
    fn parse_cmds(params: &mut EnsembleInputParameters, cmds: &[EnsembleCmd]) {
        cmds.iter().for_each(|cmd| match cmd {
            &EnsembleCmd::InputLimit(i, limit) => {
                if i < params.limits.len() {
                    params.limits[i] = limit;
                }
            }
            &EnsembleCmd::InputNominalLevel(i, level) => {
                if i < params.levels.len() {
                    params.levels[i] = level;
                }
            }
            &EnsembleCmd::MicGain(i, gain) => {
                if i < params.gains.len() {
                    params.gains[i] = gain;
                }
            }
            &EnsembleCmd::MicPower(i, phantom) => {
                if i < params.phantoms.len() {
                    params.phantoms[i] = phantom;
                }
            }
            &EnsembleCmd::MicPolarity(i, polarity) => {
                if i < params.polarities.len() {
                    params.polarities[i] = polarity;
                }
            }
            &EnsembleCmd::InputOptIface(opt_iface_mode) => {
                params.opt_iface_mode = opt_iface_mode;
            }
            _ => (),
        })
    }

    fn build_cmds(params: &EnsembleInputParameters) -> Vec<EnsembleCmd> {
        let mut cmds = Vec::new();

        params
            .limits
            .iter()
            .enumerate()
            .for_each(|(i, &limit)| cmds.push(EnsembleCmd::InputLimit(i, limit)));

        params
            .levels
            .iter()
            .enumerate()
            .for_each(|(i, &level)| cmds.push(EnsembleCmd::InputNominalLevel(i, level)));

        params
            .gains
            .iter()
            .enumerate()
            .for_each(|(i, &gain)| cmds.push(EnsembleCmd::MicGain(i, gain)));

        params
            .phantoms
            .iter()
            .enumerate()
            .for_each(|(i, &phantom)| cmds.push(EnsembleCmd::MicPower(i, phantom)));

        params
            .polarities
            .iter()
            .enumerate()
            .for_each(|(i, &polarity)| cmds.push(EnsembleCmd::MicPolarity(i, polarity)));

        cmds.push(EnsembleCmd::InputOptIface(params.opt_iface_mode));

        cmds
    }
}

impl From<&EnsembleInputParameters> for Vec<EnsembleCmd> {
    fn from(params: &EnsembleInputParameters) -> Self {
        EnsembleInputProtocol::build_cmds(params)
    }
}

impl EnsembleParameterProtocol<EnsembleInputParameters> for EnsembleInputProtocol {}

/// Parameters of analog/digital outputs.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct EnsembleOutputParameters {
    /// The volume of 1st pair of analog outputs.
    pub vol: u8,
    /// The volume of headphone outputs.
    pub headphone_vols: [u8; 2],
    /// The nominal level of outputs.
    pub levels: [OutputNominalLevel; 8],
    /// The mode of optical outputs.
    pub opt_iface_mode: OptIfaceMode,
}

/// Protocol implementation for output parameters.
#[derive(Default, Debug)]
pub struct EnsembleOutputProtocol;

impl EnsembleOutputProtocol {
    /// The minimum value of volume.
    pub const VOL_MIN: u8 = 0;

    /// The maximum value of volume.
    pub const VOL_MAX: u8 = 0x7f;

    /// The step of value of volume.
    pub const VOL_STEP: u8 = 1;

    fn coef_from_vol(vol: u8) -> u8 {
        Self::VOL_MAX - vol
    }

    fn coef_to_vol(coef: u8) -> u8 {
        Self::VOL_MAX - coef
    }
}

impl EnsembleParametersConverter<EnsembleOutputParameters> for EnsembleOutputProtocol {
    fn parse_cmds(params: &mut EnsembleOutputParameters, cmds: &[EnsembleCmd]) {
        cmds.iter().for_each(|cmd| match cmd {
            &EnsembleCmd::OutVol(i, coef) => {
                let vol = Self::coef_to_vol(coef);
                match i {
                    0 => params.vol = vol,
                    1..=2 => {
                        params.headphone_vols[i - 1] = vol;
                    }
                    _ => (),
                }
            }
            &EnsembleCmd::OutputNominalLevel(i, level) => {
                if i < params.levels.len() {
                    params.levels[i] = level;
                }
            }
            &EnsembleCmd::OutputOptIface(opt_iface_mode) => {
                params.opt_iface_mode = opt_iface_mode;
            }
            _ => (),
        })
    }

    fn build_cmds(params: &EnsembleOutputParameters) -> Vec<EnsembleCmd> {
        let mut cmds = Vec::new();

        [params.vol]
            .iter()
            .chain(&params.headphone_vols)
            .enumerate()
            .for_each(|(i, &vol)| cmds.push(EnsembleCmd::OutVol(i, Self::coef_from_vol(vol))));

        params
            .levels
            .iter()
            .enumerate()
            .for_each(|(i, &level)| cmds.push(EnsembleCmd::OutputNominalLevel(i, level)));

        cmds.push(EnsembleCmd::OutputOptIface(params.opt_iface_mode));

        cmds
    }
}

impl From<&EnsembleOutputParameters> for Vec<EnsembleCmd> {
    fn from(params: &EnsembleOutputParameters) -> Self {
        EnsembleOutputProtocol::build_cmds(params)
    }
}

impl EnsembleParameterProtocol<EnsembleOutputParameters> for EnsembleOutputProtocol {}

/// Parameters of input/output source.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EnsembleSourceParameters {
    /// To (18):
    ///   analog-output-0, analog-output-1, analog-output-2, analog-output-3,
    ///   analog-output-4, analog-output-5, analog-output-6, analog-output-7,
    ///   spdif-output-0, spdif-output-1,
    ///   adat-output-0, adat-output-1, adat-input-output-2, adat-output-3,
    ///   adat-output-4, adat-output-5, adat-input-output-6, adat-output-7
    /// From (40):
    ///   analog-input-0, analog-input-1, analog-input-2, analog-input-3,
    ///   analog-input-4, analog-input-5, analog-input-6, analog-input-7,
    ///   stream-input-0, stream-input-1, stream-input-2, stream-input-3,
    ///   stream-input-4, stream-input-5, stream-input-6, stream-input-7,
    ///   stream-input-8, stream-input-9, stream-input-10, stream-input-11,
    ///   stream-input-12, stream-input-13, stream-input-14, stream-input-15,
    ///   stream-input-16, stream-input-17,
    ///   spdif-input-0, spdif-input-1,
    ///   adat-input-0, adat-input-1, adat-input-2, adat-input-3,
    ///   adat-input-4, adat-input-5, adat-input-6, adat-input-7,
    ///   mixer-output-1, mixer-output-2, mixer-output-3, mixer-output-4,
    pub output_sources: [usize; 18],
    /// To (18):
    ///  stream-output-0, stream-output-1, stream-output-2, stream-output-3,
    ///  stream-output-4, stream-output-5, stream-output-6, stream-output-7,
    ///  stream-output-8, stream-output-9, stream-output-10, stream-output-11,
    ///  stream-output-12, stream-output-13, stream-output-14, stream-output-15,
    ///  stream-output-16, stream-output-17,
    /// From(18):
    ///  analog-input-0, analog-input-1, analog-input-2, analog-input-3,
    ///  analog-input-4, analog-input-5, analog-input-6, analog-input-7,
    ///  spdif-input-0, spdif-input-1,
    ///  adat-input-0, adat-input-1, adat-input-2, adat-input-3,
    ///  adat-input-4, adat-input-5, adat-input-6, adat-input-7,
    pub capture_sources: [usize; 18],
    /// From (6):
    ///  analog-output-1/2, analog-output-3/4, analog-output-5/6, analog-output-7/8,
    ///  spdif-output-1/2, none,
    pub headphone_sources: [usize; 2],
}

impl Default for EnsembleSourceParameters {
    fn default() -> Self {
        let mut output_sources = [0; 18];
        output_sources
            .iter_mut()
            .enumerate()
            .for_each(|(i, v)| *v = i + 8);

        let mut capture_sources = [0; 18];
        capture_sources
            .iter_mut()
            .enumerate()
            .for_each(|(i, v)| *v = i);

        let mut headphone_sources = [0; 2];
        headphone_sources
            .iter_mut()
            .enumerate()
            .for_each(|(i, v)| *v = i);

        EnsembleSourceParameters {
            output_sources,
            capture_sources,
            headphone_sources,
        }
    }
}

/// Protocol implementation for source parameters.
#[derive(Default, Debug)]
pub struct EnsembleSourceProtocol;

impl EnsembleSourceProtocol {
    fn dst_id_from_out_idx(idx: usize) -> Option<usize> {
        match idx {
            0..=7 => Some(idx),
            8..=17 => Some(idx + 18),
            _ => None,
        }
    }

    fn dst_id_to_out_idx(dst_id: usize) -> Option<usize> {
        match dst_id {
            0..=7 => Some(dst_id),
            8..=25 => None,
            26..=35 => Some(dst_id - 18),
            _ => None,
        }
    }

    fn dst_id_from_cap_idx(idx: usize) -> Option<usize> {
        if idx < 18 {
            Some(idx + 8)
        } else {
            None
        }
    }

    fn dst_id_to_cap_idx(dst_id: usize) -> Option<usize> {
        match dst_id {
            0..=7 => None,
            8..=25 => Some(dst_id - 8),
            _ => None,
        }
    }

    fn src_id_from_stream_idx(src_id: usize) -> Option<usize> {
        match src_id {
            0..=7 => Some(src_id),
            8..=17 => Some(src_id + 18),
            _ => None,
        }
    }

    fn src_id_to_stream_idx(idx: usize) -> Option<usize> {
        match idx {
            0..=7 => Some(idx),
            8..=25 => None,
            26..=35 => Some(idx - 18),
            _ => None,
        }
    }
}

impl EnsembleParametersConverter<EnsembleSourceParameters> for EnsembleSourceProtocol {
    fn build_cmds(params: &EnsembleSourceParameters) -> Vec<EnsembleCmd> {
        let mut cmds = Vec::new();

        params
            .output_sources
            .iter()
            .enumerate()
            .for_each(|(out_idx, &src_id)| {
                Self::dst_id_from_out_idx(out_idx).map(|dst_id| {
                    cmds.push(EnsembleCmd::IoRouting(dst_id, src_id));
                });
            });

        params
            .capture_sources
            .iter()
            .enumerate()
            .for_each(|(cap_idx, &stream_idx)| {
                Self::dst_id_from_cap_idx(cap_idx).map(|dst_id| {
                    Self::src_id_from_stream_idx(stream_idx).map(|src_id| {
                        cmds.push(EnsembleCmd::IoRouting(dst_id, src_id));
                    });
                });
            });

        params
            .headphone_sources
            .iter()
            .enumerate()
            .for_each(|(dst_id, &src_id)| {
                cmds.push(EnsembleCmd::HpSrc(dst_id, src_id));
            });

        cmds
    }

    fn parse_cmds(params: &mut EnsembleSourceParameters, cmds: &[EnsembleCmd]) {
        cmds.iter().for_each(|cmd| match cmd {
            &EnsembleCmd::IoRouting(dst_id, src_id) => {
                if let Some(cap_idx) = Self::dst_id_to_cap_idx(dst_id) {
                    Self::src_id_to_stream_idx(src_id).map(|stream_idx| {
                        params.capture_sources[cap_idx] = stream_idx;
                    });
                } else if let Some(out_idx) = Self::dst_id_to_out_idx(dst_id) {
                    params.output_sources[out_idx] = src_id;
                }
            }
            &EnsembleCmd::HpSrc(dst_id, src_id) => {
                if dst_id < params.headphone_sources.len() {
                    params.headphone_sources[dst_id] = src_id;
                }
            }
            _ => (),
        })
    }
}
impl From<&EnsembleSourceParameters> for Vec<EnsembleCmd> {
    fn from(params: &EnsembleSourceParameters) -> Self {
        EnsembleSourceProtocol::build_cmds(params)
    }
}

impl EnsembleParameterProtocol<EnsembleSourceParameters> for EnsembleSourceProtocol {}

/// Parameters of signal multiplexer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EnsembleMixerParameters {
    /// To (4):
    ///   mixer-output-0, mixer-output-1, mixer-output-2, mixer-output-3
    ///
    /// From (36):
    /// analog-input-0, analog-input-1, analog-input-2, analog-input-3,
    /// analog-input-4, analog-input-5, analog-input-6, analog-input-7,
    /// stream-input-0, stream-input-1, stream-input-2, stream-input-3,
    /// stream-input-4, stream-input-5, stream-input-6, stream-input-7,
    /// stream-input-8,stream-input-9, stream-input-10, stream-input-11,
    /// stream-input-12, stream-input-13, stream-input-14, stream-input-15,
    /// stream-input-16, stream-input-17,
    /// adat-input-0, adat-input-1, adat-input-2, adat-input-3,
    /// adat-input-4, adat-input-5, adat-input-6, adat-input-7,
    /// spdif-input-0, spdif-input-1,
    pub src_gains: [[i16; 36]; 4],
}

impl Default for EnsembleMixerParameters {
    fn default() -> Self {
        let mut src_gains = [[0; 36]; 4];

        src_gains.iter_mut().enumerate().for_each(|(i, gains)| {
            gains
                .iter_mut()
                .enumerate()
                .filter(|(j, _)| i % 2 == j % 2)
                .for_each(|(_, gain)| *gain = EnsembleMixerProtocol::GAIN_MAX);
        });
        Self { src_gains }
    }
}

/// Mixer protocol.
#[derive(Default, Debug)]
pub struct EnsembleMixerProtocol;

impl EnsembleMixerProtocol {
    /// The minimum value of gain.
    pub const GAIN_MIN: i16 = 0;

    /// The maximum value of gain.
    pub const GAIN_MAX: i16 = 0xff;

    /// The step of value of gain.
    pub const GAIN_STEP: i16 = 0x01;

    fn array_indices_to_cmd_indices(dst_idx: usize, src_idx: usize) -> (usize, usize, usize) {
        let pair_idx = dst_idx / 2;
        let target_idx = src_idx / 9;
        let coef_idx = (dst_idx % 2) + (src_idx % 9) * 2;

        (pair_idx, target_idx, coef_idx)
    }

    fn array_indices_from_cmd_indices(
        pair_idx: usize,
        target_idx: usize,
        coef_idx: usize,
    ) -> (usize, usize) {
        let dst_idx = pair_idx * 2 + coef_idx % 2;
        let src_idx = target_idx * 9 + coef_idx / 2;

        (dst_idx, src_idx)
    }

    fn partial_update_by_cmd_content(
        params: &mut EnsembleMixerParameters,
        pair_idx: usize,
        target_idx: usize,
        gains: &[i16],
    ) {
        assert!(pair_idx < 2);
        assert!(target_idx < 4);
        assert_eq!(gains.len(), MIXER_COEFFICIENT_COUNT);

        gains.iter().enumerate().for_each(|(coef_idx, &gain)| {
            let (dst_idx, src_idx) =
                Self::array_indices_from_cmd_indices(pair_idx, target_idx, coef_idx);
            params.src_gains[dst_idx][src_idx] = gain;
        });
    }
}

// NOTE:
//
// Array layout:
//
//                src_idx 0-35
//              +--------------------------------+
//    dst_idx 0 |   A (L)  B (L)  C (L)  D (L)   |
//              +--------------------------------+
//    dst_idx 1 |   A (R)  B (R)  C (R)  D (R)   |
//              +--------------------------------+
//    dst_idx 2 |   E (L)  F (L)  G (L)  H (L)   |
//              +--------------------------------+
//    dst_idx 3 |   E (R)  F (R)  G (R)  H (R)   |
//              +--------------------------------+
//
// Command content layout:
//
//                 pair_idx 0        pair_idx 1
//               coef_idx 0-18     coef_idx 0-18
//              +--------------+  +--------------+
// target_idx 0 |  A (LRLR..)  |  |  E (LRLR..)  |
//              +--------------+  +--------------+
// target_idx 1 |  B (LRLR..)  |  |  F (LRLR..)  |
//              +--------------+  +--------------+
// target_idx 2 |  C (LRLR..)  |  |  G (LRLR..)  |
//              +--------------+  +--------------+
// target_idx 3 |  D (LRLR..)  |  |  H (LRLR..)  |
//              +--------------+  +--------------+
//
impl EnsembleParametersConverter<EnsembleMixerParameters> for EnsembleMixerProtocol {
    fn build_cmds(params: &EnsembleMixerParameters) -> Vec<EnsembleCmd> {
        let mut cmds = Vec::new();

        (0..2).for_each(|pair_idx| {
            let mut src0_gains = [0; MIXER_COEFFICIENT_COUNT];
            let mut src1_gains = [0; MIXER_COEFFICIENT_COUNT];
            let mut src2_gains = [0; MIXER_COEFFICIENT_COUNT];
            let mut src3_gains = [0; MIXER_COEFFICIENT_COUNT];

            params
                .src_gains
                .iter()
                .skip(pair_idx * 2)
                .take(2)
                .enumerate()
                .for_each(|(idx, gains)| {
                    let dst_idx = pair_idx * 2 + idx;
                    gains.iter().enumerate().for_each(|(src_idx, &gain)| {
                        let (_, target_idx, coef_idx) =
                            Self::array_indices_to_cmd_indices(dst_idx, src_idx);
                        let src_gains = match target_idx {
                            0 => &mut src0_gains,
                            1 => &mut src1_gains,
                            2 => &mut src2_gains,
                            3 => &mut src3_gains,
                            _ => unreachable!(),
                        };
                        src_gains[coef_idx] = gain;
                    });
                });

            cmds.push(EnsembleCmd::MixerSrc0(pair_idx, src0_gains));
            cmds.push(EnsembleCmd::MixerSrc1(pair_idx, src1_gains));
            cmds.push(EnsembleCmd::MixerSrc2(pair_idx, src2_gains));
            cmds.push(EnsembleCmd::MixerSrc3(pair_idx, src3_gains));
        });

        cmds
    }

    fn parse_cmds(params: &mut EnsembleMixerParameters, cmds: &[EnsembleCmd]) {
        cmds.iter().for_each(|cmd| match cmd {
            EnsembleCmd::MixerSrc0(pair_idx, gains) => {
                Self::partial_update_by_cmd_content(params, *pair_idx, 0, gains);
            }
            EnsembleCmd::MixerSrc1(pair_idx, gains) => {
                Self::partial_update_by_cmd_content(params, *pair_idx, 1, gains);
            }
            EnsembleCmd::MixerSrc2(pair_idx, gains) => {
                Self::partial_update_by_cmd_content(params, *pair_idx, 2, gains);
            }
            EnsembleCmd::MixerSrc3(pair_idx, gains) => {
                Self::partial_update_by_cmd_content(params, *pair_idx, 3, gains);
            }
            _ => (),
        });
    }
}
impl From<&EnsembleMixerParameters> for Vec<EnsembleCmd> {
    fn from(params: &EnsembleMixerParameters) -> Self {
        EnsembleMixerProtocol::build_cmds(params)
    }
}

impl EnsembleParameterProtocol<EnsembleMixerParameters> for EnsembleMixerProtocol {}

/// Parameters of stream mode.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct EnsembleStreamParameters {
    /// The mode of isochronous stream in IEEE 1394 bus.
    pub mode: StreamMode,
}

/// Protocol implementation for stream mode parameters.
#[derive(Default, Debug)]
pub struct EnsembleStreamProtocol;

impl EnsembleParametersConverter<EnsembleStreamParameters> for EnsembleStreamProtocol {
    fn build_cmds(params: &EnsembleStreamParameters) -> Vec<EnsembleCmd> {
        vec![EnsembleCmd::Hw(HwCmd::StreamMode(params.mode))]
    }

    fn parse_cmds(params: &mut EnsembleStreamParameters, cmds: &[EnsembleCmd]) {
        cmds.iter().for_each(|cmd| match cmd {
            &EnsembleCmd::Hw(HwCmd::StreamMode(mode)) => {
                params.mode = mode;
            }
            _ => (),
        });
    }
}

impl From<&EnsembleStreamParameters> for Vec<EnsembleCmd> {
    fn from(params: &EnsembleStreamParameters) -> Self {
        EnsembleStreamProtocol::build_cmds(params)
    }
}

impl EnsembleParameterProtocol<EnsembleStreamParameters> for EnsembleStreamProtocol {
    fn whole_update(
        avc: &BebobAvc,
        params: &mut EnsembleStreamParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let plug_addr =
            BcoPlugAddr::new_for_unit(BcoPlugDirection::Output, BcoPlugAddrUnitType::Isoc, 0);
        let mut op = ExtendedStreamFormatSingle::new(&plug_addr);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let info = op
            .stream_format
            .as_bco_compound_am824_stream()
            .ok_or_else(|| {
                let label = "Bco Compound AM824 stream is not available for the unit";
                Error::new(FileError::Nxio, &label)
            })?;
        let count = info
            .entries
            .iter()
            .filter(|entry| entry.format == BcoCompoundAm824StreamFormat::MultiBitLinearAudioRaw)
            .fold(0, |count, entry| count + entry.count as usize);
        params.mode = match count {
            18 => StreamMode::Format18x18,
            10 => StreamMode::Format10x10,
            _ => StreamMode::Format8x8,
        };

        Ok(())
    }
}

/// The converter between parameters and specific commands.
pub trait EnsembleParametersConverter<T: Copy> {
    /// Parse the given vector of commands for parameters.
    fn parse_cmds(params: &mut T, cmds: &[EnsembleCmd]);
    /// Build vector of commands by the given parameters.
    fn build_cmds(params: &T) -> Vec<EnsembleCmd>;
}

/// The trait for parameter protocol.
pub trait EnsembleParameterProtocol<T: Copy>: EnsembleParametersConverter<T> {
    /// Update the hardware for all of the parameters
    fn whole_update(avc: &BebobAvc, params: &mut T, timeout_ms: u32) -> Result<(), Error> {
        Self::build_cmds(params).into_iter().try_for_each(|cmd| {
            let mut op = EnsembleOperation::new(cmd);
            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
        })
    }

    /// Update the hardware for the part of parameters.
    fn partial_update(avc: &BebobAvc, new: &T, old: &mut T, timeout_ms: u32) -> Result<(), Error> {
        Self::build_cmds(new)
            .into_iter()
            .zip(Self::build_cmds(old))
            .filter(|(n, o)| !n.eq(o))
            .try_for_each(|(n, _)| {
                let mut op = EnsembleOperation::new(n);
                avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
            })
            .map(|_| *old = *new)
    }
}

/// The target of input for knob.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KnobInputTarget {
    /// 1st microphone.
    Mic0,
    /// 2nd microphone.
    Mic1,
    /// 3rd microphone.
    Mic2,
    /// 4th microphone.
    Mic3,
}

impl Default for KnobInputTarget {
    fn default() -> Self {
        Self::Mic1
    }
}

/// The target of output for knob.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KnobOutputTarget {
    /// 1st pair of analog outputs.
    AnalogOutputPair0,
    /// 1st pair of headphone outputs.
    HeadphonePair0,
    /// 2nd pair of headphone outputs.
    HeadphonePair1,
}

impl Default for KnobOutputTarget {
    fn default() -> Self {
        Self::AnalogOutputPair0
    }
}

/// Information of hardware metering.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EnsembleMeter {
    /// The target of hardware input knob.
    pub knob_input_target: KnobInputTarget,
    /// The target of hardware output knob.
    pub knob_output_target: KnobOutputTarget,
    /// The value of hardware input knob.
    pub knob_input_vals: [u8; 4],
    /// The value of hardware output knob.
    pub knob_output_vals: [u8; 3],
    /// The detected signal level of inputs.
    pub phys_inputs: [u8; 18],
    /// The detected signal level of outputs.
    pub phys_outputs: [u8; 16],
}

impl Default for EnsembleMeter {
    fn default() -> Self {
        Self {
            knob_input_target: Default::default(),
            knob_output_target: Default::default(),
            knob_input_vals: Default::default(),
            knob_output_vals: Default::default(),
            phys_inputs: [0; 18],
            phys_outputs: [0; 16],
        }
    }
}

/// The protocol implementation for meter information.
#[derive(Default, Debug)]
pub struct EnsembleMeterProtocol;

const KNOB_IN_TARGET_MASK: u8 = 0x03;
const KNOB_IN_TARGET_SHIFT: usize = 3;

const KNOB_OUT_TARGET_MASK: u8 = 0x07;
const KNOB_OUT_TARGET_SHIFT: usize = 0;

// 33-34: mixer-out-3/4
// 35: unknown
// 36-52: stream-in-0..16, missing 17
const SELECT_POS: usize = 4;
const IN_GAIN_POS: [usize; 4] = [0, 1, 2, 3];
const OUT_VOL_POS: [usize; 3] = [7, 6, 5];
const IN_METER_POS: [usize; 18] = [
    12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
];
const OUT_METER_POS: [usize; 16] = [
    35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
];

/// Protocol implementation for hardware metering.
impl EnsembleMeterProtocol {
    /// The minimum value of hardware output knob.
    pub const OUT_KNOB_VAL_MIN: u8 = EnsembleOutputProtocol::VOL_MIN;
    /// The maximum value of hardware output knob.
    pub const OUT_KNOB_VAL_MAX: u8 = EnsembleOutputProtocol::VOL_MAX;
    /// The step value of hardware output knob.
    pub const OUT_KNOB_VAL_STEP: u8 = EnsembleOutputProtocol::VOL_STEP;

    /// The minimum value of hardware input knob.
    pub const IN_KNOB_VAL_MIN: u8 = EnsembleInputProtocol::GAIN_MIN;
    /// The maximum value of hardware input knob.
    pub const IN_KNOB_VAL_MAX: u8 = EnsembleInputProtocol::GAIN_MAX;
    /// The step value of hardware input knob.
    pub const IN_KNOB_VAL_STEP: u8 = EnsembleInputProtocol::GAIN_STEP;

    /// The minimum value of detected signal level.
    pub const LEVEL_MIN: u8 = u8::MIN;
    /// The maximum value of detected signal level.
    pub const LEVEL_MAX: u8 = u8::MAX;
    /// The step value of detected signal level.
    pub const LEVEL_STEP: u8 = 1;

    /// Update the given parameters by the state of hardware.
    pub fn whole_update(
        avc: &mut BebobAvc,
        meter: &mut EnsembleMeter,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let cmd = EnsembleCmd::HwStatusLong([0; METER_LONG_FRAME_SIZE]);
        let mut op = EnsembleOperation::new(cmd);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            if let EnsembleCmd::HwStatusLong(frame) = &op.cmd {
                let val = (frame[SELECT_POS] >> KNOB_IN_TARGET_SHIFT) & KNOB_IN_TARGET_MASK;
                meter.knob_input_target = match val & 0x03 {
                    3 => KnobInputTarget::Mic3,
                    2 => KnobInputTarget::Mic2,
                    1 => KnobInputTarget::Mic1,
                    _ => KnobInputTarget::Mic0,
                };

                let val = (frame[SELECT_POS] >> KNOB_OUT_TARGET_SHIFT) & KNOB_OUT_TARGET_MASK;
                meter.knob_output_target = match val {
                    4 => KnobOutputTarget::HeadphonePair1,
                    2 => KnobOutputTarget::HeadphonePair0,
                    _ => KnobOutputTarget::AnalogOutputPair0,
                };

                IN_GAIN_POS
                    .iter()
                    .zip(&mut meter.knob_input_vals)
                    .for_each(|(&i, m)| *m = frame[i]);

                OUT_VOL_POS
                    .iter()
                    .zip(&mut meter.knob_output_vals)
                    .for_each(|(&i, m)| {
                        *m = Self::OUT_KNOB_VAL_MAX - (frame[i] & Self::OUT_KNOB_VAL_MAX);
                    });

                IN_METER_POS
                    .iter()
                    .zip(&mut meter.phys_inputs)
                    .for_each(|(&i, m)| *m = frame[i]);

                OUT_METER_POS
                    .iter()
                    .zip(&mut meter.phys_outputs)
                    .for_each(|(&i, m)| *m = frame[i]);
            }
        })
    }
}

/// The nominal level of analog input 0-7.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InputNominalLevel {
    /// +4 dBu.
    Professional,
    /// -10 dBV.
    Consumer,
    /// Widely adjustable with preamp. Available only for channel 0-3.
    Microphone,
}

impl Default for InputNominalLevel {
    fn default() -> Self {
        Self::Professional
    }
}

/// The nominal level of analog ouput 0-7.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OutputNominalLevel {
    /// +4 dBu.
    Professional,
    /// -10 dBV.
    Consumer,
}

impl Default for OutputNominalLevel {
    fn default() -> Self {
        Self::Professional
    }
}

/// The mode of signal in optical interface.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OptIfaceMode {
    Spdif,
    Adat,
}

impl Default for OptIfaceMode {
    fn default() -> Self {
        Self::Spdif
    }
}

fn opt_iface_mode_to_val(mode: &OptIfaceMode) -> u8 {
    if mode.eq(&OptIfaceMode::Adat) {
        1
    } else {
        0
    }
}

fn opt_iface_mode_from_val(val: u8) -> OptIfaceMode {
    if val > 0 {
        OptIfaceMode::Adat
    } else {
        OptIfaceMode::Spdif
    }
}

/// The target to convert sample format from 24 bit to 16 bit.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FormatConvertTarget {
    /// Disabled.
    Disabled,
    /// 1st pair of analog inputs.
    AnalogInputPair0,
    /// 2nd pair of analog inputs.
    AnalogInputPair1,
    /// 3rd pair of analog inputs.
    AnalogInputPair2,
    /// 4th pair of analog inputs.
    AnalogInputPair3,
    /// The pair of S/PDIF input in optical interface.
    SpdifOpticalInputPair0,
    /// The pair of S/PDIF input in coaxial interface.
    SpdifCoaxialInputPair0,
    /// The pair of S/PDIF output in optical interface.
    SpdifCoaxialOutputPair0,
    /// The pair of S/PDIF output in coaxial interface.
    SpdifOpticalOutputPair0,
}

impl Default for FormatConvertTarget {
    fn default() -> Self {
        Self::Disabled
    }
}

/// The target to convert sample rate.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RateConvertTarget {
    /// Disabled.
    Disabled,
    /// The pair of S/PDIF output in optical interface.
    SpdifOpticalOutputPair0,
    /// The pair of S/PDIF output in coaxial interface.
    SpdifCoaxialOutputPair0,
    /// The pair of S/PDIF input in optical interface.
    SpdifOpticalInputPair0,
    /// The pair of S/PDIF input in coaxial interface.
    SpdifCoaxialInputPair0,
}

impl Default for RateConvertTarget {
    fn default() -> Self {
        Self::Disabled
    }
}

/// The rate to convert sample rate.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RateConvertRate {
    /// To 44.1 kHz.
    R44100,
    /// To 48.0 kHz.
    R48000,
    /// To 88.2 kHz.
    R88200,
    /// To 96.0 kHz.
    R96000,
    /// To 176.0 kHz.
    R176400,
    /// To 192.0 kHz.
    R192000,
}

impl Default for RateConvertRate {
    fn default() -> Self {
        Self::R44100
    }
}

const METER_SHORT_FRAME_SIZE: usize = 17;
const METER_LONG_FRAME_SIZE: usize = 56;
const MIXER_COEFFICIENT_COUNT: usize = 18;

/// Command specific to Apogee Ensemble.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EnsembleCmd {
    /// Limitter for 8 analog inputs.
    InputLimit(usize, bool), // index, state
    /// Phantom powering for 4 microphone inputs.
    MicPower(usize, bool), // index, state
    /// The nominal level of 8 analog inputs.
    InputNominalLevel(usize, InputNominalLevel),
    /// The nominal level of 8 analog outputs.
    OutputNominalLevel(usize, OutputNominalLevel),
    /// The routing between source and destination.
    IoRouting(usize, usize), // destination, source
    /// Hardware type of command.
    Hw(HwCmd),
    /// The source of 4 headphone outputs.
    HpSrc(usize, usize), // destination, source
    /// The source of 1st nine pairs of coefficients for two pairs of mixers.
    MixerSrc0(usize, [i16; MIXER_COEFFICIENT_COUNT]),
    /// The source of 2nd nine pairs of coefficients for two pairs of mixers.
    MixerSrc1(usize, [i16; MIXER_COEFFICIENT_COUNT]),
    /// The source of 3rd nine pairs of coefficients for two pairs of mixers.
    MixerSrc2(usize, [i16; MIXER_COEFFICIENT_COUNT]),
    /// The source of 4th nine pairs of coefficients for two pairs of mixers.
    MixerSrc3(usize, [i16; MIXER_COEFFICIENT_COUNT]),
    /// The gain of 4 microphone inputs, between 10 and 75.
    MicGain(usize, u8), // 1/2/3/4, dB(10-75), also available as knob control
    /// The mode of output in optical interface.
    OutputOptIface(OptIfaceMode),
    /// The mode of input in optical interface.
    InputOptIface(OptIfaceMode),
    /// The target of converter for sample format.
    FormatConvert(FormatConvertTarget),
    /// The parameters of sampling rate converter.
    RateConvert(RateConvertTarget, RateConvertRate),
    /// The polarity for 4 microphone inputs.
    MicPolarity(usize, bool), // index, state
    /// The volume of output.
    OutVol(usize, u8), // main/hp0/hp1, dB(127-0), also available as knob control
    /// The short expression of hardware status.
    HwStatusShort([u8; METER_SHORT_FRAME_SIZE]),
    /// The long expression of hardware status.
    HwStatusLong([u8; METER_LONG_FRAME_SIZE]),
    Reserved(Vec<u8>),
}

impl Default for EnsembleCmd {
    fn default() -> Self {
        Self::Reserved(Vec::new())
    }
}

impl EnsembleCmd {
    const INPUT_LIMIT: u8 = 0xe4;
    const MIC_POWER: u8 = 0xe5;
    const IO_NOMINAL_LEVEL: u8 = 0xe8;
    const IO_ROUTING: u8 = 0xef;
    const HW: u8 = 0xeb;
    const HP_SRC: u8 = 0xab;
    const MIXER_SRC0: u8 = 0xb0;
    const MIXER_SRC1: u8 = 0xb1;
    const MIXER_SRC2: u8 = 0xb2;
    const MIXER_SRC3: u8 = 0xb3;
    const MIC_GAIN: u8 = 0xe6;
    const OPT_IFACE_MODE: u8 = 0xf1;
    const FORMAT_CONVERT: u8 = 0xf2;
    const RATE_CONVERT: u8 = 0xf3;
    const MIC_POLARITY: u8 = 0xf5;
    const OUT_VOL: u8 = 0xf6;
    const HW_STATUS: u8 = 0xff;
}

impl From<&EnsembleCmd> for Vec<u8> {
    fn from(cmd: &EnsembleCmd) -> Self {
        match cmd {
            EnsembleCmd::InputLimit(ch, state) => {
                vec![EnsembleCmd::INPUT_LIMIT, *ch as u8, *state as u8]
            }
            EnsembleCmd::MicPower(ch, state) => {
                vec![EnsembleCmd::MIC_POWER, *ch as u8, *state as u8]
            }
            EnsembleCmd::InputNominalLevel(ch, state) => {
                let val = match state {
                    InputNominalLevel::Professional => 0,
                    InputNominalLevel::Consumer => 1,
                    InputNominalLevel::Microphone => 2,
                };
                vec![EnsembleCmd::IO_NOMINAL_LEVEL, *ch as u8, 0x01, val]
            }
            EnsembleCmd::OutputNominalLevel(ch, state) => {
                let val = match state {
                    OutputNominalLevel::Professional => 0,
                    OutputNominalLevel::Consumer => 1,
                };
                vec![EnsembleCmd::IO_NOMINAL_LEVEL, *ch as u8, 0x00, val]
            }
            EnsembleCmd::IoRouting(dst, src) => {
                vec![EnsembleCmd::IO_ROUTING, *dst as u8, *src as u8]
            }
            EnsembleCmd::Hw(op) => {
                let mut params = Into::<Vec<u8>>::into(op);
                params.insert(0, EnsembleCmd::HW);
                params
            }
            EnsembleCmd::HpSrc(dst, src) => {
                vec![
                    EnsembleCmd::HP_SRC,
                    (1 + *dst as u8) % 2,
                    (1 + 2 * *src as u8),
                ]
            }
            EnsembleCmd::MixerSrc0(pair, coefs) => {
                let mut data = Vec::with_capacity(2 + 2 * MIXER_COEFFICIENT_COUNT);
                data.push(EnsembleCmd::MIXER_SRC0);
                data.push(*pair as u8);
                coefs
                    .iter()
                    .for_each(|coef| data.extend_from_slice(&coef.to_be_bytes()));
                data
            }
            EnsembleCmd::MixerSrc1(pair, coefs) => {
                let mut data = Vec::with_capacity(2 + 2 * MIXER_COEFFICIENT_COUNT);
                data.push(EnsembleCmd::MIXER_SRC1);
                data.push(*pair as u8);
                coefs
                    .iter()
                    .for_each(|coef| data.extend_from_slice(&coef.to_be_bytes()));
                data
            }
            EnsembleCmd::MixerSrc2(pair, coefs) => {
                let mut data = Vec::with_capacity(2 + 2 * MIXER_COEFFICIENT_COUNT);
                data.push(EnsembleCmd::MIXER_SRC2);
                data.push(*pair as u8);
                coefs
                    .iter()
                    .for_each(|coef| data.extend_from_slice(&coef.to_be_bytes()));
                data
            }
            EnsembleCmd::MixerSrc3(pair, coefs) => {
                let mut data = Vec::with_capacity(2 + 2 * MIXER_COEFFICIENT_COUNT);
                data.push(EnsembleCmd::MIXER_SRC3);
                data.push(*pair as u8);
                coefs
                    .iter()
                    .for_each(|coef| data.extend_from_slice(&coef.to_be_bytes()));
                data
            }
            EnsembleCmd::MicGain(target, val) => {
                vec![EnsembleCmd::MIC_GAIN, *target as u8, *val]
            }
            EnsembleCmd::OutputOptIface(mode) => {
                vec![EnsembleCmd::OPT_IFACE_MODE, 0, opt_iface_mode_to_val(mode)]
            }
            EnsembleCmd::InputOptIface(mode) => {
                vec![EnsembleCmd::OPT_IFACE_MODE, 1, opt_iface_mode_to_val(mode)]
            }
            EnsembleCmd::FormatConvert(state) => {
                let val = match state {
                    FormatConvertTarget::Disabled => 0,
                    FormatConvertTarget::AnalogInputPair0 => 1,
                    FormatConvertTarget::AnalogInputPair1 => 2,
                    FormatConvertTarget::AnalogInputPair2 => 3,
                    FormatConvertTarget::AnalogInputPair3 => 4,
                    FormatConvertTarget::SpdifOpticalInputPair0 => 5,
                    FormatConvertTarget::SpdifCoaxialInputPair0 => 6,
                    FormatConvertTarget::SpdifCoaxialOutputPair0 => 7,
                    FormatConvertTarget::SpdifOpticalOutputPair0 => 8,
                };
                vec![EnsembleCmd::FORMAT_CONVERT, val]
            }
            EnsembleCmd::RateConvert(target, rate) => {
                let triplet = match target {
                    RateConvertTarget::Disabled => [0, 0, 0],
                    RateConvertTarget::SpdifOpticalOutputPair0 => [1, 0, 0],
                    RateConvertTarget::SpdifCoaxialOutputPair0 => [1, 1, 0],
                    RateConvertTarget::SpdifOpticalInputPair0 => [1, 0, 1],
                    RateConvertTarget::SpdifCoaxialInputPair0 => [1, 1, 1],
                };
                let val = match rate {
                    RateConvertRate::R44100 => 0,
                    RateConvertRate::R48000 => 1,
                    RateConvertRate::R88200 => 2,
                    RateConvertRate::R96000 => 3,
                    RateConvertRate::R176400 => 4,
                    RateConvertRate::R192000 => 5,
                };
                vec![
                    EnsembleCmd::RATE_CONVERT,
                    triplet[0],
                    triplet[1],
                    triplet[2],
                    val,
                ]
            }
            EnsembleCmd::MicPolarity(ch, state) => {
                vec![EnsembleCmd::MIC_POLARITY, *ch as u8, *state as u8]
            }
            EnsembleCmd::OutVol(target, vol) => {
                vec![EnsembleCmd::OUT_VOL, *target as u8, *vol]
            }
            EnsembleCmd::HwStatusShort(_) => vec![EnsembleCmd::HW_STATUS, 0],
            EnsembleCmd::HwStatusLong(_) => vec![EnsembleCmd::HW_STATUS, 1],
            EnsembleCmd::Reserved(r) => r.to_vec(),
        }
    }
}

impl From<&[u8]> for EnsembleCmd {
    fn from(raw: &[u8]) -> Self {
        match raw[0] {
            Self::INPUT_LIMIT => Self::InputLimit(raw[1] as usize, raw[2] > 0),
            Self::MIC_POWER => Self::MicPower(raw[1] as usize, raw[2] > 0),
            Self::IO_NOMINAL_LEVEL => {
                if raw[2] > 0 {
                    let state = match raw[3] {
                        2 => InputNominalLevel::Microphone,
                        1 => InputNominalLevel::Consumer,
                        _ => InputNominalLevel::Professional,
                    };
                    Self::InputNominalLevel(raw[1] as usize, state)
                } else {
                    let state = match raw[3] {
                        1 => OutputNominalLevel::Consumer,
                        _ => OutputNominalLevel::Professional,
                    };
                    Self::OutputNominalLevel(raw[1] as usize, state)
                }
            }
            Self::IO_ROUTING => Self::IoRouting(raw[1] as usize, raw[2] as usize),
            Self::HW => Self::Hw(HwCmd::from(&raw[1..])),
            Self::HP_SRC => Self::HpSrc((1 + raw[1] as usize) % 2, (raw[2] as usize) / 2),
            Self::MIXER_SRC0 => {
                let mut doublet = [0; 2];
                let mut coefs = [0; MIXER_COEFFICIENT_COUNT];
                coefs.iter_mut().enumerate().for_each(|(i, coef)| {
                    let pos = 2 + i * 2;
                    doublet.copy_from_slice(&raw[pos..(pos + 2)]);
                    *coef = i16::from_be_bytes(doublet);
                });
                Self::MixerSrc0(raw[1] as usize, coefs)
            }
            Self::MIXER_SRC1 => {
                let mut doublet = [0; 2];
                let mut coefs = [0; MIXER_COEFFICIENT_COUNT];
                coefs.iter_mut().enumerate().for_each(|(i, coef)| {
                    let pos = 2 + i * 2;
                    doublet.copy_from_slice(&raw[pos..(pos + 2)]);
                    *coef = i16::from_be_bytes(doublet);
                });
                Self::MixerSrc1(raw[1] as usize, coefs)
            }
            Self::MIXER_SRC2 => {
                let mut doublet = [0; 2];
                let mut coefs = [0; MIXER_COEFFICIENT_COUNT];
                coefs.iter_mut().enumerate().for_each(|(i, coef)| {
                    let pos = 2 + i * 2;
                    doublet.copy_from_slice(&raw[pos..(pos + 2)]);
                    *coef = i16::from_be_bytes(doublet);
                });
                Self::MixerSrc2(raw[1] as usize, coefs)
            }
            Self::MIXER_SRC3 => {
                let mut doublet = [0; 2];
                let mut coefs = [0; MIXER_COEFFICIENT_COUNT];
                coefs.iter_mut().enumerate().for_each(|(i, coef)| {
                    let pos = 2 + i * 2;
                    doublet.copy_from_slice(&raw[pos..(pos + 2)]);
                    *coef = i16::from_be_bytes(doublet);
                });
                Self::MixerSrc3(raw[1] as usize, coefs)
            }
            Self::MIC_GAIN => Self::MicGain(raw[1] as usize, raw[2]),
            Self::OPT_IFACE_MODE => {
                let mode = opt_iface_mode_from_val(raw[2]);
                if raw[1] > 0 {
                    Self::InputOptIface(mode)
                } else {
                    Self::OutputOptIface(mode)
                }
            }
            Self::FORMAT_CONVERT => {
                let target = match raw[1] {
                    8 => FormatConvertTarget::SpdifOpticalOutputPair0,
                    7 => FormatConvertTarget::SpdifCoaxialOutputPair0,
                    6 => FormatConvertTarget::SpdifCoaxialInputPair0,
                    5 => FormatConvertTarget::SpdifOpticalInputPair0,
                    4 => FormatConvertTarget::AnalogInputPair3,
                    3 => FormatConvertTarget::AnalogInputPair2,
                    2 => FormatConvertTarget::AnalogInputPair1,
                    1 => FormatConvertTarget::AnalogInputPair0,
                    _ => FormatConvertTarget::Disabled,
                };
                Self::FormatConvert(target)
            }
            Self::RATE_CONVERT => {
                let target = if raw[1] == 0 {
                    RateConvertTarget::Disabled
                } else {
                    match (raw[2], raw[3]) {
                        (0, 0) => RateConvertTarget::SpdifOpticalOutputPair0,
                        (1, 0) => RateConvertTarget::SpdifCoaxialOutputPair0,
                        (0, 1) => RateConvertTarget::SpdifOpticalInputPair0,
                        (1, 1) => RateConvertTarget::SpdifCoaxialInputPair0,
                        _ => RateConvertTarget::Disabled,
                    }
                };
                let rate = match raw[4] {
                    5 => RateConvertRate::R192000,
                    4 => RateConvertRate::R176400,
                    3 => RateConvertRate::R96000,
                    2 => RateConvertRate::R88200,
                    1 => RateConvertRate::R48000,
                    _ => RateConvertRate::R44100,
                };
                Self::RateConvert(target, rate)
            }
            Self::MIC_POLARITY => Self::MicPolarity(raw[1] as usize, raw[2] > 0),
            Self::OUT_VOL => Self::OutVol(raw[1] as usize, raw[2]),
            Self::HW_STATUS => {
                if raw[1] > 0 {
                    let mut params = [0; METER_LONG_FRAME_SIZE];
                    params.copy_from_slice(&raw[1..]);
                    Self::HwStatusLong(params)
                } else {
                    let mut params = [0; METER_SHORT_FRAME_SIZE];
                    params.copy_from_slice(&raw[1..]);
                    Self::HwStatusShort(params)
                }
            }
            _ => Self::Reserved(raw.to_vec()),
        }
    }
}

/// The mode of stream format.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StreamMode {
    /// For 18 channels capture and 18 channels playback.
    Format18x18,
    /// For 10 channels capture and 10 channels playback.
    Format10x10,
    /// For 8 channels capture and 8 channels playback.
    Format8x8,
}

impl Default for StreamMode {
    fn default() -> Self {
        StreamMode::Format8x8
    }
}

/// The target of display meter.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DisplayMeterTarget {
    /// For outputs.
    Output,
    /// For inputs.
    Input,
}

impl Default for DisplayMeterTarget {
    fn default() -> Self {
        Self::Output
    }
}

/// Command for functions in hardware category specific to Apogee Ensemble.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HwCmd {
    /// The mode of isochronous stream in IEEE 1394 bus. Any change generates bus reset in the bus.
    StreamMode(StreamMode),
    /// Whether to illuminate display.
    DisplayIlluminate(bool),
    /// Whether to enable/disable display.
    DisplayMode(bool),
    /// The target of metering display.
    DisplayTarget(DisplayMeterTarget),
    /// Whether to enable/disable overhold of peak detection.
    DisplayOverhold(bool),
    /// Reset display metering.
    MeterReset,
    /// The mode of CD (44.1 kHz/16 bit sample).
    CdMode(bool),
    Reserved(Vec<u8>),
}

impl HwCmd {
    const STREAM_MODE: u8 = 0x06;
    const DISPLAY_ILLUMINATE: u8 = 0x08;
    const DISPLAY_MODE: u8 = 0x09;
    const DISPLAY_TARGET: u8 = 0x0a;
    const DISPLAY_OVERHOLD: u8 = 0x0e;
    const METER_RESET: u8 = 0x0f;
    const CD_MODE: u8 = 0xf5;
}

impl From<&HwCmd> for Vec<u8> {
    fn from(op: &HwCmd) -> Self {
        match op {
            HwCmd::StreamMode(mode) => {
                let val = match mode {
                    StreamMode::Format18x18 => 0,
                    StreamMode::Format10x10 => 1,
                    StreamMode::Format8x8 => 2,
                };
                vec![HwCmd::STREAM_MODE, val]
            }
            HwCmd::DisplayIlluminate(state) => vec![HwCmd::DISPLAY_ILLUMINATE, *state as u8],
            HwCmd::DisplayMode(state) => vec![HwCmd::DISPLAY_MODE, *state as u8],
            HwCmd::DisplayTarget(target) => {
                let val = match target {
                    DisplayMeterTarget::Output => 0,
                    DisplayMeterTarget::Input => 1,
                };
                vec![HwCmd::DISPLAY_TARGET, val]
            }
            HwCmd::DisplayOverhold(state) => vec![HwCmd::DISPLAY_OVERHOLD, *state as u8],
            HwCmd::MeterReset => vec![HwCmd::METER_RESET],
            HwCmd::CdMode(state) => vec![HwCmd::CD_MODE, *state as u8],
            HwCmd::Reserved(val) => val.to_vec(),
        }
    }
}

impl From<&[u8]> for HwCmd {
    fn from(vals: &[u8]) -> HwCmd {
        match vals[0] {
            HwCmd::STREAM_MODE => {
                let mode = match vals[1] {
                    2 => StreamMode::Format8x8,
                    1 => StreamMode::Format10x10,
                    _ => StreamMode::Format18x18,
                };
                HwCmd::StreamMode(mode)
            }
            HwCmd::DISPLAY_ILLUMINATE => HwCmd::DisplayIlluminate(vals[1] > 0),
            HwCmd::DISPLAY_MODE => HwCmd::DisplayMode(vals[1] > 0),
            HwCmd::DISPLAY_TARGET => {
                let target = if vals[1] > 0 {
                    DisplayMeterTarget::Input
                } else {
                    DisplayMeterTarget::Output
                };
                HwCmd::DisplayTarget(target)
            }
            HwCmd::DISPLAY_OVERHOLD => HwCmd::DisplayOverhold(vals[1] > 0),
            HwCmd::METER_RESET => HwCmd::MeterReset,
            HwCmd::CD_MODE => HwCmd::CdMode(vals[1] > 0),
            _ => HwCmd::Reserved(vals.to_vec()),
        }
    }
}

/// The protocol implementation of AV/C vendor-dependent command specific to Apogee Ensemble.
#[derive(Debug)]
pub struct EnsembleOperation {
    pub cmd: EnsembleCmd,
    op: VendorDependent,
}

impl Default for EnsembleOperation {
    fn default() -> Self {
        Self {
            cmd: Default::default(),
            op: VendorDependent {
                company_id: APOGEE_OUI,
                data: Default::default(),
            },
        }
    }
}

impl EnsembleOperation {
    pub fn new(cmd: EnsembleCmd) -> Self {
        Self {
            cmd,
            ..Default::default()
        }
    }
}

impl AvcOp for EnsembleOperation {
    const OPCODE: u8 = VendorDependent::OPCODE;
}

impl AvcControl for EnsembleOperation {
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        self.op.data = Into::<Vec<u8>>::into(&self.cmd);

        // At least, 6 bytes should be required to align to 3 quadlets. Unless, the target unit is freezed.
        while self.op.data.len() < 6 {
            self.op.data.push(0xff);
        }

        AvcControl::build_operands(&mut self.op, addr)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AvcControl::parse_operands(&mut self.op, addr, operands).map(|_| {
            // NOTE: parameters are retrieved by HwStatus command only.
            match &mut self.cmd {
                EnsembleCmd::HwStatusShort(buf) => buf.copy_from_slice(&self.op.data[2..]),
                EnsembleCmd::HwStatusLong(buf) => buf.copy_from_slice(&self.op.data[2..]),
                _ => (),
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn converter_params_and_cmds() {
        let params = EnsembleConverterParameters {
            format_target: FormatConvertTarget::SpdifCoaxialInputPair0,
            rate_target: RateConvertTarget::SpdifOpticalOutputPair0,
            converted_rate: RateConvertRate::R88200,
            cd_mode: true,
        };
        let cmds = EnsembleConverterProtocol::build_cmds(&params);
        let mut p = EnsembleConverterParameters::default();
        EnsembleConverterProtocol::parse_cmds(&mut p, &cmds);

        assert_eq!(params, p);
    }

    #[test]
    fn display_params_and_cmds() {
        let params = EnsembleDisplayParameters {
            enabled: false,
            illuminate: true,
            target: DisplayMeterTarget::Output,
            overhold: true,
        };
        let cmds = EnsembleDisplayProtocol::build_cmds(&params);
        let mut p = EnsembleDisplayParameters::default();
        EnsembleDisplayProtocol::parse_cmds(&mut p, &cmds);

        assert_eq!(params, p);
    }

    #[test]
    fn input_params_and_cmds() {
        let params = EnsembleInputParameters {
            limits: [false, true, false, true, true, false, true, false],
            levels: [
                InputNominalLevel::Professional,
                InputNominalLevel::Consumer,
                InputNominalLevel::Microphone,
                InputNominalLevel::Professional,
                InputNominalLevel::Consumer,
                InputNominalLevel::Microphone,
                InputNominalLevel::Professional,
                InputNominalLevel::Consumer,
            ],
            gains: [10, 20, 30, 40],
            phantoms: [true, false, false, true],
            polarities: [false, true, true, false],
            opt_iface_mode: OptIfaceMode::Adat,
        };
        let cmds = EnsembleInputProtocol::build_cmds(&params);
        let mut p = EnsembleInputParameters::default();
        EnsembleInputProtocol::parse_cmds(&mut p, &cmds);

        assert_eq!(params, p);
    }

    #[test]
    fn output_params_and_cmds() {
        let params = EnsembleOutputParameters {
            vol: 0x72,
            headphone_vols: [0x4f, 0x5a],
            levels: [
                OutputNominalLevel::Professional,
                OutputNominalLevel::Consumer,
                OutputNominalLevel::Professional,
                OutputNominalLevel::Consumer,
                OutputNominalLevel::Professional,
                OutputNominalLevel::Consumer,
                OutputNominalLevel::Professional,
                OutputNominalLevel::Consumer,
            ],
            opt_iface_mode: OptIfaceMode::Adat,
        };
        let cmds = EnsembleOutputProtocol::build_cmds(&params);
        let mut p = EnsembleOutputParameters::default();
        EnsembleOutputProtocol::parse_cmds(&mut p, &cmds);

        assert_eq!(params, p);
    }

    #[test]
    fn source_params_and_cmds() {
        let params = EnsembleSourceParameters {
            output_sources: [9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 17, 16, 15, 14, 13, 12, 11, 10],
            capture_sources: [9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 17, 16, 15, 14, 13, 12, 11, 10],
            headphone_sources: [7, 2],
        };
        let cmds = EnsembleSourceProtocol::build_cmds(&params);
        let mut p = EnsembleSourceParameters::default();
        EnsembleSourceProtocol::parse_cmds(&mut p, &cmds);

        assert_eq!(params, p);
    }

    #[test]
    fn mixer_params_and_cmds() {
        let params = EnsembleMixerParameters {
            src_gains: [
                [
                    39, -84, 33, 93, -55, 26, -14, -16, 36, 25, -76, 27, -90, -22, -92, 15, -98,
                    90, 55, 58, 0, -33, -3, -86, 62, -57, 45, 2, 51, -39, 53, 41, -58, -18, -88,
                    -38,
                ],
                [
                    -12, -78, -72, -43, -50, -73, 19, -9, 21, 28, -15, 36, 55, -58, 22, 56, 39, 43,
                    10, -1, 60, -6, -29, 15, -98, 46, 90, -67, 32, 83, -55, 66, 54, 48, 62, -49,
                ],
                [
                    10, -100, 90, 18, -3, -61, -2, -37, -29, -60, 99, -16, 54, 28, -17, 17, -69,
                    33, -81, -56, -39, 3, 22, 85, -35, -52, -21, 40, 21, -67, 45, 80, 0, 42, -88,
                    63,
                ],
                [
                    4, -72, 18, -56, 10, 68, -82, 82, 94, -8, -9, 6, -79, 64, 30, -50, -88, -23,
                    -34, 23, -33, 77, -28, -7, 21, -32, -42, -58, -1, 71, 84, 37, -80, -19, 88, 0,
                ],
            ],
        };
        let cmds = EnsembleMixerProtocol::build_cmds(&params);
        let mut p = EnsembleMixerParameters::default();
        EnsembleMixerProtocol::parse_cmds(&mut p, &cmds);

        assert_eq!(params, p);
    }

    #[test]
    fn stream_params() {
        let params = EnsembleStreamParameters {
            mode: StreamMode::Format10x10,
        };
        let cmds = EnsembleStreamProtocol::build_cmds(&params);
        let mut p = EnsembleStreamParameters::default();
        EnsembleStreamProtocol::parse_cmds(&mut p, &cmds);

        assert_eq!(params, p);
    }

    #[test]
    fn vendorcmd_from() {
        let cmd = EnsembleCmd::InputLimit(1, true);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::MicPower(1, true);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::InputNominalLevel(1, InputNominalLevel::Microphone);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::OutputNominalLevel(1, OutputNominalLevel::Consumer);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::IoRouting(1, 11);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::Hw(HwCmd::StreamMode(StreamMode::Format10x10));
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::HpSrc(1, 31);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::MixerSrc0(3, [3; MIXER_COEFFICIENT_COUNT]);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::MixerSrc1(2, [11; MIXER_COEFFICIENT_COUNT]);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::MixerSrc2(1, [17; MIXER_COEFFICIENT_COUNT]);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::MixerSrc3(0, [21; MIXER_COEFFICIENT_COUNT]);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::MicGain(195, 233);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::OutputOptIface(OptIfaceMode::Adat);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::InputOptIface(OptIfaceMode::Spdif);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::FormatConvert(FormatConvertTarget::AnalogInputPair0);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::RateConvert(
            RateConvertTarget::SpdifOpticalInputPair0,
            RateConvertRate::R88200,
        );
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::MicPolarity(0, true);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::OutVol(0, 113);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );
    }
}
