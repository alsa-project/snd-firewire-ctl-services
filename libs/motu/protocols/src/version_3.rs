// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol used in version 3 devices of MOTU FireWire series.
//!
//! The modules includes structure, enumeration, and trait and its implementation for protocol
//! used in version 3 devices of Mark of the Unicorn FireWire series.

use super::{command_dsp::*, register_dsp::*, *};

/// The enumeration to express source of sampling clock.
pub enum V3ClkSrc {
    /// Internal.
    Internal,
    /// S/PDIF on coaxial interface.
    SpdifCoax,
    /// Word clock on BNC interface.
    WordClk,
    /// AES/EBU on XLR interface.
    AesEbuXlr,
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
        (TargetPort::PhonePair, 0x01),     // = Stream-1/2
        (TargetPort::MainPair, 0x02),      // = Stream-5/6
        (TargetPort::AnalogPair(0), 0x06), // = Stream-3/4
        (TargetPort::SpdifPair, 0x07),     // = Stream-7/8
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

impl RegisterDspMixerReturnOperation for AudioExpressProtocol {}

impl RegisterDspMixerStereoSourceOperation for AudioExpressProtocol {}

impl RegisterDspOutputOperation for AudioExpressProtocol {}

impl RegisterDspStereoInputOperation for AudioExpressProtocol {
    const MIC_COUNT: usize = 2;
}

impl RegisterDspMeterOperation for AudioExpressProtocol {
    const SELECTABLE: bool = false;
    const INPUT_PORTS: &'static [TargetPort] = &[
        TargetPort::Analog(0),
        TargetPort::Analog(1),
        TargetPort::Analog(2),
        TargetPort::Analog(3),
        TargetPort::Analog(0),
        TargetPort::Analog(1),
        TargetPort::Analog(2),
        TargetPort::Analog(3),
        TargetPort::Spdif(0),
        TargetPort::Spdif(1),
    ];
    const OUTPUT_PORT_PAIRS: &'static [(TargetPort, [usize; 2])] = &[
        (TargetPort::PhonePair, [0, 1]),
        (TargetPort::MainPair, [2, 3]),
        (TargetPort::AnalogPair(0), [10, 11]),
        (TargetPort::SpdifPair, [12, 13]),
    ];
}

impl AudioExpressProtocol {
    /// Notification mask for footswitch.
    pub const NOTIFY_FOOTSWITCH_MASK: u32 = 0x01000000;
}

/// The protocol implementation for 828mk3 (FireWire only).
#[derive(Default)]
pub struct F828mk3Protocol;

const F828MK3_ASSIGN_PORTS: &[(TargetPort, u8)] = &[
    (TargetPort::MainPair, 0x00),        // = Stream-10/13
    (TargetPort::AnalogPair(0), 0x01),   // = Stream-2/3
    (TargetPort::AnalogPair(1), 0x02),   // = Stream-4/5
    (TargetPort::AnalogPair(2), 0x03),   // = Stream-6/7
    (TargetPort::AnalogPair(3), 0x04),   // = Stream-8/9
    (TargetPort::SpdifPair, 0x05),       // = Stream-12/13
    (TargetPort::PhonePair, 0x06),       // = Stream-0/1
    (TargetPort::OpticalAPair(0), 0x07), // = Stream-14/15
    (TargetPort::OpticalAPair(1), 0x08), // = Stream-16/17
    (TargetPort::OpticalAPair(2), 0x09), // = Stream-18/19
    (TargetPort::OpticalAPair(3), 0x0a), // = Stream-20/21
    (TargetPort::OpticalBPair(0), 0x0b), // = Stream-22/23
    (TargetPort::OpticalBPair(1), 0x0c), // = Stream-24/25
    (TargetPort::OpticalBPair(2), 0x0d), // = Stream-26/27
    (TargetPort::OpticalBPair(3), 0x0e), // = Stream-28/29
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
    TargetPort::MainPair,
    TargetPort::AnalogPair(0),
    TargetPort::AnalogPair(1),
    TargetPort::AnalogPair(2),
    TargetPort::AnalogPair(3),
    TargetPort::SpdifPair,
    TargetPort::PhonePair,
    TargetPort::OpticalAPair(0),
    TargetPort::OpticalAPair(1),
    TargetPort::OpticalAPair(2),
    TargetPort::OpticalAPair(3),
    TargetPort::OpticalBPair(0),
    TargetPort::OpticalBPair(1),
    TargetPort::OpticalBPair(2),
    TargetPort::OpticalBPair(3),
];

const F828MK3_MIXER_SOURCE_PORTS: &[TargetPort] = &[
    TargetPort::Analog(0), // Mic-0
    TargetPort::Analog(1), // Mic-1
    TargetPort::Analog(2),
    TargetPort::Analog(3),
    TargetPort::Analog(4),
    TargetPort::Analog(5),
    TargetPort::Analog(6),
    TargetPort::Analog(7),
    TargetPort::Analog(8),
    TargetPort::Analog(9),
    TargetPort::Spdif(0),
    TargetPort::Spdif(1),
    TargetPort::OpticalA(0),
    TargetPort::OpticalA(1),
    TargetPort::OpticalA(2),
    TargetPort::OpticalA(3),
    TargetPort::OpticalA(4),
    TargetPort::OpticalA(5),
    TargetPort::OpticalA(6),
    TargetPort::OpticalA(7),
    TargetPort::OpticalB(0),
    TargetPort::OpticalB(1),
    TargetPort::OpticalB(2),
    TargetPort::OpticalB(3),
    TargetPort::OpticalB(4),
    TargetPort::OpticalB(5),
    TargetPort::OpticalB(6),
    TargetPort::OpticalB(7),
];

const F828MK3_MIXER_OUTPUT_PORTS: &[TargetPort] = &[
    TargetPort::Disabled,
    TargetPort::MainPair,
    TargetPort::AnalogPair(0),
    TargetPort::AnalogPair(1),
    TargetPort::AnalogPair(2),
    TargetPort::AnalogPair(3),
    TargetPort::SpdifPair,
    TargetPort::PhonePair,
    TargetPort::OpticalAPair(0),
    TargetPort::OpticalAPair(1),
    TargetPort::OpticalAPair(2),
    TargetPort::OpticalAPair(3),
    TargetPort::OpticalBPair(0),
    TargetPort::OpticalBPair(1),
    TargetPort::OpticalBPair(2),
    TargetPort::OpticalBPair(3),
];

const F828MK3_INPUT_PORTS: &[TargetPort] = &[
    TargetPort::Analog(0), // Mic-0
    TargetPort::Analog(1), // Mic-1
    TargetPort::Analog(2),
    TargetPort::Analog(3),
    TargetPort::Analog(4),
    TargetPort::Analog(5),
    TargetPort::Analog(6),
    TargetPort::Analog(7),
    TargetPort::Analog(8),
    TargetPort::Analog(9),
    TargetPort::Spdif(0),
    TargetPort::Spdif(1),
    TargetPort::OpticalA(0),
    TargetPort::OpticalA(1),
    TargetPort::OpticalA(2),
    TargetPort::OpticalA(3),
    TargetPort::OpticalA(4),
    TargetPort::OpticalA(5),
    TargetPort::OpticalA(6),
    TargetPort::OpticalA(7),
    TargetPort::OpticalB(0),
    TargetPort::OpticalB(1),
    TargetPort::OpticalB(2),
    TargetPort::OpticalB(3),
    TargetPort::OpticalB(4),
    TargetPort::OpticalB(5),
    TargetPort::OpticalB(6),
    TargetPort::OpticalB(7),
];

const F828MK3_OUTPUT_PORTS: &[TargetPort] = &[
    TargetPort::MainPair,
    TargetPort::AnalogPair(0),
    TargetPort::AnalogPair(1),
    TargetPort::AnalogPair(2),
    TargetPort::AnalogPair(3),
    TargetPort::SpdifPair,
    TargetPort::PhonePair,
    TargetPort::OpticalAPair(0),
    TargetPort::OpticalAPair(1),
    TargetPort::OpticalAPair(2),
    TargetPort::OpticalAPair(3),
    TargetPort::OpticalBPair(0),
    TargetPort::OpticalBPair(1),
    TargetPort::OpticalBPair(2),
    TargetPort::OpticalBPair(3),
];

const F828MK3_METER_INPUT_PORTS: &[(TargetPort, usize)] = &[
    (TargetPort::Analog(0), 2),
    (TargetPort::Analog(1), 3),
    (TargetPort::Analog(2), 4),
    (TargetPort::Analog(3), 5),
    (TargetPort::Analog(4), 6),
    (TargetPort::Analog(5), 7),
    (TargetPort::Analog(6), 8),
    (TargetPort::Analog(7), 9),
    (TargetPort::Analog(8), 10),
    (TargetPort::Analog(9), 11),
    (TargetPort::Spdif(0), 12),
    (TargetPort::Spdif(1), 13),
    (TargetPort::OpticalA(0), 14),
    (TargetPort::OpticalA(1), 15),
    (TargetPort::OpticalA(2), 16),
    (TargetPort::OpticalA(3), 17),
    (TargetPort::OpticalA(4), 18),
    (TargetPort::OpticalA(5), 19),
    (TargetPort::OpticalA(6), 20),
    (TargetPort::OpticalA(7), 21),
    (TargetPort::OpticalB(0), 22),
    (TargetPort::OpticalB(1), 23),
    (TargetPort::OpticalB(2), 24),
    (TargetPort::OpticalB(3), 25),
    (TargetPort::OpticalB(4), 26),
    (TargetPort::OpticalB(5), 27),
    (TargetPort::OpticalB(6), 28),
    (TargetPort::OpticalB(7), 29),
    (TargetPort::Analog(0), 46),
    (TargetPort::Analog(1), 47),
    (TargetPort::Analog(2), 48),
    (TargetPort::Analog(3), 49),
    (TargetPort::Analog(4), 50),
    (TargetPort::Analog(5), 51),
    (TargetPort::Analog(6), 52),
    (TargetPort::Analog(7), 53),
    (TargetPort::Analog(8), 54),
    (TargetPort::Analog(9), 55),
    (TargetPort::Spdif(0), 56),
    (TargetPort::Spdif(1), 57),
    (TargetPort::OpticalA(0), 58),
    (TargetPort::OpticalA(1), 59),
    (TargetPort::OpticalA(2), 60),
    (TargetPort::OpticalA(3), 61),
    (TargetPort::OpticalA(4), 62),
    (TargetPort::OpticalA(5), 63),
    (TargetPort::OpticalA(6), 64),
    (TargetPort::OpticalA(7), 65),
    (TargetPort::OpticalB(0), 66),
    (TargetPort::OpticalB(1), 67),
    (TargetPort::OpticalB(2), 68),
    (TargetPort::OpticalB(3), 69),
    (TargetPort::OpticalB(4), 70),
    (TargetPort::OpticalB(5), 71),
    (TargetPort::OpticalB(6), 72),
    (TargetPort::OpticalB(7), 73),
];

const F828MK3_METER_OUTPUT_PORTS: &[(TargetPort, usize)] = &[
    (TargetPort::Phone(0), 86),
    (TargetPort::Phone(1), 87),
    (TargetPort::Analog(0), 76),
    (TargetPort::Analog(1), 77),
    (TargetPort::Analog(2), 78),
    (TargetPort::Analog(3), 79),
    (TargetPort::Analog(4), 80),
    (TargetPort::Analog(5), 81),
    (TargetPort::Analog(6), 82),
    (TargetPort::Analog(7), 83),
    (TargetPort::Main(0), 74),
    (TargetPort::Main(1), 75),
    (TargetPort::Spdif(0), 84),
    (TargetPort::Spdif(1), 85),
    (TargetPort::OpticalA(0), 88),
    (TargetPort::OpticalA(1), 89),
    (TargetPort::OpticalA(2), 90),
    (TargetPort::OpticalA(3), 91),
    (TargetPort::OpticalA(4), 92),
    (TargetPort::OpticalA(5), 93),
    (TargetPort::OpticalA(6), 94),
    (TargetPort::OpticalA(7), 95),
    (TargetPort::OpticalB(0), 96),
    (TargetPort::OpticalB(1), 97),
    (TargetPort::OpticalB(2), 98),
    (TargetPort::OpticalB(3), 99),
    (TargetPort::OpticalB(4), 100),
    (TargetPort::OpticalB(5), 101),
    (TargetPort::OpticalB(6), 102),
    (TargetPort::OpticalB(7), 103),
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

impl CommandDspMeterOperation for F828mk3Protocol {
    const INPUT_PORTS: &'static [(TargetPort, usize)] = F828MK3_METER_INPUT_PORTS;
    const OUTPUT_PORTS: &'static [(TargetPort, usize)] = F828MK3_METER_OUTPUT_PORTS;
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

impl CommandDspMeterOperation for F828mk3HybridProtocol {
    const INPUT_PORTS: &'static [(TargetPort, usize)] = F828MK3_METER_INPUT_PORTS;
    const OUTPUT_PORTS: &'static [(TargetPort, usize)] = F828MK3_METER_OUTPUT_PORTS;
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
        (TargetPort::PhonePair, 0x01),     // = Stream-1/2
        (TargetPort::MainPair, 0x02),      // = Stream-5/6
        (TargetPort::AnalogPair(0), 0x06), // = Stream-3/4
        (TargetPort::SpdifPair, 0x07),     // = Stream-7/8
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

impl RegisterDspMixerReturnOperation for H4preProtocol {}

impl RegisterDspMixerStereoSourceOperation for H4preProtocol {}

impl RegisterDspOutputOperation for H4preProtocol {}

impl RegisterDspStereoInputOperation for H4preProtocol {
    const MIC_COUNT: usize = 4;
}

impl RegisterDspMeterOperation for H4preProtocol {
    const SELECTABLE: bool = false;
    const INPUT_PORTS: &'static [TargetPort] = &[
        TargetPort::Analog(0),
        TargetPort::Analog(1),
        TargetPort::Analog(2),
        TargetPort::Analog(3),
        TargetPort::Analog(0),
        TargetPort::Analog(1),
        TargetPort::Analog(2),
        TargetPort::Analog(3),
        TargetPort::Spdif(0),
        TargetPort::Spdif(1),
    ];
    const OUTPUT_PORT_PAIRS: &'static [(TargetPort, [usize; 2])] = &[
        (TargetPort::PhonePair, [0, 1]),
        (TargetPort::MainPair, [2, 3]),
        (TargetPort::AnalogPair(0), [10, 11]),
        (TargetPort::SpdifPair, [12, 13]),
    ];
}

/// The protocol implementation for Ultralite mk3 (FireWire only).
#[derive(Default)]
pub struct UltraliteMk3Protocol;

const ULTRALITE_MK3_ASSIGN_PORTS: &[(TargetPort, u8)] = &[
    (TargetPort::MainPair, 0x00),      // = Stream-0/1
    (TargetPort::AnalogPair(0), 0x01), // = Stream-2/3
    (TargetPort::AnalogPair(1), 0x02), // = Stream-4/5
    (TargetPort::AnalogPair(2), 0x03), // = Stream-6/7
    (TargetPort::AnalogPair(3), 0x04), // = Stream-8/9
    (TargetPort::SpdifPair, 0x05),     // = Stream-12/13
    (TargetPort::PhonePair, 0x06),     // = Stream-10/11
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
    TargetPort::MainPair,
    TargetPort::AnalogPair(0),
    TargetPort::AnalogPair(1),
    TargetPort::AnalogPair(2),
    TargetPort::AnalogPair(3),
    TargetPort::SpdifPair,
    TargetPort::PhonePair,
];

const ULTRALITE_MK3_MIXER_SOURCE_PORTS: &[TargetPort] = &[
    TargetPort::Analog(0),
    TargetPort::Analog(1),
    TargetPort::Analog(2),
    TargetPort::Analog(3),
    TargetPort::Analog(4),
    TargetPort::Analog(5),
    TargetPort::Analog(6),
    TargetPort::Analog(7),
    TargetPort::Spdif(0),
    TargetPort::Spdif(1),
];

const ULTRALITE_MK3_MIXER_OUTPUT_PORTS: &[TargetPort] = &[
    TargetPort::MainPair,
    TargetPort::AnalogPair(0),
    TargetPort::AnalogPair(1),
    TargetPort::AnalogPair(2),
    TargetPort::AnalogPair(3),
    TargetPort::SpdifPair,
    TargetPort::PhonePair,
];

const ULTRALITE_MK3_INPUT_PORTS: &[TargetPort] = &[
    TargetPort::Analog(0),
    TargetPort::Analog(1),
    TargetPort::Analog(2),
    TargetPort::Analog(3),
    TargetPort::Analog(4),
    TargetPort::Analog(5),
    TargetPort::Analog(6),
    TargetPort::Analog(7),
    TargetPort::Spdif(0),
    TargetPort::Spdif(1),
];

const ULTRALITE_MK3_OUTPUT_PORTS: &[TargetPort] = &[
    TargetPort::MainPair,
    TargetPort::AnalogPair(0),
    TargetPort::AnalogPair(1),
    TargetPort::AnalogPair(2),
    TargetPort::AnalogPair(3),
    TargetPort::SpdifPair,
    TargetPort::PhonePair,
];

const ULTRALITEMK3_METER_INPUT_PORTS: &[(TargetPort, usize)] = &[
    (TargetPort::Analog(0), 2),
    (TargetPort::Analog(1), 3),
    (TargetPort::Analog(2), 4),
    (TargetPort::Analog(3), 5),
    (TargetPort::Analog(4), 6),
    (TargetPort::Analog(5), 7),
    (TargetPort::Analog(6), 8),
    (TargetPort::Analog(7), 9),
    (TargetPort::Spdif(0), 10),
    (TargetPort::Spdif(1), 11),
    (TargetPort::Analog(0), 28),
    (TargetPort::Analog(1), 29),
    (TargetPort::Analog(2), 30),
    (TargetPort::Analog(3), 31),
    (TargetPort::Analog(4), 32),
    (TargetPort::Analog(5), 33),
    (TargetPort::Analog(6), 34),
    (TargetPort::Analog(7), 35),
    (TargetPort::Spdif(0), 36),
    (TargetPort::Spdif(1), 37),
];
const ULTRALITEMK3_METER_OUTPUT_PORTS: &[(TargetPort, usize)] = &[
    (TargetPort::Spdif(0), 40),
    (TargetPort::Spdif(1), 41),
    (TargetPort::Analog(0), 42),
    (TargetPort::Analog(1), 43),
    (TargetPort::Analog(2), 44),
    (TargetPort::Analog(3), 45),
    (TargetPort::Analog(4), 46),
    (TargetPort::Analog(5), 47),
    (TargetPort::Analog(6), 48),
    (TargetPort::Analog(7), 49),
    (TargetPort::Phone(0), 50),
    (TargetPort::Phone(1), 51),
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

impl CommandDspMeterOperation for UltraliteMk3Protocol {
    const INPUT_PORTS: &'static [(TargetPort, usize)] = ULTRALITEMK3_METER_INPUT_PORTS;
    const OUTPUT_PORTS: &'static [(TargetPort, usize)] = ULTRALITEMK3_METER_OUTPUT_PORTS;
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

impl CommandDspMeterOperation for UltraliteMk3HybridProtocol {
    const INPUT_PORTS: &'static [(TargetPort, usize)] = ULTRALITEMK3_METER_INPUT_PORTS;
    const OUTPUT_PORTS: &'static [(TargetPort, usize)] = ULTRALITEMK3_METER_OUTPUT_PORTS;
}

impl UltraliteMk3HybridProtocol {
    /// Notification mask for main assignment, return assignment, and phone assignment. The change
    /// of phone assignment is also notified in command message.
    pub const NOTIFY_PORT_CHANGE: u32 = 0x40000000;
}

/// The protocol implementation for Traveler mk3.
#[derive(Default)]
pub struct TravelerMk3Protocol;

impl AssignOperation for TravelerMk3Protocol {
    const ASSIGN_PORTS: &'static [(TargetPort, u8)] = &[
        (TargetPort::AnalogPair(0), 0x00),   // = Stream-2/3
        (TargetPort::AnalogPair(1), 0x01),   // = Stream-4/5
        (TargetPort::AnalogPair(2), 0x02),   // = Stream-6/7
        (TargetPort::AnalogPair(3), 0x03),   // = Stream-8/9
        (TargetPort::AesEbuPair, 0x04),      // = Stream-10/11
        (TargetPort::SpdifPair, 0x05),       // = Stream-12/13
        (TargetPort::PhonePair, 0x06),       // = Stream-0/1
        (TargetPort::OpticalAPair(0), 0x07), // = Stream-14/15
        (TargetPort::OpticalAPair(1), 0x08), // = Stream-16/17
        (TargetPort::OpticalAPair(2), 0x09), // = Stream-18/19
        (TargetPort::OpticalAPair(3), 0x0a), // = Stream-20/21
        (TargetPort::OpticalBPair(0), 0x0b), // = Stream-22/23
        (TargetPort::OpticalBPair(1), 0x0c), // = Stream-24/25
        (TargetPort::OpticalBPair(2), 0x0d), // = Stream-26/27
        (TargetPort::OpticalBPair(3), 0x0e), // = Stream-28/29
    ];
}

impl V3ClkOperation for TravelerMk3Protocol {
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
        (V3ClkSrc::AesEbuXlr, 0x08),
        (V3ClkSrc::SpdifCoax, 0x10),
        (V3ClkSrc::SignalOptA, 0x18),
        (V3ClkSrc::SignalOptB, 0x19),
    ];
    const HAS_LCD: bool = true;
}

impl V3PortAssignOperation for TravelerMk3Protocol {}

impl V3OptIfaceOperation for TravelerMk3Protocol {}

impl WordClkOperation for TravelerMk3Protocol {}

impl CommandDspOperation for TravelerMk3Protocol {}

impl TravelerMk3Protocol {
    /// Notification mask for main assignment, return assignment, and phone assignment. The change
    /// of phone assignment is also notified in command message.
    pub const NOTIFY_PORT_CHANGE: u32 = 0x40000000;
}
