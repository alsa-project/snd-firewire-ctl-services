// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by Apogee Electronics for Duet FireWire.
//!
//! The module includes protocol implementation defined by Apogee Electronics for Duet FireWire.

use glib::{Error, FileError};

use hinawa::{FwFcp, FwNode, FwReq, FwReqExtManual, FwTcode};

use ta1394::{general::VendorDependent, *};

const APOGEE_OUI: [u8; 3] = [0x00, 0x03, 0xdb];

const METER_OFFSET_BASE: u64 = 0xfffff0080000;
const METER_INPUT_OFFSET: u64 = 0x0004;
const METER_MIXER_OFFSET: u64 = 0x0404;

/// The state of meter for analog input.
#[derive(Default, Debug)]
pub struct DuetFwInputMeterState(pub [i32; 2]);

/// The protocol implementation of meter for analog input.
#[derive(Default, Debug)]
pub struct DuetFwInputMeterProtocol;

impl DuetFwInputMeterProtocol {
    const ANALOG_INPUT_SIZE: usize = 8;

    pub const LEVEL_MIN: i32 = 0;
    pub const LEVEL_MAX: i32 = i32::MAX;
    pub const LEVEL_STEP: i32 = 0x100;

    pub fn read_state(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut DuetFwInputMeterState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut frame = [0; Self::ANALOG_INPUT_SIZE];

        req.transaction_sync(
            node,
            FwTcode::ReadBlockRequest,
            METER_OFFSET_BASE + METER_INPUT_OFFSET,
            frame.len(),
            &mut frame,
            timeout_ms,
        )
        .map(|_| {
            let mut quadlet = [0; 4];
            state.0.iter_mut().enumerate().for_each(|(i, meter)| {
                let pos = i * 4;
                quadlet.copy_from_slice(&frame[pos..(pos + 4)]);
                *meter = i32::from_be_bytes(quadlet);
            });
        })
    }
}

/// The state of meter for mixer source/output.
#[derive(Default, Debug)]
pub struct DuetFwMixerMeterState {
    pub stream_inputs: [i32; 2],
    pub mixer_outputs: [i32; 2],
}

/// The protocol implementation of meter for mixer source/output.
#[derive(Default, Debug)]
pub struct DuetFwMixerMeterProtocol;

impl DuetFwMixerMeterProtocol {
    const MIXER_IO_SIZE: usize = 16;

    pub const LEVEL_MIN: i32 = 0;
    pub const LEVEL_MAX: i32 = i32::MAX;
    pub const LEVEL_STEP: i32 = 0x100;

    pub fn read_state(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut DuetFwMixerMeterState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut frame = [0; Self::MIXER_IO_SIZE];

        req.transaction_sync(
            node,
            FwTcode::ReadBlockRequest,
            METER_OFFSET_BASE + METER_MIXER_OFFSET,
            frame.len(),
            &mut frame,
            timeout_ms,
        )
        .map(|_| {
            let mut quadlet = [0; 4];
            state.stream_inputs.iter_mut()
                .chain(state.mixer_outputs.iter_mut())
                .enumerate()
                .for_each(|(i, meter)| {
                    let pos = i * 4;
                    quadlet.copy_from_slice(&frame[pos..(pos + 4)]);
                    *meter = i32::from_be_bytes(quadlet);
                });
        })
    }
}

/// The enumeration for target of knob.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DuetFwKnobTarget {
    OutputPair0,
    InputPair0,
    InputPair1,
}

impl Default for DuetFwKnobTarget {
    fn default() -> Self {
        Self::OutputPair0
    }
}

/// The state of rotary knob.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct DuetFwKnobState {
    pub output_mute: bool,
    pub target: DuetFwKnobTarget,
    pub output_volume: u8,
    pub input_gains: [u8; 2],
}

/// The protocol implementation of rotary knob.
#[derive(Default)]
pub struct DuetFwKnobProtocol;

impl DuetFwKnobProtocol {
    pub const VOLUME_MIN: u8 = 0;
    pub const VOLUME_MAX: u8 = 64;
    pub const VOLUME_STEP: u8 = 1;

    pub const GAIN_MIN: u8 = 10;
    pub const GAIN_MAX: u8 = 75;
    pub const GAIN_STEP: u8 = 1;
}

impl DuetFwKnobProtocol {
    pub fn read_state(
        avc: &mut FwFcp,
        state: &mut DuetFwKnobState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::HwState(Default::default()));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            if let VendorCmd::HwState(raw) = &op.cmd {
                state.output_mute = raw[0] > 0;
                state.target = match raw[1] {
                    2 => DuetFwKnobTarget::InputPair1,
                    1 => DuetFwKnobTarget::InputPair0,
                    _ => DuetFwKnobTarget::OutputPair0,
                };
                state.output_volume = Self::VOLUME_MAX - raw[3];
                state.input_gains[0] = raw[4];
                state.input_gains[1] = raw[5];
            }
        })
    }
}

/// The enumeration for source of output.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DuetFwOutputSource {
    StreamInputPair0,
    MixerOutputPair0,
}

impl Default for DuetFwOutputSource {
    fn default() -> Self {
        Self::StreamInputPair0
    }
}

/// The enumeration for level of output.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

/// The enumeration for level of output.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DuetFwOutputMuteMode {
    Never,
    Normal,
    Swapped,
}

impl Default for DuetFwOutputMuteMode {
    fn default() -> Self {
        Self::Never
    }
}

/// The protocol implementation of output parameters.
#[derive(Default)]
pub struct DuetFwOutputProtocol;

impl DuetFwOutputProtocol {
    pub const VOLUME_MIN: u8 = 0;
    pub const VOLUME_MAX: u8 = 64;
    pub const VOLUME_STEP: u8 = 1;

    pub fn read_mute(avc: &mut FwFcp, mute: &mut bool, timeout_ms: u32) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::OutMute);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)
            .map(|_| *mute = op.get_enum() > 0)
    }

    pub fn write_mute(avc: &mut FwFcp, mute: bool, timeout_ms: u32) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::OutMute);
        op.put_enum(mute as u32);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_volume(avc: &mut FwFcp, volume: &mut u8, timeout_ms: u32) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::OutVolume);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)
            .map(|_| *volume = Self::VOLUME_MAX - op.vals[0])
    }

    pub fn write_volume(avc: &mut FwFcp, volume: u8, timeout_ms: u32) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::OutVolume);
        op.vals.push(Self::VOLUME_MAX - volume);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_src(
        avc: &mut FwFcp,
        src: &mut DuetFwOutputSource,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::UseMixerOut);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            *src = if op.get_enum() > 0 {
                DuetFwOutputSource::MixerOutputPair0
            } else {
                DuetFwOutputSource::StreamInputPair0
            };
        })
    }

    pub fn write_src(
        avc: &mut FwFcp,
        src: DuetFwOutputSource,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = if src == DuetFwOutputSource::MixerOutputPair0 {
            1
        } else {
            0
        };
        let mut op = ApogeeCmd::new(VendorCmd::UseMixerOut);
        op.put_enum(val);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_nominal_level(
        avc: &mut FwFcp,
        level: &mut DuetFwOutputNominalLevel,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::OutAttr);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            *level = if op.get_enum() > 0 {
                DuetFwOutputNominalLevel::Consumer
            } else {
                DuetFwOutputNominalLevel::Instrument
            };
        })
    }

    pub fn write_nominal_level(
        avc: &mut FwFcp,
        level: DuetFwOutputNominalLevel,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = if level == DuetFwOutputNominalLevel::Consumer {
            1
        } else {
            0
        };
        let mut op = ApogeeCmd::new(VendorCmd::OutAttr);
        op.put_enum(val);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    fn parse_mute_mode(mute: bool, unmute: bool) -> DuetFwOutputMuteMode {
        match (mute, unmute) {
            (true, true) => DuetFwOutputMuteMode::Never,
            (true, false) => DuetFwOutputMuteMode::Swapped,
            (false, true) => DuetFwOutputMuteMode::Normal,
            (false, false) => DuetFwOutputMuteMode::Never,
        }
    }

    fn build_mute_mode(mode: DuetFwOutputMuteMode) -> (bool, bool) {
        match mode {
            DuetFwOutputMuteMode::Never => (true, true),
            DuetFwOutputMuteMode::Normal => (false, true),
            DuetFwOutputMuteMode::Swapped => (true, false),
        }
    }

    pub fn read_mute_mode_for_analog_output(
        avc: &mut FwFcp,
        mode: &mut DuetFwOutputMuteMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::MuteForLineOut);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
        let mute_enabled = op.get_enum() > 0;

        let mut op = ApogeeCmd::new(VendorCmd::UnmuteForLineOut);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
        let unmute_enabled = op.get_enum() > 0;

        *mode = Self::parse_mute_mode(mute_enabled, unmute_enabled);

        Ok(())
    }

    pub fn write_mute_mode_for_analog_output(
        avc: &mut FwFcp,
        mode: DuetFwOutputMuteMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let (mute_enabled, unmute_enabled) = Self::build_mute_mode(mode);

        let mut op = ApogeeCmd::new(VendorCmd::MuteForLineOut);
        op.put_enum(mute_enabled as u32);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = ApogeeCmd::new(VendorCmd::UnmuteForLineOut);
        op.put_enum(unmute_enabled as u32);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_mute_mode_for_hp(
        avc: &mut FwFcp,
        mode: &mut DuetFwOutputMuteMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::MuteForHpOut);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
        let mute_enabled = op.get_enum() > 0;

        let mut op = ApogeeCmd::new(VendorCmd::UnmuteForHpOut);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
        let unmute_enabled = op.get_enum() > 0;

        *mode = Self::parse_mute_mode(mute_enabled, unmute_enabled);

        Ok(())
    }

    pub fn write_mute_mode_for_hp(
        avc: &mut FwFcp,
        mode: DuetFwOutputMuteMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let (mute_enabled, unmute_enabled) = Self::build_mute_mode(mode);

        let mut op = ApogeeCmd::new(VendorCmd::MuteForHpOut);
        op.put_enum(mute_enabled as u32);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = ApogeeCmd::new(VendorCmd::UnmuteForHpOut);
        op.put_enum(unmute_enabled as u32);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }
}

/// The enumeration for source of input.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

/// The enumeration for nominal level of input.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

/// The protocol implementation of input parameters.
#[derive(Default)]
pub struct DuetFwInputProtocol;

impl DuetFwInputProtocol {
    pub const GAIN_MIN: u8 = 10;
    pub const GAIN_MAX: u8 = 75;
    pub const GAIN_STEP: u8 = 1;

    pub fn read_gain(
        avc: &mut FwFcp,
        idx: usize,
        gain: &mut u8,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::InGain(idx, Default::default()));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            if let VendorCmd::InGain(_, g) = &op.cmd {
                *gain = *g
            }
        })
    }

    pub fn write_gain(avc: &mut FwFcp, idx: usize, gain: u8, timeout_ms: u32) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::InGain(idx, gain));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_polarity(
        avc: &mut FwFcp,
        idx: usize,
        polarity: &mut bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::MicPolarity(idx as u8));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)
            .map(|_| *polarity = op.get_enum() > 0)
    }

    pub fn write_polarity(
        avc: &mut FwFcp,
        idx: usize,
        polarity: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::MicPolarity(idx as u8));
        op.put_enum(polarity as u32);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_xlr_nominal_level(
        avc: &mut FwFcp,
        idx: usize,
        level: &mut DuetFwInputXlrNominalLevel,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::PhoneInLine(idx as u8));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
        let is_line_level = op.get_enum() > 0;
        let mut op = ApogeeCmd::new(VendorCmd::LineInLevel(idx as u8));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
        let is_consumer_level = op.get_enum() > 0;
        *level = if is_line_level {
            if is_consumer_level {
                DuetFwInputXlrNominalLevel::Consumer
            } else {
                DuetFwInputXlrNominalLevel::Professional
            }
        } else {
            DuetFwInputXlrNominalLevel::Microphone
        };
        Ok(())
    }

    pub fn write_xlr_nominal_level(
        avc: &mut FwFcp,
        idx: usize,
        level: DuetFwInputXlrNominalLevel,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let (is_line_level, is_consumer_level) = match level {
            DuetFwInputXlrNominalLevel::Consumer => (true, true),
            DuetFwInputXlrNominalLevel::Professional => (true, false),
            DuetFwInputXlrNominalLevel::Microphone => (false, true),
        };
        let mut op = ApogeeCmd::new(VendorCmd::PhoneInLine(idx as u8));
        op.put_enum(is_line_level as u32);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
        let mut op = ApogeeCmd::new(VendorCmd::LineInLevel(idx as u8));
        op.put_enum(is_consumer_level as u32);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_phantom_powering(
        avc: &mut FwFcp,
        idx: usize,
        enable: &mut bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::MicPhantom(idx as u8));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)
            .map(|_| *enable = op.get_enum() > 0)
    }

    pub fn write_phantom_powering(
        avc: &mut FwFcp,
        idx: usize,
        enable: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::MicPhantom(idx as u8));
        op.put_enum(enable as u32);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_src(
        avc: &mut FwFcp,
        idx: usize,
        src: &mut DuetFwInputSource,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::MicIn(idx as u8));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            *src = if op.get_enum() > 0 {
                DuetFwInputSource::Phone
            } else {
                DuetFwInputSource::Xlr
            };
        })
    }

    pub fn write_src(
        avc: &mut FwFcp,
        idx: usize,
        src: DuetFwInputSource,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = if src == DuetFwInputSource::Phone {
            1
        } else {
            0
        };
        let mut op = ApogeeCmd::new(VendorCmd::MicIn(idx as u8));
        op.put_enum(val);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_clickless(
        avc: &mut FwFcp,
        enable: &mut bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::InClickless);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)
            .map(|_| *enable = op.get_enum() > 0)
    }

    pub fn write_clickless(avc: &mut FwFcp, enable: bool, timeout_ms: u32) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::InClickless);
        op.put_enum(enable as u32);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }
}

/// The protocol implementation of mixer.
#[derive(Default)]
pub struct DuetFwMixerProtocol;

impl DuetFwMixerProtocol {
    pub const GAIN_MIN: u16 = 0;
    pub const GAIN_MAX: u16 = 0x3fff;
    pub const GAIN_STEP: u16 = 0x80;

    pub fn read_source_gain(
        avc: &mut FwFcp,
        dst: usize,
        src: usize,
        gain: &mut u16,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::MixerSrc(src, dst, Default::default()));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            if let VendorCmd::MixerSrc(_, _, g) = &op.cmd {
                *gain = *g
            }
        })
    }

    pub fn write_source_gain(
        avc: &mut FwFcp,
        dst: usize,
        src: usize,
        gain: u16,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::MixerSrc(src, dst, gain));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }
}

/// The enumeration for target of display.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DuetFwDisplayTarget {
    Output,
    Input,
}

impl Default for DuetFwDisplayTarget {
    fn default() -> Self {
        Self::Output
    }
}

/// The enumeration for mode of display.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DuetFwDisplayMode {
    Independent,
    FollowingToKnobTarget,
}

impl Default for DuetFwDisplayMode {
    fn default() -> Self {
        Self::Independent
    }
}

/// The enumeration for overhold of display.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DuetFwDisplayOverhold {
    Infinite,
    TwoSeconds,
}

impl Default for DuetFwDisplayOverhold {
    fn default() -> Self {
        Self::Infinite
    }
}

/// The protocol implementation of display.
#[derive(Default)]
pub struct DuetFwDisplayProtocol;

impl DuetFwDisplayProtocol {
    pub fn read_target(
        avc: &mut FwFcp,
        target: &mut DuetFwDisplayTarget,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::DisplayInput);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            *target = if op.get_enum() > 0 {
                DuetFwDisplayTarget::Input
            } else {
                DuetFwDisplayTarget::Output
            }
        })
    }

    pub fn write_target(
        avc: &mut FwFcp,
        target: DuetFwDisplayTarget,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = if target == DuetFwDisplayTarget::Input {
            1
        } else {
            0
        };
        let mut op = ApogeeCmd::new(VendorCmd::DisplayInput);
        op.put_enum(val);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_mode(
        avc: &mut FwFcp,
        mode: &mut DuetFwDisplayMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::DisplayFollow);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            *mode = if op.get_enum() > 0 {
                DuetFwDisplayMode::FollowingToKnobTarget
            } else {
                DuetFwDisplayMode::Independent
            }
        })
    }

    pub fn write_mode(
        avc: &mut FwFcp,
        mode: DuetFwDisplayMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = if mode == DuetFwDisplayMode::FollowingToKnobTarget {
            1
        } else {
            0
        };
        let mut op = ApogeeCmd::new(VendorCmd::DisplayFollow);
        op.put_enum(val);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_overhold(
        avc: &mut FwFcp,
        mode: &mut DuetFwDisplayOverhold,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::DisplayOverhold);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            *mode = if op.get_enum() > 0 {
                DuetFwDisplayOverhold::TwoSeconds
            } else {
                DuetFwDisplayOverhold::Infinite
            };
        })
    }

    pub fn write_overhold(
        avc: &mut FwFcp,
        mode: DuetFwDisplayOverhold,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = if mode == DuetFwDisplayOverhold::TwoSeconds {
            1
        } else {
            0
        };
        let mut op = ApogeeCmd::new(VendorCmd::DisplayOverhold);
        op.put_enum(val);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }
}

/// The enumeration to represent type of command for Apogee Duet FireWire.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
// Usually 5 params.
pub enum VendorCmd {
    MicPolarity(u8),
    PhoneInLine(u8),
    LineInLevel(u8),
    MicPhantom(u8),
    OutAttr,
    InGain(usize, u8),
    HwState([u8; 11]),
    OutMute,
    MicIn(u8),
    MixerSrc(usize, usize, u16),
    UseMixerOut,
    DisplayOverhold,
    DisplayClear,
    OutVolume,
    MuteForLineOut,
    MuteForHpOut,
    UnmuteForLineOut,
    UnmuteForHpOut,
    DisplayInput,
    InClickless,
    DisplayFollow,
}

impl VendorCmd {
    const APOGEE_PREFIX: [u8; 3] = [0x50, 0x43, 0x4d]; // 'P', 'C', 'M'

    const MIC_POLARITY: u8 = 0x00;
    const PHONE_IN_LEVEL: u8 = 0x01;
    const LINE_IN_LEVEL: u8 = 0x02;
    const MIC_PHANTOM: u8 = 0x03;
    const OUT_ATTR: u8 = 0x04;
    const IN_GAIN: u8 = 0x05;
    const HW_STATE: u8 = 0x07;
    const OUT_MUTE: u8 = 0x09;
    const USE_LINE_IN: u8 = 0x0c;
    const MIXER_SRC: u8 = 0x10;
    const USE_MIXER_OUT: u8 = 0x11;
    const DISPLAY_OVERHOLD: u8 = 0x13;
    const DISPLAY_CLEAR: u8 = 0x14;
    const OUT_VOLUME: u8 = 0x15;
    const MUTE_FOR_LINE_OUT: u8 = 0x16;
    const MUTE_FOR_HP_OUT: u8 = 0x17;
    const UNMUTE_FOR_LINE_OUT: u8 = 0x18;
    const UNMUTE_FOR_HP_OUT: u8 = 0x19;
    const DISPLAY_INPUT: u8 = 0x1b;
    const IN_CLICKLESS: u8 = 0x1e;
    const DISPLAY_FOLLOW: u8 = 0x22;

    const ON: u8 = 0x70;
    const OFF: u8 = 0x60;

    fn build_args(&self) -> Vec<u8> {
        let mut args = Vec::with_capacity(6);
        args.extend_from_slice(&Self::APOGEE_PREFIX);
        args.extend_from_slice(&[0xff; 3]);

        match self {
            Self::MicPolarity(ch) => {
                args[3] = Self::MIC_POLARITY;
                args[4] = 0x80;
                args[5] = *ch;
            }
            Self::PhoneInLine(ch) => {
                args[3] = Self::PHONE_IN_LEVEL;
                args[4] = 0x80;
                args[5] = *ch;
            }
            Self::LineInLevel(ch) => {
                args[3] = Self::LINE_IN_LEVEL;
                args[4] = 0x80;
                args[5] = *ch;
            }
            Self::MicPhantom(ch) => {
                args[3] = Self::MIC_PHANTOM;
                args[4] = 0x80;
                args[5] = *ch;
            }
            Self::OutAttr => {
                args[3] = Self::OUT_ATTR;
                args[4] = 0x80;
            }
            Self::InGain(ch, _) => {
                args[3] = Self::IN_GAIN;
                args[4] = 0x80;
                args[5] = *ch as u8;
            }
            Self::HwState(_) => args[3] = Self::HW_STATE,
            Self::OutMute => {
                args[3] = Self::OUT_MUTE;
                args[4] = 0x80;
            }
            Self::MicIn(ch) => {
                args[3] = Self::USE_LINE_IN;
                args[4] = 0x80;
                args[5] = *ch;
            }
            Self::MixerSrc(src, dst, _) => {
                args[3] = Self::MIXER_SRC;
                args[4] = (((*src / 2) << 4) | (*src % 2)) as u8;
                args[5] = *dst as u8;
            }
            Self::UseMixerOut => args[3] = Self::USE_MIXER_OUT,
            Self::DisplayOverhold => args[3] = Self::DISPLAY_OVERHOLD,
            Self::DisplayClear => args[3] = Self::DISPLAY_CLEAR,
            Self::OutVolume => {
                args[3] = Self::OUT_VOLUME;
                args[4] = 0x80;
            }
            Self::MuteForLineOut => {
                args[3] = Self::MUTE_FOR_LINE_OUT;
                args[4] = 0x80;
            }
            Self::MuteForHpOut => {
                args[3] = Self::MUTE_FOR_HP_OUT;
                args[4] = 0x80;
            }
            Self::UnmuteForLineOut => {
                args[3] = Self::UNMUTE_FOR_LINE_OUT;
                args[4] = 0x80;
            }
            Self::UnmuteForHpOut => {
                args[3] = Self::UNMUTE_FOR_HP_OUT;
                args[4] = 0x80;
            }
            Self::DisplayInput => args[3] = Self::DISPLAY_INPUT,
            Self::InClickless => args[3] = Self::IN_CLICKLESS,
            Self::DisplayFollow => args[3] = Self::DISPLAY_FOLLOW,
        }

        args
    }

    fn append_variable(&self, data: &mut Vec<u8>) {
        match self {
            Self::InGain(_, gain) => data.push(*gain),
            Self::MixerSrc(_, _, gain) => data.extend_from_slice(&gain.to_be_bytes()),
            _ => (),
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
            _ => Ok(()),
        }
    }
}

/// The structure to represent protocol of Apogee Duet FireWire.
pub struct ApogeeCmd {
    cmd: VendorCmd,
    vals: Vec<u8>,
    op: VendorDependent,
}

impl ApogeeCmd {
    pub fn new(cmd: VendorCmd) -> Self {
        ApogeeCmd{
            cmd,
            vals: Vec::new(),
            op: VendorDependent::new(&APOGEE_OUI),
        }
    }

    fn parse_data(&mut self) -> Result<(), Error> {
        let args = self.cmd.build_args();
        if self.op.data[..6] != args[..6] {
            let label = format!("Unexpected arguments in response: {:?} but {:?}", args, self.op.data);
            Err(Error::new(Ta1394AvcError::UnexpectedRespOperands, &label))
        } else {
            self.vals = self.op.data.split_off(6);
            Ok(())
        }
    }

    pub fn get_enum(&self) -> u32 {
        assert!(self.vals.len() > 0, "Unexpected read operation as bool argument.");
        (self.vals[0] == VendorCmd::ON) as u32
    }

    pub fn put_enum(&mut self, val: u32) {
        assert!(self.vals.len() == 0, "Unexpected write operation as bool argument.");
        self.vals.push(if val > 0 { VendorCmd::ON } else { VendorCmd::OFF })
    }
}

impl AvcOp for ApogeeCmd {
    const OPCODE: u8 = VendorDependent::OPCODE;
}

impl AvcControl for ApogeeCmd {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        let mut data = self.cmd.build_args();
        self.cmd.append_variable(&mut data);
        data.extend_from_slice(&self.vals);
        self.op.data = data;
        AvcControl::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcControl::parse_operands(&mut self.op, addr, operands)?;
        self.parse_data()
    }
}

impl AvcStatus for ApogeeCmd {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        self.op.data = self.cmd.build_args();
        AvcStatus::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcStatus::parse_operands(&mut self.op, addr, operands)?;
        self.cmd.parse_variable(&self.op.data)?;
        self.parse_data()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn apogee_cmd_proto_operands() {
        // No argument command.
        let mut op = ApogeeCmd::new(VendorCmd::UseMixerOut);
        let operands = [0x00, 0x03, 0xdb, 0x50, 0x43, 0x4d, 0x11, 0xff, 0xff, 0xe3];
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.vals, &[0xe3]);

        let mut o = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands[..9]);

        let mut op = ApogeeCmd::new(VendorCmd::UseMixerOut);
        let operands = [0x00, 0x03, 0xdb, 0x50, 0x43, 0x4d, 0x11, 0xff, 0xff, 0xe3];
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.vals, &[0xe3]);

        let mut o = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands);

        // One argument command.
        let mut op = ApogeeCmd::new(VendorCmd::LineInLevel(1));
        let operands = [0x00, 0x03, 0xdb, 0x50, 0x43, 0x4d, 0x02, 0x80, 0x01, 0xb9];
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.vals, &[0xb9]);

        let mut o = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands[..9]);

        let mut op = ApogeeCmd::new(VendorCmd::LineInLevel(1));
        let operands = [0x00, 0x03, 0xdb, 0x50, 0x43, 0x4d, 0x02, 0x80, 0x01, 0xb9];
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.vals, &[0xb9]);

        let mut o = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands);

        // Two arguments command.
        let mut op = ApogeeCmd::new(VendorCmd::MixerSrc(1, 0, Default::default()));
        let operands = [0x00, 0x03, 0xdb, 0x50, 0x43, 0x4d, 0x10, 0x01, 0x00, 0xde, 0x00, 0xad, 0x01, 0xbe, 0x02, 0xef];
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        if let VendorCmd::MixerSrc(src, dst, gain) = &op.cmd {
            assert_eq!(*src, 1);
            assert_eq!(*dst, 0);
            assert_eq!(*gain, 0xde00);
        } else {
            unreachable!();
        }

        let mut o = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands[..9]);

        let mut op = ApogeeCmd::new(VendorCmd::MixerSrc(1, 0, 0xde00));
        let operands = [0x00, 0x03, 0xdb, 0x50, 0x43, 0x4d, 0x10, 0x01, 0x00, 0xde, 0x00, 0xad, 0x01, 0xbe, 0x02, 0xef];
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();

        let mut o = Vec::new();
        op.vals.clear();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands[..11]);

        // Command for block request.
        let mut op = ApogeeCmd::new(VendorCmd::HwState(Default::default()));
        let operands = [0x00, 0x03, 0xdb, 0x50, 0x43, 0x4d, 0x07, 0xff, 0xff,
                        0xde, 0x00, 0xad, 0x01, 0xbe, 0x02, 0xef, 0xde, 0xad, 0xbe, 0xef];
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        if let VendorCmd::HwState(raw) = &op.cmd {
            assert_eq!(raw, &[0xde, 0x00, 0xad, 0x01, 0xbe, 0x02, 0xef, 0xde, 0xad, 0xbe, 0xef]);
        } else {
            unreachable!();
        }

        let mut o = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands[..9]);
    }
}
