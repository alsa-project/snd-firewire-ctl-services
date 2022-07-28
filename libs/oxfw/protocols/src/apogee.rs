// SPDX-License-Identifier: GPL-3.0-or-later
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
            state
                .stream_inputs
                .iter_mut()
                .chain(&mut state.mixer_outputs)
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
        avc: &mut OxfwAvc,
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

    pub fn read_mute(avc: &mut OxfwAvc, mute: &mut bool, timeout_ms: u32) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::OutMute(Default::default()));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            if let VendorCmd::OutMute(e) = &op.cmd {
                *mute = *e
            }
        })
    }

    pub fn write_mute(avc: &mut OxfwAvc, mute: bool, timeout_ms: u32) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::OutMute(mute));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_volume(avc: &mut OxfwAvc, volume: &mut u8, timeout_ms: u32) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::OutVolume(Default::default()));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            if let VendorCmd::OutVolume(v) = &op.cmd {
                *volume = Self::VOLUME_MAX - *v
            }
        })
    }

    pub fn write_volume(avc: &mut OxfwAvc, volume: u8, timeout_ms: u32) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::OutVolume(Self::VOLUME_MAX - volume));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_src(
        avc: &mut OxfwAvc,
        src: &mut DuetFwOutputSource,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::OutSourceIsMixer(Default::default()));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            if let VendorCmd::OutSourceIsMixer(enabled) = &op.cmd {
                *src = if *enabled {
                    DuetFwOutputSource::MixerOutputPair0
                } else {
                    DuetFwOutputSource::StreamInputPair0
                };
            }
        })
    }

    pub fn write_src(
        avc: &mut OxfwAvc,
        src: DuetFwOutputSource,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let enable = src == DuetFwOutputSource::MixerOutputPair0;
        let mut op = ApogeeCmd::new(VendorCmd::OutSourceIsMixer(enable));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_nominal_level(
        avc: &mut OxfwAvc,
        level: &mut DuetFwOutputNominalLevel,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::OutIsConsumerLevel(Default::default()));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            if let VendorCmd::OutIsConsumerLevel(enabled) = &op.cmd {
                *level = if *enabled {
                    DuetFwOutputNominalLevel::Consumer
                } else {
                    DuetFwOutputNominalLevel::Instrument
                }
            }
        })
    }

    pub fn write_nominal_level(
        avc: &mut OxfwAvc,
        level: DuetFwOutputNominalLevel,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let enable = level == DuetFwOutputNominalLevel::Consumer;
        let mut op = ApogeeCmd::new(VendorCmd::OutIsConsumerLevel(enable));
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
        avc: &mut OxfwAvc,
        mode: &mut DuetFwOutputMuteMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::MuteForLineOut(Default::default()));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
        let mute_enabled = match &op.cmd {
            VendorCmd::MuteForLineOut(enabled) => *enabled,
            _ => unreachable!(),
        };

        let mut op = ApogeeCmd::new(VendorCmd::UnmuteForLineOut(Default::default()));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
        let unmute_enabled = match &op.cmd {
            VendorCmd::UnmuteForLineOut(enabled) => *enabled,
            _ => unreachable!(),
        };

        *mode = Self::parse_mute_mode(mute_enabled, unmute_enabled);

        Ok(())
    }

    pub fn write_mute_mode_for_analog_output(
        avc: &mut OxfwAvc,
        mode: DuetFwOutputMuteMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let (mute_enabled, unmute_enabled) = Self::build_mute_mode(mode);

        let mut op = ApogeeCmd::new(VendorCmd::MuteForLineOut(mute_enabled));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = ApogeeCmd::new(VendorCmd::UnmuteForLineOut(unmute_enabled));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_mute_mode_for_hp(
        avc: &mut OxfwAvc,
        mode: &mut DuetFwOutputMuteMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::MuteForHpOut(Default::default()));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
        let mute_enabled = match &op.cmd {
            VendorCmd::MuteForHpOut(enabled) => *enabled,
            _ => unreachable!(),
        };

        let mut op = ApogeeCmd::new(VendorCmd::UnmuteForHpOut(Default::default()));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
        let unmute_enabled = match &op.cmd {
            VendorCmd::UnmuteForHpOut(enabled) => *enabled,
            _ => unreachable!(),
        };

        *mode = Self::parse_mute_mode(mute_enabled, unmute_enabled);

        Ok(())
    }

    pub fn write_mute_mode_for_hp(
        avc: &mut OxfwAvc,
        mode: DuetFwOutputMuteMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let (mute_enabled, unmute_enabled) = Self::build_mute_mode(mode);

        let mut op = ApogeeCmd::new(VendorCmd::MuteForHpOut(mute_enabled));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = ApogeeCmd::new(VendorCmd::UnmuteForHpOut(unmute_enabled));
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

/// The structure of input parameters.
#[derive(Default, Debug)]
pub struct DuetFwInputParameters {
    pub gains: [u8; 2],
    pub polarities: [bool; 2],
    pub xlr_nominal_levels: [DuetFwInputXlrNominalLevel; 2],
    pub phantom_powerings: [bool; 2],
    pub srcs: [DuetFwInputSource; 2],
    pub clickless: bool,
}

/// The protocol implementation of input parameters.
#[derive(Default)]
pub struct DuetFwInputProtocol;

impl DuetFwInputProtocol {
    pub const GAIN_MIN: u8 = 10;
    pub const GAIN_MAX: u8 = 75;
    pub const GAIN_STEP: u8 = 1;

    pub fn read_parameters(
        avc: &mut OxfwAvc,
        params: &mut DuetFwInputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        params
            .gains
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, gain)| {
                let mut op = ApogeeCmd::new(VendorCmd::InGain(i, Default::default()));
                avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
                    if let VendorCmd::InGain(_, g) = &op.cmd {
                        *gain = *g
                    }
                })
            })?;

        params
            .polarities
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, polarity)| {
                let mut op = ApogeeCmd::new(VendorCmd::MicPolarity(i, Default::default()));
                avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
                    if let VendorCmd::MicPolarity(_, enabled) = &op.cmd {
                        *polarity = *enabled
                    }
                })
            })?;

        params
            .xlr_nominal_levels
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, level)| {
                let mut op = ApogeeCmd::new(VendorCmd::XlrIsMicLevel(i, Default::default()));
                avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
                let is_mic_level = match &op.cmd {
                    VendorCmd::XlrIsMicLevel(_, enabled) => *enabled,
                    _ => unreachable!(),
                };

                let mut op = ApogeeCmd::new(VendorCmd::XlrIsConsumerLevel(i, Default::default()));
                avc.status(&AvcAddr::Unit, &mut op, timeout_ms)
                    .map(|_| {
                        let is_consumer_level = match &op.cmd {
                            VendorCmd::XlrIsConsumerLevel(_, enabled) => *enabled,
                            _ => unreachable!(),
                        };

                        *level = if is_mic_level {
                            DuetFwInputXlrNominalLevel::Microphone
                        } else {
                            if is_consumer_level {
                                DuetFwInputXlrNominalLevel::Consumer
                            } else {
                                DuetFwInputXlrNominalLevel::Professional
                            }
                        };
                    })
            })?;

        params
            .phantom_powerings
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, phantom)| {
                let mut op = ApogeeCmd::new(VendorCmd::MicPhantom(i, Default::default()));
                avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
                    if let VendorCmd::MicPhantom(_, e) = &op.cmd {
                        *phantom = *e
                    }
                })
            })?;

        params
            .srcs
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, src)| {
                let mut op = ApogeeCmd::new(VendorCmd::InputSourceIsPhone(i, Default::default()));
                avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
                    if let VendorCmd::InputSourceIsPhone(_, enabled) = &op.cmd {
                        *src = if *enabled {
                            DuetFwInputSource::Phone
                        } else {
                            DuetFwInputSource::Xlr
                        };
                    }
                })
            })?;

        let mut op = ApogeeCmd::new(VendorCmd::InClickless(Default::default()));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            if let VendorCmd::InClickless(e) = &op.cmd {
                params.clickless = *e
            }
        })?;

        Ok(())
    }

    pub fn write_gain(
        avc: &mut OxfwAvc,
        idx: usize,
        gain: u8,
        params: &mut DuetFwInputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if params.srcs[idx] == DuetFwInputSource::Xlr
            && params.xlr_nominal_levels[idx] != DuetFwInputXlrNominalLevel::Microphone
        {
            let msg = format!("Gain is not adjustable for line level of XLR.");
            Err(Error::new(FileError::Inval, &msg))
        } else {
            let mut op = ApogeeCmd::new(VendorCmd::InGain(idx, gain));
            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
                .map(|_| params.gains[idx] = gain)
        }
    }

    pub fn write_polarity(
        avc: &mut OxfwAvc,
        idx: usize,
        polarity: bool,
        params: &mut DuetFwInputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::MicPolarity(idx, polarity));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
            .map(|_| params.polarities[idx] = polarity)
    }

    pub fn write_xlr_nominal_level(
        avc: &mut OxfwAvc,
        idx: usize,
        level: DuetFwInputXlrNominalLevel,
        params: &mut DuetFwInputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let (is_mic_level, is_consumer_level) = match level {
            DuetFwInputXlrNominalLevel::Consumer => (false, true),
            DuetFwInputXlrNominalLevel::Professional => (false, false),
            DuetFwInputXlrNominalLevel::Microphone => (true, true),
        };

        let mut op = ApogeeCmd::new(VendorCmd::XlrIsMicLevel(idx, is_mic_level));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
        let mut op = ApogeeCmd::new(VendorCmd::XlrIsConsumerLevel(idx, is_consumer_level));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
        params.xlr_nominal_levels[idx] = level;
        Ok(())
    }

    pub fn write_phantom_powering(
        avc: &mut OxfwAvc,
        idx: usize,
        enable: bool,
        params: &mut DuetFwInputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if params.xlr_nominal_levels[idx] != DuetFwInputXlrNominalLevel::Microphone {
            let msg = "Phantom powering is available for microphone nominal level";
            Err(Error::new(FileError::Inval, &msg))
        } else {
            let mut op = ApogeeCmd::new(VendorCmd::MicPhantom(idx, enable));
            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
                .map(|_| params.phantom_powerings[idx] = enable)
        }
    }

    pub fn write_src(
        avc: &mut OxfwAvc,
        idx: usize,
        src: DuetFwInputSource,
        params: &mut DuetFwInputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let is_phone = src == DuetFwInputSource::Phone;
        let mut op = ApogeeCmd::new(VendorCmd::InputSourceIsPhone(idx, is_phone));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
            .map(|_| params.srcs[idx] = src)
    }

    pub fn write_clickless(
        avc: &mut OxfwAvc,
        enable: bool,
        params: &mut DuetFwInputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::InClickless(enable));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
            .map(|_| params.clickless = enable)
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
        avc: &mut OxfwAvc,
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
        avc: &mut OxfwAvc,
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
        avc: &mut OxfwAvc,
        target: &mut DuetFwDisplayTarget,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::DisplayIsInput(Default::default()));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            if let VendorCmd::DisplayIsInput(enabled) = &op.cmd {
                *target = if *enabled {
                    DuetFwDisplayTarget::Input
                } else {
                    DuetFwDisplayTarget::Output
                };
            }
        })
    }

    pub fn write_target(
        avc: &mut OxfwAvc,
        target: DuetFwDisplayTarget,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let is_input = target == DuetFwDisplayTarget::Input;
        let mut op = ApogeeCmd::new(VendorCmd::DisplayIsInput(is_input));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_mode(
        avc: &mut OxfwAvc,
        mode: &mut DuetFwDisplayMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::DisplayFollowToKnob(Default::default()));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            if let VendorCmd::DisplayFollowToKnob(enabled) = &op.cmd {
                *mode = if *enabled {
                    DuetFwDisplayMode::FollowingToKnobTarget
                } else {
                    DuetFwDisplayMode::Independent
                }
            }
        })
    }

    pub fn write_mode(
        avc: &mut OxfwAvc,
        mode: DuetFwDisplayMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let enable = mode == DuetFwDisplayMode::FollowingToKnobTarget;
        let mut op = ApogeeCmd::new(VendorCmd::DisplayFollowToKnob(enable));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    pub fn read_overhold(
        avc: &mut OxfwAvc,
        mode: &mut DuetFwDisplayOverhold,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = ApogeeCmd::new(VendorCmd::DisplayOverholdTwoSec(Default::default()));
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            if let VendorCmd::DisplayOverholdTwoSec(enabled) = &op.cmd {
                *mode = if *enabled {
                    DuetFwDisplayOverhold::TwoSeconds
                } else {
                    DuetFwDisplayOverhold::Infinite
                };
            }
        })
    }

    pub fn write_overhold(
        avc: &mut OxfwAvc,
        mode: DuetFwDisplayOverhold,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let enable = mode == DuetFwDisplayOverhold::TwoSeconds;
        let mut op = ApogeeCmd::new(VendorCmd::DisplayOverholdTwoSec(enable));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }
}

/// The enumeration to represent type of command for Apogee Duet FireWire.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
// Usually 5 params.
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

/// The structure to represent protocol of Apogee Duet FireWire.
pub struct ApogeeCmd {
    cmd: VendorCmd,
    op: VendorDependent,
}

impl ApogeeCmd {
    pub fn new(cmd: VendorCmd) -> Self {
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
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        let mut data = self.cmd.build_args();
        self.cmd.append_variable(&mut data);
        self.op.data = data;
        AvcControl::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcControl::parse_operands(&mut self.op, addr, operands)
    }
}

impl AvcStatus for ApogeeCmd {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        self.op.data = self.cmd.build_args();
        AvcStatus::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcStatus::parse_operands(&mut self.op, addr, operands)?;
        self.cmd.parse_variable(&self.op.data)
    }
}

#[cfg(test)]
mod test {
    use super::*;

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

        let mut o = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands[..9]);

        let mut op = ApogeeCmd::new(VendorCmd::OutSourceIsMixer(Default::default()));
        let operands = [0x00, 0x03, 0xdb, 0x50, 0x43, 0x4d, 0x11, 0xff, 0xff, 0x60];
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();

        let mut o = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
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

        let mut o = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
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

        let mut o = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
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

        let mut o = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands[..9]);

        let mut op = ApogeeCmd::new(VendorCmd::MixerSrc(1, 0, 0xde00));
        let operands = [
            0x00, 0x03, 0xdb, 0x50, 0x43, 0x4d, 0x10, 0x01, 0x00, 0xde, 0x00, 0xad, 0x01, 0xbe,
            0x02, 0xef,
        ];
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();

        let mut o = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
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

        let mut o = Vec::new();
        AvcStatus::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(o, operands[..9]);
    }
}
