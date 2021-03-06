// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol used in version 3 devices of MOTU FireWire series.
//!
//! The modules includes structure, enumeration, and trait and its implementation for protocol
//! used in version 3 devices of Mark of the Unicorn FireWire series.

use glib::Error;

use hinawa::FwReq;
use hinawa::SndMotu;

use super::*;

/// The enumeration to express source of sampling clock.
pub enum V3ClkSrc {
    /// Internal.
    Internal,
    /// S/PDIF on coaxial interface.
    SpdifCoax,
    /// Word clock on BNC interface.
    WordClk,
    /// Any signal on optical interface A.
    SignalOptA,
    /// Any signal on optical interface B.
    SignalOptB,
}

const CLK_RATE_LABEL: &str = "clock-rate-v3";
const CLK_RATE_MASK: u32 = 0x0000ff00;
const CLK_RATE_SHIFT: usize = 8;

const CLK_SRC_LABEL: &str = "clock-source-v3";
const CLK_SRC_MASK: u32 = 0x000000ff;
const CLK_SRC_SHIFT: usize = 0;

/// The trait for sampling clock protocol in version 3.
pub trait V3ClkProtocol<'a>: CommonProtocol<'a> {
    const CLK_RATES: &'a [(ClkRate, u8)];
    const CLK_SRCS: &'a [(V3ClkSrc, u8)];

    const HAS_LCD: bool;

    fn get_clk_rate(&self, unit: &SndMotu, timeout_ms: u32) -> Result<usize, Error> {
        let vals: Vec<u8> = Self::CLK_RATES.iter().map(|e| e.1).collect();
        self.get_idx_from_val(
            Self::OFFSET_CLK,
            CLK_RATE_MASK,
            CLK_RATE_SHIFT,
            CLK_RATE_LABEL,
            unit,
            &vals,
            timeout_ms,
        )
    }

    fn set_clk_rate(&self, unit: &SndMotu, idx: usize, timeout_ms: u32) -> Result<(), Error> {
        let vals: Vec<u8> = Self::CLK_RATES.iter().map(|e| e.1).collect();
        self.set_idx_to_val(
            Self::OFFSET_CLK,
            CLK_RATE_MASK,
            CLK_RATE_SHIFT,
            CLK_RATE_LABEL,
            unit,
            &vals,
            idx,
            timeout_ms,
        )
    }

    fn get_clk_src(&self, unit: &SndMotu, timeout_ms: u32) -> Result<usize, Error> {
        let vals: Vec<u8> = Self::CLK_SRCS.iter().map(|e| e.1).collect();
        self.get_idx_from_val(
            Self::OFFSET_CLK,
            CLK_SRC_MASK,
            CLK_SRC_SHIFT,
            CLK_SRC_LABEL,
            unit,
            &vals,
            timeout_ms,
        )
    }

    fn set_clk_src(&self, unit: &SndMotu, idx: usize, timeout_ms: u32) -> Result<(), Error> {
        let vals: Vec<u8> = Self::CLK_SRCS.iter().map(|e| e.1).collect();
        self.set_idx_to_val(
            Self::OFFSET_CLK,
            CLK_SRC_MASK,
            CLK_SRC_SHIFT,
            CLK_SRC_LABEL,
            unit,
            &vals,
            idx,
            timeout_ms,
        )
    }
}

const PORT_MAIN_LABEL: &str = "main-out-assign-v3";
const PORT_MAIN_MASK: u32 = 0x000000f0;
const PORT_MAIN_SHIFT: usize = 4;

const PORT_RETURN_LABEL: &str = "return-assign-v3";
const PORT_RETURN_MASK: u32 = 0x00000f00;
const PORT_RETURN_SHIFT: usize = 8;

/// The trait for main/return assignment protocol in version 3.
pub trait V3PortAssignProtocol<'a>: AssignProtocol<'a> {
    fn get_main_assign(&self, unit: &SndMotu, timeout_ms: u32) -> Result<usize, Error> {
        let vals: Vec<u8> = Self::ASSIGN_PORTS.iter().map(|e| e.1).collect();
        self.get_idx_from_val(
            Self::OFFSET_PORT,
            PORT_MAIN_MASK,
            PORT_MAIN_SHIFT,
            PORT_MAIN_LABEL,
            unit,
            &vals,
            timeout_ms,
        )
    }

    fn set_main_assign(&self, unit: &SndMotu, idx: usize, timeout_ms: u32) -> Result<(), Error> {
        let vals: Vec<u8> = Self::ASSIGN_PORTS.iter().map(|e| e.1).collect();
        self.set_idx_to_val(
            Self::OFFSET_PORT,
            PORT_MAIN_MASK,
            PORT_MAIN_SHIFT,
            PORT_MAIN_LABEL,
            unit,
            &vals,
            idx,
            timeout_ms,
        )
    }

    fn get_return_assign(&self, unit: &SndMotu, timeout_ms: u32) -> Result<usize, Error> {
        let vals: Vec<u8> = Self::ASSIGN_PORTS.iter().map(|e| e.1).collect();
        self.get_idx_from_val(
            Self::OFFSET_PORT,
            PORT_RETURN_MASK,
            PORT_RETURN_SHIFT,
            PORT_RETURN_LABEL,
            unit,
            &vals,
            timeout_ms,
        )
    }

    fn set_return_assign(&self, unit: &SndMotu, idx: usize, timeout_ms: u32) -> Result<(), Error> {
        let vals: Vec<u8> = Self::ASSIGN_PORTS.iter().map(|e| e.1).collect();
        self.set_idx_to_val(
            Self::OFFSET_PORT,
            PORT_RETURN_MASK,
            PORT_RETURN_SHIFT,
            PORT_RETURN_LABEL,
            unit,
            &vals,
            idx,
            timeout_ms,
        )
    }
}

const OFFSET_OPT: u32 = 0x0c94;

/// The trait for optical interface protocol in version 3.
pub trait V3OptIfaceProtocol<'a>: CommonProtocol<'a> {
    fn get_opt_iface_masks(&self, is_out: bool, is_b: bool) -> (u32, u32) {
        let mut enabled_mask = 0x00000001;
        if is_out {
            enabled_mask <<= 8;
        }
        if is_b {
            enabled_mask <<= 1;
        }

        let mut no_adat_mask = 0x00010000;
        if is_out {
            no_adat_mask <<= 2;
        }
        if is_b {
            no_adat_mask <<= 4;
        }

        (enabled_mask, no_adat_mask)
    }

    fn set_opt_iface_mode(
        &self,
        unit: &SndMotu,
        is_out: bool,
        is_b: bool,
        enable: bool,
        no_adat: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let (enabled_mask, no_adat_mask) = self.get_opt_iface_masks(is_out, is_b);
        self.read_quad(unit, OFFSET_OPT, timeout_ms)
            .and_then(|mut quad| {
                quad &= !enabled_mask;
                quad &= !no_adat_mask;
                if enable {
                    quad |= enabled_mask;
                }
                if no_adat {
                    quad |= no_adat_mask;
                }
                self.write_quad(unit, OFFSET_OPT, quad, timeout_ms)
            })
    }

    fn get_opt_iface_mode(
        &self,
        unit: &SndMotu,
        is_out: bool,
        is_b: bool,
        timeout_ms: u32,
    ) -> Result<(bool, bool), Error> {
        self.read_quad(unit, OFFSET_OPT, timeout_ms).map(|quad| {
            let (enabled_mask, no_adat_mask) = self.get_opt_iface_masks(is_out, is_b);
            let enabled = (quad & enabled_mask) > 0;
            let no_adat = (quad & no_adat_mask) > 0;

            (enabled, no_adat)
        })
    }
}

/// The protocol implementation for Audio Express.
#[derive(Default)]
pub struct AudioExpressProtocol(FwReq);

impl AsRef<FwReq> for AudioExpressProtocol {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

impl<'a> CommonProtocol<'a> for AudioExpressProtocol {}

impl<'a> AssignProtocol<'a> for AudioExpressProtocol {
    const ASSIGN_PORTS: &'a [(&'a str, u8)] = &[
        ("Phone-1/2", 0x01),  // = Stream-1/2
        ("Main-1/2", 0x02),   // = Stream-5/6
        ("Andlog-1/2", 0x06), // = Stream-3/4
        ("S/PDIF-1/2", 0x07), // = Stream-7/8
                              // Blank for Stream-9/10
    ];
}

impl<'a> V3ClkProtocol<'a> for AudioExpressProtocol {
    const CLK_RATES: &'a [(ClkRate, u8)] = &[
        (ClkRate::R44100, 0x00),
        (ClkRate::R48000, 0x01),
        (ClkRate::R88200, 0x02),
        (ClkRate::R96000, 0x03),
    ];

    const CLK_SRCS: &'a [(V3ClkSrc, u8)] =
        &[(V3ClkSrc::Internal, 0x00), (V3ClkSrc::SpdifCoax, 0x01)];

    const HAS_LCD: bool = false;
}

/// The protocol implementation for 828mk3.
#[derive(Default)]
pub struct F828mk3Protocol(FwReq);

impl AsRef<FwReq> for F828mk3Protocol {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

impl<'a> CommonProtocol<'a> for F828mk3Protocol {}

impl<'a> AssignProtocol<'a> for F828mk3Protocol {
    const ASSIGN_PORTS: &'a [(&'a str, u8)] = &[
        ("Main-1/2", 0x00),      // = Stream-11/12
        ("Analog-1/2", 0x01),    // = Stream-3/4
        ("Analog-3/4", 0x02),    // = Stream-5/6
        ("Analog-5/6", 0x03),    // = Stream-7/8
        ("Analog-7/8", 0x04),    // = Stream-9/10
        ("S/PDIF-1/2", 0x05),    // = Stream-13/14
        ("Phone-1/2", 0x06),     // = Stream-1/2
        ("Optical-A-1/2", 0x07), // = Stream-15/16
        ("Optical-A-3/4", 0x08), // = Stream-17/18
        ("Optical-A-5/6", 0x09), // = Stream-19/20
        ("Optical-A-7/8", 0x0a), // = Stream-21/22
        ("Optical-B-1/2", 0x0b), // = Stream-23/24
        ("Optical-B-3/4", 0x0c), // = Stream-25/26
        ("Optical-B-5/6", 0x0d), // = Stream-27/28
        ("Optical-B-7/8", 0x0e), // = Stream-29/30
    ];
}

impl<'a> WordClkProtocol<'a> for F828mk3Protocol {}

impl<'a> V3ClkProtocol<'a> for F828mk3Protocol {
    const CLK_RATES: &'a [(ClkRate, u8)] = &[
        (ClkRate::R44100, 0x00),
        (ClkRate::R48000, 0x01),
        (ClkRate::R88200, 0x02),
        (ClkRate::R96000, 0x03),
        (ClkRate::R176400, 0x04),
        (ClkRate::R192000, 0x05),
    ];

    const CLK_SRCS: &'a [(V3ClkSrc, u8)] = &[
        (V3ClkSrc::Internal, 0x00),
        (V3ClkSrc::WordClk, 0x01),
        (V3ClkSrc::SpdifCoax, 0x10),
        (V3ClkSrc::SignalOptA, 0x18),
        (V3ClkSrc::SignalOptB, 0x19),
    ];

    const HAS_LCD: bool = true;
}

impl<'a> V3PortAssignProtocol<'a> for F828mk3Protocol {}

impl<'a> V3OptIfaceProtocol<'a> for F828mk3Protocol {}

/// The protocol implementation for 4pre.
#[derive(Default)]
pub struct H4preProtocol(FwReq);

impl AsRef<FwReq> for H4preProtocol {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

impl<'a> CommonProtocol<'a> for H4preProtocol {}

impl<'a> AssignProtocol<'a> for H4preProtocol {
    const ASSIGN_PORTS: &'a [(&'a str, u8)] = &[
        ("Phone-1/2", 0x01),  // = Stream-1/2
        ("Main-1/2", 0x02),   // = Stream-5/6
        ("Andlog-1/2", 0x06), // = Stream-3/4
        ("S/PDIF-1/2", 0x07), // = Stream-7/8
                              // Blank for Stream-9/10
    ];
}

impl<'a> V3ClkProtocol<'a> for H4preProtocol {
    const CLK_RATES: &'a [(ClkRate, u8)] = &[
        (ClkRate::R44100, 0x00),
        (ClkRate::R48000, 0x01),
        (ClkRate::R88200, 0x02),
        (ClkRate::R96000, 0x03),
    ];

    const CLK_SRCS: &'a [(V3ClkSrc, u8)] =
        &[(V3ClkSrc::Internal, 0x00), (V3ClkSrc::SpdifCoax, 0x01)];

    const HAS_LCD: bool = false;
}

/// The protocol implementation for Ultralite mk3.
#[derive(Default)]
pub struct UltraliteMk3Protocol(FwReq);

impl AsRef<FwReq> for UltraliteMk3Protocol {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

impl<'a> CommonProtocol<'a> for UltraliteMk3Protocol {}

impl<'a> AssignProtocol<'a> for UltraliteMk3Protocol {
    const ASSIGN_PORTS: &'a [(&'a str, u8)] = &[
        ("Main-1/2", 0x00),   // = Stream-1/2
        ("Analog-1/2", 0x01), // = Stream-3/4
        ("Analog-3/4", 0x02), // = Stream-5/6
        ("Analog-5/6", 0x03), // = Stream-7/8
        ("Analog-7/8", 0x04), // = Stream-9/10
        ("S/PDIF-1/2", 0x05), // = Stream-13/14
        ("Phone-1/2", 0x06),  // = Stream-11/12
    ];
}

impl<'a> V3ClkProtocol<'a> for UltraliteMk3Protocol {
    const CLK_RATES: &'a [(ClkRate, u8)] = &[
        (ClkRate::R44100, 0x00),
        (ClkRate::R48000, 0x01),
        (ClkRate::R88200, 0x02),
        (ClkRate::R96000, 0x03),
    ];

    const CLK_SRCS: &'a [(V3ClkSrc, u8)] =
        &[(V3ClkSrc::Internal, 0x00), (V3ClkSrc::SpdifCoax, 0x01)];

    const HAS_LCD: bool = true;
}

impl<'a> V3PortAssignProtocol<'a> for UltraliteMk3Protocol {}
