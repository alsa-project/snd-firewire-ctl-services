// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Desktop Konnekt 6.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Desktop Konnekt 6.
//!
//! ## Diagram of internal signal flow
//!
//! ```text
//!                  +-------+
//! XLR input -----> |       |
//!                  | input | -> analog input-1/2 -----------> stream-output-1/2
//! Phone input 1 -> | scene |          |          (unused) --> stream-output-3/4
//! Phone input 2 -> |       |          |          (unused) --> stream-output-5/6
//!                  +-------+          v
//!                                ++=======++
//!                                || 4 x 2 ||
//!                         +----> ||       || -----+---------> analog-output-1/2 (main)
//!                         |      || mixer ||      |
//!                         |      ++=======++      |
//! stream-input-1/2 -------+                       |
//! stream-input-3/4 -------------------------------or--------> analog-output-3/4 (headphone)
//! stream-input-5/6 --> (unused)
//! ```
//!
//! Reverb effect is not implemented in hardware, while control items are in hardware surface.

use super::tcelectronic::*;

const DESKTOP_HW_STATE_NOTIFY_FLAG: u32 = 0x00010000;
const DESKTOP_CONFIG_NOTIFY_FLAG: u32 = 0x00020000;
const DESKTOP_MIXER_STATE_NOTIFY_FLAG: u32 = 0x00040000;
const DESKTOP_PANEL_NOTIFY_FLAG: u32 = 0x00080000;

/// Protocol implementation of Desktop Konnekt 6.
#[derive(Default, Debug)]
pub struct Desktopk6Protocol;

impl TcatOperation for Desktopk6Protocol {}

impl TcatGlobalSectionSpecification for Desktopk6Protocol {}

/// Segment for panel. 0x0008..0x0097 (36 quads).
pub type Desktopk6HwStateSegment = TcKonnektSegment<DesktopHwState>;

/// Segment for configuration. 0x0098..0x00b7 (8 quads).
pub type Desktopk6ConfigSegment = TcKonnektSegment<DesktopConfig>;

/// Segment for state of mixer. 0x00b8..0x0367 (172 quads).
pub type Desktopk6MixerStateSegment = TcKonnektSegment<DesktopMixerState>;

/// Segment for panel. 0x2008..0x2047 (15 quads).
pub type Desktopk6PanelSegment = TcKonnektSegment<DesktopPanel>;

/// Segment for meter. 0x20e4..0x213f (23 quads).
pub type Desktopk6MeterSegment = TcKonnektSegment<DesktopMeter>;

macro_rules! segment_default {
    ($p:ty, $t:ty) => {
        impl Default for TcKonnektSegment<$t> {
            fn default() -> Self {
                Self {
                    data: <$t>::default(),
                    raw: vec![0; <$p as TcKonnektSegmentSerdes<$t>>::SIZE],
                }
            }
        }
    };
}

segment_default!(Desktopk6Protocol, DesktopHwState);
segment_default!(Desktopk6Protocol, DesktopConfig);
segment_default!(Desktopk6Protocol, DesktopMixerState);
segment_default!(Desktopk6Protocol, DesktopPanel);
segment_default!(Desktopk6Protocol, DesktopMeter);

/// Target of meter.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MeterTarget {
    /// Analog input 1/2.
    Input,
    /// Mixer output 1/2 before volume adjustment.
    Pre,
    /// Mixer output 1/2 after volume adjustment.
    Post,
}

impl Default for MeterTarget {
    fn default() -> Self {
        Self::Input
    }
}

const METER_TARGETS: &[MeterTarget] = &[MeterTarget::Input, MeterTarget::Pre, MeterTarget::Post];

const METER_TARGET_LABEL: &str = "meter-target";

fn serialize_meter_target(target: &MeterTarget, raw: &mut [u8]) -> Result<(), String> {
    serialize_position(METER_TARGETS, target, raw, METER_TARGET_LABEL)
}

fn deserialize_meter_target(target: &mut MeterTarget, raw: &[u8]) -> Result<(), String> {
    deserialize_position(METER_TARGETS, target, raw, METER_TARGET_LABEL)
}

/// Current scene for analog input 1/2.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InputScene {
    /// Microphone and instrument.
    MicInst,
    /// Two instruments.
    DualInst,
    /// Two line inputs for stereo.
    StereoIn,
}

impl Default for InputScene {
    fn default() -> Self {
        Self::MicInst
    }
}

const INPUT_SCENES: &[InputScene] = &[
    InputScene::MicInst,
    InputScene::DualInst,
    InputScene::StereoIn,
];

const INPUT_SCENE_LABEL: &str = "inputscene";

fn serialize_input_scene(scene: &InputScene, raw: &mut [u8]) -> Result<(), String> {
    serialize_position(INPUT_SCENES, scene, raw, INPUT_SCENE_LABEL)
}

fn deserialize_input_scene(scene: &mut InputScene, raw: &[u8]) -> Result<(), String> {
    deserialize_position(INPUT_SCENES, scene, raw, INPUT_SCENE_LABEL)
}

/// General state of hardware.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DesktopHwState {
    /// The target of meter in surface.
    pub meter_target: MeterTarget,
    /// Use mixer output as monaural.
    pub mixer_output_monaural: bool,
    /// Whether to adjust volume of headphone output by main knob.
    pub knob_assign_to_hp: bool,
    /// Whether to dim main output.
    pub mixer_output_dim_enabled: bool,
    /// The volume of main output if dimmed between -1000..-60. (-94.0..-6.0 dB)
    pub mixer_output_dim_volume: i32,
    /// The use case of analog input 1/2.
    pub input_scene: InputScene,
    /// Multiplex reverb signal to stream input 1/2 in advance.
    pub reverb_to_master: bool,
    /// Multiplex reverb signal to stream input 3/4 in advance.
    pub reverb_to_hp: bool,
    /// Turn on backlight in master knob.
    pub master_knob_backlight: bool,
    /// Phantom powering in microphone 1.
    pub mic_0_phantom: bool,
    /// Signal boost in microphone 1 by 12 dB.
    pub mic_0_boost: bool,
}

impl DesktopHwState {
    const REVERB_TO_MAIN_MASK: u32 = 0x00000001;
    const REVERB_TO_HP_MASK: u32 = 0x00000002;
}

impl TcKonnektSegmentSerdes<DesktopHwState> for Desktopk6Protocol {
    const NAME: &'static str = "hardware-state";
    const OFFSET: usize = 0x0008;
    const SIZE: usize = 144;

    fn serialize(params: &DesktopHwState, raw: &mut [u8]) -> Result<(), String> {
        serialize_meter_target(&params.meter_target, &mut raw[..4])?;
        params.mixer_output_monaural.build_quadlet(&mut raw[4..8]);
        params.knob_assign_to_hp.build_quadlet(&mut raw[8..12]);
        params
            .mixer_output_dim_enabled
            .build_quadlet(&mut raw[12..16]);
        params
            .mixer_output_dim_volume
            .build_quadlet(&mut raw[16..20]);
        serialize_input_scene(&params.input_scene, &mut raw[20..24])?;

        let mut val = 0;
        if params.reverb_to_master {
            val |= DesktopHwState::REVERB_TO_MAIN_MASK;
        }
        if params.reverb_to_hp {
            val |= DesktopHwState::REVERB_TO_HP_MASK;
        }
        val.build_quadlet(&mut raw[28..32]);

        params.master_knob_backlight.build_quadlet(&mut raw[32..36]);
        params.mic_0_phantom.build_quadlet(&mut raw[52..56]);
        params.mic_0_boost.build_quadlet(&mut raw[56..60]);

        Ok(())
    }

    fn deserialize(params: &mut DesktopHwState, raw: &[u8]) -> Result<(), String> {
        deserialize_meter_target(&mut params.meter_target, &raw[..4])?;
        params.mixer_output_monaural.parse_quadlet(&raw[4..8]);
        params.knob_assign_to_hp.parse_quadlet(&raw[8..12]);
        params.mixer_output_dim_enabled.parse_quadlet(&raw[12..16]);
        params.mixer_output_dim_volume.parse_quadlet(&raw[16..20]);
        deserialize_input_scene(&mut params.input_scene, &raw[20..24])?;

        let mut val = 0;
        val.parse_quadlet(&raw[28..32]);
        params.reverb_to_master = val & DesktopHwState::REVERB_TO_MAIN_MASK > 0;
        params.reverb_to_hp = val & DesktopHwState::REVERB_TO_HP_MASK > 0;

        params.master_knob_backlight.parse_quadlet(&raw[32..36]);
        params.mic_0_phantom.parse_quadlet(&raw[52..56]);
        params.mic_0_boost.parse_quadlet(&raw[56..60]);

        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<DesktopHwState> for Desktopk6Protocol {}

impl TcKonnektNotifiedSegmentOperation<DesktopHwState> for Desktopk6Protocol {
    const NOTIFY_FLAG: u32 = DESKTOP_HW_STATE_NOTIFY_FLAG;
}

/// Configuration.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DesktopConfig {
    /// Sampling rate at standalone mode.
    pub standalone_rate: TcKonnektStandaloneClockRate,
}

impl TcKonnektSegmentSerdes<DesktopConfig> for Desktopk6Protocol {
    const NAME: &'static str = "configuration";
    const OFFSET: usize = 0x0098;
    const SIZE: usize = 32;

    fn serialize(params: &DesktopConfig, raw: &mut [u8]) -> Result<(), String> {
        serialize_standalone_clock_rate(&params.standalone_rate, &mut raw[4..8])
    }

    fn deserialize(params: &mut DesktopConfig, raw: &[u8]) -> Result<(), String> {
        deserialize_standalone_clock_rate(&mut params.standalone_rate, &raw[4..8])
    }
}

impl TcKonnektMutableSegmentOperation<DesktopConfig> for Desktopk6Protocol {}

impl TcKonnektNotifiedSegmentOperation<DesktopConfig> for Desktopk6Protocol {
    const NOTIFY_FLAG: u32 = DESKTOP_CONFIG_NOTIFY_FLAG;
}

/// Source of headphone.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DesktopHpSrc {
    /// Stream input 3/4.
    Stream23,
    /// Mixer output 1/2.
    Mixer01,
}

impl Default for DesktopHpSrc {
    fn default() -> Self {
        Self::Stream23
    }
}

impl From<u32> for DesktopHpSrc {
    fn from(val: u32) -> Self {
        match val {
            0x05 => Self::Stream23,
            _ => Self::Mixer01,
        }
    }
}

impl From<DesktopHpSrc> for u32 {
    fn from(src: DesktopHpSrc) -> Self {
        match src {
            DesktopHpSrc::Stream23 => 0x05,
            DesktopHpSrc::Mixer01 => 0x0b,
        }
    }
}

/// State of mixer.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DesktopMixerState {
    /// The input level of microphone 1 and phone 1 for instrument, between -1000 and 0 (-94.0 and
    /// 0.0 dB)
    pub mic_inst_level: [i32; 2],
    /// The LR balance of microphone 1 and phone 1 for instrument, between -50 and 50.
    pub mic_inst_pan: [i32; 2],
    /// The level to send from microphone 1 and phone 1 for instrument, between -1000 and 0 (-94.0
    /// and 0.0 dB)
    pub mic_inst_send: [i32; 2],
    /// The input level of both phone 1 and 2 for instrument, between -1000 and 0 (-94.0 and
    /// 0.0 dB)
    pub dual_inst_level: [i32; 2],
    /// The LR balance  of both phone 1 and 2 for instrument, between -50 and 50.
    pub dual_inst_pan: [i32; 2],
    /// The level to send from both phone 1 and 2 for instrument, between -1000 and 0 (-94.0 and
    /// 0.0 dB)
    pub dual_inst_send: [i32; 2],
    /// The input level of both phone 1 and 2 for line, between -1000 and 0 (-94.0 and 0.0 dB)
    pub stereo_in_level: i32,
    /// The LR balance of both phone 1 and 2 for line, between -50 and 50.
    pub stereo_in_pan: i32,
    /// The level to send of both phone 1 and 2 for line, between -1000 and 0 (-94.0 and 0.0 dB)
    pub stereo_in_send: i32,
    /// The source of headphone output.
    pub hp_src: DesktopHpSrc,
}

impl TcKonnektSegmentSerdes<DesktopMixerState> for Desktopk6Protocol {
    const NAME: &'static str = "mixer-state";
    const OFFSET: usize = 0x00b8;
    const SIZE: usize = 688;

    fn serialize(params: &DesktopMixerState, raw: &mut [u8]) -> Result<(), String> {
        params.mic_inst_level[0].build_quadlet(&mut raw[12..16]);
        params.mic_inst_pan[0].build_quadlet(&mut raw[16..20]);
        params.mic_inst_send[0].build_quadlet(&mut raw[20..24]);
        params.mic_inst_level[1].build_quadlet(&mut raw[28..32]);
        params.mic_inst_pan[1].build_quadlet(&mut raw[32..36]);
        params.mic_inst_send[1].build_quadlet(&mut raw[40..44]);

        params.dual_inst_level[0].build_quadlet(&mut raw[228..232]);
        params.dual_inst_pan[0].build_quadlet(&mut raw[232..236]);
        params.dual_inst_send[0].build_quadlet(&mut raw[240..244]);
        params.dual_inst_level[1].build_quadlet(&mut raw[248..252]);
        params.dual_inst_pan[1].build_quadlet(&mut raw[252..256]);
        params.dual_inst_send[1].build_quadlet(&mut raw[260..264]);

        params.stereo_in_level.build_quadlet(&mut raw[444..448]);
        params.stereo_in_pan.build_quadlet(&mut raw[448..452]);
        params.stereo_in_send.build_quadlet(&mut raw[452..456]);

        params.hp_src.build_quadlet(&mut raw[648..652]);
        Ok(())
    }

    fn deserialize(params: &mut DesktopMixerState, raw: &[u8]) -> Result<(), String> {
        params.mic_inst_level[0].parse_quadlet(&raw[12..16]);
        params.mic_inst_pan[0].parse_quadlet(&raw[16..20]);
        params.mic_inst_send[0].parse_quadlet(&raw[20..24]);
        params.mic_inst_level[1].parse_quadlet(&raw[28..32]);
        params.mic_inst_pan[1].parse_quadlet(&raw[32..36]);
        params.mic_inst_send[1].parse_quadlet(&raw[40..44]);

        params.dual_inst_level[0].parse_quadlet(&raw[228..232]);
        params.dual_inst_pan[0].parse_quadlet(&raw[232..236]);
        params.dual_inst_send[0].parse_quadlet(&raw[240..244]);
        params.dual_inst_level[1].parse_quadlet(&raw[248..252]);
        params.dual_inst_pan[1].parse_quadlet(&raw[252..256]);
        params.dual_inst_send[1].parse_quadlet(&raw[260..264]);

        params.stereo_in_level.parse_quadlet(&raw[444..448]);
        params.stereo_in_pan.parse_quadlet(&raw[448..452]);
        params.stereo_in_send.parse_quadlet(&raw[452..456]);

        params.hp_src.parse_quadlet(&raw[648..652]);
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<DesktopMixerState> for Desktopk6Protocol {}

impl TcKonnektNotifiedSegmentOperation<DesktopMixerState> for Desktopk6Protocol {
    const NOTIFY_FLAG: u32 = DESKTOP_MIXER_STATE_NOTIFY_FLAG;
}

/// Panel in hardware surface.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DesktopPanel {
    /// The count of panel button to push.
    pub panel_button_count: u32,
    /// The value of main knob, between -1000 and 0.
    pub main_knob_value: i32,
    /// The value of phone knob, between -1000 and 0.
    pub phone_knob_value: i32,
    /// The value of mix knob, between 0 and 1000.
    pub mix_knob_value: u32,
    /// The state of reverb LED.
    pub reverb_led_on: bool,
    /// The value of reverb knob, between -1000 and 0.
    pub reverb_knob_value: i32,
    /// The state of FireWire LED.
    pub firewire_led: FireWireLedState,
}

impl TcKonnektSegmentSerdes<DesktopPanel> for Desktopk6Protocol {
    const NAME: &'static str = "hardware-panel";
    const OFFSET: usize = 0x2008;
    const SIZE: usize = 64;

    fn serialize(params: &DesktopPanel, raw: &mut [u8]) -> Result<(), String> {
        params.panel_button_count.build_quadlet(&mut raw[..4]);
        params.main_knob_value.build_quadlet(&mut raw[4..8]);
        params.phone_knob_value.build_quadlet(&mut raw[8..12]);
        params.mix_knob_value.build_quadlet(&mut raw[12..16]);
        params.reverb_led_on.build_quadlet(&mut raw[16..20]);
        params.reverb_knob_value.build_quadlet(&mut raw[24..28]);
        serialize_fw_led_state(&params.firewire_led, &mut raw[36..40])?;
        Ok(())
    }

    fn deserialize(params: &mut DesktopPanel, raw: &[u8]) -> Result<(), String> {
        params.panel_button_count.parse_quadlet(&raw[..4]);
        params.main_knob_value.parse_quadlet(&raw[4..8]);
        params.phone_knob_value.parse_quadlet(&raw[8..12]);
        params.mix_knob_value.parse_quadlet(&raw[12..16]);
        params.reverb_led_on.parse_quadlet(&raw[16..20]);
        params.reverb_knob_value.parse_quadlet(&raw[24..28]);
        deserialize_fw_led_state(&mut params.firewire_led, &raw[36..40])?;
        Ok(())
    }
}

impl TcKonnektMutableSegmentOperation<DesktopPanel> for Desktopk6Protocol {}

impl TcKonnektNotifiedSegmentOperation<DesktopPanel> for Desktopk6Protocol {
    const NOTIFY_FLAG: u32 = DESKTOP_PANEL_NOTIFY_FLAG;
}

/// Hardware metering.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DesktopMeter {
    pub analog_inputs: [i32; 2],
    pub mixer_outputs: [i32; 2],
    pub stream_inputs: [i32; 2],
}

impl TcKonnektSegmentSerdes<DesktopMeter> for Desktopk6Protocol {
    const NAME: &'static str = "hardware-meter";
    const OFFSET: usize = 0x20e4;
    const SIZE: usize = 92;

    fn serialize(params: &DesktopMeter, raw: &mut [u8]) -> Result<(), String> {
        params.analog_inputs.build_quadlet_block(&mut raw[..8]);
        params.mixer_outputs.build_quadlet_block(&mut raw[40..48]);
        params.stream_inputs.build_quadlet_block(&mut raw[48..56]);
        Ok(())
    }

    fn deserialize(params: &mut DesktopMeter, raw: &[u8]) -> Result<(), String> {
        params.analog_inputs.parse_quadlet_block(&raw[..8]);
        params.mixer_outputs.parse_quadlet_block(&raw[40..48]);
        params.stream_inputs.parse_quadlet_block(&raw[48..56]);
        Ok(())
    }
}
