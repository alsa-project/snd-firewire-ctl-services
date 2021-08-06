// SPDX-License-Identifier: GPL-3.0-or-later
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

use crate::*;

use super::*;

use ta1394::ccm::{SignalAddr, SignalSubunitAddr, SignalUnitAddr};
use ta1394::MUSIC_SUBUNIT_0;

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

/// The structure of sample format converter.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct EnsembleConvertParameters {
    pub format_target: FormatConvertTarget,
    pub rate_target: RateConvertTarget,
    pub converted_rate: RateConvertRate,
    pub cd_mode: bool,
}

impl From<&EnsembleConvertParameters> for Vec<EnsembleCmd> {
    fn from(params: &EnsembleConvertParameters) -> Self {
        vec![
            EnsembleCmd::FormatConvert(params.format_target),
            EnsembleCmd::RateConvert(params.rate_target, params.converted_rate),
            EnsembleCmd::Hw(HwCmd::CdMode(params.cd_mode)),
        ]
    }
}

impl EnsembleParameterProtocol<EnsembleConvertParameters> for BebobAvc {}

/// The structure for parameters of display meters.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct EnsembleDisplayParameters {
    pub enabled: bool,
    pub illuminate: bool,
    pub target: DisplayMeterTarget,
    pub overhold: bool,
}

impl From<&EnsembleDisplayParameters> for Vec<EnsembleCmd> {
    fn from(params: &EnsembleDisplayParameters) -> Self {
        vec![
            EnsembleCmd::Hw(HwCmd::DisplayMode(params.enabled)),
            EnsembleCmd::Hw(HwCmd::DisplayIlluminate(params.illuminate)),
            EnsembleCmd::Hw(HwCmd::DisplayTarget(params.target)),
            EnsembleCmd::Hw(HwCmd::DisplayOverhold(params.overhold)),
        ]
    }
}

impl EnsembleParameterProtocol<EnsembleDisplayParameters> for BebobAvc {}

/// The parameters of analog/digital inputs. The gains, phantoms, and polarities parameters
/// are available when channel 0-3 levels are for mic.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct EnsembleInputParameters {
    pub limits: [bool; 8],
    pub levels: [InputNominalLevel; 8],
    pub gains: [u8; 4],
    pub phantoms: [bool; 4],
    pub polarities: [bool; 4],
    pub opt_iface_mode: OptIfaceMode,
}

impl EnsembleInputParameters {
    pub const GAIN_MIN: u8 = 0;
    pub const GAIN_MAX: u8 = 75;
    pub const GAIN_STEP: u8 = 1;
}

impl From<&EnsembleInputParameters> for Vec<EnsembleCmd> {
    fn from(params: &EnsembleInputParameters) -> Self {
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

impl EnsembleParameterProtocol<EnsembleInputParameters> for BebobAvc {}

/// The structure for parameters of analog/digital outputs.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct EnsembleOutputParameters {
    pub vol: u8,
    pub headphone_vols: [u8; 2],
    pub levels: [OutputNominalLevel; 8],
    pub opt_iface_mode: OptIfaceMode,
}

impl EnsembleOutputParameters {
    pub const VOL_MIN: u8 = 0;
    pub const VOL_MAX: u8 = 0x7f;
    pub const VOL_STEP: u8 = 1;
}

impl From<&EnsembleOutputParameters> for Vec<EnsembleCmd> {
    fn from(params: &EnsembleOutputParameters) -> Self {
        let mut cmds = Vec::new();

        [params.vol]
            .iter()
            .chain(params.headphone_vols.iter())
            .enumerate()
            .for_each(|(i, &vol)| {
                cmds.push(EnsembleCmd::OutVol(
                    i,
                    EnsembleOutputParameters::VOL_MAX - vol,
                ))
            });

        params
            .levels
            .iter()
            .enumerate()
            .for_each(|(i, &level)| cmds.push(EnsembleCmd::OutputNominalLevel(i, level)));

        cmds.push(EnsembleCmd::OutputOptIface(params.opt_iface_mode));

        cmds
    }
}

impl EnsembleParameterProtocol<EnsembleOutputParameters> for BebobAvc {}

/// The structure for parameters of input/output source.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

impl From<&EnsembleSourceParameters> for Vec<EnsembleCmd> {
    fn from(params: &EnsembleSourceParameters) -> Self {
        let mut cmds = Vec::new();

        params
            .output_sources
            .iter()
            .enumerate()
            .for_each(|(dst_id, &src_id)| {
                let dst_id = if dst_id < 8 { dst_id } else { dst_id + 18 };
                cmds.push(EnsembleCmd::IoRouting(dst_id, src_id));
            });

        params
            .capture_sources
            .iter()
            .enumerate()
            .for_each(|(dst_id, &src)| {
                let dst_id = dst_id + 8;
                let src_id = if src < 8 { src } else { src + 18 };
                cmds.push(EnsembleCmd::IoRouting(dst_id, src_id));
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
}

impl EnsembleParameterProtocol<EnsembleSourceParameters> for BebobAvc {}

/// The structure for mixer parameters.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

impl EnsembleMixerParameters {
    pub const GAIN_MIN: i16 = 0;
    pub const GAIN_MAX: i16 = 0xff;
    pub const GAIN_STEP: i16 = 0x01;
}

impl Default for EnsembleMixerParameters {
    fn default() -> Self {
        let mut src_gains = [[0; 36]; 4];

        src_gains.iter_mut().enumerate().for_each(|(i, gains)| {
            gains
                .iter_mut()
                .enumerate()
                .filter(|(j, _)| i % 2 == j % 2)
                .for_each(|(_, gain)| *gain = Self::GAIN_MAX);
        });
        Self { src_gains }
    }
}

impl From<&EnsembleMixerParameters> for Vec<EnsembleCmd> {
    fn from(params: &EnsembleMixerParameters) -> Self {
        let mut cmds = Vec::new();

        (0..2).for_each(|i| {
            let mut src0_gains = [0; MIXER_COEFFICIENT_COUNT];
            let mut src1_gains = [0; MIXER_COEFFICIENT_COUNT];
            let mut src2_gains = [0; MIXER_COEFFICIENT_COUNT];
            let mut src3_gains = [0; MIXER_COEFFICIENT_COUNT];

            params.src_gains[i * 2]
                .iter()
                .zip(params.src_gains[i * 2 + 1].iter())
                .enumerate()
                .for_each(|(j, (&l, &r))| {
                    let (gains, pos) = match j {
                        0..=8 => (&mut src0_gains, j),
                        9..=17 => (&mut src1_gains, j - 9),
                        18..=26 => (&mut src2_gains, j - 18),
                        _ => (&mut src3_gains, j - 27),
                    };
                    gains[pos * 2] = l;
                    gains[pos * 2 + 1] = r;
                });

            cmds.push(EnsembleCmd::MixerSrc0(i, src0_gains));
            cmds.push(EnsembleCmd::MixerSrc1(i, src1_gains));
            cmds.push(EnsembleCmd::MixerSrc2(i, src2_gains));
            cmds.push(EnsembleCmd::MixerSrc3(i, src3_gains));
        });

        cmds
    }
}

impl EnsembleParameterProtocol<EnsembleMixerParameters> for BebobAvc {}

/// The trait for parameter protocol.
pub trait EnsembleParameterProtocol<T>: Ta1394Avc
where
    for<'a> Vec<EnsembleCmd>: From<&'a T>,
    T: Copy,
{
    fn init_params(&mut self, params: &mut T, timeout_ms: u32) -> Result<(), Error> {
        Vec::<EnsembleCmd>::from(&(*params))
            .into_iter()
            .try_for_each(|cmd| {
                let mut op = EnsembleOperation::new(cmd);
                self.control(&AvcAddr::Unit, &mut op, timeout_ms)
            })
    }

    fn update_params(&mut self, new: &T, old: &mut T, timeout_ms: u32) -> Result<(), Error> {
        Vec::<EnsembleCmd>::from(new)
            .into_iter()
            .zip(Vec::<EnsembleCmd>::from(&(*old)).iter())
            .filter(|(n, o)| !n.eq(o))
            .try_for_each(|(n, _)| {
                let mut op = EnsembleOperation::new(n);
                self.control(&AvcAddr::Unit, &mut op, timeout_ms)
            })
            .map(|_| *old = *new)
    }
}

/// The target of input for knob.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum KnobInputTarget {
    Mic0,
    Mic1,
    Mic2,
    Mic3,
}

impl Default for KnobInputTarget {
    fn default() -> Self {
        Self::Mic1
    }
}

/// The target of output for knob.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum KnobOutputTarget {
    AnalogOutputPair0,
    HeadphonePair0,
    HeadphonePair1,
}

impl Default for KnobOutputTarget {
    fn default() -> Self {
        Self::AnalogOutputPair0
    }
}

/// The structure for meter information.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct EnsembleMeter {
    pub knob_input_target: KnobInputTarget,
    pub knob_output_target: KnobOutputTarget,
    pub knob_input_vals: [u8; 4],
    pub knob_output_vals: [u8; 3],
    pub phys_inputs: [u8; 18],
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
#[derive(Default)]
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

/// The trait of operation for meter information.
impl EnsembleMeterProtocol {
    pub const OUT_KNOB_VAL_MIN: u8 = EnsembleOutputParameters::VOL_MIN;
    pub const OUT_KNOB_VAL_MAX: u8 = EnsembleOutputParameters::VOL_MAX;
    pub const OUT_KNOB_VAL_STEP: u8 = EnsembleOutputParameters::VOL_STEP;

    pub const IN_KNOB_VAL_MIN: u8 = EnsembleInputParameters::GAIN_MIN;
    pub const IN_KNOB_VAL_MAX: u8 = EnsembleInputParameters::GAIN_MAX;
    pub const IN_KNOB_VAL_STEP: u8 = EnsembleInputParameters::GAIN_STEP;

    pub const LEVEL_MIN: u8 = u8::MIN;
    pub const LEVEL_MAX: u8 = u8::MAX;
    pub const LEVEL_STEP: u8 = 1;

    pub fn measure_meter(
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
                    .zip(meter.knob_input_vals.iter_mut())
                    .for_each(|(&i, m)| *m = frame[i]);

                OUT_VOL_POS
                    .iter()
                    .zip(meter.knob_output_vals.iter_mut())
                    .for_each(|(&i, m)| {
                        *m = Self::OUT_KNOB_VAL_MAX - (frame[i] & Self::OUT_KNOB_VAL_MAX);
                    });

                IN_METER_POS
                    .iter()
                    .zip(meter.phys_inputs.iter_mut())
                    .for_each(|(&i, m)| *m = frame[i]);

                OUT_METER_POS
                    .iter()
                    .zip(meter.phys_outputs.iter_mut())
                    .for_each(|(&i, m)| *m = frame[i]);
            }
        })
    }
}

/// The nominal level of analog input 0-7.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FormatConvertTarget {
    Disabled,
    AnalogInputPair0,
    AnalogInputPair1,
    AnalogInputPair2,
    AnalogInputPair3,
    SpdifOpticalInputPair0,
    SpdifCoaxialInputPair0,
    SpdifCoaxialOutputPair0,
    SpdifOpticalOutputPair0,
}

impl Default for FormatConvertTarget {
    fn default() -> Self {
        Self::Disabled
    }
}

/// The target to convert sample rate.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RateConvertTarget {
    Disabled,
    SpdifOpticalOutputPair0,
    SpdifCoaxialOutputPair0,
    SpdifOpticalInputPair0,
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
    R44100,
    R48000,
    R88200,
    R96000,
    R176400,
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

/// The enumeration of command specific to Apogee Ensemble.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EnsembleCmd {
    InputLimit(usize, bool), // index, state
    MicPower(usize, bool),   // index, state
    InputNominalLevel(usize, InputNominalLevel),
    OutputNominalLevel(usize, OutputNominalLevel),
    IoRouting(usize, usize), // destination, source
    Hw(HwCmd),
    HpSrc(usize, usize), // destination, source
    MixerSrc0(usize, [i16; MIXER_COEFFICIENT_COUNT]),
    MixerSrc1(usize, [i16; MIXER_COEFFICIENT_COUNT]),
    MixerSrc2(usize, [i16; MIXER_COEFFICIENT_COUNT]),
    MixerSrc3(usize, [i16; MIXER_COEFFICIENT_COUNT]),
    MicGain(usize, u8), // 1/2/3/4, dB(10-75), also available as knob control
    OutputOptIface(OptIfaceMode),
    InputOptIface(OptIfaceMode),
    FormatConvert(FormatConvertTarget),
    RateConvert(RateConvertTarget, RateConvertRate),
    MicPolarity(usize, bool), // index, state
    OutVol(usize, u8),        // main/hp0/hp1, dB(127-0), also available as knob control
    HwStatusShort([u8; METER_SHORT_FRAME_SIZE]),
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
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StreamMode {
    Format18x18,
    Format10x10,
    Format8x8,
}

impl Default for StreamMode {
    fn default() -> Self {
        StreamMode::Format8x8
    }
}

/// The target of display meter.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DisplayMeterTarget {
    Output,
    Input,
}

impl Default for DisplayMeterTarget {
    fn default() -> Self {
        Self::Output
    }
}

/// The enumeration of command for hardware operation.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum HwCmd {
    /// STREAM_MODE command generates bus reset to change available stream formats.
    StreamMode(StreamMode),
    DisplayIlluminate(bool),
    DisplayMode(bool),
    DisplayTarget(DisplayMeterTarget),
    DisplayOverhold(bool),
    MeterReset,
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
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        self.op.data = Into::<Vec<u8>>::into(&self.cmd);

        // At least, 6 bytes should be required to align to 3 quadlets. Unless, the target unit is freezed.
        while self.op.data.len() < 6 {
            self.op.data.push(0xff);
        }

        AvcControl::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
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
