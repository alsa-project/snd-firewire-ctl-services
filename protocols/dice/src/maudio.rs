// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Hardware specification and application protocol specific to M-Audio ProFire series.
//!
//! The modules includes structure, enumeration, and trait and its implementation for hardware
//! specification and application protocol specific to M-Audio ProFire series.
//!
//! ## Diagram of internal signal flow for Profire 2626
//!
//! ```text
//!
//! XLR input 1 -------+-------+
//! Phone input 1 -----+       |
//!                            |
//! XLR input 2 -------+-------+
//! Phone input 2 -----+       |
//!                            +----------------> analog-input-1/2
//! XLR input 3/4 ------------------------------> analog-input-3/4
//! XLR input 5/6 ------------------------------> analog-input-5/6
//! XLR input 7/8 ------------------------------> analog-input-7/8
//! Coaxial input ------------------------------> spdif-input-1/2
//! Optical input A ----------------------------> adat-input-1..8
//! Optical input B -----------or---------------> adat-input-8..16
//!                             +---------------> spdif-input-3/4
//!
//!                          ++=============++
//! analog-input-1/2 ------> || 70 x 68     || --> analog-output-1/2
//! analog-input-3/4 ------> || router      || --> analog-output-3/4
//! analog-input-5/6 ------> || up to       || --> analog-output-5/6
//! analog-input-7/8 ------> || 128 entries || --> analog-output-7/8
//!                          ||             ||
//! spdif-input-1/2 -------> ||             || --> spdif-output-1/2
//! spdif-input-3/4 -------> ||             || --> spdif-output-3/4
//!                          ||             ||
//! adat-input-1/2 --------> ||             || --> adat-output-1/2
//! adat-input-3/4 --------> ||             || --> adat-output-3/4
//! adat-input-5/6 --------> ||             || --> adat-output-5/6
//! adat-input-7/8 --------> ||             || --> adat-output-7/8
//!                          ||             ||
//! adat-input-9/10 -------> ||             || --> adat-output-9/10
//! adat-input-11/12 ------> ||             || --> adat-output-11/12
//! adat-input-13/14 ------> ||             || --> adat-output-13/14
//! adat-input-15/16 ------> ||             || --> adat-output-15/16
//!                          ||             ||
//! stream-input-A-1/2 ----> ||             || --> stream-output-A-1/2
//! stream-input-A-3/4 ----> ||             || --> stream-output-A-3/4
//! stream-input-A-5/6 ----> ||             || --> stream-output-A-5/6
//! stream-input-A-7/8 ----> ||             || --> stream-output-A-7/8
//! stream-input-A-9/10 ---> ||             || --> stream-output-A-9/10
//!                          ||             ||
//! stream-input-B-1/2 ----> ||             || --> stream-output-B-1/2
//! stream-input-B-3/4 ----> ||             || --> stream-output-B-3/4
//! stream-input-B-5/6 ----> ||             || --> stream-output-B-5/6
//! stream-input-B-7/8 ----> ||             || --> stream-output-B-7/8
//! stream-input-B-9/10 ---> ||             || --> stream-output-B-9/10
//! stream-input-B-11/12 --> ||             || --> stream-output-B-11/12
//! stream-input-B-13/14 --> ||             || --> stream-output-B-13/14
//! stream-input-B-15/16 --> ||             || --> stream-output-B-15/16
//!                          ||             ||
//! mixer-output-1/2 ------> ||             || --> mixer-input-1/2
//! mixer-output-3/4 ------> ||             || --> mixer-input-3/4
//! mixer-output-5/6 ------> ||             || --> mixer-input-5/6
//! mixer-output-7/8 ------> ||             || --> mixer-input-7/8
//! mixer-output-9/10 -----> ||             || --> mixer-input-9/10
//! mixer-output-11/12 ----> ||             || --> mixer-input-11/12
//! mixer-output-13/14 ----> ||             || --> mixer-input-13/14
//! mixer-output-15/16 ----> ||             || --> mixer-input-15/16
//!                          ||             || --> mixer-input-17/18
//!                          ++=============++
//!
//!                          ++============++
//! mixer-input-1/2 ----->   ||            || --> mixer-output-1/2
//! mixer-input-3/4 ----->   ||            || --> mixer-output-3/4
//! mixer-input-5/6 ----->   ||            || --> mixer-output-5/6
//! mixer-input-7/8 ----->   ||  18 x 16   || --> mixer-output-7/8
//! mixer-input-9/10 ---->   ||            || --> mixer-output-9/10
//! mixer-input-11/11 --->   ||   mixer    || --> mixer-output-11/12
//! mixer-input-13/14 --->   ||            || --> mixer-output-13/14
//! mixer-input-15/16 --->   ||            || --> mixer-output-15/16
//! mixer-input-17/18 --->   ||            ||
//!                          ++============++
//!
//! analog-output-1/2 ------------+-------------> Phone output 1/2
//!                               +-------------> Headphone output 1/2
//! analog-output-3/4 ------------+-------------> Phone output 1/2
//!                               +-------------> Headphone output 3/4
//! analog-output-5/6 --------------------------> Phone output 1/2
//! analog-output-7/8 --------------------------> Phone output 1/2
//!
//! spdif-output-1/2 ---------------------------> Coaxial output
//!
//! adat-output-1..8 ---------------------------> Optical A output
//!
//! adat-output-9..16 ------------or------------> Optical B output
//! spdif-output-3/4 -------------+
//!
//! ```
//!
//! ## Diagram of internal signal flow for Profire 610
//!
//! At 176.4/192.0 kHz, both stream-input-A and stream-input-B are available for
//! analog-output-1..6 and spdif-output-1/2 per each.
//!
//! ```text
//!
//! XLR input 1/2 ------------------------------> analog-input-1/2
//! Phone input 3/4 ----------------------------> analog-input-3/4
//! coaxial input 1/2 --------------------------> spdif-input-1/2
//!
//!                          ++=============++
//! analog-input-1/2 ------> || 32 x 30     || --> analog-output-1/2
//! analog-input-3/4 ------> || router      || --> analog-output-3/4
//!                          || up to       ||
//! spdif-input-1/2 -------> || 128 entries || --> spdif-output-1/2
//!                          ||             ||
//! stream-input-1/2 ------> ||             || --> stream-output-1/2
//! stream-input-3/4 ------> ||             || --> stream-output-3/4
//! stream-input-5/6 ------> ||             || --> stream-output-5/6
//! stream-input-7/8 ------> ||             ||
//! stream-input-9/10 -----> ||             ||
//!                          ||             ||
//! mixer-output-1/2 ------> ||             || --> mixer-input-1/2
//! mixer-output-3/4 ------> ||             || --> mixer-input-3/4
//! mixer-output-5/6 ------> ||             || --> mixer-input-5/6
//! mixer-output-7/8 ------> ||             || --> mixer-input-7/8
//! mixer-output-9/10 -----> ||             || --> mixer-input-9/10
//! mixer-output-11/12 ----> ||             || --> mixer-input-11/12
//! mixer-output-13/14 ----> ||             || --> mixer-input-13/14
//! mixer-output-15/16 ----> ||             || --> mixer-input-15/16
//!                          ||             || --> mixer-input-17/18
//!                          ++=============++
//!
//!                          ++============++
//! mixer-input-1/2 ----->   ||            || --> mixer-output-1/2
//! mixer-input-3/4 ----->   ||            || --> mixer-output-3/4
//! mixer-input-5/6 ----->   ||            || --> mixer-output-5/6
//! mixer-input-7/8 ----->   ||  18 x 16   || --> mixer-output-7/8
//! mixer-input-9/10 ---->   ||            || --> mixer-output-9/10
//! mixer-input-11/11 --->   ||   mixer    || --> mixer-output-11/12
//! mixer-input-13/14 --->   ||            || --> mixer-output-13/14
//! mixer-input-15/16 --->   ||            || --> mixer-output-15/16
//! mixer-input-17/18 --->   ||            ||
//!                          ++============++
//!
//! analog-output-1/2 ------------+-------------> Phone output 1/2
//!                               +-------------> Headphone output 1/2
//! analog-output-3/4 ------------+-------------> Phone output 3/4
//!                               +-------------> Headphone output 1/2
//!
//! spdif-output-1/2 ---------------------------> Coaxial output
//!

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

impl TcatExtensionOperation for Pfire2626Protocol {}

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
            id: SrcBlkId::Aes,
            offset: 0,
            count: 2,
            label: None,
        },
        Input {
            id: SrcBlkId::Adat,
            offset: 0,
            count: 8,
            label: None,
        },
        // NOTE: share the same optical interface.
        Input {
            id: SrcBlkId::Adat,
            offset: 8,
            count: 8,
            label: None,
        },
        Input {
            id: SrcBlkId::Aes,
            offset: 6,
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
            id: DstBlkId::Aes,
            offset: 0,
            count: 2,
            label: None,
        },
        Output {
            id: DstBlkId::Adat,
            offset: 0,
            count: 8,
            label: None,
        },
        // NOTE: share the same optical interface.
        Output {
            id: DstBlkId::Adat,
            offset: 8,
            count: 8,
            label: None,
        },
        Output {
            id: DstBlkId::Aes,
            offset: 6,
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

impl TcatExtensionOperation for Pfire610Protocol {}

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
