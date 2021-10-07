// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol used in version 3 devices of MOTU FireWire series.
//!
//! The modules includes structure, enumeration, and trait and its implementation for protocol
//! used in version 3 devices of Mark of the Unicorn FireWire series.

use glib::Error;

use hinawa::FwReq;

use super::{register_dsp::*, *};

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
pub trait V3ClkOperation {
    const CLK_RATES: &'static [(ClkRate, u8)];
    const CLK_SRCS: &'static [(V3ClkSrc, u8)];

    const HAS_LCD: bool;

    fn get_clk_rate(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32
    ) -> Result<usize, Error> {
        let vals: Vec<u8> = Self::CLK_RATES.iter().map(|e| e.1).collect();
        get_idx_from_val(
            OFFSET_CLK,
            CLK_RATE_MASK,
            CLK_RATE_SHIFT,
            CLK_RATE_LABEL,
            req,
            node,
            &vals,
            timeout_ms,
        )
    }

    fn set_clk_rate(
        req: &mut FwReq,
        node: &mut FwNode,
        idx: usize,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let vals: Vec<u8> = Self::CLK_RATES.iter().map(|e| e.1).collect();
        set_idx_to_val(
            OFFSET_CLK,
            CLK_RATE_MASK,
            CLK_RATE_SHIFT,
            CLK_RATE_LABEL,
            req,
            node,
            &vals,
            idx,
            timeout_ms,
        )
    }

    fn get_clk_src(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32
    ) -> Result<usize, Error> {
        let vals: Vec<u8> = Self::CLK_SRCS.iter().map(|e| e.1).collect();
        get_idx_from_val(
            OFFSET_CLK,
            CLK_SRC_MASK,
            CLK_SRC_SHIFT,
            CLK_SRC_LABEL,
            req,
            node,
            &vals,
            timeout_ms,
        )
    }

    fn set_clk_src(
        req: &mut FwReq,
        node: &mut FwNode,
        idx: usize,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let vals: Vec<u8> = Self::CLK_SRCS.iter().map(|e| e.1).collect();
        set_idx_to_val(
            OFFSET_CLK,
            CLK_SRC_MASK,
            CLK_SRC_SHIFT,
            CLK_SRC_LABEL,
            req,
            node,
            &vals,
            idx,
            timeout_ms,
        )
    }

    fn update_clk_display(
        req: &mut FwReq,
        node: &mut FwNode,
        label: &str,
        timeout_ms: u32
    ) -> Result<(), Error> {
        update_clk_display(req, node, label, timeout_ms)
    }
}

const PORT_MAIN_LABEL: &str = "main-out-assign-v3";
const PORT_MAIN_MASK: u32 = 0x000000f0;
const PORT_MAIN_SHIFT: usize = 4;

const PORT_RETURN_LABEL: &str = "return-assign-v3";
const PORT_RETURN_MASK: u32 = 0x00000f00;
const PORT_RETURN_SHIFT: usize = 8;

/// The trait for main/return assignment protocol in version 3.
pub trait V3PortAssignOperation: AssignOperation {
    fn get_main_assign(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32
    ) -> Result<usize, Error> {
        let vals: Vec<u8> = Self::ASSIGN_PORTS.iter().map(|e| e.1).collect();
        get_idx_from_val(
            OFFSET_PORT,
            PORT_MAIN_MASK,
            PORT_MAIN_SHIFT,
            PORT_MAIN_LABEL,
            req,
            node,
            &vals,
            timeout_ms,
        )
    }

    fn set_main_assign(
        req: &mut FwReq,
        node: &mut FwNode,
        idx: usize,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let vals: Vec<u8> = Self::ASSIGN_PORTS.iter().map(|e| e.1).collect();
        set_idx_to_val(
            OFFSET_PORT,
            PORT_MAIN_MASK,
            PORT_MAIN_SHIFT,
            PORT_MAIN_LABEL,
            req,
            node,
            &vals,
            idx,
            timeout_ms,
        )
    }

    fn get_return_assign(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32
    ) -> Result<usize, Error> {
        let vals: Vec<u8> = Self::ASSIGN_PORTS.iter().map(|e| e.1).collect();
        get_idx_from_val(
            OFFSET_PORT,
            PORT_RETURN_MASK,
            PORT_RETURN_SHIFT,
            PORT_RETURN_LABEL,
            req,
            node,
            &vals,
            timeout_ms,
        )
    }

    fn set_return_assign(
        req: &mut FwReq,
        node: &mut FwNode,
        idx: usize,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let vals: Vec<u8> = Self::ASSIGN_PORTS.iter().map(|e| e.1).collect();
        set_idx_to_val(
            OFFSET_PORT,
            PORT_RETURN_MASK,
            PORT_RETURN_SHIFT,
            PORT_RETURN_LABEL,
            req,
            node,
            &vals,
            idx,
            timeout_ms,
        )
    }
}

/// The enumeration for direction of optical interface.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum V3OptIfaceTarget {
    A,
    B,
}

impl Default for V3OptIfaceTarget {
    fn default() -> Self {
        Self::A
    }
}

/// The enumeration for mode of optical interface.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum V3OptIfaceMode {
    Disabled,
    Adat,
    Spdif,
}

impl Default for V3OptIfaceMode {
    fn default() -> Self {
        Self::Disabled
    }
}

fn get_opt_iface_masks(target: V3OptIfaceTarget, is_out: bool) -> (u32, u32) {
    let mut enabled_mask = 0x00000001;
    if is_out {
        enabled_mask <<= 8;
    }
    if target == V3OptIfaceTarget::B {
        enabled_mask <<= 1;
    }

    let mut no_adat_mask = 0x00010000;
    if is_out {
        no_adat_mask <<= 2;
    }
    if target == V3OptIfaceTarget::B {
        no_adat_mask <<= 4;
    }

    (enabled_mask, no_adat_mask)
}

const OFFSET_OPT: u32 = 0x0c94;

fn set_opt_iface_mode(
    req: &mut FwReq,
    node: &mut FwNode,
    target: V3OptIfaceTarget,
    is_out: bool,
    mode: V3OptIfaceMode,
    timeout_ms: u32,
) -> Result<(), Error> {
    let (enabled_mask, no_adat_mask) = get_opt_iface_masks(target, is_out);
    read_quad(req, node, OFFSET_OPT, timeout_ms).and_then(|mut quad| {
        match mode {
            V3OptIfaceMode::Disabled => {
                quad &= !enabled_mask;
                quad &= !no_adat_mask;
            }
            V3OptIfaceMode::Adat => {
                quad |= enabled_mask;
                quad &= !no_adat_mask;
            }
            V3OptIfaceMode::Spdif => {
                quad |= enabled_mask;
                quad |= no_adat_mask;
            }
        }
        write_quad(req, node, OFFSET_OPT, quad, timeout_ms)
    })
}

fn get_opt_iface_mode(
    req: &mut FwReq,
    node: &mut FwNode,
    target: V3OptIfaceTarget,
    is_out: bool,
    timeout_ms: u32,
) -> Result<V3OptIfaceMode, Error> {
    read_quad(req, node, OFFSET_OPT, timeout_ms).map(|quad| {
        let (enabled_mask, no_adat_mask) = get_opt_iface_masks(target, is_out);
        match (quad & enabled_mask > 0, quad & no_adat_mask > 0) {
            (false, false) |
            (false, true) => V3OptIfaceMode::Disabled,
            (true, false) => V3OptIfaceMode::Adat,
            (true, true) => V3OptIfaceMode::Spdif,
        }
    })
}

/// The trait for optical interface protocol in version 3.
pub trait V3OptIfaceOperation {
    fn set_opt_input_iface_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        target: V3OptIfaceTarget,
        mode: V3OptIfaceMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        set_opt_iface_mode(req, node, target, false, mode, timeout_ms)
    }

    fn get_opt_input_iface_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        target: V3OptIfaceTarget,
        timeout_ms: u32,
    ) -> Result<V3OptIfaceMode, Error> {
        get_opt_iface_mode(req, node, target, false, timeout_ms)
    }

    fn set_opt_output_iface_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        target: V3OptIfaceTarget,
        mode: V3OptIfaceMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        set_opt_iface_mode(req, node, target, true, mode, timeout_ms)
    }

    fn get_opt_output_iface_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        target: V3OptIfaceTarget,
        timeout_ms: u32,
    ) -> Result<V3OptIfaceMode, Error> {
        get_opt_iface_mode(req, node, target, true, timeout_ms)
    }
}

/// The protocol implementation for Audio Express.
#[derive(Default)]
pub struct AudioExpressProtocol;

impl AssignOperation for AudioExpressProtocol {
    const ASSIGN_PORTS: &'static [(TargetPort, u8)] = &[
        (TargetPort::PhonePair0, 0x01), // = Stream-1/2
        (TargetPort::MainPair0, 0x02), // = Stream-5/6
        (TargetPort::AnalogPair0, 0x06), // = Stream-3/4
        (TargetPort::SpdifPair0, 0x07), // = Stream-7/8
                              // Blank for Stream-9/10
    ];
}

impl V3ClkOperation for AudioExpressProtocol {
    const CLK_RATES: &'static [(ClkRate, u8)] = &[
        (ClkRate::R44100, 0x00),
        (ClkRate::R48000, 0x01),
        (ClkRate::R88200, 0x02),
        (ClkRate::R96000, 0x03),
    ];

    const CLK_SRCS: &'static [(V3ClkSrc, u8)] =
        &[(V3ClkSrc::Internal, 0x00), (V3ClkSrc::SpdifCoax, 0x01)];

    const HAS_LCD: bool = false;
}

impl RegisterDspMixerOutputOperation for AudioExpressProtocol {
    const OUTPUT_DESTINATIONS: &'static [TargetPort] = &[];
}

impl RegisterDspMixerReturnOperation for AudioExpressProtocol {
    const RETURN_SOURCES: &'static [TargetPort] = &[];
}

impl RegisterDspMixerStereoSourceOperation for AudioExpressProtocol {}

/// The protocol implementation for 828mk3.
#[derive(Default)]
pub struct F828mk3Protocol;

impl AssignOperation for F828mk3Protocol {
    const ASSIGN_PORTS: &'static [(TargetPort, u8)] = &[
        (TargetPort::MainPair0, 0x00), // = Stream-11/12
        (TargetPort::AnalogPair0, 0x01), // = Stream-3/4
        (TargetPort::AnalogPair1, 0x02), // = Stream-5/6
        (TargetPort::AnalogPair2, 0x03), // = Stream-7/8
        (TargetPort::AnalogPair3, 0x04), // = Stream-9/10
        (TargetPort::SpdifPair0, 0x05), // = Stream-13/14
        (TargetPort::PhonePair0, 0x06), // = Stream-1/2
        (TargetPort::OpticalAPair0, 0x07), // = Stream-15/16
        (TargetPort::OpticalAPair1, 0x08), // = Stream-17/18
        (TargetPort::OpticalAPair2, 0x09), // = Stream-19/20
        (TargetPort::OpticalAPair3, 0x0a), // = Stream-21/22
        (TargetPort::OpticalBPair0, 0x0b), // = Stream-23/24
        (TargetPort::OpticalBPair1, 0x0c), // = Stream-25/26
        (TargetPort::OpticalBPair2, 0x0d), // = Stream-27/28
        (TargetPort::OpticalBPair3, 0x0e), // = Stream-29/30
    ];
}

impl WordClkOperation for F828mk3Protocol {}

impl V3ClkOperation for F828mk3Protocol {
    const CLK_RATES: &'static [(ClkRate, u8)] = &[
        (ClkRate::R44100, 0x00),
        (ClkRate::R48000, 0x01),
        (ClkRate::R88200, 0x02),
        (ClkRate::R96000, 0x03),
        (ClkRate::R176400, 0x04),
        (ClkRate::R192000, 0x05),
    ];

    const CLK_SRCS: &'static [(V3ClkSrc, u8)] = &[
        (V3ClkSrc::Internal, 0x00),
        (V3ClkSrc::WordClk, 0x01),
        (V3ClkSrc::SpdifCoax, 0x10),
        (V3ClkSrc::SignalOptA, 0x18),
        (V3ClkSrc::SignalOptB, 0x19),
    ];

    const HAS_LCD: bool = true;
}

impl V3PortAssignOperation for F828mk3Protocol {}

impl V3OptIfaceOperation for F828mk3Protocol {}

/// The protocol implementation for 4pre.
#[derive(Default)]
pub struct H4preProtocol;

impl AssignOperation for H4preProtocol {
    const ASSIGN_PORTS: &'static [(TargetPort, u8)] = &[
        (TargetPort::PhonePair0, 0x01), // = Stream-1/2
        (TargetPort::MainPair0, 0x02), // = Stream-5/6
        (TargetPort::AnalogPair0, 0x06), // = Stream-3/4
        (TargetPort::SpdifPair0, 0x07), // = Stream-7/8
                              // Blank for Stream-9/10
    ];
}

impl V3ClkOperation for H4preProtocol {
    const CLK_RATES: &'static [(ClkRate, u8)] = &[
        (ClkRate::R44100, 0x00),
        (ClkRate::R48000, 0x01),
        (ClkRate::R88200, 0x02),
        (ClkRate::R96000, 0x03),
    ];

    const CLK_SRCS: &'static [(V3ClkSrc, u8)] =
        &[(V3ClkSrc::Internal, 0x00), (V3ClkSrc::SpdifCoax, 0x01)];

    const HAS_LCD: bool = false;
}

impl RegisterDspMixerOutputOperation for H4preProtocol {
    const OUTPUT_DESTINATIONS: &'static [TargetPort] = &[];
}

impl RegisterDspMixerReturnOperation for H4preProtocol {
    const RETURN_SOURCES: &'static [TargetPort] = &[];
}

impl RegisterDspMixerStereoSourceOperation for H4preProtocol {}

/// The protocol implementation for Ultralite mk3.
#[derive(Default)]
pub struct UltraliteMk3Protocol;

impl AssignOperation for UltraliteMk3Protocol {
    const ASSIGN_PORTS: &'static [(TargetPort, u8)] = &[
        (TargetPort::MainPair0, 0x00), // = Stream-1/2
        (TargetPort::AnalogPair0, 0x01), // = Stream-3/4
        (TargetPort::AnalogPair1, 0x02), // = Stream-5/6
        (TargetPort::AnalogPair2, 0x03), // = Stream-7/8
        (TargetPort::AnalogPair3, 0x04), // = Stream-9/10
        (TargetPort::SpdifPair0, 0x05), // = Stream-13/14
        (TargetPort::PhonePair0, 0x06), // = Stream-11/12
    ];
}

impl V3ClkOperation for UltraliteMk3Protocol {
    const CLK_RATES: &'static [(ClkRate, u8)] = &[
        (ClkRate::R44100, 0x00),
        (ClkRate::R48000, 0x01),
        (ClkRate::R88200, 0x02),
        (ClkRate::R96000, 0x03),
    ];

    const CLK_SRCS: &'static [(V3ClkSrc, u8)] =
        &[(V3ClkSrc::Internal, 0x00), (V3ClkSrc::SpdifCoax, 0x01)];

    const HAS_LCD: bool = true;
}

impl V3PortAssignOperation for UltraliteMk3Protocol {}
