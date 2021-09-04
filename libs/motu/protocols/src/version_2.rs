// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol used in version 2 devices of MOTU FireWire series.
//!
//! The modules includes structure, enumeration, and trait and its implementation for protocol
//! used in version 2 devices of Mark of the Unicorn FireWire series.

use glib::Error;

use hinawa::FwReq;
use hinawa::SndMotu;

use super::*;

/// The enumeration to express source of sampling clock.
pub enum V2ClkSrc {
    /// Internal.
    Internal,
    /// S/PDIF on coaxial interface.
    SpdifCoax,
    /// Word clock on BNC interface.
    WordClk,
    /// Any signal on optical interface.
    SignalOpt,
    /// ADAT on optical interface.
    AdatOpt,
    /// ADAT on D-Sub interface.
    AdatDsub,
    /// AES/EBU on XLR interface.
    AesebuXlr,
}

const CLK_RATE_LABEL: &str = "clock-rate-v2";
const CLK_RATE_MASK: u32 = 0x00000038;
const CLK_RATE_SHIFT: usize = 3;

const CLK_SRC_LABEL: &str = "clock-source-v2";
const CLK_SRC_MASK: u32 = 0x00000007;
const CLK_SRC_SHIFT: usize = 0;

/// The trait for version 2 protocol.
pub trait V2ClkProtocol: CommonProtocol {
    const CLK_RATES: &'static [(ClkRate, u8)];
    const CLK_SRCS: &'static [(V2ClkSrc, u8)];

    const HAS_LCD: bool;

    fn get_clk_rate(&self, unit: &SndMotu, timeout_ms: u32) -> Result<usize, Error> {
        let vals: Vec<u8> = Self::CLK_RATES.iter().map(|e| e.1).collect();
        Self::get_idx_from_val(
            Self::OFFSET_CLK,
            CLK_RATE_MASK,
            CLK_RATE_SHIFT,
            CLK_RATE_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &vals,
            timeout_ms,
        )
    }

    fn set_clk_rate(&self, unit: &SndMotu, idx: usize, timeout_ms: u32) -> Result<(), Error> {
        let vals: Vec<u8> = Self::CLK_RATES.iter().map(|e| e.1).collect();
        Self::set_idx_to_val(
            Self::OFFSET_CLK,
            CLK_RATE_MASK,
            CLK_RATE_SHIFT,
            CLK_RATE_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &vals,
            idx,
            timeout_ms,
        )
    }

    fn get_clk_src(&self, unit: &SndMotu, timeout_ms: u32) -> Result<usize, Error> {
        let vals: Vec<u8> = Self::CLK_SRCS.iter().map(|e| e.1).collect();
        Self::get_idx_from_val(
            Self::OFFSET_CLK,
            CLK_SRC_MASK,
            CLK_SRC_SHIFT,
            CLK_SRC_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &vals,
            timeout_ms,
        )
    }

    fn set_clk_src(&self, unit: &SndMotu, idx: usize, timeout_ms: u32) -> Result<(), Error> {
        let vals: Vec<u8> = Self::CLK_SRCS.iter().map(|e| e.1).collect();
        Self::set_idx_to_val(
            Self::OFFSET_CLK,
            CLK_SRC_MASK,
            CLK_SRC_SHIFT,
            CLK_SRC_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &vals,
            idx,
            timeout_ms,
        )
    }
}

const MAIN_VOL_LABEL: &str = "main-vol-target-v2";
const MAIN_VOL_MASK: u32 = 0x000f0000;
const MAIN_VOL_SHIFT: usize = 16;

/// The trait for main volume knob assignment in version 2.
pub trait V2MainAssignProtocol: CommonProtocol {
    const KNOB_TARGETS: &'static [(&'static str, u8)];

    fn get_main_vol_assign(&self, unit: &SndMotu, timeout_ms: u32) -> Result<usize, Error> {
        let vals: Vec<u8> = Self::KNOB_TARGETS.iter().map(|e| e.1).collect();
        Self::get_idx_from_val(
            Self::OFFSET_PORT,
            MAIN_VOL_MASK,
            MAIN_VOL_SHIFT,
            MAIN_VOL_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &vals,
            timeout_ms,
        )
    }

    fn set_main_vol_assign(
        &self,
        unit: &SndMotu,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let vals: Vec<u8> = Self::KNOB_TARGETS.iter().map(|e| e.1).collect();
        Self::set_idx_to_val(
            Self::OFFSET_PORT,
            MAIN_VOL_MASK,
            MAIN_VOL_SHIFT,
            MAIN_VOL_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            &vals,
            idx,
            timeout_ms,
        )
    }
}

/// The enumeration to express mode of optical interface.
pub enum V2OptIfaceMode {
    None,
    Adat,
    Spdif,
}

const OPT_IN_IFACE_LABEL: &str = "optical-input-iface-v2";
const OPT_IN_IFACE_MASK: u32 = 0x00000300;
const OPT_IN_IFACE_SHIFT: usize = 8;

const OPT_OUT_IFACE_LABEL: &str = "optical-output-iface-v2";
const OPT_OUT_IFACE_MASK: u32 = 0x00000c00;
const OPT_OUT_IFACE_SHIFT: usize = 10;

const OPT_IFACE_MODE_VALS: &[u8] = &[0x00, 0x01, 0x02];

/// The trait for optical interface mode in version 2.
pub trait V2OptIfaceProtocol: CommonProtocol {
    const OPT_IFACE_MODES: &'static [(V2OptIfaceMode, u8)];

    fn get_opt_in_iface_mode(&self, unit: &SndMotu, timeout_ms: u32) -> Result<usize, Error> {
        Self::get_idx_from_val(
            Self::OFFSET_PORT,
            OPT_IN_IFACE_MASK,
            OPT_IN_IFACE_SHIFT,
            OPT_IN_IFACE_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            OPT_IFACE_MODE_VALS,
            timeout_ms,
        )
    }

    fn set_opt_in_iface_mode(
        &self,
        unit: &SndMotu,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::set_idx_to_val(
            Self::OFFSET_PORT,
            OPT_IN_IFACE_MASK,
            OPT_IN_IFACE_SHIFT,
            OPT_IN_IFACE_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            OPT_IFACE_MODE_VALS,
            idx,
            timeout_ms,
        )
    }

    fn get_opt_out_iface_mode(&self, unit: &SndMotu, timeout_ms: u32) -> Result<usize, Error> {
        Self::get_idx_from_val(
            Self::OFFSET_PORT,
            OPT_OUT_IFACE_MASK,
            OPT_OUT_IFACE_SHIFT,
            OPT_OUT_IFACE_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            OPT_IFACE_MODE_VALS,
            timeout_ms,
        )
    }

    fn set_opt_out_iface_mode(
        &self,
        unit: &SndMotu,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::set_idx_to_val(
            Self::OFFSET_PORT,
            OPT_OUT_IFACE_MASK,
            OPT_OUT_IFACE_SHIFT,
            OPT_OUT_IFACE_LABEL,
            self.as_ref(),
            &mut unit.get_node(),
            OPT_IFACE_MODE_VALS,
            idx,
            timeout_ms,
        )
    }
}

/// The protocol implementation for 828mkII.
#[derive(Default)]
pub struct F828mk2Protocol(FwReq);

impl AsRef<FwReq> for F828mk2Protocol {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

impl CommonProtocol for F828mk2Protocol {}

impl AssignProtocol for F828mk2Protocol {
    const ASSIGN_PORTS: &'static [(&'static str, u8)] = &[
        ("Phone-1/2", 0x01),  // = Stream-1/2
        ("Analog-1/2", 0x02), // = Stream-3/4
        ("Analog-3/4", 0x03), // = Stream-5/6
        ("Analog-5/6", 0x04), // = Stream-7/8
        ("Analog-7/8", 0x05), // = Stream-9/10
        ("Main-1/2", 0x06),   // = Stream-11/12
        ("S/PDIF-1/2", 0x07), // = Stream-13/14
        ("ADAT-1/2", 0x08),   // = Stream-15/16
        ("ADAT-3/4", 0x09),   // = Stream-17/18
        ("ADAT-5/6", 0x0a),   // = Stream-19/20
        ("ADAT-7/8", 0x0b),   // = Stream-21/22
    ];
}

impl WordClkProtocol for F828mk2Protocol {}

impl V2ClkProtocol for F828mk2Protocol {
    const CLK_RATES: &'static [(ClkRate, u8)] = &[
        (ClkRate::R44100, 0x00),
        (ClkRate::R48000, 0x01),
        (ClkRate::R88200, 0x02),
        (ClkRate::R96000, 0x03),
    ];

    const CLK_SRCS: &'static [(V2ClkSrc, u8)] = &[
        (V2ClkSrc::Internal, 0x00),
        (V2ClkSrc::SignalOpt, 0x01),
        (V2ClkSrc::SpdifCoax, 0x02),
        (V2ClkSrc::WordClk, 0x04),
        (V2ClkSrc::AdatDsub, 0x05),
    ];

    const HAS_LCD: bool = true;
}

impl V2OptIfaceProtocol for F828mk2Protocol {
    const OPT_IFACE_MODES: &'static [(V2OptIfaceMode, u8)] = &[
        (V2OptIfaceMode::None, 0x00),
        (V2OptIfaceMode::Adat, 0x01),
        (V2OptIfaceMode::Spdif, 0x02),
    ];
}

/// The protocol implementation for 8pre.
#[derive(Default)]
pub struct F8preProtocol(FwReq);

impl AsRef<FwReq> for F8preProtocol {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

impl CommonProtocol for F8preProtocol {}

impl AssignProtocol for F8preProtocol {
    const ASSIGN_PORTS: &'static [(&'static str, u8)] = &[("Phone-1/2", 0x01), ("Main-1/2", 0x02)];
}

impl V2ClkProtocol for F8preProtocol {
    const CLK_RATES: &'static [(ClkRate, u8)] = &[
        (ClkRate::R44100, 0x00),
        (ClkRate::R48000, 0x01),
        (ClkRate::R88200, 0x02),
        (ClkRate::R96000, 0x03),
    ];

    const CLK_SRCS: &'static [(V2ClkSrc, u8)] = &[(V2ClkSrc::Internal, 0x00), (V2ClkSrc::AdatOpt, 0x01)];

    const HAS_LCD: bool = false;
}

impl V2OptIfaceProtocol for F8preProtocol {
    const OPT_IFACE_MODES: &'static [(V2OptIfaceMode, u8)] =
        &[(V2OptIfaceMode::None, 0x00), (V2OptIfaceMode::Adat, 0x01)];
}

/// The protocol implementation for Traveler.
#[derive(Default)]
pub struct TravelerProtocol(FwReq);

impl AsRef<FwReq> for TravelerProtocol {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

impl CommonProtocol for TravelerProtocol {}

impl AssignProtocol for TravelerProtocol {
    const ASSIGN_PORTS: &'static [(&'static str, u8)] = &[
        ("Phone-1/2", 0x01),   // = Stream-1/2
        ("Analog-1/2", 0x02),  // = Stream-3/4
        ("Analog-3/4", 0x03),  // = Stream-5/6
        ("Analog-5/6", 0x04),  // = Stream-7/8
        ("Analog-7/8", 0x05),  // = Stream-9/10
        ("AES/EBU-1/2", 0x06), // = Stream-11/12
        ("S/PDIF-1/2", 0x07),  // = Stream-13/14
        ("ADAT-1/2", 0x08),    // = Stream-15/16
        ("ADAT-3/4", 0x09),    // = Stream-17/18
        ("ADAT-5/6", 0x0a),    // = Stream-19/20
        ("ADAT-7/8", 0x0b),    // = Stream-21/22
    ];
}

impl WordClkProtocol for TravelerProtocol {}

impl V2ClkProtocol for TravelerProtocol {
    const CLK_RATES: &'static [(ClkRate, u8)] = &[
        (ClkRate::R44100, 0x00),
        (ClkRate::R48000, 0x01),
        (ClkRate::R88200, 0x02),
        (ClkRate::R96000, 0x03),
        (ClkRate::R176400, 0x04),
        (ClkRate::R192000, 0x05),
    ];

    const CLK_SRCS: &'static [(V2ClkSrc, u8)] = &[
        (V2ClkSrc::Internal, 0x00),
        (V2ClkSrc::SignalOpt, 0x01),
        (V2ClkSrc::SpdifCoax, 0x02),
        (V2ClkSrc::WordClk, 0x04),
        (V2ClkSrc::AdatDsub, 0x05),
        (V2ClkSrc::AesebuXlr, 0x07),
    ];

    const HAS_LCD: bool = true;
}

impl V2OptIfaceProtocol for TravelerProtocol {
    const OPT_IFACE_MODES: &'static [(V2OptIfaceMode, u8)] = &[
        (V2OptIfaceMode::None, 0x00),
        (V2OptIfaceMode::Adat, 0x01),
        (V2OptIfaceMode::Spdif, 0x02),
    ];
}

/// The protocol implementation for Ultralite.
#[derive(Default)]
pub struct UltraliteProtocol(FwReq);

impl AsRef<FwReq> for UltraliteProtocol {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

impl CommonProtocol for UltraliteProtocol {}

impl AssignProtocol for UltraliteProtocol {
    const ASSIGN_PORTS: &'static [(&'static str, u8)] = &[
        ("Phone-1/2", 0x01),  // Stream-1/2
        ("Analog-1/2", 0x02), // Stream-3/4
        ("Analog-3/4", 0x03), // Stream-5/6
        ("Analog-5/6", 0x04), // Stream-7/8
        ("Analog-7/8", 0x05), // Stream-9/10
        ("Main-1/2", 0x06),   // Stream-11/12
        ("S/PDIF-1/2", 0x07), // Stream-13/14
    ];
}

impl V2ClkProtocol for UltraliteProtocol {
    const CLK_RATES: &'static [(ClkRate, u8)] = &[
        (ClkRate::R44100, 0x00),
        (ClkRate::R48000, 0x01),
        (ClkRate::R88200, 0x02),
        (ClkRate::R96000, 0x03),
    ];

    const CLK_SRCS: &'static [(V2ClkSrc, u8)] =
        &[(V2ClkSrc::Internal, 0x00), (V2ClkSrc::SpdifCoax, 0x02)];

    const HAS_LCD: bool = true;
}

impl V2MainAssignProtocol for UltraliteProtocol {
    const KNOB_TARGETS: &'static [(&'static str, u8)] = &[
        ("Main-out-1/2", 0x00),
        ("Analog-1/2/3/4/5/6", 0x01),
        ("Analog-1/2/3/4/5/6/7/8", 0x02),
        ("S/PDIF-1/2", 0x03),
    ];
}

/// The protocol implementation for 896HD.
#[derive(Default)]
pub struct F896hdProtocol(FwReq);

impl AsRef<FwReq> for F896hdProtocol {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

impl CommonProtocol for F896hdProtocol {}

impl AssignProtocol for F896hdProtocol {
    const ASSIGN_PORTS: &'static [(&'static str, u8)] = &[
        ("Phone-1/2", 0x01),
        ("Analog-1/2", 0x02),   // Stream-1/2
        ("Analog-3/4", 0x03),   // Stream-3/4
        ("Analog-5/6", 0x04),   // Stream-5/6
        ("Analog-7/8", 0x05),   // Stream-7/8
        ("Main-out-1/2", 0x06), // Stream-9/10
        ("AES/EBU-1/2", 0x07),  // Stream-11/12
        ("ADAT-1/2", 0x08),     // Stream-13/14
        ("ADAT-3/4", 0x09),     // Stream-15/16
        ("ADAT-5/6", 0x0a),     // Stream-17/18
        ("ADAT-7/8", 0x0b),     // Stream-19/20
    ];
}

impl WordClkProtocol for F896hdProtocol {}

impl AesebuRateConvertProtocol for F896hdProtocol {
    const AESEBU_RATE_CONVERT_MASK: u32 = 0x00000300;
    const AESEBU_RATE_CONVERT_SHIFT: usize = 8;
}

impl LevelMetersProtocol for F896hdProtocol {}

impl V2ClkProtocol for F896hdProtocol {
    const CLK_RATES: &'static [(ClkRate, u8)] = &[
        (ClkRate::R44100, 0x00),
        (ClkRate::R48000, 0x01),
        (ClkRate::R88200, 0x02),
        (ClkRate::R96000, 0x03),
        (ClkRate::R176400, 0x04),
        (ClkRate::R192000, 0x05),
    ];

    const CLK_SRCS: &'static [(V2ClkSrc, u8)] = &[
        (V2ClkSrc::Internal, 0x00),
        (V2ClkSrc::AdatOpt, 0x01),
        (V2ClkSrc::AesebuXlr, 0x02),
        (V2ClkSrc::WordClk, 0x04),
        (V2ClkSrc::AdatDsub, 0x05),
    ];

    const HAS_LCD: bool = false;
}

impl V2OptIfaceProtocol for F896hdProtocol {
    const OPT_IFACE_MODES: &'static [(V2OptIfaceMode, u8)] =
        &[(V2OptIfaceMode::None, 0x00), (V2OptIfaceMode::Adat, 0x01)];
}
