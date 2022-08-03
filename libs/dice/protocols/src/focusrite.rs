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
    tcat::extension::{appl_section::*, *},
    *,
};

/// The trait for software notice protocol to update hardware parameter.
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

/// The structure to represent a set of entries for output control.
#[derive(Default, Debug)]
pub struct OutGroupState {
    pub vols: Vec<i8>,
    pub vol_mutes: Vec<bool>,
    pub vol_hwctls: Vec<bool>,

    pub mute_enabled: bool,
    pub mute_hwctls: Vec<bool>,

    pub dim_enabled: bool,
    pub dim_hwctls: Vec<bool>,

    pub hw_knob_value: i8,
}

/// The trait for output group protocol.
pub trait SaffireproOutGroupOperation: SaffireproSwNoticeOperation {
    const ENTRY_COUNT: usize;
    const HAS_VOL_HWCTL: bool;

    const OUT_CTL_OFFSET: usize;

    const SRC_NOTICE: u32;
    const DIM_MUTE_NOTICE: u32;

    const MUTE_OFFSET: usize = Self::OUT_CTL_OFFSET + 0x0000;
    const DIM_OFFSET: usize = Self::OUT_CTL_OFFSET + 0x0004;
    const VOL_OFFSET: usize = Self::OUT_CTL_OFFSET + 0x0008;
    const VOL_HWCTL_OFFSET: usize = Self::OUT_CTL_OFFSET + 0x001c;
    const DIM_MUTE_HWCTL_OFFSET: usize = Self::OUT_CTL_OFFSET + 0x0030;
    const HW_KNOB_VALUE_OFFSET: usize = Self::OUT_CTL_OFFSET + 0x0048;

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
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

const MIC_INPUT_LEVEL_INSTRUMENT_FLAG: u16 = 0x0002;
const LINE_INPUT_LEVEL_HIGH_FLAG: u16 = 0x0001;

// The trait for input protocol specific to Pro 14 and Pro 24.
pub trait SaffireproInputOperation: SaffireproSwNoticeOperation {
    const MIC_INPUT_OFFSET: usize;
    const LINE_INPUT_OFFSET: usize;

    const SW_NOTICE: u32 = 0x00000004;

    const MIC_INPUT_COUNT: usize = 2;
    const LINE_INPUT_COUNT: usize = 2;

    fn read_mic_level(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        levels: &mut [SaffireproMicInputLevel],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(levels.len(), Self::MIC_INPUT_COUNT);

        let mut raw = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            Self::MIC_INPUT_OFFSET,
            &mut raw,
            timeout_ms,
        )
        .map(|_| {
            let val = u32::from_be_bytes(raw);

            levels.iter_mut().enumerate().for_each(|(i, level)| {
                let flag = (MIC_INPUT_LEVEL_INSTRUMENT_FLAG as u32) << (i * 16);
                *level = if val & flag > 0 {
                    SaffireproMicInputLevel::Instrument
                } else {
                    SaffireproMicInputLevel::Line
                }
            });
        })
    }

    fn write_mic_level(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        levels: &[SaffireproMicInputLevel],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(levels.len(), Self::MIC_INPUT_COUNT);

        let val = levels
            .iter()
            .enumerate()
            .fold(0u32, |mut val, (i, &level)| {
                if level == SaffireproMicInputLevel::Instrument {
                    val |= (MIC_INPUT_LEVEL_INSTRUMENT_FLAG as u32) << (i * 16);
                }
                val
            });

        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            Self::MIC_INPUT_OFFSET,
            &mut val.to_be_bytes(),
            timeout_ms,
        )?;
        Self::write_sw_notice(req, node, sections, Self::SW_NOTICE, timeout_ms)
    }

    fn read_line_level(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        levels: &mut [SaffireproLineInputLevel],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(levels.len(), Self::LINE_INPUT_COUNT);

        let mut raw = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            Self::LINE_INPUT_OFFSET,
            &mut raw,
            timeout_ms,
        )
        .map(|_| {
            let val = u32::from_be_bytes(raw);

            levels.iter_mut().enumerate().for_each(|(i, level)| {
                let flag = (LINE_INPUT_LEVEL_HIGH_FLAG as u32) << (i * 16);
                *level = if val & flag > 0 {
                    SaffireproLineInputLevel::High
                } else {
                    SaffireproLineInputLevel::Low
                }
            });
        })
    }

    fn write_line_level(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        levels: &[SaffireproLineInputLevel],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(levels.len(), Self::LINE_INPUT_COUNT);

        let val = levels
            .iter()
            .enumerate()
            .fold(0u32, |mut val, (i, &level)| {
                if level == SaffireproLineInputLevel::High {
                    val |= (LINE_INPUT_LEVEL_HIGH_FLAG as u32) << (i * 16);
                }
                val
            });

        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            Self::LINE_INPUT_OFFSET,
            &mut val.to_be_bytes(),
            timeout_ms,
        )?;
        Self::write_sw_notice(req, node, sections, Self::SW_NOTICE, timeout_ms)
    }
}
