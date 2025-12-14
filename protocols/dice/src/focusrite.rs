// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol specific to Focusrite Saffire series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Focusrite for Saffire series.

pub mod liquids56;
pub mod spro14;
pub mod spro24;
pub mod spro24dsp;
pub mod spro26;
pub mod spro40;
pub mod spro40d3;

use super::{
    tcat::{extension::*, *},
    *,
};

/// Software notice protocol to update hardware parameter.
pub trait SaffireproSwNoticeOperation: TcatExtensionOperation {
    const SW_NOTICE_OFFSET: usize;

    fn write_sw_notice(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        notice: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        serialize_u32(&notice, &mut raw);
        Self::write_extension(
            req,
            node,
            &sections.application,
            Self::SW_NOTICE_OFFSET,
            &mut raw,
            timeout_ms,
        )
    }
}

/// A set of entries for output control. Output volumes corresponding to the entries are
/// controlled by single software/hardware operation if enabled.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct OutGroupState {
    /// Volume of each analog output, between 0x00 and 0x7f.
    pub vols: Vec<i8>,

    /// Whether to mute each analog output.
    pub vol_mutes: Vec<bool>,

    /// Whether to control volume of each analog output by hardware `monitor` knob.
    pub vol_hwctls: Vec<bool>,

    /// Whether mute is enabled or not.
    pub mute_enabled: bool,

    /// Whether to control volume of each analog output by hardware `mute` button.
    pub mute_hwctls: Vec<bool>,

    /// Whether dim is enabled or not.
    pub dim_enabled: bool,

    /// Whether to control volume of each analog output by hardware `dim` button.
    pub dim_hwctls: Vec<bool>,

    /// Current value of hardware `monitor` knob, supported by Liquid Saffire 56 and
    /// Saffire Pro 40.
    pub hw_knob_value: i8,
}

const OUT_GROUP_STATE_SIZE: usize = 0x50;

const VOL_MIN: i8 = 0;
const VOL_MAX: i8 = i8::MAX;

fn serialize_out_group_state(state: &OutGroupState, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= OUT_GROUP_STATE_SIZE);

    serialize_bool(&state.mute_enabled, &mut raw[0x00..0x04]);
    serialize_bool(&state.dim_enabled, &mut raw[0x04..0x08]);

    (0..(state.vols.len() / 2)).for_each(|i| {
        let mut val = 0u32;
        state.vols[(i * 2)..(i * 2 + 2)]
            .iter()
            .enumerate()
            .for_each(|(j, &vol)| {
                // NOTE: inverted.
                let v = VOL_MAX - vol;
                val |= (v as u32) << (8 * j);
            });
        let pos = 0x08 + i * 4;
        serialize_u32(&val, &mut raw[pos..(pos + 4)]);
    });

    (0..(state.vol_hwctls.len() / 2)).for_each(|i| {
        let mut val = 0u32;
        let idx = i * 2;

        state.vol_hwctls[idx..(idx + 2)]
            .iter()
            .enumerate()
            .filter(|(_, &vol_hwctl)| vol_hwctl)
            .for_each(|(i, _)| val |= 1 << i);

        state.vol_mutes[idx..(idx + 2)]
            .iter()
            .enumerate()
            .filter(|(_, &vol_mute)| vol_mute)
            .for_each(|(i, _)| val |= 1 << (i + 2));

        let pos = 0x1c + i * 4;
        serialize_u32(&val, &mut raw[pos..(pos + 4)]);
    });

    let mut val = 0u32;
    state
        .dim_hwctls
        .iter()
        .enumerate()
        .filter(|(_, &assigned)| assigned)
        .for_each(|(i, _)| val |= 1 << (i + 10));
    state
        .mute_hwctls
        .iter()
        .enumerate()
        .filter(|(_, &assigned)| assigned)
        .for_each(|(i, _)| val |= 1 << i);
    serialize_u32(&val, &mut raw[0x30..0x34]);

    serialize_i8(&state.hw_knob_value, &mut raw[0x48..0x4c]);

    Ok(())
}

fn deserialize_out_group_state(state: &mut OutGroupState, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= OUT_GROUP_STATE_SIZE);

    deserialize_bool(&mut state.mute_enabled, &raw[0x00..0x04]);
    deserialize_bool(&mut state.dim_enabled, &raw[0x04..0x08]);

    (0..(state.vols.len() / 2)).for_each(|i| {
        let pos = 0x08 + i * 4;
        let mut val = 0u32;
        deserialize_u32(&mut val, &raw[pos..(pos + 4)]);
        state.vols[(i * 2)..(i * 2 + 2)]
            .iter_mut()
            .enumerate()
            .for_each(|(j, vol)| {
                let v = ((val >> (j * 8)) & 0xff) as i8;
                // NOTE: inverted.
                *vol = VOL_MAX - v;
            });
    });

    (0..(state.vol_hwctls.len() / 2)).for_each(|i| {
        let pos = 0x1c + i * 4;
        let mut val = 0u32;
        deserialize_u32(&mut val, &raw[pos..(pos + 4)]);
        let idx = i * 2;

        state.vol_hwctls[idx..(idx + 2)]
            .iter_mut()
            .enumerate()
            .for_each(|(i, vol_hwctl)| *vol_hwctl = val & (1 << i) > 0);

        state.vol_mutes[idx..(idx + 2)]
            .iter_mut()
            .enumerate()
            .for_each(|(i, vol_mute)| {
                *vol_mute = val & (1 << (i + 2)) > 0;
            });
    });

    let mut val = 0u32;
    deserialize_u32(&mut val, &raw[0x30..0x34]);
    state
        .dim_hwctls
        .iter_mut()
        .enumerate()
        .for_each(|(i, assign)| *assign = val & (1 << (i + 10)) > 0);
    state
        .mute_hwctls
        .iter_mut()
        .enumerate()
        .for_each(|(i, assign)| *assign = val & (1 << i) > 0);

    deserialize_i8(&mut state.hw_knob_value, &raw[0x48..0x4c]);

    Ok(())
}

const NOTIFY_DIM_MUTE_CHANGE: u32 = 0x00200000;
const NOTIFY_VOL_CHANGE: u32 = 0x00400000;

/// Output group operation.
pub trait SaffireproOutGroupSpecification: SaffireproSwNoticeOperation {
    /// Offset of output group state.
    const OUT_GROUP_STATE_OFFSET: usize;

    /// The number of outputs to be controlled.
    const ENTRY_COUNT: usize;

    /// Support hardware knob to control volume.
    const HAS_VOL_HWCTL: bool;

    /// Software notification for source of output group.
    const SRC_NOTICE: u32;

    /// Software notification for dim and mute of output group.
    const DIM_MUTE_NOTICE: u32;

    /// The minimum value of volume.
    const VOL_MIN: i8 = VOL_MIN;

    /// The maximum value of volume.
    const VOL_MAX: i8 = VOL_MAX;

    /// Instantiate structure for output group state.
    fn create_out_group_state() -> OutGroupState {
        OutGroupState {
            vols: vec![0; Self::ENTRY_COUNT],
            vol_mutes: vec![false; Self::ENTRY_COUNT],
            vol_hwctls: vec![false; Self::ENTRY_COUNT],
            mute_enabled: false,
            mute_hwctls: vec![false; Self::ENTRY_COUNT],
            dim_enabled: false,
            dim_hwctls: vec![false; Self::ENTRY_COUNT],
            hw_knob_value: 0,
        }
    }
}

impl<O: TcatExtensionOperation + SaffireproOutGroupSpecification>
    TcatExtensionSectionParamsOperation<OutGroupState> for O
{
    fn cache_extension_whole_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        _: &ExtensionCaps,
        params: &mut OutGroupState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0u8; OUT_GROUP_STATE_SIZE];
        Self::read_extension(
            req,
            node,
            &sections.application,
            O::OUT_GROUP_STATE_OFFSET,
            &mut raw,
            timeout_ms,
        )?;
        deserialize_out_group_state(params, &raw)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }
}

impl<O: TcatExtensionOperation + SaffireproOutGroupSpecification>
    TcatExtensionSectionPartialMutableParamsOperation<OutGroupState> for O
{
    fn update_extension_partial_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        _: &ExtensionCaps,
        params: &OutGroupState,
        prev: &mut OutGroupState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut new = vec![0u8; OUT_GROUP_STATE_SIZE];
        serialize_out_group_state(params, &mut new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        let mut old = vec![0u8; OUT_GROUP_STATE_SIZE];
        serialize_out_group_state(prev, &mut old)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        (0..OUT_GROUP_STATE_SIZE).step_by(4).try_for_each(|pos| {
            if new[pos..(pos + 4)] != old[pos..(pos + 4)] {
                Self::write_extension(
                    req,
                    node,
                    &sections.application,
                    Self::OUT_GROUP_STATE_OFFSET + pos,
                    &mut new[pos..(pos + 4)],
                    timeout_ms,
                )
            } else {
                Ok(())
            }
        })?;

        if new[..0x08] != old[..0x08] {
            Self::write_sw_notice(req, node, sections, Self::DIM_MUTE_NOTICE, timeout_ms)?;
        }

        if new[0x08..0x34] != old[0x08..0x34] {
            Self::write_sw_notice(req, node, sections, Self::SRC_NOTICE, timeout_ms)?;
        }

        deserialize_out_group_state(prev, &new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }
}

impl<O: TcatExtensionOperation + SaffireproOutGroupSpecification>
    TcatExtensionSectionNotifiedParamsOperation<OutGroupState> for O
{
    fn cache_extension_notified_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        params: &mut OutGroupState,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if msg & (NOTIFY_DIM_MUTE_CHANGE | NOTIFY_VOL_CHANGE) > 0 {
            Self::cache_extension_whole_params(req, node, sections, caps, params, timeout_ms)?;

            if msg & NOTIFY_VOL_CHANGE > 0 {
                let vol_hwctls = params.vol_hwctls.clone();
                let hw_knob_value = params.hw_knob_value;
                params
                    .vols
                    .iter_mut()
                    .zip(vol_hwctls)
                    .filter(|(_, vol_hwctl)| *vol_hwctl)
                    .for_each(|(vol, _)| *vol = hw_knob_value);
            }
        }

        Ok(())
    }
}

/// The level of microphone input.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SaffireproMicInputLevel {
    /// Gain range: -10dB to +36 dB.
    Line,
    /// Gain range: +13 to +60 dB, headroom: +8dBu.
    Instrument,
}

impl Default for SaffireproMicInputLevel {
    fn default() -> Self {
        Self::Line
    }
}

/// The level of line input.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SaffireproLineInputLevel {
    /// +16 dBu.
    Low,
    /// -10dBV.
    High,
}

impl Default for SaffireproLineInputLevel {
    fn default() -> Self {
        Self::Low
    }
}

/// Parameters for analog inputs.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SaffireproInputParams {
    /// Nominal level of microphone inputs.
    pub mic_levels: [SaffireproMicInputLevel; 2],
    /// Nominal level of line inputs.
    pub line_levels: [SaffireproLineInputLevel; 2],
}

const MIC_INPUT_LEVEL_INSTRUMENT_FLAG: u16 = 0x0002;
const LINE_INPUT_LEVEL_HIGH_FLAG: u16 = 0x0001;

const INPUT_PARAMS_SIZE: usize = 8;

fn serialize_input_params(params: &SaffireproInputParams, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= INPUT_PARAMS_SIZE);

    let mut val = 0u32;
    params.mic_levels.iter().enumerate().for_each(|(i, level)| {
        if *level == SaffireproMicInputLevel::Instrument {
            val |= (MIC_INPUT_LEVEL_INSTRUMENT_FLAG as u32) << (16 * i);
        };
    });
    serialize_u32(&val, &mut raw[..4]);

    let mut val = 0u32;
    params
        .line_levels
        .iter()
        .enumerate()
        .for_each(|(i, level)| {
            if *level == SaffireproLineInputLevel::High {
                val |= (LINE_INPUT_LEVEL_HIGH_FLAG as u32) << (16 * i);
            }
        });
    serialize_u32(&val, &mut raw[4..8]);

    Ok(())
}

fn deserialize_input_params(params: &mut SaffireproInputParams, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= INPUT_PARAMS_SIZE);

    let mut val = 0u32;

    deserialize_u32(&mut val, &raw[..4]);
    params
        .mic_levels
        .iter_mut()
        .enumerate()
        .for_each(|(i, level)| {
            let flag = (val >> (16 * i)) as u16;
            *level = if flag & MIC_INPUT_LEVEL_INSTRUMENT_FLAG > 0 {
                SaffireproMicInputLevel::Instrument
            } else {
                SaffireproMicInputLevel::Line
            };
        });

    deserialize_u32(&mut val, &raw[4..8]);
    params
        .line_levels
        .iter_mut()
        .enumerate()
        .for_each(|(i, level)| {
            let flag = (val >> (16 * i)) as u16;
            *level = if flag & LINE_INPUT_LEVEL_HIGH_FLAG > 0 {
                SaffireproLineInputLevel::High
            } else {
                SaffireproLineInputLevel::Low
            };
        });

    Ok(())
}

/// Input protocol specific to Pro 14 and Pro 24.
pub trait SaffireproInputSpecification: SaffireproSwNoticeOperation {
    const INPUT_PARAMS_OFFSET: usize;

    const SW_NOTICE: u32 = 0x00000004;

    const MIC_INPUT_COUNT: usize = 2;
    const LINE_INPUT_COUNT: usize = 2;
}

impl<O: TcatExtensionOperation + SaffireproInputSpecification>
    TcatExtensionSectionParamsOperation<SaffireproInputParams> for O
{
    fn cache_extension_whole_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        _: &ExtensionCaps,
        params: &mut SaffireproInputParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0u8; INPUT_PARAMS_SIZE];
        Self::read_extension(
            req,
            node,
            &sections.application,
            O::INPUT_PARAMS_OFFSET,
            &mut raw,
            timeout_ms,
        )?;
        deserialize_input_params(params, &raw)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }
}

impl<O: TcatExtensionOperation + SaffireproInputSpecification>
    TcatExtensionSectionPartialMutableParamsOperation<SaffireproInputParams> for O
{
    fn update_extension_partial_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        _: &ExtensionCaps,
        params: &SaffireproInputParams,
        prev: &mut SaffireproInputParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut new = vec![0u8; INPUT_PARAMS_SIZE];
        serialize_input_params(params, &mut new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        let mut old = vec![0u8; INPUT_PARAMS_SIZE];
        serialize_input_params(prev, &mut old)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        (0..INPUT_PARAMS_SIZE).step_by(4).try_for_each(|pos| {
            if new[pos..(pos + 4)] != old[pos..(pos + 4)] {
                Self::write_extension(
                    req,
                    node,
                    &sections.application,
                    Self::INPUT_PARAMS_OFFSET + pos,
                    &mut new[pos..(pos + 4)],
                    timeout_ms,
                )
            } else {
                Ok(())
            }
        })?;

        if new != old {
            Self::write_sw_notice(req, node, sections, Self::SW_NOTICE, timeout_ms)?;
        }

        deserialize_input_params(prev, &new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }
}

/// Type of signal for optical output interface.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OpticalOutIfaceMode {
    /// For ADAT signal.
    Adat,
    /// For S/PDIF signal.
    Spdif,
    /// For AES/EBU signal.
    AesEbu,
}

impl Default for OpticalOutIfaceMode {
    fn default() -> Self {
        Self::Adat
    }
}

impl OpticalOutIfaceMode {
    const ADAT_VALUE: u32 = 0x00000000;
    const SPDIF_VALUE: u32 = 0x00000001;
    const AESEBU_VALUE: u32 = 0x00000002;
}

const OPTICAL_OUT_IFACE_MODE_MASK: u32 = 0x00000003;
const MIC_AMP_TRANSFORMER_CH0_FLAG: u32 = 0x00000008;
const MIC_AMP_TRANSFORMER_CH1_FLAG: u32 = 0x00000010;

fn serialize_optical_out_iface_mode(
    mode: &OpticalOutIfaceMode,
    aesebu_is_supported: bool,
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    deserialize_u32(&mut val, raw);
    val &= !OPTICAL_OUT_IFACE_MODE_MASK;

    let v = match mode {
        OpticalOutIfaceMode::Adat => OpticalOutIfaceMode::ADAT_VALUE,
        OpticalOutIfaceMode::Spdif => OpticalOutIfaceMode::SPDIF_VALUE,
        OpticalOutIfaceMode::AesEbu => {
            if aesebu_is_supported {
                OpticalOutIfaceMode::AESEBU_VALUE
            } else {
                Err(format!("AES/EBU is not supported but selected"))?
            }
        }
    };
    val |= v;

    serialize_u32(&val, raw);

    Ok(())
}

fn deserialize_optical_out_iface_mode(
    mode: &mut OpticalOutIfaceMode,
    aesebu_is_supported: bool,
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    deserialize_u32(&mut val, raw);
    val &= OPTICAL_OUT_IFACE_MODE_MASK;

    *mode = match val {
        OpticalOutIfaceMode::ADAT_VALUE => OpticalOutIfaceMode::Adat,
        OpticalOutIfaceMode::SPDIF_VALUE => OpticalOutIfaceMode::Spdif,
        OpticalOutIfaceMode::AESEBU_VALUE => {
            if aesebu_is_supported {
                OpticalOutIfaceMode::AesEbu
            } else {
                Err(format!("AES/EBU is not supported but detected"))?
            }
        }
        _ => Err(format!(
            "Optical interface mode not found for value {}",
            val
        ))?,
    };

    Ok(())
}

const MIC_AMP_TRANSFORMER_FLAGS: [u32; 2] =
    [MIC_AMP_TRANSFORMER_CH0_FLAG, MIC_AMP_TRANSFORMER_CH1_FLAG];

fn serialize_mic_amp_transformers(transformers: &[bool; 2], raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    deserialize_u32(&mut val, raw);
    val &= !(MIC_AMP_TRANSFORMER_CH0_FLAG | MIC_AMP_TRANSFORMER_CH1_FLAG);

    transformers
        .iter()
        .zip(MIC_AMP_TRANSFORMER_FLAGS)
        .filter(|(&enabled, _)| enabled)
        .for_each(|(_, flag)| val |= flag);

    serialize_u32(&val, raw);

    Ok(())
}

fn deserialize_mic_amp_transformers(
    transformers: &mut [bool; 2],
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    deserialize_u32(&mut val, raw);
    val &= MIC_AMP_TRANSFORMER_CH0_FLAG | MIC_AMP_TRANSFORMER_CH1_FLAG;

    transformers
        .iter_mut()
        .zip(MIC_AMP_TRANSFORMER_FLAGS)
        .for_each(|(enabled, flag)| *enabled = val & flag > 0);

    Ok(())
}

/// General input/output configuration for Liquid Saffire 56 and Saffire Pro 40.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SaffireproIoParams {
    /// Whether to activate attenuation for analog output 0/1.
    pub analog_out_0_1_pad: bool,
    /// Mode of signal for optical output interface.
    pub opt_out_iface_mode: OpticalOutIfaceMode,
    /// Whether to enable transformer of microhphone amplifiers.
    pub mic_amp_transformers: [bool; 2],
}

const IO_PARAMS_OFFSET: usize = 0x40;
const IO_PARAMS_SIZE: usize = 0x20;

fn serialize_io_params(
    params: &SaffireproIoParams,
    aesebu_is_supported: bool,
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= IO_PARAMS_SIZE);

    serialize_bool(&params.analog_out_0_1_pad, &mut raw[..4]);
    serialize_optical_out_iface_mode(
        &params.opt_out_iface_mode,
        aesebu_is_supported,
        &mut raw[0x1c..0x20],
    )?;
    serialize_mic_amp_transformers(&params.mic_amp_transformers, &mut raw[0x1c..0x20])?;

    Ok(())
}

fn deserialize_io_params(
    params: &mut SaffireproIoParams,
    aesebu_is_supported: bool,
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= IO_PARAMS_SIZE);

    deserialize_bool(&mut params.analog_out_0_1_pad, &raw[..4]);
    deserialize_optical_out_iface_mode(
        &mut params.opt_out_iface_mode,
        aesebu_is_supported,
        &raw[0x1c..0x20],
    )?;
    deserialize_mic_amp_transformers(&mut params.mic_amp_transformers, &raw[0x1c..0x20])?;

    Ok(())
}

/// Operation for parameters of input/output configuration.
pub trait SaffireproIoParamsSpecification: SaffireproSwNoticeOperation {
    /// Whether to support AES/EBU signal in optical interface.
    const AESEBU_IS_SUPPORTED: bool;

    /// Whether to support transformer function for microphone pre-amplifier.
    const MIC_PREAMP_TRANSFORMER_IS_SUPPORTED: bool;
}

impl<O: TcatExtensionOperation + SaffireproIoParamsSpecification>
    TcatExtensionSectionParamsOperation<SaffireproIoParams> for O
{
    fn cache_extension_whole_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        _: &ExtensionCaps,
        params: &mut SaffireproIoParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0u8; IO_PARAMS_SIZE];
        Self::read_extension(
            req,
            node,
            &sections.application,
            IO_PARAMS_OFFSET,
            &mut raw,
            timeout_ms,
        )?;
        deserialize_io_params(params, O::AESEBU_IS_SUPPORTED, &raw)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }
}

impl<O: TcatExtensionOperation + SaffireproIoParamsSpecification>
    TcatExtensionSectionPartialMutableParamsOperation<SaffireproIoParams> for O
{
    fn update_extension_partial_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        _: &ExtensionCaps,
        params: &SaffireproIoParams,
        prev: &mut SaffireproIoParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut new = vec![0u8; IO_PARAMS_SIZE];
        serialize_io_params(params, Self::AESEBU_IS_SUPPORTED, &mut new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        let mut old = vec![0u8; IO_PARAMS_SIZE];
        serialize_io_params(prev, Self::AESEBU_IS_SUPPORTED, &mut old)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        (0..IO_PARAMS_SIZE).step_by(4).try_for_each(|pos| {
            if new[pos..(pos + 4)] != old[pos..(pos + 4)] {
                Self::write_extension(
                    req,
                    node,
                    &sections.application,
                    IO_PARAMS_OFFSET + pos,
                    &mut new[pos..(pos + 4)],
                    timeout_ms,
                )
            } else {
                Ok(())
            }
        })?;

        if new[..0x04] != old[..0x04] {
            Self::write_sw_notice(req, node, sections, 0x00000003, timeout_ms)?;
        }

        if new[0x1c..0x20] != old[0x1c..0x20] {
            let mut n = 0u32;
            deserialize_u32(&mut n, &new[0x1c..0x20]);
            let mut o = 0u32;
            deserialize_u32(&mut o, &old[0x1c..0x20]);

            if (n ^ o) & 0x00000003 > 0 {
                Self::write_sw_notice(req, node, sections, 0x00000004, timeout_ms)?;
            }

            if (n ^ o) & 0x00000008 > 0 {
                Self::write_sw_notice(req, node, sections, 0x00000008, timeout_ms)?;
            }

            if (n ^ o) & 0x00000010 > 0 {
                Self::write_sw_notice(req, node, sections, 0x00000010, timeout_ms)?;
            }
        }

        deserialize_io_params(prev, Self::AESEBU_IS_SUPPORTED, &new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn input_params_serdes() {
        let params = SaffireproInputParams {
            mic_levels: [
                SaffireproMicInputLevel::Instrument,
                SaffireproMicInputLevel::Line,
            ],
            line_levels: [
                SaffireproLineInputLevel::High,
                SaffireproLineInputLevel::Low,
            ],
        };

        let mut raw = vec![0u8; INPUT_PARAMS_SIZE];
        serialize_input_params(&params, &mut raw).unwrap();

        let mut p = SaffireproInputParams::default();
        deserialize_input_params(&mut p, &raw).unwrap();

        assert_eq!(params, p);
    }

    #[test]
    fn out_group_serdes() {
        let params = OutGroupState {
            vols: vec![0, 1, 2, 3, 4, 5],
            vol_mutes: vec![false, true, true, false, false, true],
            vol_hwctls: vec![true, false, false, true, true, false],
            mute_enabled: true,
            mute_hwctls: vec![true, true, true, false, false, false],
            dim_enabled: true,
            dim_hwctls: vec![false, false, false, true, true, true],
            hw_knob_value: 33,
        };

        let mut raw = vec![0; 0x100];
        serialize_out_group_state(&params, &mut raw).unwrap();

        let mut p = OutGroupState {
            vols: vec![Default::default(); 6],
            vol_mutes: vec![Default::default(); 6],
            vol_hwctls: vec![Default::default(); 6],
            mute_enabled: true,
            mute_hwctls: vec![Default::default(); 6],
            dim_enabled: true,
            dim_hwctls: vec![Default::default(); 6],
            hw_knob_value: 33,
        };
        deserialize_out_group_state(&mut p, &raw).unwrap();

        assert_eq!(params, p);
    }

    #[test]
    fn io_params_serdes() {
        let params = SaffireproIoParams {
            analog_out_0_1_pad: true,
            opt_out_iface_mode: OpticalOutIfaceMode::Spdif,
            mic_amp_transformers: [true, false],
        };

        let mut raw = vec![0u8; IO_PARAMS_SIZE];
        serialize_io_params(&params, false, &mut raw).unwrap();

        let mut p = SaffireproIoParams::default();
        deserialize_io_params(&mut p, false, &raw).unwrap();

        assert_eq!(params, p);
    }
}
