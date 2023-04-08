// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol used in version 2 devices of MOTU FireWire series.
//!
//! The modules includes structure, enumeration, and trait and its implementation for protocol
//! used in version 2 devices of Mark of the Unicorn FireWire series.

use super::{register_dsp::*, *};

/// Signal source of sampling clock.
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
pub trait V2ClkOperation {
    const CLK_RATES: &'static [(ClkRate, u8)];
    const CLK_SRCS: &'static [(V2ClkSrc, u8)];

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

/// Mode of optical interface.
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
pub trait V2OptIfaceOperation {
    const OPT_IFACE_MODES: &'static [(V2OptIfaceMode, u8)];

    fn get_opt_in_iface_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<usize, Error> {
        get_idx_from_val(
            OFFSET_PORT,
            OPT_IN_IFACE_MASK,
            OPT_IN_IFACE_SHIFT,
            OPT_IN_IFACE_LABEL,
            req,
            node,
            OPT_IFACE_MODE_VALS,
            timeout_ms,
        )
    }

    fn set_opt_in_iface_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        set_idx_to_val(
            OFFSET_PORT,
            OPT_IN_IFACE_MASK,
            OPT_IN_IFACE_SHIFT,
            OPT_IN_IFACE_LABEL,
            req,
            node,
            OPT_IFACE_MODE_VALS,
            idx,
            timeout_ms,
        )
    }

    fn get_opt_out_iface_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<usize, Error> {
        get_idx_from_val(
            OFFSET_PORT,
            OPT_OUT_IFACE_MASK,
            OPT_OUT_IFACE_SHIFT,
            OPT_OUT_IFACE_LABEL,
            req,
            node,
            OPT_IFACE_MODE_VALS,
            timeout_ms,
        )
    }

    fn set_opt_out_iface_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        set_idx_to_val(
            OFFSET_PORT,
            OPT_OUT_IFACE_MASK,
            OPT_OUT_IFACE_SHIFT,
            OPT_OUT_IFACE_LABEL,
            req,
            node,
            OPT_IFACE_MODE_VALS,
            idx,
            timeout_ms,
        )
    }
}

/// The protocol implementation for 828mkII.
#[derive(Default)]
pub struct F828mk2Protocol;

impl AssignOperation for F828mk2Protocol {
    const ASSIGN_PORTS: &'static [(TargetPort, u8)] = &[
        (TargetPort::PhonePair, 0x01),     // = Stream-0/1
        (TargetPort::AnalogPair(0), 0x02), // = Stream-2/3
        (TargetPort::AnalogPair(1), 0x03), // = Stream-4/5
        (TargetPort::AnalogPair(2), 0x04), // = Stream-6/7
        (TargetPort::AnalogPair(3), 0x05), // = Stream-8/9
        (TargetPort::MainPair, 0x06),      // = Stream-10/11
        (TargetPort::SpdifPair, 0x07),     // = Stream-12/13
        (TargetPort::AdatPair(0), 0x08),   // = Stream-14/15
        (TargetPort::AdatPair(1), 0x09),   // = Stream-16/17
        (TargetPort::AdatPair(2), 0x0a),   // = Stream-18/19
        (TargetPort::AdatPair(3), 0x0b),   // = Stream-20/21
    ];
}

impl MotuWordClockOutputSpecification for F828mk2Protocol {}

impl MotuClockNameDisplaySpecification for F828mk2Protocol {}

impl V2ClkOperation for F828mk2Protocol {
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

impl V2OptIfaceOperation for F828mk2Protocol {
    const OPT_IFACE_MODES: &'static [(V2OptIfaceMode, u8)] = &[
        (V2OptIfaceMode::None, 0x00),
        (V2OptIfaceMode::Adat, 0x01),
        (V2OptIfaceMode::Spdif, 0x02),
    ];
}

impl RegisterDspMixerOutputOperation for F828mk2Protocol {
    const OUTPUT_DESTINATIONS: &'static [TargetPort] = &[
        TargetPort::Disabled,
        TargetPort::PhonePair,
        TargetPort::AnalogPair(0),
        TargetPort::AnalogPair(1),
        TargetPort::AnalogPair(2),
        TargetPort::AnalogPair(3),
        TargetPort::MainPair,
        TargetPort::SpdifPair,
        TargetPort::AdatPair(0),
        TargetPort::AdatPair(1),
        TargetPort::AdatPair(2),
        TargetPort::AdatPair(3),
    ];
}
impl RegisterDspMixerReturnOperation for F828mk2Protocol {}

impl RegisterDspMixerMonauralSourceOperation for F828mk2Protocol {
    const MIXER_SOURCES: &'static [TargetPort] = &[
        TargetPort::Analog(0),
        TargetPort::Analog(1),
        TargetPort::Analog(2),
        TargetPort::Analog(3),
        TargetPort::Analog(4),
        TargetPort::Analog(5),
        TargetPort::Analog(6),
        TargetPort::Analog(7),
        TargetPort::Analog(8), // Mic-0
        TargetPort::Analog(9), // Mic-1
        TargetPort::Spdif(0),
        TargetPort::Spdif(1),
        TargetPort::Adat(0),
        TargetPort::Adat(1),
        TargetPort::Adat(2),
        TargetPort::Adat(3),
        TargetPort::Adat(4),
        TargetPort::Adat(5),
        TargetPort::Adat(6),
        TargetPort::Adat(7),
    ];
}

impl RegisterDspOutputOperation for F828mk2Protocol {}

impl Traveler828mk2LineInputOperation for F828mk2Protocol {
    const LINE_INPUT_COUNT: usize = 8;
    const CH_OFFSET: usize = 0;
}

impl F828mk2Protocol {
    /// Notification mask for speed of word clock output, and phone assignment. The change of phone
    /// assignment is also notified in message delivered by the sequence of isochronous packets.
    pub const NOTIFY_PORT_CHANGE: u32 = 0x40000000;

    /// Notification mask for footswitch.
    pub const NOTIFY_FOOTSWITCH_MASK: u32 = 0x01000000;
}

impl RegisterDspMeterOperation for F828mk2Protocol {
    const SELECTABLE: bool = true;
    const INPUT_PORTS: &'static [TargetPort] = &[
        TargetPort::Analog(0),
        TargetPort::Analog(1),
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
        TargetPort::Adat(0),
        TargetPort::Adat(1),
        TargetPort::Adat(2),
        TargetPort::Adat(3),
        TargetPort::Adat(4),
        TargetPort::Adat(5),
        TargetPort::Adat(6),
        TargetPort::Adat(7),
    ];
    const OUTPUT_PORT_PAIRS: &'static [(TargetPort, [usize; 2])] = &[
        (TargetPort::PhonePair, [2, 3]),
        (TargetPort::AnalogPair(0), [4, 5]),
        (TargetPort::AnalogPair(1), [6, 7]),
        (TargetPort::AnalogPair(2), [8, 9]),
        (TargetPort::AnalogPair(3), [10, 11]),
        (TargetPort::MainPair, [12, 13]),
        (TargetPort::SpdifPair, [14, 15]),
        (TargetPort::AdatPair(0), [16, 17]),
        (TargetPort::AdatPair(1), [18, 19]),
        (TargetPort::AdatPair(2), [20, 21]),
        (TargetPort::AdatPair(3), [22, 23]),
    ];
}

/// The protocol implementation for 8pre.
#[derive(Default)]
pub struct F8preProtocol;

impl AssignOperation for F8preProtocol {
    const ASSIGN_PORTS: &'static [(TargetPort, u8)] =
        &[(TargetPort::PhonePair, 0x01), (TargetPort::MainPair, 0x02)];
}

impl V2ClkOperation for F8preProtocol {
    const CLK_RATES: &'static [(ClkRate, u8)] = &[
        (ClkRate::R44100, 0x00),
        (ClkRate::R48000, 0x01),
        (ClkRate::R88200, 0x02),
        (ClkRate::R96000, 0x03),
    ];

    const CLK_SRCS: &'static [(V2ClkSrc, u8)] =
        &[(V2ClkSrc::Internal, 0x00), (V2ClkSrc::AdatOpt, 0x01)];

    const HAS_LCD: bool = false;
}

impl V2OptIfaceOperation for F8preProtocol {
    const OPT_IFACE_MODES: &'static [(V2OptIfaceMode, u8)] =
        &[(V2OptIfaceMode::None, 0x00), (V2OptIfaceMode::Adat, 0x01)];
}

impl RegisterDspMixerOutputOperation for F8preProtocol {
    const OUTPUT_DESTINATIONS: &'static [TargetPort] = &[
        TargetPort::Disabled,
        TargetPort::PhonePair,
        TargetPort::MainPair,
        TargetPort::AdatPair(0),
        TargetPort::AdatPair(1),
        TargetPort::AdatPair(2),
        TargetPort::AdatPair(3),
    ];
}

impl RegisterDspMixerReturnOperation for F8preProtocol {}

impl RegisterDspMixerMonauralSourceOperation for F8preProtocol {
    const MIXER_SOURCES: &'static [TargetPort] = &[
        TargetPort::Analog(0),
        TargetPort::Analog(1),
        TargetPort::Analog(2),
        TargetPort::Analog(3),
        TargetPort::Analog(4),
        TargetPort::Analog(5),
        TargetPort::Analog(6),
        TargetPort::Analog(7),
        TargetPort::Adat(0),
        TargetPort::Adat(1),
        TargetPort::Adat(2),
        TargetPort::Adat(3),
        TargetPort::Adat(4),
        TargetPort::Adat(5),
        TargetPort::Adat(6),
        TargetPort::Adat(7),
    ];
}

impl RegisterDspOutputOperation for F8preProtocol {}

impl RegisterDspMeterOperation for F8preProtocol {
    const SELECTABLE: bool = false;
    const INPUT_PORTS: &'static [TargetPort] = &[
        TargetPort::Analog(0),
        TargetPort::Analog(1),
        TargetPort::Analog(2),
        TargetPort::Analog(3),
        TargetPort::Analog(4),
        TargetPort::Analog(5),
        TargetPort::Analog(6),
        TargetPort::Analog(7),
        TargetPort::Adat(0),
        TargetPort::Adat(1),
        TargetPort::Adat(2),
        TargetPort::Adat(3),
        TargetPort::Adat(4),
        TargetPort::Adat(5),
        TargetPort::Adat(6),
        TargetPort::Adat(7),
    ];
    const OUTPUT_PORT_PAIRS: &'static [(TargetPort, [usize; 2])] = &[
        (TargetPort::PhonePair, [2, 3]),
        (TargetPort::AnalogPair(0), [4, 5]),
        (TargetPort::AdatPair(0), [6, 7]),
        (TargetPort::AdatPair(1), [8, 9]),
        (TargetPort::AdatPair(2), [10, 11]),
        (TargetPort::AdatPair(3), [12, 13]),
    ];
}

/// The protocol implementation for Traveler.
#[derive(Default)]
pub struct TravelerProtocol;

impl AssignOperation for TravelerProtocol {
    const ASSIGN_PORTS: &'static [(TargetPort, u8)] = &[
        (TargetPort::PhonePair, 0x01),     // = Stream-0/1
        (TargetPort::AnalogPair(0), 0x02), // = Stream-2/3
        (TargetPort::AnalogPair(1), 0x03), // = Stream-4/5
        (TargetPort::AnalogPair(2), 0x04), // = Stream-6/7
        (TargetPort::AnalogPair(3), 0x05), // = Stream-8/9
        (TargetPort::AesEbuPair, 0x06),    // = Stream-10/11
        (TargetPort::SpdifPair, 0x07),     // = Stream-12/13
        (TargetPort::AdatPair(0), 0x08),   // = Stream-14/15
        (TargetPort::AdatPair(1), 0x09),   // = Stream-16/17
        (TargetPort::AdatPair(2), 0x0a),   // = Stream-18/19
        (TargetPort::AdatPair(3), 0x0b),   // = Stream-20/21
    ];
}

impl MotuWordClockOutputSpecification for TravelerProtocol {}

impl MotuClockNameDisplaySpecification for TravelerProtocol {}

impl V2ClkOperation for TravelerProtocol {
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

impl V2OptIfaceOperation for TravelerProtocol {
    const OPT_IFACE_MODES: &'static [(V2OptIfaceMode, u8)] = &[
        (V2OptIfaceMode::None, 0x00),
        (V2OptIfaceMode::Adat, 0x01),
        (V2OptIfaceMode::Spdif, 0x02),
    ];
}

impl RegisterDspMixerOutputOperation for TravelerProtocol {
    const OUTPUT_DESTINATIONS: &'static [TargetPort] = &[
        TargetPort::Disabled,
        TargetPort::PhonePair,
        TargetPort::AnalogPair(0),
        TargetPort::AnalogPair(1),
        TargetPort::AnalogPair(2),
        TargetPort::AnalogPair(3),
        TargetPort::AesEbuPair,
        TargetPort::SpdifPair,
        TargetPort::AdatPair(0),
        TargetPort::AdatPair(1),
        TargetPort::AdatPair(2),
        TargetPort::AdatPair(3),
    ];
}

impl RegisterDspMixerReturnOperation for TravelerProtocol {}

impl RegisterDspMixerMonauralSourceOperation for TravelerProtocol {
    const MIXER_SOURCES: &'static [TargetPort] = &[
        TargetPort::Analog(0),
        TargetPort::Analog(1),
        TargetPort::Analog(2),
        TargetPort::Analog(3),
        TargetPort::Analog(4),
        TargetPort::Analog(5),
        TargetPort::Analog(6),
        TargetPort::Analog(7),
        TargetPort::AesEbu(0),
        TargetPort::AesEbu(1),
        TargetPort::Spdif(0),
        TargetPort::Spdif(1),
        TargetPort::Adat(0),
        TargetPort::Adat(1),
        TargetPort::Adat(2),
        TargetPort::Adat(3),
        TargetPort::Adat(4),
        TargetPort::Adat(5),
        TargetPort::Adat(6),
        TargetPort::Adat(7),
    ];
}

impl RegisterDspOutputOperation for TravelerProtocol {}

impl Traveler828mk2LineInputOperation for TravelerProtocol {
    const LINE_INPUT_COUNT: usize = 4;
    const CH_OFFSET: usize = 4;
}

impl RegisterDspMeterOperation for TravelerProtocol {
    const SELECTABLE: bool = true;
    const INPUT_PORTS: &'static [TargetPort] = &[
        TargetPort::Analog(0),
        TargetPort::Analog(1),
        TargetPort::Analog(2),
        TargetPort::Analog(3),
        TargetPort::Analog(4),
        TargetPort::Analog(5),
        TargetPort::Analog(6),
        TargetPort::Analog(7),
        TargetPort::AesEbu(8),
        TargetPort::AesEbu(9),
        TargetPort::Spdif(0),
        TargetPort::Spdif(1),
        TargetPort::Adat(0),
        TargetPort::Adat(1),
        TargetPort::Adat(2),
        TargetPort::Adat(3),
        TargetPort::Adat(4),
        TargetPort::Adat(5),
        TargetPort::Adat(6),
        TargetPort::Adat(7),
    ];
    const OUTPUT_PORT_PAIRS: &'static [(TargetPort, [usize; 2])] = &[
        (TargetPort::PhonePair, [2, 3]),
        (TargetPort::AnalogPair(0), [4, 5]),
        (TargetPort::AnalogPair(1), [6, 7]),
        (TargetPort::AnalogPair(2), [8, 9]),
        (TargetPort::AnalogPair(3), [10, 11]),
        (TargetPort::AesEbuPair, [12, 13]),
        (TargetPort::SpdifPair, [14, 15]),
        (TargetPort::AdatPair(0), [16, 17]),
        (TargetPort::AdatPair(1), [18, 19]),
        (TargetPort::AdatPair(2), [20, 21]),
        (TargetPort::AdatPair(3), [22, 23]),
    ];
}

/// State of inputs in Traveler.
#[derive(Default, Debug)]
pub struct TravelerMicInputState {
    pub gain: [u8; TravelerProtocol::MIC_INPUT_COUNT],
    pub pad: [bool; TravelerProtocol::MIC_INPUT_COUNT],
}

const TRAVELER_MIC_PARAM_OFFSET: usize = 0x0c1c;
const TRAVELER_MIC_GAIN_MASK: u8 = 0x3f;
const TRAVELER_MIC_PAD_FLAG: u8 = 0x40;
const TRAVELER_MIC_CHANGE_FLAG: u8 = 0x80;

impl TravelerProtocol {
    /// Notification mask for mic gain, and pad.
    pub const NOTIFY_MIC_PARAM_MASK: u32 = 0x20000000;

    /// Notification mask for speed of word clock output, phone assignment.
    pub const NOTIFY_PORT_CHANGE: u32 = 0x40000000;

    /// Notification mask for signal format of optical input/output interfaces.
    pub const NOTIFY_FORMAT_CHANGE: u32 = 0x08000000;

    pub const MIC_INPUT_COUNT: usize = 4;

    pub const MIC_GAIN_MIN: u8 = 0x00;
    pub const MIC_GAIN_MAX: u8 = 0x35;
    pub const MIC_GAIN_STEP: u8 = 0x01;

    pub fn read_mic_input_state(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut TravelerMicInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        read_quad(req, node, TRAVELER_MIC_PARAM_OFFSET as u32, timeout_ms).map(|val| {
            (0..Self::MIC_INPUT_COUNT).for_each(|i| {
                let v = ((val >> (i * 8)) & 0xff) as u8;
                state.gain[i] = v & TRAVELER_MIC_GAIN_MASK;
                state.pad[i] = v & TRAVELER_MIC_PAD_FLAG > 0;
            });
        })
    }

    pub fn write_mic_gain(
        req: &mut FwReq,
        node: &mut FwNode,
        gain: &[u8],
        state: &mut TravelerMicInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(gain.len(), Self::MIC_INPUT_COUNT);

        let val = gain
            .iter()
            .enumerate()
            .filter(|&(i, g)| !state.gain[i].eq(g))
            .fold(0u32, |val, (i, &g)| {
                let mut v = TRAVELER_MIC_CHANGE_FLAG;
                if state.pad[i] {
                    v |= TRAVELER_MIC_PAD_FLAG;
                }
                v |= g & TRAVELER_MIC_GAIN_MASK;
                val | ((v as u32) << (i * 8))
            });
        write_quad(req, node, TRAVELER_MIC_PARAM_OFFSET as u32, val, timeout_ms)
            .map(|_| state.gain.copy_from_slice(gain))
    }

    pub fn write_mic_pad(
        req: &mut FwReq,
        node: &mut FwNode,
        pad: &[bool],
        state: &mut TravelerMicInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(pad.len(), Self::MIC_INPUT_COUNT);

        let val = pad
            .iter()
            .enumerate()
            .filter(|&(i, p)| !state.pad[i].eq(p))
            .fold(0u32, |val, (i, &p)| {
                let mut v = TRAVELER_MIC_CHANGE_FLAG;
                if p {
                    v |= TRAVELER_MIC_PAD_FLAG;
                }
                v |= state.gain[i] & TRAVELER_MIC_GAIN_MASK;
                val | ((v as u32) << (i * 8))
            });
        write_quad(req, node, TRAVELER_MIC_PARAM_OFFSET as u32, val, timeout_ms)
            .map(|_| state.pad.copy_from_slice(pad))
    }
}

/// The protocol implementation for Ultralite.
#[derive(Default)]
pub struct UltraliteProtocol;

impl AssignOperation for UltraliteProtocol {
    const ASSIGN_PORTS: &'static [(TargetPort, u8)] = &[
        (TargetPort::PhonePair, 0x01),     // Stream-0/1
        (TargetPort::AnalogPair(0), 0x02), // Stream-2/3
        (TargetPort::AnalogPair(1), 0x03), // Stream-4/5
        (TargetPort::AnalogPair(2), 0x04), // Stream-6/7
        (TargetPort::AnalogPair(3), 0x05), // Stream-8/9
        (TargetPort::MainPair, 0x06),      // Stream-10/11
        (TargetPort::SpdifPair, 0x07),     // Stream-12/13
    ];
}

impl MotuClockNameDisplaySpecification for UltraliteProtocol {}

impl V2ClkOperation for UltraliteProtocol {
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

impl RegisterDspMixerOutputOperation for UltraliteProtocol {
    const OUTPUT_DESTINATIONS: &'static [TargetPort] = &[
        TargetPort::Disabled,
        TargetPort::PhonePair,
        TargetPort::AnalogPair(0),
        TargetPort::AnalogPair(1),
        TargetPort::AnalogPair(2),
        TargetPort::AnalogPair(3),
        TargetPort::MainPair,
        TargetPort::SpdifPair,
    ];
}

impl RegisterDspMixerReturnOperation for UltraliteProtocol {}

impl RegisterDspMixerMonauralSourceOperation for UltraliteProtocol {
    const MIXER_SOURCES: &'static [TargetPort] = &[
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
}

impl RegisterDspOutputOperation for UltraliteProtocol {}

impl RegisterDspMonauralInputOperation for UltraliteProtocol {}

impl RegisterDspMeterOperation for UltraliteProtocol {
    const SELECTABLE: bool = false;
    const INPUT_PORTS: &'static [TargetPort] = &[
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
    const OUTPUT_PORT_PAIRS: &'static [(TargetPort, [usize; 2])] = &[
        (TargetPort::PhonePair, [2, 3]),
        (TargetPort::AnalogPair(0), [4, 5]),
        (TargetPort::AnalogPair(1), [6, 7]),
        (TargetPort::AnalogPair(2), [8, 9]),
        (TargetPort::AnalogPair(3), [10, 11]),
        (TargetPort::MainPair, [12, 13]),
        (TargetPort::SpdifPair, [14, 15]),
    ];
}

const ULTRALITE_MAIN_ASSIGN_MASK: u32 = 0x000f0000;
const ULTRALITE_MAIN_ASSIGN_SHIFT: usize = 16;
const ULTRALITE_MAIN_ASSIGN_LABEL: &str = "ultralite-main-assign";

impl UltraliteProtocol {
    /// Notification mask for main assignment, and phone assignment. The change of phone assignment
    /// is also notified in message delivered by the sequence of isochronous packets.
    pub const NOTIFY_PORT_CHANGE: u32 = 0x40000000;

    pub const KNOB_TARGETS: &'static [(TargetPort, u8)] = &[
        (TargetPort::MainPair, 0x00),
        (TargetPort::Analog6Pairs, 0x01),
        (TargetPort::Analog8Pairs, 0x02),
        (TargetPort::SpdifPair, 0x03),
    ];

    pub const INPUT_COUNT: usize = 10;

    pub const INPUT_GAIN_MIN: u8 = 0x00;
    pub const INPUT_GAIN_MAX: u8 = 0x18;
    pub const INPUT_GAIN_STEP: u8 = 0x01;

    pub fn get_main_assign(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<usize, Error> {
        let vals: Vec<u8> = Self::KNOB_TARGETS.iter().map(|e| e.1).collect();
        get_idx_from_val(
            OFFSET_PORT,
            ULTRALITE_MAIN_ASSIGN_MASK,
            ULTRALITE_MAIN_ASSIGN_SHIFT,
            ULTRALITE_MAIN_ASSIGN_LABEL,
            req,
            node,
            &vals,
            timeout_ms,
        )
    }

    pub fn set_main_assign(
        req: &mut FwReq,
        node: &mut FwNode,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let vals: Vec<u8> = Self::KNOB_TARGETS.iter().map(|e| e.1).collect();
        set_idx_to_val(
            OFFSET_PORT,
            ULTRALITE_MAIN_ASSIGN_MASK,
            ULTRALITE_MAIN_ASSIGN_SHIFT,
            ULTRALITE_MAIN_ASSIGN_LABEL,
            req,
            node,
            &vals,
            idx,
            timeout_ms,
        )
    }
}

/// The protocol implementation for 896HD.
#[derive(Default)]
pub struct F896hdProtocol;

impl F896hdProtocol {
    /// Notification mask for programmable meter.
    pub const NOTIFY_PROGRAMMABLE_METER_MASK: u32 = 0x40000000;

    /// Notification mask for footswitch.
    pub const NOTIFY_FOOTSWITCH_MASK: u32 = 0x01000000;
}

impl AssignOperation for F896hdProtocol {
    const ASSIGN_PORTS: &'static [(TargetPort, u8)] = &[
        (TargetPort::PhonePair, 0x01),     // Stream-0/1
        (TargetPort::AnalogPair(0), 0x02), // Stream-2/3
        (TargetPort::AnalogPair(1), 0x03), // Stream-4/5
        (TargetPort::AnalogPair(2), 0x04), // Stream-6/7
        (TargetPort::AnalogPair(3), 0x05), // Stream-8/9
        (TargetPort::MainPair, 0x06),      // Stream-10/11
        (TargetPort::AesEbuPair, 0x07),    // Stream-12/13
        (TargetPort::AdatPair(0), 0x08),   // Stream-14/15
        (TargetPort::AdatPair(1), 0x09),   // Stream-16/17
        (TargetPort::AdatPair(2), 0x0a),   // Stream-18/19
        (TargetPort::AdatPair(3), 0x0b),   // Stream-20/21
    ];
}

impl MotuWordClockOutputSpecification for F896hdProtocol {}

impl MotuAesebuRateConvertSpecification for F896hdProtocol {
    const AESEBU_RATE_CONVERT_MASK: u32 = 0x00000300;
    const AESEBU_RATE_CONVERT_SHIFT: usize = 8;
}

impl MotuLevelMetersSpecification for F896hdProtocol {}

impl V2ClkOperation for F896hdProtocol {
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

impl V2OptIfaceOperation for F896hdProtocol {
    const OPT_IFACE_MODES: &'static [(V2OptIfaceMode, u8)] =
        &[(V2OptIfaceMode::None, 0x00), (V2OptIfaceMode::Adat, 0x01)];
}

impl RegisterDspMixerOutputOperation for F896hdProtocol {
    const OUTPUT_DESTINATIONS: &'static [TargetPort] = &[
        TargetPort::Disabled,
        TargetPort::PhonePair,
        TargetPort::AnalogPair(0),
        TargetPort::AnalogPair(1),
        TargetPort::AnalogPair(2),
        TargetPort::AnalogPair(3),
        TargetPort::MainPair,
        TargetPort::AesEbuPair,
        TargetPort::AdatPair(0),
        TargetPort::AdatPair(1),
        TargetPort::AdatPair(2),
        TargetPort::AdatPair(3),
    ];
}

impl RegisterDspMixerReturnOperation for F896hdProtocol {}

impl RegisterDspMixerMonauralSourceOperation for F896hdProtocol {
    const MIXER_SOURCES: &'static [TargetPort] = &[
        TargetPort::Analog(0),
        TargetPort::Analog(1),
        TargetPort::Analog(2),
        TargetPort::Analog(3),
        TargetPort::Analog(4),
        TargetPort::Analog(5),
        TargetPort::Analog(6),
        TargetPort::Analog(7),
        TargetPort::AesEbu(0),
        TargetPort::AesEbu(1),
        TargetPort::Adat(0),
        TargetPort::Adat(1),
        TargetPort::Adat(2),
        TargetPort::Adat(3),
        TargetPort::Adat(4),
        TargetPort::Adat(5),
        TargetPort::Adat(6),
        TargetPort::Adat(7),
    ];
}

impl RegisterDspOutputOperation for F896hdProtocol {}

impl RegisterDspMeterOperation for F896hdProtocol {
    const SELECTABLE: bool = true;
    const INPUT_PORTS: &'static [TargetPort] = &[
        TargetPort::Analog(0),
        TargetPort::Analog(1),
        TargetPort::Analog(2),
        TargetPort::Analog(3),
        TargetPort::Analog(4),
        TargetPort::Analog(5),
        TargetPort::Analog(6),
        TargetPort::Analog(7),
        TargetPort::AesEbu(0),
        TargetPort::AesEbu(1),
        TargetPort::Adat(0),
        TargetPort::Adat(1),
        TargetPort::Adat(2),
        TargetPort::Adat(3),
        TargetPort::Adat(4),
        TargetPort::Adat(5),
        TargetPort::Adat(6),
        TargetPort::Adat(7),
    ];
    const OUTPUT_PORT_PAIRS: &'static [(TargetPort, [usize; 2])] = &[
        (TargetPort::PhonePair, [2, 3]),
        (TargetPort::AnalogPair(0), [4, 5]),
        (TargetPort::AnalogPair(1), [6, 7]),
        (TargetPort::AnalogPair(2), [8, 9]),
        (TargetPort::AnalogPair(3), [10, 11]),
        (TargetPort::MainPair, [12, 13]),
        (TargetPort::AesEbuPair, [14, 15]),
        (TargetPort::AdatPair(0), [16, 17]),
        (TargetPort::AdatPair(1), [18, 19]),
        (TargetPort::AdatPair(2), [20, 21]),
        (TargetPort::AdatPair(3), [22, 23]),
    ];
}
