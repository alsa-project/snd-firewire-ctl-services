// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol used in version 3 devices of MOTU FireWire series.
//!
//! The modules includes structure, enumeration, and trait and its implementation for protocol
//! used in version 3 devices of Mark of the Unicorn FireWire series.

use glib::Error;

use hinawa::FwReq;

use super::{command_dsp::*, register_dsp::*, *};

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

    fn get_clk_rate(req: &mut FwReq, node: &mut FwNode, timeout_ms: u32) -> Result<usize, Error> {
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
        timeout_ms: u32,
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

    fn get_clk_src(req: &mut FwReq, node: &mut FwNode, timeout_ms: u32) -> Result<usize, Error> {
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
        timeout_ms: u32,
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
        timeout_ms: u32,
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
        timeout_ms: u32,
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
        timeout_ms: u32,
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
        timeout_ms: u32,
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
        timeout_ms: u32,
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
            (false, false) | (false, true) => V3OptIfaceMode::Disabled,
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
        (TargetPort::PhonePair0, 0x01),  // = Stream-1/2
        (TargetPort::MainPair0, 0x02),   // = Stream-5/6
        (TargetPort::AnalogPair0, 0x06), // = Stream-3/4
        (TargetPort::SpdifPair0, 0x07),  // = Stream-7/8
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

impl RegisterDspOutputOperation for AudioExpressProtocol {}

impl Audioexpress4preInputOperation for AudioExpressProtocol {
    const MIC_COUNT: usize = 2;
}

impl AudioExpressProtocol {
    /// Notification mask for footswitch.
    pub const NOTIFY_FOOTSWITCH_MASK: u32 = 0x01000000;
}

/// The protocol implementation for 828mk3 (FireWire only).
#[derive(Default)]
pub struct F828mk3Protocol;

const F828MK3_ASSIGN_PORTS: &[(TargetPort, u8)] = &[
    (TargetPort::MainPair0, 0x00),     // = Stream-11/12
    (TargetPort::AnalogPair0, 0x01),   // = Stream-3/4
    (TargetPort::AnalogPair1, 0x02),   // = Stream-5/6
    (TargetPort::AnalogPair2, 0x03),   // = Stream-7/8
    (TargetPort::AnalogPair3, 0x04),   // = Stream-9/10
    (TargetPort::SpdifPair0, 0x05),    // = Stream-13/14
    (TargetPort::PhonePair0, 0x06),    // = Stream-1/2
    (TargetPort::OpticalAPair0, 0x07), // = Stream-15/16
    (TargetPort::OpticalAPair1, 0x08), // = Stream-17/18
    (TargetPort::OpticalAPair2, 0x09), // = Stream-19/20
    (TargetPort::OpticalAPair3, 0x0a), // = Stream-21/22
    (TargetPort::OpticalBPair0, 0x0b), // = Stream-23/24
    (TargetPort::OpticalBPair1, 0x0c), // = Stream-25/26
    (TargetPort::OpticalBPair2, 0x0d), // = Stream-27/28
    (TargetPort::OpticalBPair3, 0x0e), // = Stream-29/30
];

const F828MK3_CLK_RATES: &[(ClkRate, u8)] = &[
    (ClkRate::R44100, 0x00),
    (ClkRate::R48000, 0x01),
    (ClkRate::R88200, 0x02),
    (ClkRate::R96000, 0x03),
    (ClkRate::R176400, 0x04),
    (ClkRate::R192000, 0x05),
];

const F828MK3_CLK_SRCS: &[(V3ClkSrc, u8)] = &[
    (V3ClkSrc::Internal, 0x00),
    (V3ClkSrc::WordClk, 0x01),
    (V3ClkSrc::SpdifCoax, 0x10),
    (V3ClkSrc::SignalOptA, 0x18),
    (V3ClkSrc::SignalOptB, 0x19),
];

const F828MK3_RETURN_ASSIGN_TARGETS: &[TargetPort] = &[
    TargetPort::MainPair0,
    TargetPort::AnalogPair0,
    TargetPort::AnalogPair1,
    TargetPort::AnalogPair2,
    TargetPort::AnalogPair3,
    TargetPort::SpdifPair0,
    TargetPort::PhonePair0,
    TargetPort::OpticalAPair0,
    TargetPort::OpticalAPair1,
    TargetPort::OpticalAPair2,
    TargetPort::OpticalAPair3,
    TargetPort::OpticalBPair0,
    TargetPort::OpticalBPair1,
    TargetPort::OpticalBPair2,
    TargetPort::OpticalBPair3,
];

const F828MK3_MIXER_SOURCE_PORTS: &[TargetPort] = &[
    TargetPort::Mic0,
    TargetPort::Mic1,
    TargetPort::Analog0,
    TargetPort::Analog1,
    TargetPort::Analog2,
    TargetPort::Analog3,
    TargetPort::Analog4,
    TargetPort::Analog5,
    TargetPort::Analog6,
    TargetPort::Analog7,
    TargetPort::Spdif0,
    TargetPort::Spdif1,
    TargetPort::OpticalA0,
    TargetPort::OpticalA1,
    TargetPort::OpticalA2,
    TargetPort::OpticalA3,
    TargetPort::OpticalA4,
    TargetPort::OpticalA5,
    TargetPort::OpticalA6,
    TargetPort::OpticalA7,
    TargetPort::OpticalB0,
    TargetPort::OpticalB1,
    TargetPort::OpticalB2,
    TargetPort::OpticalB3,
    TargetPort::OpticalB4,
    TargetPort::OpticalB5,
    TargetPort::OpticalB6,
    TargetPort::OpticalB7,
];

const F828MK3_MIXER_OUTPUT_PORTS: &[TargetPort] = &[
    TargetPort::Disabled,
    TargetPort::MainPair0,
    TargetPort::AnalogPair0,
    TargetPort::AnalogPair1,
    TargetPort::AnalogPair2,
    TargetPort::AnalogPair3,
    TargetPort::SpdifPair0,
    TargetPort::PhonePair0,
    TargetPort::OpticalAPair0,
    TargetPort::OpticalAPair1,
    TargetPort::OpticalAPair2,
    TargetPort::OpticalAPair3,
    TargetPort::OpticalBPair0,
    TargetPort::OpticalBPair1,
    TargetPort::OpticalBPair2,
    TargetPort::OpticalBPair3,
];

const F828MK3_INPUT_PORTS: &[TargetPort] = &[
    TargetPort::Mic0,
    TargetPort::Mic1,
    TargetPort::Analog0,
    TargetPort::Analog1,
    TargetPort::Analog2,
    TargetPort::Analog3,
    TargetPort::Analog4,
    TargetPort::Analog5,
    TargetPort::Analog6,
    TargetPort::Analog7,
    TargetPort::Spdif0,
    TargetPort::Spdif1,
    TargetPort::OpticalA0,
    TargetPort::OpticalA1,
    TargetPort::OpticalA2,
    TargetPort::OpticalA3,
    TargetPort::OpticalA4,
    TargetPort::OpticalA5,
    TargetPort::OpticalA6,
    TargetPort::OpticalA7,
    TargetPort::OpticalB0,
    TargetPort::OpticalB1,
    TargetPort::OpticalB2,
    TargetPort::OpticalB3,
    TargetPort::OpticalB4,
    TargetPort::OpticalB5,
    TargetPort::OpticalB6,
    TargetPort::OpticalB7,
];

const F828MK3_OUTPUT_PORTS: &[TargetPort] = &[
    TargetPort::MainPair0,
    TargetPort::AnalogPair0,
    TargetPort::AnalogPair1,
    TargetPort::AnalogPair2,
    TargetPort::AnalogPair3,
    TargetPort::SpdifPair0,
    TargetPort::PhonePair0,
    TargetPort::OpticalAPair0,
    TargetPort::OpticalAPair1,
    TargetPort::OpticalAPair2,
    TargetPort::OpticalAPair3,
    TargetPort::OpticalBPair0,
    TargetPort::OpticalBPair1,
    TargetPort::OpticalBPair2,
    TargetPort::OpticalBPair3,
];

impl AssignOperation for F828mk3Protocol {
    const ASSIGN_PORTS: &'static [(TargetPort, u8)] = F828MK3_ASSIGN_PORTS;
}

impl WordClkOperation for F828mk3Protocol {}

impl V3ClkOperation for F828mk3Protocol {
    const CLK_RATES: &'static [(ClkRate, u8)] = F828MK3_CLK_RATES;
    const CLK_SRCS: &'static [(V3ClkSrc, u8)] = F828MK3_CLK_SRCS;
    const HAS_LCD: bool = true;
}

impl V3PortAssignOperation for F828mk3Protocol {}

impl V3OptIfaceOperation for F828mk3Protocol {}

impl CommandDspOperation for F828mk3Protocol {}

impl CommandDspReverbOperation for F828mk3Protocol {}

impl CommandDspMonitorOperation for F828mk3Protocol {
    const RETURN_ASSIGN_TARGETS: &'static [TargetPort] = F828MK3_RETURN_ASSIGN_TARGETS;
}

impl CommandDspMixerOperation for F828mk3Protocol {
    const SOURCE_PORTS: &'static [TargetPort] = F828MK3_MIXER_SOURCE_PORTS;
    const OUTPUT_PORTS: &'static [TargetPort] = F828MK3_MIXER_OUTPUT_PORTS;
}

impl CommandDspInputOperation for F828mk3Protocol {
    const INPUT_PORTS: &'static [TargetPort] = F828MK3_INPUT_PORTS;
    const MIC_COUNT: usize = 0;
}

impl CommandDspOutputOperation for F828mk3Protocol {
    const OUTPUT_PORTS: &'static [TargetPort] = F828MK3_OUTPUT_PORTS;
}

impl F828mk3Protocol {
    /// Notification mask for main assignment, return assignment, and phone assignment. The change
    /// of phone assignment is also notified in command message.
    pub const NOTIFY_PORT_CHANGE: u32 = 0x40000000;

    /// Notification mask for footswitch.
    pub const NOTIFY_FOOTSWITCH_MASK: u32 = 0x01000000;
}

/// The protocol implementation for 828mk3 Hybrid.
#[derive(Default)]
pub struct F828mk3HybridProtocol;

impl AssignOperation for F828mk3HybridProtocol {
    const ASSIGN_PORTS: &'static [(TargetPort, u8)] = F828MK3_ASSIGN_PORTS;
}

impl WordClkOperation for F828mk3HybridProtocol {}

impl V3ClkOperation for F828mk3HybridProtocol {
    const CLK_RATES: &'static [(ClkRate, u8)] = F828MK3_CLK_RATES;
    const CLK_SRCS: &'static [(V3ClkSrc, u8)] = F828MK3_CLK_SRCS;
    const HAS_LCD: bool = true;
}

impl V3PortAssignOperation for F828mk3HybridProtocol {}

impl V3OptIfaceOperation for F828mk3HybridProtocol {}

impl CommandDspOperation for F828mk3HybridProtocol {}

impl CommandDspReverbOperation for F828mk3HybridProtocol {}

impl CommandDspMonitorOperation for F828mk3HybridProtocol {
    const RETURN_ASSIGN_TARGETS: &'static [TargetPort] = F828MK3_RETURN_ASSIGN_TARGETS;
}

impl CommandDspMixerOperation for F828mk3HybridProtocol {
    const SOURCE_PORTS: &'static [TargetPort] = F828MK3_MIXER_SOURCE_PORTS;
    const OUTPUT_PORTS: &'static [TargetPort] = F828MK3_MIXER_OUTPUT_PORTS;
}

impl CommandDspInputOperation for F828mk3HybridProtocol {
    const INPUT_PORTS: &'static [TargetPort] = F828MK3_INPUT_PORTS;
    // The mic functions are not configureble by command. They are just hard-wired.
    const MIC_COUNT: usize = 2;
}

impl CommandDspOutputOperation for F828mk3HybridProtocol {
    const OUTPUT_PORTS: &'static [TargetPort] = F828MK3_OUTPUT_PORTS;
}

impl F828mk3HybridProtocol {
    /// Notification mask for main assignment, return assignment, and phone assignment. The change
    /// of phone assignment is also notified in command message.
    pub const NOTIFY_PORT_CHANGE: u32 = 0x40000000;

    /// Notification mask for footswitch.
    pub const NOTIFY_FOOTSWITCH_MASK: u32 = 0x01000000;
}

/// The protocol implementation for 4pre.
#[derive(Default)]
pub struct H4preProtocol;

impl AssignOperation for H4preProtocol {
    const ASSIGN_PORTS: &'static [(TargetPort, u8)] = &[
        (TargetPort::PhonePair0, 0x01),  // = Stream-1/2
        (TargetPort::MainPair0, 0x02),   // = Stream-5/6
        (TargetPort::AnalogPair0, 0x06), // = Stream-3/4
        (TargetPort::SpdifPair0, 0x07),  // = Stream-7/8
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

impl RegisterDspOutputOperation for H4preProtocol {}

impl Audioexpress4preInputOperation for H4preProtocol {
    const MIC_COUNT: usize = 4;
}

/// The protocol implementation for Ultralite mk3 (FireWire only).
#[derive(Default)]
pub struct UltraliteMk3Protocol;

const ULTRALITE_MK3_ASSIGN_PORTS: &[(TargetPort, u8)] = &[
    (TargetPort::MainPair0, 0x00),   // = Stream-1/2
    (TargetPort::AnalogPair0, 0x01), // = Stream-3/4
    (TargetPort::AnalogPair1, 0x02), // = Stream-5/6
    (TargetPort::AnalogPair2, 0x03), // = Stream-7/8
    (TargetPort::AnalogPair3, 0x04), // = Stream-9/10
    (TargetPort::SpdifPair0, 0x05),  // = Stream-13/14
    (TargetPort::PhonePair0, 0x06),  // = Stream-11/12
];

const ULTRALITE_MK3_CLK_RATES: &[(ClkRate, u8)] = &[
    (ClkRate::R44100, 0x00),
    (ClkRate::R48000, 0x01),
    (ClkRate::R88200, 0x02),
    (ClkRate::R96000, 0x03),
];

const ULTRALITE_MK3_CLK_SRCS: &[(V3ClkSrc, u8)] =
    &[(V3ClkSrc::Internal, 0x00), (V3ClkSrc::SpdifCoax, 0x01)];

const ULTRALITE_MK3_RETURN_ASSIGN_TARGETS: &[TargetPort] = &[
    TargetPort::MainPair0,
    TargetPort::AnalogPair0,
    TargetPort::AnalogPair1,
    TargetPort::AnalogPair2,
    TargetPort::AnalogPair3,
    TargetPort::SpdifPair0,
    TargetPort::PhonePair0,
];

const ULTRALITE_MK3_MIXER_SOURCE_PORTS: &[TargetPort] = &[
    TargetPort::Analog0,
    TargetPort::Analog1,
    TargetPort::Analog2,
    TargetPort::Analog3,
    TargetPort::Analog4,
    TargetPort::Analog5,
    TargetPort::Analog6,
    TargetPort::Analog7,
    TargetPort::Spdif0,
    TargetPort::Spdif1,
];

const ULTRALITE_MK3_MIXER_OUTPUT_PORTS: &[TargetPort] = &[
    TargetPort::Disabled,
    TargetPort::MainPair0,
    TargetPort::AnalogPair0,
    TargetPort::AnalogPair1,
    TargetPort::AnalogPair2,
    TargetPort::AnalogPair3,
    TargetPort::SpdifPair0,
    TargetPort::PhonePair0,
];

const ULTRALITE_MK3_INPUT_PORTS: &[TargetPort] = &[
    TargetPort::Analog0,
    TargetPort::Analog1,
    TargetPort::Analog2,
    TargetPort::Analog3,
    TargetPort::Analog4,
    TargetPort::Analog5,
    TargetPort::Analog6,
    TargetPort::Analog7,
    TargetPort::Spdif0,
    TargetPort::Spdif1,
];

const ULTRALITE_MK3_OUTPUT_PORTS: &[TargetPort] = &[
    TargetPort::MainPair0,
    TargetPort::AnalogPair0,
    TargetPort::AnalogPair1,
    TargetPort::AnalogPair2,
    TargetPort::AnalogPair3,
    TargetPort::SpdifPair0,
    TargetPort::PhonePair0,
];

impl AssignOperation for UltraliteMk3Protocol {
    const ASSIGN_PORTS: &'static [(TargetPort, u8)] = ULTRALITE_MK3_ASSIGN_PORTS;
}

impl V3ClkOperation for UltraliteMk3Protocol {
    const CLK_RATES: &'static [(ClkRate, u8)] = ULTRALITE_MK3_CLK_RATES;
    const CLK_SRCS: &'static [(V3ClkSrc, u8)] = ULTRALITE_MK3_CLK_SRCS;
    const HAS_LCD: bool = true;
}

impl V3PortAssignOperation for UltraliteMk3Protocol {}

impl CommandDspOperation for UltraliteMk3Protocol {}

impl CommandDspReverbOperation for UltraliteMk3Protocol {}

impl CommandDspMonitorOperation for UltraliteMk3Protocol {
    const RETURN_ASSIGN_TARGETS: &'static [TargetPort] = ULTRALITE_MK3_RETURN_ASSIGN_TARGETS;
}

impl CommandDspMixerOperation for UltraliteMk3Protocol {
    const SOURCE_PORTS: &'static [TargetPort] = ULTRALITE_MK3_MIXER_SOURCE_PORTS;
    const OUTPUT_PORTS: &'static [TargetPort] = ULTRALITE_MK3_MIXER_OUTPUT_PORTS;
}

impl CommandDspInputOperation for UltraliteMk3Protocol {
    const INPUT_PORTS: &'static [TargetPort] = ULTRALITE_MK3_INPUT_PORTS;
    // The mic functions are not configureble by command. They are just hard-wired.
    const MIC_COUNT: usize = 0;
}

impl CommandDspOutputOperation for UltraliteMk3Protocol {
    const OUTPUT_PORTS: &'static [TargetPort] = ULTRALITE_MK3_OUTPUT_PORTS;
}

impl UltraliteMk3Protocol {
    /// Notification mask for main assignment, return assignment, and phone assignment. The change
    /// of phone assignment is also notified in command message.
    pub const NOTIFY_PORT_CHANGE: u32 = 0x40000000;
}

/// The protocol implementation for Ultralite mk3 Hybrid.
#[derive(Default)]
pub struct UltraliteMk3HybridProtocol;

impl AssignOperation for UltraliteMk3HybridProtocol {
    const ASSIGN_PORTS: &'static [(TargetPort, u8)] = ULTRALITE_MK3_ASSIGN_PORTS;
}

impl V3ClkOperation for UltraliteMk3HybridProtocol {
    const CLK_RATES: &'static [(ClkRate, u8)] = ULTRALITE_MK3_CLK_RATES;
    const CLK_SRCS: &'static [(V3ClkSrc, u8)] = ULTRALITE_MK3_CLK_SRCS;
    const HAS_LCD: bool = true;
}

impl V3PortAssignOperation for UltraliteMk3HybridProtocol {}

impl CommandDspOperation for UltraliteMk3HybridProtocol {}

impl CommandDspReverbOperation for UltraliteMk3HybridProtocol {}

impl CommandDspMonitorOperation for UltraliteMk3HybridProtocol {
    const RETURN_ASSIGN_TARGETS: &'static [TargetPort] = ULTRALITE_MK3_RETURN_ASSIGN_TARGETS;
}

impl CommandDspMixerOperation for UltraliteMk3HybridProtocol {
    const SOURCE_PORTS: &'static [TargetPort] = ULTRALITE_MK3_MIXER_SOURCE_PORTS;
    const OUTPUT_PORTS: &'static [TargetPort] = ULTRALITE_MK3_MIXER_OUTPUT_PORTS;
}

impl CommandDspInputOperation for UltraliteMk3HybridProtocol {
    const INPUT_PORTS: &'static [TargetPort] = ULTRALITE_MK3_INPUT_PORTS;
    const MIC_COUNT: usize = 2;
}

impl CommandDspOutputOperation for UltraliteMk3HybridProtocol {
    const OUTPUT_PORTS: &'static [TargetPort] = ULTRALITE_MK3_OUTPUT_PORTS;
}

impl UltraliteMk3HybridProtocol {
    /// Notification mask for main assignment, return assignment, and phone assignment. The change
    /// of phone assignment is also notified in command message.
    pub const NOTIFY_PORT_CHANGE: u32 = 0x40000000;
}
