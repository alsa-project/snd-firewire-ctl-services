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
    pub const OUT_KNOB_VAL_MIN: u8 = 0;
    pub const OUT_KNOB_VAL_MAX: u8 = 0x7f;
    pub const OUT_KNOB_VAL_STEP: u8 = 1;

    pub const IN_KNOB_VAL_MIN: u8 = 0;
    pub const IN_KNOB_VAL_MAX: u8 = 75;
    pub const IN_KNOB_VAL_STEP: u8 = 1;

    pub const LEVEL_MIN: u8 = u8::MIN;
    pub const LEVEL_MAX: u8 = u8::MAX;
    pub const LEVEL_STEP: u8 = 1;

    pub fn measure_meter(
        avc: &mut BebobAvc,
        meter: &mut EnsembleMeter,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let cmd = EnsembleCmd::HwStatusLong([0; METER_LONG_FRAME_SIZE]);
        let mut op = EnsembleOperation::new(cmd, &[]);
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

const METER_SHORT_FRAME_SIZE: usize = 17;
const METER_LONG_FRAME_SIZE: usize = 56;
const MIXER_COEFFICIENT_COUNT: usize = 18;

/// The enumeration of command specific to Apogee Ensemble.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EnsembleCmd {
    InputLimit(u8), // index, state
    MicPower(u8),   // index, state
    InputNominalLevel(usize, InputNominalLevel),
    OutputNominalLevel(usize, OutputNominalLevel),
    IoRouting(usize, usize), // destination, source
    Hw(HwCmd),
    HpSrc(u8), // destination, source
    MixerSrc0(usize, [i16; MIXER_COEFFICIENT_COUNT]),
    MixerSrc1(usize, [i16; MIXER_COEFFICIENT_COUNT]),
    MixerSrc2(usize, [i16; MIXER_COEFFICIENT_COUNT]),
    MixerSrc3(usize, [i16; MIXER_COEFFICIENT_COUNT]),
    MicGain(u8), // 1/2/3/4, dB(10-75), also available as knob control
    OutputOptIface(OptIfaceMode),
    InputOptIface(OptIfaceMode),
    Downgrade,       // on/off
    SpdifResample,   // on/off, iface, direction, rate
    MicPolarity(u8), // index, state
    OutVol(u8),      // main/hp0/hp1, dB(127-0), also available as knob control
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
    const IN_VOL: u8 = 0xe6;
    const OPT_IFACE_MODE: u8 = 0xf1;
    const DOWNGRADE: u8 = 0xf2;
    const SPDIF_RESAMPLE: u8 = 0xf3;
    const MIC_POLARITY: u8 = 0xf5;
    const OUT_VOL: u8 = 0xf6;
    const HW_STATUS: u8 = 0xff;
}

impl From<&EnsembleCmd> for Vec<u8> {
    fn from(cmd: &EnsembleCmd) -> Self {
        match cmd {
            EnsembleCmd::InputLimit(ch) => {
                vec![EnsembleCmd::INPUT_LIMIT, *ch]
            }
            EnsembleCmd::MicPower(ch) => {
                vec![EnsembleCmd::MIC_POWER, *ch]
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
                vec![EnsembleCmd::HW, u8::from(*op)]
            }
            EnsembleCmd::HpSrc(dst) => {
                vec![EnsembleCmd::HP_SRC, (*dst + 1) % 2]
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
            EnsembleCmd::MicGain(target) => {
                vec![EnsembleCmd::IN_VOL, *target]
            }
            EnsembleCmd::OutputOptIface(mode) => {
                vec![EnsembleCmd::OPT_IFACE_MODE, 0, opt_iface_mode_to_val(mode)]
            }
            EnsembleCmd::InputOptIface(mode) => {
                vec![EnsembleCmd::OPT_IFACE_MODE, 1, opt_iface_mode_to_val(mode)]
            }
            EnsembleCmd::Downgrade => {
                vec![EnsembleCmd::DOWNGRADE]
            }
            EnsembleCmd::SpdifResample => {
                vec![EnsembleCmd::SPDIF_RESAMPLE]
            }
            EnsembleCmd::MicPolarity(ch) => {
                vec![EnsembleCmd::MIC_POLARITY, *ch]
            }
            EnsembleCmd::OutVol(target) => {
                vec![EnsembleCmd::OUT_VOL, *target]
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
            Self::INPUT_LIMIT => Self::InputLimit(raw[1]),
            Self::MIC_POWER => Self::MicPower(raw[1]),
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
            Self::HW => Self::Hw(HwCmd::from(raw[1])),
            Self::HP_SRC => Self::HpSrc((raw[1] + 1) % 2),
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
            Self::OPT_IFACE_MODE => {
                let mode = opt_iface_mode_from_val(raw[2]);
                if raw[1] > 0 {
                    Self::InputOptIface(mode)
                } else {
                    Self::OutputOptIface(mode)
                }
            }
            Self::DOWNGRADE => Self::Downgrade,
            Self::SPDIF_RESAMPLE => Self::SpdifResample,
            Self::MIC_POLARITY => Self::MicPolarity(raw[1]),
            Self::OUT_VOL => Self::OutVol(raw[1]),
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

/// The enumeration of command for hardware operation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HwCmd {
    /// STREAM_MODE command generates bus reset to change available stream formats.
    StreamMode,
    DisplayIlluminate,
    DisplayMode,
    DisplayTarget,
    DisplayOverhold,
    MeterReset,
    CdMode,
    Reserved(u8),
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

impl From<HwCmd> for u8 {
    fn from(op: HwCmd) -> Self {
        match op {
            HwCmd::StreamMode => HwCmd::STREAM_MODE,
            HwCmd::DisplayIlluminate => HwCmd::DISPLAY_ILLUMINATE,
            HwCmd::DisplayMode => HwCmd::DISPLAY_MODE,
            HwCmd::DisplayTarget => HwCmd::DISPLAY_TARGET,
            HwCmd::DisplayOverhold => HwCmd::DISPLAY_OVERHOLD,
            HwCmd::MeterReset => HwCmd::METER_RESET,
            HwCmd::CdMode => HwCmd::CD_MODE,
            HwCmd::Reserved(val) => val,
        }
    }
}

impl From<u8> for HwCmd {
    fn from(val: u8) -> HwCmd {
        match val {
            HwCmd::STREAM_MODE => HwCmd::StreamMode,
            HwCmd::DISPLAY_ILLUMINATE => HwCmd::DisplayIlluminate,
            HwCmd::DISPLAY_MODE => HwCmd::DisplayMode,
            HwCmd::DISPLAY_TARGET => HwCmd::DisplayTarget,
            HwCmd::DISPLAY_OVERHOLD => HwCmd::DisplayOverhold,
            HwCmd::METER_RESET => HwCmd::MeterReset,
            HwCmd::CD_MODE => HwCmd::CdMode,
            _ => HwCmd::Reserved(val),
        }
    }
}

/// The protocol implementation of AV/C vendor-dependent command specific to Apogee Ensemble.
#[derive(Debug)]
pub struct EnsembleOperation {
    pub cmd: EnsembleCmd,
    pub params: Vec<u8>,
    op: VendorDependent,
}

impl Default for EnsembleOperation {
    fn default() -> Self {
        Self {
            cmd: Default::default(),
            params: Default::default(),
            op: VendorDependent {
                company_id: APOGEE_OUI,
                data: Default::default(),
            },
        }
    }
}

impl EnsembleOperation {
    pub fn new(cmd: EnsembleCmd, params: &[u8]) -> Self {
        let mut op = EnsembleOperation::default();
        op.cmd = cmd;
        op.params.extend_from_slice(params);
        op
    }
}

impl AvcOp for EnsembleOperation {
    const OPCODE: u8 = VendorDependent::OPCODE;
}

impl AvcControl for EnsembleOperation {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        self.op.data.clear();
        self.op.data.append(&mut Into::<Vec<u8>>::into(&self.cmd));
        self.op.data.append(&mut self.params.clone());

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
        let cmd = EnsembleCmd::InputLimit(1);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::MicPower(1);
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

        let cmd = EnsembleCmd::Hw(HwCmd::StreamMode);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::HpSrc(1);
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

        let cmd = EnsembleCmd::Downgrade;
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::SpdifResample;
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::MicPolarity(0);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::OutVol(0);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );
    }
}
