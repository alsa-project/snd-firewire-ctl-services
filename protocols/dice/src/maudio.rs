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
        *,
    },
    *,
};

/// Protocol implementation specific to ProFire 2626.
#[derive(Default, Debug)]
pub struct Pfire2626Protocol;

impl TcatOperation for Pfire2626Protocol {}

impl TcatGlobalSectionSpecification for Pfire2626Protocol {
    // NOTE: ClockSource::Tdif is used for second optical interface as 'ADAT_AUX'.
    const AVAILABLE_CLOCK_SOURCE_OVERRIDE: Option<&'static [ClockSource]> = Some(&[
        ClockSource::Aes1,
        ClockSource::Aes4,
        ClockSource::Adat,
        ClockSource::Tdif,
        ClockSource::WordClock,
        ClockSource::Internal,
    ]);
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

impl TcatOperation for Pfire610Protocol {}

impl TcatGlobalSectionSpecification for Pfire610Protocol {
    const AVAILABLE_CLOCK_SOURCE_OVERRIDE: Option<&'static [ClockSource]> =
        Some(&[ClockSource::Aes1, ClockSource::Internal]);
}

impl PfireSpecificOperation for Pfire610Protocol {
    const HAS_OPT_IFACE_B: bool = false;
    const SUPPORT_STANDALONE_CONVERTER: bool = false;
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OptIfaceMode {
    /// For S/PDIF signal.
    Spdif,
    /// For ADAT signal.
    Adat,
}

impl Default for OptIfaceMode {
    fn default() -> Self {
        Self::Spdif
    }
}

/// Mode of standalone converter.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StandaloneConverterMode {
    /// For A/D and D/A conversion.
    AdDa,
    /// For A/D conversion only.
    AdOnly,
}

impl Default for StandaloneConverterMode {
    fn default() -> Self {
        Self::AdDa
    }
}

/// Mode of standalone converter.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct PfireSpecificParams {
    /// Whether volumes of 4 analog output pairs have assignment to hardware knob.
    pub knob_assigns: [bool; 4],
    /// Mode of optical interface B.
    pub opt_iface_b_mode: OptIfaceMode,
    /// Mode of converter at standalone.
    pub standalone_mode: StandaloneConverterMode,
}

// const KNOB_ASSIGN_OFFSET: usize = 0x00;
// const STANDALONE_MODE_OFFSET: usize = 0x04;

const KNOB_ASSIGN_MASK: u32 = 0x0f;
const OPT_IFACE_B_IS_SPDIF_FLAG: u32 = 0x10;
const STANDALONE_CONVERTER_IS_AD_ONLY_FLAG: u32 = 0x02;

const MIN_SIZE: usize = 8;

fn serialize(params: &PfireSpecificParams, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= MIN_SIZE);

    let mut val = 0u32;
    params
        .knob_assigns
        .iter()
        .enumerate()
        .filter(|(_, &v)| v)
        .for_each(|(i, _)| val |= 1 << i);

    if params.opt_iface_b_mode == OptIfaceMode::Spdif {
        val |= OPT_IFACE_B_IS_SPDIF_FLAG;
    }
    val.build_quadlet(&mut raw[..4]);

    let mut val = 0u32;
    if params.standalone_mode == StandaloneConverterMode::AdOnly {
        val |= STANDALONE_CONVERTER_IS_AD_ONLY_FLAG;
    }
    val.build_quadlet(&mut raw[4..8]);

    Ok(())
}

fn deserialize(params: &mut PfireSpecificParams, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= MIN_SIZE);

    let mut val = 0u32;
    val.parse_quadlet(&raw[..4]);
    params
        .knob_assigns
        .iter_mut()
        .enumerate()
        .for_each(|(i, v)| *v = (val & KNOB_ASSIGN_MASK) & (1 << i) > 0);

    params.opt_iface_b_mode = if val & OPT_IFACE_B_IS_SPDIF_FLAG > 0 {
        OptIfaceMode::Spdif
    } else {
        OptIfaceMode::Adat
    };

    val.parse_quadlet(&raw[4..8]);
    params.standalone_mode = if val & STANDALONE_CONVERTER_IS_AD_ONLY_FLAG > 0 {
        StandaloneConverterMode::AdOnly
    } else {
        StandaloneConverterMode::AdDa
    };

    Ok(())
}

/// Protocol implementation specific to ProFire series.
pub trait PfireSpecificOperation {
    const HAS_OPT_IFACE_B: bool;
    const SUPPORT_STANDALONE_CONVERTER: bool;

    const KNOB_COUNT: usize = 4;

    fn cache_whole_params(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        params: &mut PfireSpecificParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0; sections.application.size];
        ApplSectionProtocol::read_appl_data(req, node, sections, 0, &mut raw, timeout_ms)?;
        deserialize(params, &raw).map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }

    fn update_partial_params(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        params: &PfireSpecificParams,
        prev: &mut PfireSpecificParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut new = vec![0; sections.application.size];
        serialize(params, &mut new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        let mut old = vec![0; sections.application.size];
        serialize(prev, &mut old)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        (0..sections.application.size)
            .step_by(4)
            .try_for_each(|pos| {
                if new[pos..(pos + 4)] != old[pos..(pos + 4)] {
                    ApplSectionProtocol::write_appl_data(
                        req,
                        node,
                        sections,
                        pos,
                        &mut new[pos..(pos + 4)],
                        timeout_ms,
                    )
                } else {
                    Ok(())
                }
            })?;

        deserialize(prev, &new).map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pfire_specific_params_serdes() {
        let params = PfireSpecificParams {
            knob_assigns: [false, true, true, false],
            opt_iface_b_mode: OptIfaceMode::Spdif,
            standalone_mode: StandaloneConverterMode::AdDa,
        };

        let mut raw = [0; MIN_SIZE];
        serialize(&params, &mut raw).unwrap();

        let mut p = PfireSpecificParams::default();
        deserialize(&mut p, &raw).unwrap();

        assert_eq!(params, p);
    }
}
