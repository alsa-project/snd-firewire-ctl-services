// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol specific to Focusrite Saffire Pro 40.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Focusrite for Saffire Pro 40.

use super::{tcat::tcd22xx_spec::*, *};

const ANALOG_OUT_0_1_PAD_OFFSET: usize = 0x0040;
const IO_FLAGS_OFFSET: usize = 0x005c;

/// Protocol implementation specific to Saffire Pro 40.
#[derive(Default, Debug)]
pub struct SPro40Protocol;

impl TcatOperation for SPro40Protocol {}

impl TcatGlobalSectionSpecification for SPro40Protocol {}

impl Tcd22xxSpecOperation for SPro40Protocol {
    const INPUTS: &'static [Input] = &[
        Input {
            id: SrcBlkId::Ins1,
            offset: 0,
            count: 8,
            label: None,
        },
        Input {
            id: SrcBlkId::Aes,
            offset: 0,
            count: 2,
            label: Some("S/PDIF-coax"),
        },
        // NOTE: share the same optical interface.
        Input {
            id: SrcBlkId::Adat,
            offset: 0,
            count: 8,
            label: None,
        },
        Input {
            id: SrcBlkId::Aes,
            offset: 4,
            count: 2,
            label: Some("S/PDIF-opt"),
        },
    ];
    const OUTPUTS: &'static [Output] = &[
        Output {
            id: DstBlkId::Ins0,
            offset: 0,
            count: 2,
            label: None,
        },
        Output {
            id: DstBlkId::Ins1,
            offset: 0,
            count: 8,
            label: None,
        },
        Output {
            id: DstBlkId::Aes,
            offset: 0,
            count: 2,
            label: Some("S/PDIF-coax"),
        },
        // NOTE: share the same optical interface.
        Output {
            id: DstBlkId::Adat,
            offset: 0,
            count: 8,
            label: None,
        },
        Output {
            id: DstBlkId::Aes,
            offset: 4,
            count: 2,
            label: Some("S/PDIF-opt"),
        },
    ];
    // NOTE: The first 8 entries in router section are used to display hardware metering.
    const FIXED: &'static [SrcBlk] = &[
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 0,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 1,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 2,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 3,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 4,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 5,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 6,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 7,
        },
    ];
}

impl SaffireproSwNoticeOperation for SPro40Protocol {
    const SW_NOTICE_OFFSET: usize = 0x0068;
}

const SRC_SW_NOTICE: u32 = 0x00000001;
const DIM_MUTE_SW_NOTICE: u32 = 0x00000002;
const OUT_PAD_SW_NOTICE: u32 = 0x00000003;
const IO_FLAG_SW_NOTICE: u32 = 0x00000004;

impl SaffireproOutGroupOperation for SPro40Protocol {
    const OUT_GROUP_STATE_OFFSET: usize = 0x000c;

    const ENTRY_COUNT: usize = 10;
    const HAS_VOL_HWCTL: bool = true;

    const SRC_NOTICE: u32 = SRC_SW_NOTICE;
    const DIM_MUTE_NOTICE: u32 = DIM_MUTE_SW_NOTICE;
}

/// Type of signal for optical output interface.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OptOutIfaceMode {
    Adat,
    Spdif,
}

impl Default for OptOutIfaceMode {
    fn default() -> Self {
        Self::Adat
    }
}

/// Protocol specific to Saffire Pro 26.
impl SPro40Protocol {
    pub fn read_analog_out_0_1_pad(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        let mut raw = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            ANALOG_OUT_0_1_PAD_OFFSET,
            &mut raw,
            timeout_ms,
        )
        .map(|_| u32::from_be_bytes(raw) > 0)
    }

    pub fn write_analog_out_0_1_pad(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        enable: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        enable.build_quadlet(&mut raw);
        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            ANALOG_OUT_0_1_PAD_OFFSET,
            &mut raw,
            timeout_ms,
        )?;
        Self::write_sw_notice(req, node, sections, OUT_PAD_SW_NOTICE, timeout_ms)
    }

    pub fn read_opt_out_iface_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<OptOutIfaceMode, Error> {
        let mut raw = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            IO_FLAGS_OFFSET,
            &mut raw,
            timeout_ms,
        )
        .map(|_| {
            let val = u32::from_be_bytes(raw);
            if val & 0x00000001 > 0 {
                OptOutIfaceMode::Spdif
            } else {
                OptOutIfaceMode::Adat
            }
        })
    }

    pub fn write_opt_out_iface_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        mode: OptOutIfaceMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            IO_FLAGS_OFFSET,
            &mut raw,
            timeout_ms,
        )?;

        let mut val = u32::from_be_bytes(raw);
        val &= !0x00000003;

        if mode == OptOutIfaceMode::Spdif {
            val |= 0x00000001;
        }
        val.build_quadlet(&mut raw);
        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            IO_FLAGS_OFFSET,
            &mut raw,
            timeout_ms,
        )?;
        Self::write_sw_notice(req, node, sections, IO_FLAG_SW_NOTICE, timeout_ms)
    }
}
