// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol specific to Focusrite Saffire Pro 40.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Focusrite for Saffire Pro 40.

use crate::tcat::extension::*;
use crate::tcat::tcd22xx_spec::*;

use super::*;

/// The structure to represent state of TCD22xx on Saffire Pro 40.
#[derive(Debug)]
pub struct SPro40State{
    tcd22xx: Tcd22xxState,
    out_grp: OutGroupState,
}

impl Default for SPro40State {
    fn default() -> Self {
        SPro40State{
            tcd22xx: Tcd22xxState::default(),
            out_grp: Self::create_out_group_state(),
        }
    }
}

impl Tcd22xxSpec for SPro40State {
    const INPUTS: &'static [Input] = &[
        Input{id: SrcBlkId::Ins1, offset: 0, count: 6, label: None},
        Input{id: SrcBlkId::Aes, offset: 0, count: 2, label: Some("S/PDIF-coax")},
        // NOTE: share the same optical interface.
        Input{id: SrcBlkId::Adat, offset: 0, count: 8, label: None},
        Input{id: SrcBlkId::Aes, offset: 4, count: 2, label: Some("S/PDIF-opt")},
    ];
    const OUTPUTS: &'static [Output] = &[
        Output{id: DstBlkId::Ins0, offset: 0, count: 2, label: None},
        Output{id: DstBlkId::Ins1, offset: 0, count: 8, label: None},
        Output{id: DstBlkId::Aes, offset: 0, count: 2, label: Some("S/PDIF-coax")},
        // NOTE: share the same optical interface.
        Output{id: DstBlkId::Adat, offset: 0, count: 8, label: None},
        Output{id: DstBlkId::Aes, offset: 4, count: 2, label: Some("S/PDIF-opt")},
    ];
    // NOTE: The first 8 entries in router section are used to display hardware metering.
    const FIXED: &'static [SrcBlk] = &[
        SrcBlk{id: SrcBlkId::Ins1, ch: 0},
        SrcBlk{id: SrcBlkId::Ins1, ch: 1},
        SrcBlk{id: SrcBlkId::Ins1, ch: 2},
        SrcBlk{id: SrcBlkId::Ins1, ch: 3},
        SrcBlk{id: SrcBlkId::Ins1, ch: 4},
        SrcBlk{id: SrcBlkId::Ins1, ch: 5},
        SrcBlk{id: SrcBlkId::Ins1, ch: 6},
        SrcBlk{id: SrcBlkId::Ins1, ch: 7},
    ];
}

impl AsMut<Tcd22xxState> for SPro40State {
    fn as_mut(&mut self) -> &mut Tcd22xxState {
        &mut self.tcd22xx
    }
}

impl AsRef<Tcd22xxState> for SPro40State {
    fn as_ref(&self) -> &Tcd22xxState {
        &self.tcd22xx
    }
}

const SW_NOTICE_OFFSET: usize = 0x0068;

const SRC_SW_NOTICE: u32 = 0x00000001;
const DIM_MUTE_SW_NOTICE: u32 = 0x00000002;
const OUT_PAD_SW_NOTICE: u32 = 0x00000003;
const IO_FLAG_SW_NOTICE: u32 = 0x00000004;

impl OutGroupSpec for SPro40State {
    const ENTRY_COUNT: usize = 10;
    const HAS_VOL_HWCTL: bool = true;
    const OUT_CTL_OFFSET: usize = 0x000c;
    const SW_NOTICE_OFFSET: usize = SW_NOTICE_OFFSET;

    const SRC_NOTICE: u32 = SRC_SW_NOTICE;
    const DIM_MUTE_NOTICE: u32 = DIM_MUTE_SW_NOTICE;
}

impl AsMut<OutGroupState> for SPro40State {
    fn as_mut(&mut self) -> &mut OutGroupState {
        &mut self.out_grp
    }
}

impl AsRef<OutGroupState> for SPro40State {
    fn as_ref(&self) -> &OutGroupState {
        &self.out_grp
    }
}

/// The enumeration to represent type of signal for optical output interface.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OptOutIfaceMode {
    Adat,
    Spdif,
}

/// The trait to represent protocol specific to Saffire Pro 26.
pub trait SPro40Protocol<T> : ApplSectionProtocol<T>
    where T: AsRef<FwNode>,
{
    const ANALOG_OUT_0_1_PAD_OFFSET: usize = 0x0040;
    const IO_FLAGS_OFFSET: usize = 0x005c;

    fn write_sw_notice(&self, node: &T, sections: &ExtensionSections, notice: u32, timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut raw = [0;4];
        notice.build_quadlet(&mut raw);
        self.write_appl_data(node, sections, SW_NOTICE_OFFSET, &mut raw, timeout_ms)
    }

    fn read_analog_out_0_1_pad_offset(&self, node: &T, sections: &ExtensionSections, timeout_ms: u32)
        ->Result<bool, Error>
    {
        let mut raw = [0;4];
        self.read_appl_data(node, sections, Self::ANALOG_OUT_0_1_PAD_OFFSET, &mut raw, timeout_ms)
            .map(|_| u32::from_be_bytes(raw) > 0)
    }

    fn write_analog_out_0_1_pad_offset(&self, node: &T, sections: &ExtensionSections, enable: bool,
                                       timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut raw = [0;4];
        enable.build_quadlet(&mut raw);
        self.write_appl_data(node, sections, Self::ANALOG_OUT_0_1_PAD_OFFSET, &mut raw, timeout_ms)?;
        self.write_sw_notice(node, sections, OUT_PAD_SW_NOTICE, timeout_ms)
    }

    fn read_opt_out_iface_mode(&self, node: &T, sections: &ExtensionSections, timeout_ms: u32)
        -> Result<OptOutIfaceMode, Error>
    {
        let mut raw = [0;4];
        self.read_appl_data(node, sections, Self::IO_FLAGS_OFFSET, &mut raw, timeout_ms)
            .map(|_| {
                let val = u32::from_be_bytes(raw);
                if val & 0x00000001 > 0 {
                    OptOutIfaceMode::Spdif
                } else {
                    OptOutIfaceMode::Adat
                }
            })
    }

    fn write_opt_out_iface_mode(&self, node: &T, sections: &ExtensionSections, mode: OptOutIfaceMode,
                                timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut raw = [0;4];
        self.read_appl_data(node, sections, Self::IO_FLAGS_OFFSET, &mut raw, timeout_ms)?;

        let mut val = u32::from_be_bytes(raw);
        val &= !0x00000003;

        if mode == OptOutIfaceMode::Spdif {
            val |= 0x00000001;
        }
        val.build_quadlet(&mut raw);
        self.write_appl_data(node, sections, Self::IO_FLAGS_OFFSET, &mut raw, timeout_ms)?;
        self.write_sw_notice(node, sections, IO_FLAG_SW_NOTICE, timeout_ms)
    }
}

impl<O: ApplSectionProtocol<T>, T: AsRef<FwNode>> SPro40Protocol<T> for O {}
