// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Hardware specification and application protocol specific to Avid Mbox 3 Pro
//!
//! The module includes structure, enumeration, and trait and its implementation for hardware
//! specification and application protocol specific to Avid Mbox 3 Pro.

use glib::{Error, FileError};

use hinawa::{FwReq, FwNode};

use super::tcat::*;
use super::tcat::extension::{*, appl_section::*};
use super::tcat::tcd22xx_spec::*;

/// The structure to represent state of TCD22xx on Mbox 3 Pro.
#[derive(Default, Debug)]
pub struct Mbox3State(Tcd22xxState);

impl Tcd22xxSpec for  Mbox3State {
    const INPUTS: &'static [Input] = &[
        Input{id: SrcBlkId::Ins0, offset: 0, count: 6, label: None},
        Input{id: SrcBlkId::Ins1, offset: 0, count: 2, label: Some("Reverb")},
        Input{id: SrcBlkId::Aes,  offset: 0, count: 2, label: None},
    ];
    const OUTPUTS: &'static [Output] = &[
        Output{id: DstBlkId::Ins0, offset: 0, count: 6, label: None},
        Output{id: DstBlkId::Ins1, offset: 0, count: 4, label: Some("Headphone")},
        Output{id: DstBlkId::Ins1, offset: 4, count: 2, label: Some("Reverb")},
        Output{id: DstBlkId::Aes,  offset: 0, count: 2, label: None},
        Output{id: DstBlkId::Reserved(0x08), offset: 0, count: 2, label: Some("ControlRoom")},
    ];
    const FIXED: &'static [SrcBlk] = &[
        SrcBlk{id: SrcBlkId::Ins0, ch: 0},
        SrcBlk{id: SrcBlkId::Ins0, ch: 1},
        SrcBlk{id: SrcBlkId::Ins0, ch: 2},
        SrcBlk{id: SrcBlkId::Ins0, ch: 3},
    ];
}

impl AsRef<Tcd22xxState> for Mbox3State {
    fn as_ref(&self) -> &Tcd22xxState {
        &self.0
    }
}

impl AsMut<Tcd22xxState> for Mbox3State {
    fn as_mut(&mut self) -> &mut Tcd22xxState {
        &mut self.0
    }
}

/// The enumeration to represent usecase of standalone mode.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StandaloneUseCase {
    Preamp,
    AdDa,
    Mixer,
}

impl Default for StandaloneUseCase {
    fn default() -> Self {
        Self::Preamp
    }
}

impl StandaloneUseCase {
    const MIXER: u32 = 0;
    const AD_DA: u32 = 1;
    const PREAMP: u32 = 2;
}

impl From<u32> for StandaloneUseCase {
    fn from(val: u32) -> Self {
        match val {
            StandaloneUseCase::MIXER => StandaloneUseCase::Mixer,
            StandaloneUseCase::AD_DA => StandaloneUseCase::AdDa,
            _ => StandaloneUseCase::Preamp,
        }
    }
}

impl From<StandaloneUseCase> for u32 {
    fn from(use_case: StandaloneUseCase) -> u32 {
        match use_case {
            StandaloneUseCase::Mixer => StandaloneUseCase::MIXER,
            StandaloneUseCase::AdDa => StandaloneUseCase::AD_DA,
            StandaloneUseCase::Preamp => StandaloneUseCase::PREAMP,
        }
    }
}

/// The trait and its implementation to represent standalone protocol for Avid Mbox 3 pro.
pub trait AvidMbox3StandaloneProtocol<T> : ApplSectionProtocol<T>
    where T: AsRef<FwNode>,
{
    const USE_CASE_OFFSET: usize = 0x00;

    fn read_standalone_use_case(&self, node: &T, sections: &ExtensionSections, timeout_ms: u32)
        -> Result<StandaloneUseCase, Error>
    {
        let mut data = [0;4];
        self.read_appl_data(node, sections, Self::USE_CASE_OFFSET, &mut data, timeout_ms)
            .map(|_| u32::from_be_bytes(data).into())
    }

    fn write_standalone_use_case(&self, node: &T, sections: &ExtensionSections,
                                 use_case: StandaloneUseCase, timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut data = u32::from(use_case).to_be_bytes().clone();
        self.write_appl_data(node, sections, Self::USE_CASE_OFFSET, &mut data, timeout_ms)
    }
}

impl<O: AsRef<FwReq>, T: AsRef<FwNode>> AvidMbox3StandaloneProtocol<T> for O {}

/// The alternative type to represent assignment map for master knob.
pub type MasterKnobAssigns = [bool;6];

/// The enumeration to represent LED state of mute button.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MuteLedState {
    Off,
    Blink,
    On,
}

impl Default for MuteLedState {
    fn default() -> Self {
        Self::Off
    }
}

impl MuteLedState {
    const LED_MASK: u32 = 0x00000003;
    const LED_BLINK_MASK: u32 = 0x00000001;
}

impl From<u32> for MuteLedState {
    fn from(val: u32) -> Self {
        if val & Self::LED_MASK == 0 {
            Self::Off
        } else if val & Self::LED_BLINK_MASK > 0 {
            Self::Blink
        } else {
            Self::On
        }
    }
}

impl From<&MuteLedState> for u32 {
    fn from(state: &MuteLedState) -> Self {
        match state {
            MuteLedState::Off => 0,
            MuteLedState::Blink => MuteLedState::LED_BLINK_MASK,
            MuteLedState::On => MuteLedState::LED_MASK & !MuteLedState::LED_BLINK_MASK,
        }
    }
}

/// The enumeration to represent LED state of mono button.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MonoLedState {
    Off,
    On,
}

impl Default for MonoLedState {
    fn default() -> Self {
        Self::Off
    }
}

impl MonoLedState {
    const LED_MASK: u32 = 0x0000000c;
    const LED_SHIFT: usize = 2;
}

impl From<u32> for MonoLedState {
    fn from(val: u32) -> Self {
        if (val & Self::LED_MASK) >> Self::LED_SHIFT > 0 {
            MonoLedState::On
        } else {
            MonoLedState::Off
        }
    }
}

impl From<&MonoLedState> for u32 {
    fn from(state: &MonoLedState) -> Self {
        match state {
            MonoLedState::On => MonoLedState::LED_MASK,
            MonoLedState::Off => 0,
        }
    }
}

/// The enumeration to represent LED state of spkr button.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SpkrLedState {
    Off,
    Green,
    GreenBlink,
    Red,
    RedBlink,
    Orange,
    OrangeBlink,
}

impl Default for SpkrLedState {
    fn default() -> Self {
        Self::Off
    }
}

impl SpkrLedState {
    const COLOR_MASK: u32 = 0x00000060;
    const COLOR_SHIFT: usize = 5;
    const BLINK_MASK: u32 = 0x00000010;
    const BLINK_SHIFT: u32 = 4;

    const NONE: u32 = 0;
    const GREEN: u32 = 1;
    const RED: u32 = 2;
    const ORANGE: u32 = 3;
}

impl From<u32> for SpkrLedState {
    fn from(val: u32) -> Self {
        let color = (val & Self::COLOR_MASK) >> Self::COLOR_SHIFT;
        let blink = (val & Self::BLINK_MASK) >> Self::BLINK_SHIFT > 0;

        match color {
            Self::NONE => SpkrLedState::Off,
            Self::GREEN => {
                if blink {
                    SpkrLedState::GreenBlink
                } else {
                    SpkrLedState::Green
                }
            }
            Self::RED => {
                if blink {
                    SpkrLedState::RedBlink
                } else {
                    SpkrLedState::Red
                }
            }
            Self::ORANGE => {
                if blink {
                    SpkrLedState::OrangeBlink
                } else {
                    SpkrLedState::Orange
                }
            }
            _ => SpkrLedState::Off,
        }
    }
}

impl From<&SpkrLedState> for u32 {
    fn from(state: &SpkrLedState) -> Self {
        let (color, blink) = match state {
            SpkrLedState::GreenBlink => (SpkrLedState::GREEN, true),
            SpkrLedState::Green => (SpkrLedState::GREEN, false),
            SpkrLedState::RedBlink => (SpkrLedState::RED, true),
            SpkrLedState::Red => (SpkrLedState::RED, false),
            SpkrLedState::OrangeBlink => (SpkrLedState::RED, true),
            SpkrLedState::Orange => (SpkrLedState::ORANGE, false),
            SpkrLedState::Off => (SpkrLedState::NONE, false),
        };

        let mut val = color << SpkrLedState::COLOR_SHIFT;
        if blink {
            val |= SpkrLedState::BLINK_MASK;
        }

        val
    }
}

/// The structure to represent status of buttons.
#[derive(Default, Debug, Copy, Clone,  Eq, PartialEq)]
pub struct ButtonLedState {
    pub mute: MuteLedState,
    pub mono: MonoLedState,
    pub spkr: SpkrLedState,
}

impl ButtonLedState {
    const SIZE: usize = 8;
}

impl From<[u8;ButtonLedState::SIZE]> for ButtonLedState {
    fn from(raw: [u8;ButtonLedState::SIZE]) -> Self {
        let mut quadlet = [0;4];
        quadlet.copy_from_slice(&raw[..4]);
        let val = u32::from_be_bytes(quadlet);
        let mute = MuteLedState::from(val);

        quadlet.copy_from_slice(&raw[4..8]);
        let val = u32::from_be_bytes(quadlet);
        let mono = MonoLedState::from(val);
        let spkr = SpkrLedState::from(val);

        ButtonLedState{mute, mono, spkr}
    }
}

impl From<&ButtonLedState> for [u8;ButtonLedState::SIZE] {
    fn from(state: &ButtonLedState) -> Self {
        let mut data = [0;ButtonLedState::SIZE];

        let val = u32::from(&state.mute);
        data[..4].copy_from_slice(&val.to_be_bytes());

        let val = u32::from(&state.mono) |
                  u32::from(&state.spkr);
        data[4..].copy_from_slice(&val.to_be_bytes());

        data
    }
}

/// The trait and its implementation to represent hardware protocol for Avid Mbox 3 pro.
pub trait AvidMbox3HwProtocol<T> : ApplSectionProtocol<T>
    where T: AsRef<FwNode>,
{
    const MASTER_KNOB_ASSIGN_OFFSET: usize = 0x0c;
    const BUTTON_LED_STATE_OFFSET: usize = 0x10;
    const DIM_LED_USAGE_OFFSET: usize = 0x1c;
    const BUTTON_HOLD_DURATION_OFFSET: usize = 0x20;
    const HPF_ENABLE_OFFSET: usize = 0x24;      // High pass filter for analog inputs.
    const OUT_0_TRIM_OFFSET: usize = 0x28;
    const OUT_1_TRIM_OFFSET: usize = 0x2c;
    const OUT_2_TRIM_OFFSET: usize = 0x30;
    const OUT_3_TRIM_OFFSET: usize = 0x34;
    const OUT_4_TRIM_OFFSET: usize = 0x38;
    const OUT_5_TRIM_OFFSET: usize = 0x3c;

    const OUTPUT_COUNT: usize = 6;

    const OUT_TRIM_OFFSETS: [usize;6] = [
        Self::OUT_0_TRIM_OFFSET, Self::OUT_1_TRIM_OFFSET,
        Self::OUT_2_TRIM_OFFSET, Self::OUT_3_TRIM_OFFSET,
        Self::OUT_4_TRIM_OFFSET, Self::OUT_5_TRIM_OFFSET,
    ];

    fn read_hw_master_knob_assign(&self, node: &T, sections: &ExtensionSections,
                                  assigns: &mut MasterKnobAssigns, timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut data = [0;4];
        self.read_appl_data(node, sections, Self::MASTER_KNOB_ASSIGN_OFFSET, &mut data, timeout_ms)
            .map(|_| {
                let val = u32::from_be_bytes(data);
                assigns.iter_mut()
                    .enumerate()
                    .for_each(|(i, assigned)| *assigned = val & (1 << i) > 0);
                ()
            })
    }

    fn write_hw_master_knob_assign(&self, node: &T, sections: &ExtensionSections,
                                   assigns: &MasterKnobAssigns, timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut data = [0;4];
        let val: u32 = assigns.iter()
            .enumerate()
            .filter(|&(_, &assigned)| assigned)
            .fold(0, |val, (i, _)| val | (1 << i));
        data.copy_from_slice(&val.to_be_bytes());
        self.write_appl_data(node, sections, Self::MASTER_KNOB_ASSIGN_OFFSET, &mut data, timeout_ms)
    }

    fn read_hw_button_led_state(&self, node: &T, sections: &ExtensionSections, timeout_ms: u32)
        -> Result<ButtonLedState, Error>
    {
        let mut data = [0;ButtonLedState::SIZE];
        self.read_appl_data(node, sections, Self::BUTTON_LED_STATE_OFFSET, &mut data, timeout_ms)
            .map(|_| ButtonLedState::from(data))
    }

    fn write_hw_button_led_state(&self, node: &T, sections: &ExtensionSections, state: &ButtonLedState,
                                 timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut data = Into::<[u8;ButtonLedState::SIZE]>::into(state);
        self.write_appl_data(node, sections, Self::BUTTON_LED_STATE_OFFSET, &mut data, timeout_ms)
    }

    fn read_hw_dim_led_usage(&self, node: &T, sections: &ExtensionSections, timeout_ms: u32)
        -> Result<bool, Error>
    {
        let mut data = [0;4];
        self.read_appl_data(node, sections, Self::DIM_LED_USAGE_OFFSET, &mut data, timeout_ms)
            .map(|_| u32::from_be_bytes(data) > 0)
    }

    fn write_hw_dim_led_usage(&self, node: &T, sections: &ExtensionSections, usage: bool, timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut data = (usage as u32).to_be_bytes().clone();
        self.write_appl_data(node, sections, Self::DIM_LED_USAGE_OFFSET, &mut data, timeout_ms)
    }

    fn read_hw_hold_duration(&self, node: &T, sections: &ExtensionSections, timeout_ms: u32)
        -> Result<u8, Error>
    {
        let mut data = [0;4];
        self.read_appl_data(node, sections, Self::BUTTON_HOLD_DURATION_OFFSET, &mut data, timeout_ms)
            .map(|_| u32::from_be_bytes(data) as u8)
    }

    fn write_hw_hold_duration(&self, node: &T, sections: &ExtensionSections, duration: u8, timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut data = (duration as u32).to_be_bytes().clone();
        self.write_appl_data(node, sections, Self::BUTTON_HOLD_DURATION_OFFSET, &mut data, timeout_ms)
    }

    fn read_hw_hpf_enable(&self, node: &T, sections: &ExtensionSections, inputs: &mut [bool;4], timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut data = [0;4];
        self.read_appl_data(node, sections, Self::HPF_ENABLE_OFFSET, &mut data, timeout_ms)
            .map(|_| {
                let val = u32::from_be_bytes(data);
                inputs.iter_mut().enumerate().for_each(|(i, v)| *v = val & (1 << i) > 0);
            })
    }

    fn write_hw_hpf_enable(&self, node: &T, sections: &ExtensionSections, inputs: [bool;4], timeout_ms: u32)
        -> Result<(), Error>
    {
        let val = inputs.iter().enumerate().filter(|(_, &v)| v).fold(0 as u32, |val, (i, _)| val | (1 << i));
        self.write_appl_data(node, sections, Self::HPF_ENABLE_OFFSET, &mut val.to_be_bytes().clone(),
                             timeout_ms)
    }

    fn read_hw_output_trim(&self, node: &T, sections: &ExtensionSections, idx: usize, timeout_ms: u32)
        -> Result<u8, Error>
    {
        Self::OUT_TRIM_OFFSETS.iter()
            .nth(idx)
            .ok_or_else(|| {
                let msg = format!("Invalid value for index of output: {} greater than {}",
                                  idx, Self::OUT_TRIM_OFFSETS.len());
                Error::new(FileError::Inval, &msg)
            })
            .and_then(|&offset| {
                let mut data = [0;4];
                self.read_appl_data(node, sections, offset, &mut data, timeout_ms)
                    .map(|_| u32::from_be_bytes(data) as u8)
            })
    }

    fn write_hw_output_trim(&self, node: &T, sections: &ExtensionSections, idx: usize, trim: u8,
                            timeout_ms: u32)
        -> Result<(), Error>
    {
        Self::OUT_TRIM_OFFSETS.iter()
            .nth(idx)
            .ok_or_else(|| {
                let msg = format!("Invalid value for index of output: {} greater than {}",
                                  idx, Self::OUT_TRIM_OFFSETS.len());
                Error::new(FileError::Inval, &msg)
            })
            .and_then(|&offset| {
                let mut data = [0;4];
                data.copy_from_slice(&(trim as u32).to_be_bytes());
                self.write_appl_data(node, sections, offset, &mut data, timeout_ms)
            })
    }
}

impl<O: AsRef<FwReq>, T: AsRef<FwNode>> AvidMbox3HwProtocol<T> for O {}

/// The enumration to represent type of reverb DSP.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ReverbType {
    Room1,
    Room2,
    Room3,
    Hall1,
    Hall2,
    Plate,
    Delay,
    Echo,
}

impl Default for ReverbType {
    fn default() -> Self {
        Self::Room1
    }
}

impl ReverbType {
    const ROOM_1: u32 = 0x01;
    const ROOM_2: u32 = 0x04;
    const ROOM_3: u32 = 0x05;
    const HALL_1: u32 = 0x06;
    const HALL_2: u32 = 0x08;
    const PLATE: u32 = 0x0b;
    const DELAY: u32 = 0x13;
    const ECHO: u32 = 0x14;
}

impl From<u32> for ReverbType {
    fn from(val: u32) -> Self {
        match val {
            ReverbType::ROOM_1 => ReverbType::Room1,
            ReverbType::ROOM_2 => ReverbType::Room2,
            ReverbType::ROOM_3 => ReverbType::Room3,
            ReverbType::HALL_1 => ReverbType::Hall1,
            ReverbType::HALL_2 => ReverbType::Hall2,
            ReverbType::PLATE => ReverbType::Plate,
            ReverbType::DELAY => ReverbType::Delay,
            _ => ReverbType::Echo,
        }
    }
}

impl From<ReverbType> for u32 {
    fn from(reverb_type: ReverbType) -> Self {
        match reverb_type {
            ReverbType::Room1 => ReverbType::ROOM_1,
            ReverbType::Room2 => ReverbType::ROOM_2,
            ReverbType::Room3 => ReverbType::ROOM_3,
            ReverbType::Hall1 => ReverbType::HALL_1,
            ReverbType::Hall2 => ReverbType::HALL_2,
            ReverbType::Plate => ReverbType::PLATE,
            ReverbType::Delay => ReverbType::DELAY,
            ReverbType::Echo => ReverbType::ECHO,
        }
    }
}

/// The trait and its implementation to represent reverb protocol for Avid Mbox 3 pro.
pub trait AvidMbox3ReverbProtocol<T> : ApplSectionProtocol<T>
    where T: AsRef<FwNode>,
{
    const REVERB_TYPE_OFFSET: usize = 0x40;
    const REVERB_VOLUME_OFFSET: usize = 0x44;
    const REVERB_DURATION_OFFSET: usize = 0x48;
    const REVERB_FEEDBACK_OFFSET: usize = 0x4c;

    fn read_reverb_type(&self, node: &T, sections: &ExtensionSections, timeout_ms: u32)
        -> Result<ReverbType, Error>
    {
        let mut data = [0;4];
        self.read_appl_data(node, sections, Self::REVERB_TYPE_OFFSET, &mut data, timeout_ms)
            .map(|_| ReverbType::from(u32::from_be_bytes(data)))
    }

    fn write_reverb_type(&self, node: &T, sections: &ExtensionSections, reverb_type: ReverbType,
                         timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut data = u32::from(reverb_type).to_be_bytes().clone();
        self.write_appl_data(node, sections, Self::REVERB_TYPE_OFFSET, &mut data, timeout_ms)
    }

    fn read_reverb_volume(&self, node: &T, sections: &ExtensionSections, timeout_ms: u32)
        -> Result<u8, Error>
    {
        let mut data = [0;4];
        self.read_appl_data(node, sections, Self::REVERB_VOLUME_OFFSET, &mut data, timeout_ms)
            .map(|_| u32::from_be_bytes(data) as u8)
    }

    fn write_reverb_volume(&self, node: &T, sections: &ExtensionSections, volume: u8,
                           timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut data = (volume as u32).to_be_bytes().clone();
        self.write_appl_data(node, sections, Self::REVERB_VOLUME_OFFSET, &mut data, timeout_ms)
    }

    fn read_reverb_duration(&self, node: &T, sections: &ExtensionSections, timeout_ms: u32)
        -> Result<u8, Error>
    {
        let mut data = [0;4];
        self.read_appl_data(node, sections, Self::REVERB_DURATION_OFFSET, &mut data, timeout_ms)
            .map(|_| u32::from_be_bytes(data) as u8)
    }

    fn write_reverb_duration(&self, node: &T, sections: &ExtensionSections, duration: u8,
                             timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut data = (duration as u32).to_be_bytes().clone();
        self.write_appl_data(node, sections, Self::REVERB_DURATION_OFFSET, &mut data, timeout_ms)
    }

    fn read_reverb_feedback(&self, node: &T, sections: &ExtensionSections, timeout_ms: u32)
        -> Result<u8, Error>
    {
        let mut data = [0;4];
        self.read_appl_data(node, sections, Self::REVERB_FEEDBACK_OFFSET, &mut data, timeout_ms)
            .map(|_| u32::from_be_bytes(data) as u8)
    }

    fn write_reverb_feedback(&self, node: &T, sections: &ExtensionSections, feedback: u8,
                              timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut data = (feedback as u32).to_be_bytes().clone();
        self.write_appl_data(node, sections, Self::REVERB_FEEDBACK_OFFSET, &mut data, timeout_ms)
    }
}

impl<O: AsRef<FwReq>, T: AsRef<FwNode>> AvidMbox3ReverbProtocol<T> for O {}

/// The trait and its implementation to parse notification defined by Avid.
pub trait AvidMbox3Notification : TcatNotification {
    const PHANTOM_BUTTON_PUSHED: u32 = 0x10000000;
    const SPKR_BUTTON_PUSHED: u32 = 0x04000000;
    const SPKR_BUTTON_HELD: u32 = 0x02000000;
    const MONO_BUTTON_PUSHED: u32 = 0x00800000;
    const MUTE_BUTTON_PUSHED: u32 = 0x00400000;
    const MUTE_BUTTON_HELD: u32 = 0x00200000;

    fn has_phantom_button_pushed(self) -> bool {
        self.bitand(Self::PHANTOM_BUTTON_PUSHED) > 0
    }

    fn has_spkr_button_pushed(self) -> bool {
        self.bitand(Self::SPKR_BUTTON_PUSHED) > 0
    }

    fn has_spkr_button_held(self) -> bool {
        self.bitand(Self::SPKR_BUTTON_HELD) > 0
    }

    fn has_mono_button_pushed(self) -> bool {
        self.bitand(Self::MONO_BUTTON_PUSHED) > 0
    }

    fn has_mute_button_pushed(self) -> bool {
        self.bitand(Self::MUTE_BUTTON_PUSHED) > 0
    }

    fn has_mute_button_held(self) -> bool {
        self.bitand(Self::MUTE_BUTTON_HELD) > 0
    }
}

impl<O: TcatNotification> AvidMbox3Notification for O {}
