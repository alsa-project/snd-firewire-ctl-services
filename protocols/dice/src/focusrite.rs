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

use super::{
    tcat::{
        extension::{appl_section::*, *},
        *,
    },
    *,
};

/// Software notice protocol to update hardware parameter.
pub trait SaffireproSwNoticeOperation {
    const SW_NOTICE_OFFSET: usize;

    fn write_sw_notice(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        notice: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        notice.build_quadlet(&mut raw);
        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
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
    /// Volume of each analog output.
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

fn serialize_out_group_state(state: &OutGroupState, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= OUT_GROUP_STATE_SIZE);

    let val = state.mute_enabled as u32;
    val.build_quadlet(&mut raw[0x00..0x04]);

    let val = state.dim_enabled as u32;
    val.build_quadlet(&mut raw[0x04..0x08]);

    (0..(state.vols.len() / 2)).for_each(|i| {
        let mut val = state.vols[2 * i] as u32;
        val |= (state.vols[2 * i + 1] as u32) << 8;
        let pos = 0x08 + i * 4;
        val.build_quadlet(&mut raw[pos..(pos + 4)]);
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
        val.build_quadlet(&mut raw[pos..(pos + 4)]);
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
    val.build_quadlet(&mut raw[0x30..0x34]);

    let val = state.hw_knob_value as u32;
    val.build_quadlet(&mut raw[0x48..0x4c]);

    Ok(())
}

fn deserialize_out_group_state(state: &mut OutGroupState, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= OUT_GROUP_STATE_SIZE);

    let mut val = 0u32;

    val.parse_quadlet(&raw[0x00..0x04]);
    state.mute_enabled = val > 0;

    val.parse_quadlet(&raw[0x04..0x08]);
    state.dim_enabled = val > 0;

    (0..(state.vols.len() / 2)).for_each(|i| {
        let pos = 0x08 + i * 4;
        val.parse_quadlet(&raw[pos..(pos + 4)]);
        state.vols[2 * i] = (val & 0x000000ff) as i8;
        state.vols[2 * i + 1] = ((val & 0x0000ff00) >> 8) as i8;
    });

    (0..(state.vol_hwctls.len() / 2)).for_each(|i| {
        let pos = 0x1c + i * 4;
        val.parse_quadlet(&raw[pos..(pos + 4)]);
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

    val.parse_quadlet(&raw[0x30..0x34]);
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

    val.parse_quadlet(&raw[0x48..0x4c]);
    state.hw_knob_value = val as i8;

    Ok(())
}

/// Output group protocol.
pub trait SaffireproOutGroupOperation: SaffireproSwNoticeOperation {
    const OUT_GROUP_STATE_OFFSET: usize;

    const ENTRY_COUNT: usize;
    const HAS_VOL_HWCTL: bool;

    const SRC_NOTICE: u32;
    const DIM_MUTE_NOTICE: u32;

    const MUTE_OFFSET: usize = Self::OUT_GROUP_STATE_OFFSET + 0x0000;
    const DIM_OFFSET: usize = Self::OUT_GROUP_STATE_OFFSET + 0x0004;
    const VOL_OFFSET: usize = Self::OUT_GROUP_STATE_OFFSET + 0x0008;
    const VOL_HWCTL_OFFSET: usize = Self::OUT_GROUP_STATE_OFFSET + 0x001c;
    const DIM_MUTE_HWCTL_OFFSET: usize = Self::OUT_GROUP_STATE_OFFSET + 0x0030;
    const HW_KNOB_VALUE_OFFSET: usize = Self::OUT_GROUP_STATE_OFFSET + 0x0048;

    const NOTIFY_DIM_MUTE_CHANGE: u32 = 0x00200000;

    /// Just supported by Liquid Saffire 56 and Saffire Pro 40.
    const NOTIFY_VOL_CHANGE: u32 = 0x00400000;

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

    /// Cache state of hardware for whole parameters.
    fn cache_whole_out_group_state(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        params: &mut OutGroupState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0u8; OUT_GROUP_STATE_SIZE];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            Self::OUT_GROUP_STATE_OFFSET,
            &mut raw,
            timeout_ms,
        )?;
        deserialize_out_group_state(params, &raw)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }

    /// Update state of hardware for partial parameters.
    fn update_partial_out_group_state(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        state: &OutGroupState,
        prev: &mut OutGroupState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut new = vec![0u8; OUT_GROUP_STATE_SIZE];
        serialize_out_group_state(state, &mut new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        let mut old = vec![0u8; OUT_GROUP_STATE_SIZE];
        serialize_out_group_state(prev, &mut old)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        (0..OUT_GROUP_STATE_SIZE).step_by(4).try_for_each(|pos| {
            if new[pos..(pos + 4)] != old[pos..(pos + 4)] {
                ApplSectionProtocol::write_appl_data(
                    req,
                    node,
                    sections,
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

    /// Cache state of hardware for parameters according to received notification.
    fn cache_notified_params(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        msg: u32,
        state: &mut OutGroupState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if msg & (Self::NOTIFY_DIM_MUTE_CHANGE | Self::NOTIFY_VOL_CHANGE) > 0 {
            Self::cache_whole_out_group_state(req, node, sections, state, timeout_ms)?;

            if msg & Self::NOTIFY_VOL_CHANGE > 0 {
                let vol_hwctls = state.vol_hwctls.clone();
                let hw_knob_value = state.hw_knob_value;
                state
                    .vols
                    .iter_mut()
                    .zip(vol_hwctls)
                    .filter(|(_, vol_hwctl)| *vol_hwctl)
                    .for_each(|(vol, _)| *vol = hw_knob_value);
            }
        }

        Ok(())
    }

    fn read_out_group_mute(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        state: &mut OutGroupState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            Self::MUTE_OFFSET,
            &mut raw,
            timeout_ms,
        )
        .map(|_| {
            let mut val = 0u32;
            val.parse_quadlet(&raw);
            state.mute_enabled = val > 0;
        })
    }

    fn write_out_group_mute(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        state: &mut OutGroupState,
        enable: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        enable.build_quadlet(&mut raw);
        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            Self::MUTE_OFFSET,
            &mut raw,
            timeout_ms,
        )?;
        Self::write_sw_notice(req, node, sections, Self::DIM_MUTE_NOTICE, timeout_ms)
            .map(|_| state.mute_enabled = enable)
    }

    fn read_out_group_dim(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        state: &mut OutGroupState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            Self::DIM_OFFSET,
            &mut raw,
            timeout_ms,
        )
        .map(|_| {
            let mut val = 0u32;
            val.parse_quadlet(&raw);
            state.dim_enabled = val > 0;
        })
    }

    fn write_out_group_dim(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        state: &mut OutGroupState,
        enable: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        enable.build_quadlet(&mut raw);
        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            Self::DIM_OFFSET,
            &mut raw,
            timeout_ms,
        )?;
        Self::write_sw_notice(req, node, sections, Self::DIM_MUTE_NOTICE, timeout_ms)
            .map(|_| state.dim_enabled = enable)
    }

    fn read_out_group_vols(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        state: &mut OutGroupState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0; (Self::ENTRY_COUNT + 1) / 2 * 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            Self::VOL_OFFSET,
            &mut raw,
            timeout_ms,
        )
        .map(|_| {
            let mut val = 0u32;
            (0..(Self::ENTRY_COUNT / 2)).for_each(|i| {
                let pos = i * 4;
                val.parse_quadlet(&raw[pos..(pos + 4)]);
                state.vols[2 * i] = (val & 0x000000ff) as i8;
                state.vols[2 * i + 1] = ((val & 0x0000ff00) >> 8) as i8;
            })
        })
    }

    fn write_out_group_vols(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        state: &mut OutGroupState,
        vols: &[i8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(state.vols.len(), vols.len());

        let mut raw = vec![0; (Self::ENTRY_COUNT + 1) / 2 * 4];
        (0..(Self::ENTRY_COUNT / 2)).for_each(|i| {
            let left_vol = state.vols[2 * i] as u32;
            let right_vol = state.vols[2 * i + 1] as u32;
            let val = (right_vol << 24) | (left_vol << 16) | (right_vol << 8) | left_vol;
            let pos = i * 4;
            val.build_quadlet(&mut raw[pos..(pos + 4)]);
        });
        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            Self::VOL_OFFSET,
            &mut raw,
            timeout_ms,
        )?;
        Self::write_sw_notice(req, node, sections, Self::SRC_NOTICE, timeout_ms)
            .map(|_| state.vols.copy_from_slice(&vols))
    }

    fn read_out_group_vol_mute_hwctls(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        state: &mut OutGroupState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0; (Self::ENTRY_COUNT + 1) / 2 * 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            Self::VOL_HWCTL_OFFSET,
            &mut raw,
            timeout_ms,
        )
        .map(|_| {
            let mut val = 0u32;
            (0..(Self::ENTRY_COUNT / 2)).for_each(|i| {
                let pos = i * 4;
                val.parse_quadlet(&raw[pos..(pos + 4)]);
                if Self::HAS_VOL_HWCTL {
                    state.vol_hwctls[2 * i] = val & 0x00000001 > 0;
                    state.vol_hwctls[2 * i + 1] = val & 0x00000002 > 0;
                } else {
                    state.vol_hwctls[2 * i] = false;
                    state.vol_hwctls[2 * i + 1] = false;
                }
                state.vol_mutes[2 * i] = val & 0x00000004 > 0;
                state.vol_mutes[2 * i + 1] = val & 0x00000008 > 0;
            })
        })
    }

    fn write_out_group_vol_mute_hwctls(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        state: &mut OutGroupState,
        vol_mutes: &[bool],
        vol_hwctls: &[bool],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(vol_mutes.len(), vol_hwctls.len());
        assert_eq!(state.vol_mutes.len(), vol_mutes.len());
        assert_eq!(state.vol_hwctls.len(), vol_hwctls.len());

        let mut raw = vec![0; (Self::ENTRY_COUNT + 1) / 2 * 4];
        (0..(Self::ENTRY_COUNT / 2)).for_each(|i| {
            let mut val = 0u32;
            if Self::HAS_VOL_HWCTL {
                if vol_hwctls[2 * i] {
                    val |= 0x00000001;
                }
                if vol_hwctls[2 * i + 1] {
                    val |= 0x00000002;
                }
            }
            if vol_mutes[2 * i] {
                val |= 0x00000004;
            }
            if vol_mutes[2 * i + 1] {
                val |= 0x00000008;
            }
            let pos = i * 4;
            val.build_quadlet(&mut raw[pos..(pos + 4)]);
        });
        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            Self::VOL_HWCTL_OFFSET,
            &mut raw,
            timeout_ms,
        )?;

        vol_hwctls.iter().enumerate().try_for_each(|(i, &hwctl)| {
            if !hwctl && state.vol_hwctls[i] {
                let pos = i / 2;
                let mut raw = [0; 4];
                ApplSectionProtocol::read_appl_data(
                    req,
                    node,
                    sections,
                    Self::VOL_OFFSET + pos * 4,
                    &mut raw,
                    timeout_ms,
                )?;
                let mut val = u32::from_be_bytes(raw);
                let ch = i % 2;
                if ch == 0 {
                    val &= 0x00ff00ff;
                    val |= (state.vols[i] as u32) << 24;
                    val |= (state.vols[i] as u32) << 8;
                } else {
                    val &= 0xff00ff00;
                    val |= (state.vols[i] as u32) << 16;
                    val |= state.vols[i] as u32;
                }
                val.build_quadlet(&mut raw);
                ApplSectionProtocol::write_appl_data(
                    req,
                    node,
                    sections,
                    Self::VOL_OFFSET + pos * 4,
                    &mut raw,
                    timeout_ms,
                )
            } else {
                Ok(())
            }
        })?;

        Self::write_sw_notice(req, node, sections, Self::SRC_NOTICE, timeout_ms).map(|_| {
            state.vol_mutes.copy_from_slice(&vol_mutes);
            state.vol_hwctls.copy_from_slice(&vol_hwctls);
        })
    }

    fn read_out_group_dim_mute_hwctls(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        state: &mut OutGroupState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            Self::DIM_MUTE_HWCTL_OFFSET,
            &mut raw,
            timeout_ms,
        )
        .map(|_| {
            let mut val = 0u32;
            val.parse_quadlet(&raw);
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
        })
    }

    fn write_out_group_dim_mute_hwctls(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        state: &mut OutGroupState,
        dim_hwctls: &[bool],
        mute_hwctls: &[bool],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(dim_hwctls.len(), mute_hwctls.len());
        assert_eq!(state.dim_hwctls.len(), dim_hwctls.len());
        assert_eq!(state.mute_hwctls.len(), mute_hwctls.len());

        let dim_assign_flags = dim_hwctls
            .iter()
            .enumerate()
            .filter(|(_, &assign)| assign)
            .fold(0u32, |v, (i, _)| v | (1 << (i + 10)));
        let mute_assign_flags = mute_hwctls
            .iter()
            .enumerate()
            .filter(|(_, &assign)| assign)
            .fold(0u32, |v, (i, _)| v + (1 << i));
        let val = dim_assign_flags | mute_assign_flags;
        let mut raw = [0; 4];
        val.build_quadlet(&mut raw);
        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            Self::DIM_MUTE_HWCTL_OFFSET,
            &mut raw,
            timeout_ms,
        )?;
        Self::write_sw_notice(req, node, sections, Self::SRC_NOTICE, timeout_ms).map(|_| {
            state.dim_hwctls.copy_from_slice(&dim_hwctls);
            state.mute_hwctls.copy_from_slice(&mute_hwctls);
        })
    }

    fn read_out_group_knob_value(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        state: &mut OutGroupState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            Self::HW_KNOB_VALUE_OFFSET,
            &mut raw,
            timeout_ms,
        )
        .map(|_| state.hw_knob_value = u32::from_be_bytes(raw) as i8)
    }

    fn has_dim_mute_change(msg: u32) -> bool {
        msg & Self::NOTIFY_DIM_MUTE_CHANGE > 0
    }

    fn has_vol_change(msg: u32) -> bool {
        msg & Self::NOTIFY_VOL_CHANGE > 0
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
    val.build_quadlet(&mut raw[..4]);

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
    val.build_quadlet(&mut raw[4..8]);

    Ok(())
}

fn deserialize_input_params(params: &mut SaffireproInputParams, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= INPUT_PARAMS_SIZE);

    let mut val = 0u32;

    val.parse_quadlet(&raw[..4]);
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

    val.parse_quadlet(&raw[4..8]);
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
pub trait SaffireproInputOperation: SaffireproSwNoticeOperation {
    const INPUT_PARAMS_OFFSET: usize;

    const SW_NOTICE: u32 = 0x00000004;

    const MIC_INPUT_COUNT: usize = 2;
    const LINE_INPUT_COUNT: usize = 2;

    /// Cache state of hardware for whole parameters.
    fn cache_whole_input_params(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        params: &mut SaffireproInputParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0u8; INPUT_PARAMS_SIZE];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            Self::INPUT_PARAMS_OFFSET,
            &mut raw,
            timeout_ms,
        )?;
        deserialize_input_params(params, &raw)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }

    /// Update state of hardware for partial parameters.
    fn update_partial_input_params(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
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
                ApplSectionProtocol::write_appl_data(
                    req,
                    node,
                    sections,
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
}
