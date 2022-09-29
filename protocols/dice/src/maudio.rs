// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Hardware specification and application protocol specific to M-Audio ProFire series.
//!
//! The modules includes structure, enumeration, and trait and its implementation for hardware
//! specification and application protocol specific to M-Audio ProFire series.

use super::{
    tcat::{
        extension::{appl_section::*, *},
        global_section::*,
        tcd22xx_spec::*,
    },
    *,
};

const KNOB_ASSIGN_OFFSET: usize = 0x00;
const STANDALONE_MODE_OFFSET: usize = 0x04;

/// Protocol implementation specific to ProFire 2626.
#[derive(Default, Debug)]
pub struct Pfire2626Protocol;

impl PfireClkSpec for Pfire2626Protocol {
    const AVAIL_CLK_SRCS: &'static [ClockSource] = &[
        ClockSource::Aes1,
        ClockSource::Aes4,
        ClockSource::Adat,
        ClockSource::Tdif,
        ClockSource::WordClock,
        ClockSource::Internal,
    ];
}

impl PfireSpecificOperation for Pfire2626Protocol {
    const HAS_OPT_IFACE_B: bool = true;
    const SUPPORT_STANDALONE_CONVERTER: bool = true;
}

impl Tcd22xxSpecOperation for Pfire2626Protocol {
    const INPUTS: &'static [Input] = &[
        Input {
            id: SrcBlkId::Ins1,
            offset: 0,
            count: 8,
            label: None,
        },
        Input {
            id: SrcBlkId::Adat,
            offset: 0,
            count: 8,
            label: None,
        },
        Input {
            id: SrcBlkId::Adat,
            offset: 8,
            count: 8,
            label: None,
        },
        Input {
            id: SrcBlkId::Aes,
            offset: 0,
            count: 2,
            label: None,
        },
    ];
    const OUTPUTS: &'static [Output] = &[
        Output {
            id: DstBlkId::Ins1,
            offset: 0,
            count: 8,
            label: None,
        },
        Output {
            id: DstBlkId::Adat,
            offset: 0,
            count: 8,
            label: None,
        },
        Output {
            id: DstBlkId::Adat,
            offset: 8,
            count: 8,
            label: None,
        },
        Output {
            id: DstBlkId::Aes,
            offset: 0,
            count: 2,
            label: None,
        },
    ];
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

/// Protocol implementation specific to ProFire 610.
#[derive(Default, Debug)]
pub struct Pfire610Protocol;

impl PfireClkSpec for Pfire610Protocol {
    const AVAIL_CLK_SRCS: &'static [ClockSource] = &[ClockSource::Aes1, ClockSource::Internal];
}

impl PfireSpecificOperation for Pfire610Protocol {
    const HAS_OPT_IFACE_B: bool = false;
    const SUPPORT_STANDALONE_CONVERTER: bool = false;
}

/// Available rates and sources of sampling clock.
pub trait PfireClkSpec {
    const AVAIL_CLK_RATES: &'static [ClockRate] = &[
        ClockRate::R32000,
        ClockRate::R44100,
        ClockRate::R48000,
        ClockRate::R88200,
        ClockRate::R96000,
        ClockRate::R176400,
        ClockRate::R192000,
    ];

    const AVAIL_CLK_SRCS: &'static [ClockSource];
}

// NOTE: the second rx stream is firstly available at higher sampling rate.
impl Tcd22xxSpecOperation for Pfire610Protocol {
    const INPUTS: &'static [Input] = &[
        Input {
            id: SrcBlkId::Ins0,
            offset: 0,
            count: 4,
            label: None,
        },
        Input {
            id: SrcBlkId::Aes,
            offset: 0,
            count: 2,
            label: None,
        },
    ];
    const OUTPUTS: &'static [Output] = &[
        Output {
            id: DstBlkId::Ins0,
            offset: 0,
            count: 8,
            label: None,
        },
        Output {
            id: DstBlkId::Aes,
            offset: 0,
            count: 2,
            label: None,
        },
    ];
    const FIXED: &'static [SrcBlk] = &[
        SrcBlk {
            id: SrcBlkId::Ins0,
            ch: 0,
        },
        SrcBlk {
            id: SrcBlkId::Ins0,
            ch: 1,
        },
    ];
}

/// Mode of optical interface.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OptIfaceMode {
    Spdif,
    Adat,
}

/// Mode of standalone converter.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StandaloneConverterMode {
    AdDa,
    AdOnly,
}

const KNOB_ASSIGN_MASK: u32 = 0x0f;
const OPT_IFACE_B_IS_SPDIF_FLAG: u32 = 0x10;
const STANDALONE_CONVERTER_IS_AD_ONLY_FLAG: u32 = 0x02;

/// Protocol implementation specific to ProFire series.
pub trait PfireSpecificOperation {
    const HAS_OPT_IFACE_B: bool;
    const SUPPORT_STANDALONE_CONVERTER: bool;

    const KNOB_COUNT: usize = 4;

    fn read_knob_assign(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        targets: &mut [bool],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut data = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            KNOB_ASSIGN_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map(|_| {
            let val = u32::from_be_bytes(data) & KNOB_ASSIGN_MASK;
            targets
                .iter_mut()
                .enumerate()
                .for_each(|(i, v)| *v = val & (1 << i) > 0)
        })
    }

    fn write_knob_assign(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        targets: &[bool],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut data = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            KNOB_ASSIGN_OFFSET,
            &mut data,
            timeout_ms,
        )?;
        let mut val = u32::from_be_bytes(data) & KNOB_ASSIGN_MASK;

        targets.iter().enumerate().for_each(|(i, knob)| {
            val &= !(1 << i);
            if *knob {
                val |= 1 << i;
            }
        });
        data.copy_from_slice(&val.to_be_bytes());

        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            KNOB_ASSIGN_OFFSET,
            &mut data,
            timeout_ms,
        )
    }

    fn read_opt_iface_b_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<OptIfaceMode, Error> {
        let mut data = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            KNOB_ASSIGN_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map(|_| {
            let val = u32::from_be_bytes(data);
            if val & OPT_IFACE_B_IS_SPDIF_FLAG > 0 {
                OptIfaceMode::Spdif
            } else {
                OptIfaceMode::Adat
            }
        })
    }

    fn write_opt_iface_b_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        mode: OptIfaceMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut data = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            KNOB_ASSIGN_OFFSET,
            &mut data,
            timeout_ms,
        )?;
        let mut val = u32::from_be_bytes(data);

        val &= !OPT_IFACE_B_IS_SPDIF_FLAG;
        if mode == OptIfaceMode::Spdif {
            val |= OPT_IFACE_B_IS_SPDIF_FLAG;
        }
        data.copy_from_slice(&val.to_be_bytes());

        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            KNOB_ASSIGN_OFFSET,
            &mut data,
            timeout_ms,
        )
    }

    fn read_standalone_converter_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<StandaloneConverterMode, Error> {
        let mut data = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            STANDALONE_MODE_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map(|_| {
            let val = u32::from_be_bytes(data);
            if val & STANDALONE_CONVERTER_IS_AD_ONLY_FLAG > 0 {
                StandaloneConverterMode::AdOnly
            } else {
                StandaloneConverterMode::AdDa
            }
        })
    }

    fn write_standalone_converter_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        mode: StandaloneConverterMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut data = [0; 4];
        ApplSectionProtocol::read_appl_data(
            req,
            node,
            sections,
            STANDALONE_MODE_OFFSET,
            &mut data,
            timeout_ms,
        )?;
        let mut val = u32::from_be_bytes(data);

        val &= !STANDALONE_CONVERTER_IS_AD_ONLY_FLAG;
        if mode == StandaloneConverterMode::AdOnly {
            val |= STANDALONE_CONVERTER_IS_AD_ONLY_FLAG;
        }
        data.copy_from_slice(&val.to_be_bytes());

        ApplSectionProtocol::write_appl_data(
            req,
            node,
            sections,
            STANDALONE_MODE_OFFSET,
            &mut data,
            timeout_ms,
        )
    }
}
