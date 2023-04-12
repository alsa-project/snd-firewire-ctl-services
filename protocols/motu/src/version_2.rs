// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol used in version 2 devices of MOTU FireWire series.
//!
//! The modules includes structure, enumeration, and trait and its implementation for protocol
//! used in version 2 devices of Mark of the Unicorn FireWire series.

use super::{register_dsp::*, *};

/// Signal source of sampling clock.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

impl Default for V2ClkSrc {
    fn default() -> Self {
        Self::Internal
    }
}

/// The parameters of media and sampling clocks.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Version2ClockParameters {
    /// The rate of media clock.
    pub rate: ClkRate,
    /// The source of sampling clock.
    pub source: V2ClkSrc,
}

const CLK_RATE_LABEL: &str = "clock-rate-v2";
const CLK_RATE_MASK: u32 = 0x00000038;
const CLK_RATE_SHIFT: usize = 3;

const CLK_SRC_LABEL: &str = "clock-source-v2";
const CLK_SRC_MASK: u32 = 0x00000007;
const CLK_SRC_SHIFT: usize = 0;

/// The trait for specification of sampling and media clocks.
pub trait MotuVersion2ClockSpecification {
    const CLK_RATES: &'static [ClkRate];
    const CLK_RATE_VALS: &'static [u8];

    const CLK_SRCS: &'static [V2ClkSrc];
    const CLK_SRC_VALS: &'static [u8];
}

impl<O> MotuWhollyCacheableParamsOperation<Version2ClockParameters> for O
where
    O: MotuVersion2ClockSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut Version2ClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let quad = read_quad(req, node, OFFSET_CLK, timeout_ms)?;

        deserialize_flag(
            &mut params.rate,
            &quad,
            CLK_RATE_MASK,
            CLK_RATE_SHIFT,
            Self::CLK_RATES,
            Self::CLK_RATE_VALS,
            CLK_RATE_LABEL,
        )?;

        deserialize_flag(
            &mut params.source,
            &quad,
            CLK_SRC_MASK,
            CLK_SRC_SHIFT,
            Self::CLK_SRCS,
            Self::CLK_SRC_VALS,
            CLK_SRC_LABEL,
        )
    }
}

impl<O> MotuWhollyUpdatableParamsOperation<Version2ClockParameters> for O
where
    O: MotuVersion2ClockSpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &Version2ClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quad = read_quad(req, node, OFFSET_CLK, timeout_ms)?;

        serialize_flag(
            &params.rate,
            &mut quad,
            CLK_RATE_MASK,
            CLK_RATE_SHIFT,
            Self::CLK_RATES,
            Self::CLK_RATE_VALS,
            CLK_RATE_LABEL,
        )?;

        serialize_flag(
            &params.source,
            &mut quad,
            CLK_SRC_MASK,
            CLK_SRC_SHIFT,
            Self::CLK_SRCS,
            Self::CLK_SRC_VALS,
            CLK_SRC_LABEL,
        )?;

        write_quad(req, node, OFFSET_CLK, quad, timeout_ms)
    }
}

/// Mode of optical interface.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum V2OptIfaceMode {
    None,
    Adat,
    Spdif,
}

impl Default for V2OptIfaceMode {
    fn default() -> Self {
        Self::None
    }
}

/// The parameters of optical interfaces.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Version2OpticalIfaceParameters {
    /// The mode of signal in optical input interface.
    pub input_mode: V2OptIfaceMode,
    /// The mode of signal in optical output interface.
    pub output_mode: V2OptIfaceMode,
}

const OPT_IN_IFACE_LABEL: &str = "optical-input-iface-v2";
const OPT_IN_IFACE_MASK: u32 = 0x00000300;
const OPT_IN_IFACE_SHIFT: usize = 8;

const OPT_OUT_IFACE_LABEL: &str = "optical-output-iface-v2";
const OPT_OUT_IFACE_MASK: u32 = 0x00000c00;
const OPT_OUT_IFACE_SHIFT: usize = 10;

const OPT_IFACE_MODE_VALS: &[u8] = &[0x00, 0x01, 0x02];

/// The trait for specificification of mode of optical input and output interfaces.
pub trait MotuVersion2OpticalIfaceSpecification {
    const OPT_IFACE_MODES: &'static [V2OptIfaceMode];
}

impl<O> MotuWhollyCacheableParamsOperation<Version2OpticalIfaceParameters> for O
where
    O: MotuVersion2OpticalIfaceSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut Version2OpticalIfaceParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let quad = read_quad(req, node, OFFSET_PORT, timeout_ms)?;

        deserialize_flag(
            &mut params.input_mode,
            &quad,
            OPT_IN_IFACE_MASK,
            OPT_IN_IFACE_SHIFT,
            Self::OPT_IFACE_MODES,
            OPT_IFACE_MODE_VALS,
            OPT_IN_IFACE_LABEL,
        )?;

        deserialize_flag(
            &mut params.output_mode,
            &quad,
            OPT_OUT_IFACE_MASK,
            OPT_OUT_IFACE_SHIFT,
            Self::OPT_IFACE_MODES,
            OPT_IFACE_MODE_VALS,
            OPT_OUT_IFACE_LABEL,
        )
    }
}

impl<O> MotuWhollyUpdatableParamsOperation<Version2OpticalIfaceParameters> for O
where
    O: MotuVersion2OpticalIfaceSpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &Version2OpticalIfaceParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quad = read_quad(req, node, OFFSET_PORT, timeout_ms)?;

        serialize_flag(
            &params.input_mode,
            &mut quad,
            OPT_IN_IFACE_MASK,
            OPT_IN_IFACE_SHIFT,
            Self::OPT_IFACE_MODES,
            OPT_IFACE_MODE_VALS,
            OPT_IN_IFACE_LABEL,
        )?;

        serialize_flag(
            &params.output_mode,
            &mut quad,
            OPT_OUT_IFACE_MASK,
            OPT_OUT_IFACE_SHIFT,
            Self::OPT_IFACE_MODES,
            OPT_IFACE_MODE_VALS,
            OPT_OUT_IFACE_LABEL,
        )?;

        write_quad(req, node, OFFSET_PORT, quad, timeout_ms)
    }
}

/// The protocol implementation for 828mkII.
#[derive(Default)]
pub struct F828mk2Protocol;

impl MotuPortAssignSpecification for F828mk2Protocol {
    const ASSIGN_PORT_TARGETS: &'static [TargetPort] = &[
        TargetPort::PhonePair,     // = Stream-0/1
        TargetPort::AnalogPair(0), // = Stream-2/3
        TargetPort::AnalogPair(1), // = Stream-4/5
        TargetPort::AnalogPair(2), // = Stream-6/7
        TargetPort::AnalogPair(3), // = Stream-8/9
        TargetPort::MainPair,      // = Stream-10/11
        TargetPort::SpdifPair,     // = Stream-12/13
        TargetPort::AdatPair(0),   // = Stream-14/15
        TargetPort::AdatPair(1),   // = Stream-16/17
        TargetPort::AdatPair(2),   // = Stream-18/19
        TargetPort::AdatPair(3),   // = Stream-20/21
    ];

    const ASSIGN_PORT_VALS: &'static [u8] = &[
        0x01, // = Stream-0/1
        0x02, // = Stream-2/3
        0x03, // = Stream-4/5
        0x04, // = Stream-6/7
        0x05, // = Stream-8/9
        0x06, // = Stream-10/11
        0x07, // = Stream-12/13
        0x08, // = Stream-14/15
        0x09, // = Stream-16/17
        0x0a, // = Stream-18/19
        0x0b, // = Stream-20/21
    ];
}

impl MotuWordClockOutputSpecification for F828mk2Protocol {}

impl MotuClockNameDisplaySpecification for F828mk2Protocol {}

impl MotuVersion2ClockSpecification for F828mk2Protocol {
    const CLK_RATES: &'static [ClkRate] = &[
        ClkRate::R44100,
        ClkRate::R48000,
        ClkRate::R88200,
        ClkRate::R96000,
    ];
    const CLK_RATE_VALS: &'static [u8] = &[0x00, 0x01, 0x02, 0x03];

    const CLK_SRCS: &'static [V2ClkSrc] = &[
        V2ClkSrc::Internal,
        V2ClkSrc::SignalOpt,
        V2ClkSrc::SpdifCoax,
        V2ClkSrc::WordClk,
        V2ClkSrc::AdatDsub,
    ];
    const CLK_SRC_VALS: &'static [u8] = &[0x00, 0x01, 0x02, 0x04, 0x05];
}

impl MotuVersion2OpticalIfaceSpecification for F828mk2Protocol {
    const OPT_IFACE_MODES: &'static [V2OptIfaceMode] = &[
        V2OptIfaceMode::None,
        V2OptIfaceMode::Adat,
        V2OptIfaceMode::Spdif,
    ];
}

impl MotuRegisterDspSpecification for F828mk2Protocol {
    const MIXER_OUTPUT_DESTINATIONS: &'static [TargetPort] = &[
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

impl MotuRegisterDspMixerMonauralSourceSpecification for F828mk2Protocol {
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

impl MotuRegisterDspLineInputSpecification for F828mk2Protocol {
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

impl MotuPortAssignSpecification for F8preProtocol {
    const ASSIGN_PORT_TARGETS: &'static [TargetPort] =
        &[TargetPort::PhonePair, TargetPort::MainPair];
    const ASSIGN_PORT_VALS: &'static [u8] = &[0x01, 0x02];
}

impl MotuVersion2ClockSpecification for F8preProtocol {
    const CLK_RATES: &'static [ClkRate] = &[
        ClkRate::R44100,
        ClkRate::R48000,
        ClkRate::R88200,
        ClkRate::R96000,
    ];
    const CLK_RATE_VALS: &'static [u8] = &[0x00, 0x01, 0x02, 0x03];

    const CLK_SRCS: &'static [V2ClkSrc] = &[V2ClkSrc::Internal, V2ClkSrc::AdatOpt];
    const CLK_SRC_VALS: &'static [u8] = &[0x00, 0x01];
}

impl MotuVersion2OpticalIfaceSpecification for F8preProtocol {
    const OPT_IFACE_MODES: &'static [V2OptIfaceMode] =
        &[V2OptIfaceMode::None, V2OptIfaceMode::Adat];
}

impl MotuRegisterDspSpecification for F8preProtocol {
    const MIXER_OUTPUT_DESTINATIONS: &'static [TargetPort] = &[
        TargetPort::Disabled,
        TargetPort::PhonePair,
        TargetPort::MainPair,
        TargetPort::AdatPair(0),
        TargetPort::AdatPair(1),
        TargetPort::AdatPair(2),
        TargetPort::AdatPair(3),
    ];
}

impl MotuRegisterDspMixerMonauralSourceSpecification for F8preProtocol {
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

impl MotuPortAssignSpecification for TravelerProtocol {
    const ASSIGN_PORT_TARGETS: &'static [TargetPort] = &[
        TargetPort::PhonePair,     // = Stream-0/1
        TargetPort::AnalogPair(0), // = Stream-2/3
        TargetPort::AnalogPair(1), // = Stream-4/5
        TargetPort::AnalogPair(2), // = Stream-6/7
        TargetPort::AnalogPair(3), // = Stream-8/9
        TargetPort::AesEbuPair,    // = Stream-10/11
        TargetPort::SpdifPair,     // = Stream-12/13
        TargetPort::AdatPair(0),   // = Stream-14/15
        TargetPort::AdatPair(1),   // = Stream-16/17
        TargetPort::AdatPair(2),   // = Stream-18/19
        TargetPort::AdatPair(3),   // = Stream-20/21
    ];

    const ASSIGN_PORT_VALS: &'static [u8] = &[
        0x01, // = Stream-0/1
        0x02, // = Stream-2/3
        0x03, // = Stream-4/5
        0x04, // = Stream-6/7
        0x05, // = Stream-8/9
        0x06, // = Stream-10/11
        0x07, // = Stream-12/13
        0x08, // = Stream-14/15
        0x09, // = Stream-16/17
        0x0a, // = Stream-18/19
        0x0b, // = Stream-20/21
    ];
}

impl MotuWordClockOutputSpecification for TravelerProtocol {}

impl MotuClockNameDisplaySpecification for TravelerProtocol {}

impl MotuVersion2ClockSpecification for TravelerProtocol {
    const CLK_RATES: &'static [ClkRate] = &[
        ClkRate::R44100,
        ClkRate::R48000,
        ClkRate::R88200,
        ClkRate::R96000,
        ClkRate::R176400,
        ClkRate::R192000,
    ];
    const CLK_RATE_VALS: &'static [u8] = &[0x00, 0x01, 0x02, 0x03, 0x04, 0x05];

    const CLK_SRCS: &'static [V2ClkSrc] = &[
        V2ClkSrc::Internal,
        V2ClkSrc::SignalOpt,
        V2ClkSrc::SpdifCoax,
        V2ClkSrc::WordClk,
        V2ClkSrc::AdatDsub,
        V2ClkSrc::AesebuXlr,
    ];
    const CLK_SRC_VALS: &'static [u8] = &[0x00, 0x01, 0x02, 0x04, 0x05, 0x07];
}

impl MotuVersion2OpticalIfaceSpecification for TravelerProtocol {
    const OPT_IFACE_MODES: &'static [V2OptIfaceMode] = &[
        V2OptIfaceMode::None,
        V2OptIfaceMode::Adat,
        V2OptIfaceMode::Spdif,
    ];
}

impl MotuRegisterDspSpecification for TravelerProtocol {
    const MIXER_OUTPUT_DESTINATIONS: &'static [TargetPort] = &[
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

impl MotuRegisterDspMixerMonauralSourceSpecification for TravelerProtocol {
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

impl MotuRegisterDspLineInputSpecification for TravelerProtocol {
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
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TravelerMicInputState {
    /// The gain of microphone input. The value is between 0x00 and 0x35.
    pub gain: [u8; TravelerProtocol::MIC_INPUT_COUNT],
    /// Whether to pad microphone input.
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

    /// The number of microphone inputs.
    pub const MIC_INPUT_COUNT: usize = 4;

    /// The minimum value of microphone input.
    pub const MIC_GAIN_MIN: u8 = 0x00;
    /// The maximum value of microphone input.
    pub const MIC_GAIN_MAX: u8 = 0x35;
    /// The step value of microphone input.
    pub const MIC_GAIN_STEP: u8 = 0x01;
}

impl MotuWhollyCacheableParamsOperation<TravelerMicInputState> for TravelerProtocol {
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut TravelerMicInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        read_quad(req, node, TRAVELER_MIC_PARAM_OFFSET as u32, timeout_ms).map(|val| {
            (0..Self::MIC_INPUT_COUNT).for_each(|i| {
                let v = ((val >> (i * 8)) & 0xff) as u8;
                params.gain[i] = v & TRAVELER_MIC_GAIN_MASK;
                params.pad[i] = v & TRAVELER_MIC_PAD_FLAG > 0;
            });
        })
    }
}

impl MotuWhollyUpdatableParamsOperation<TravelerMicInputState> for TravelerProtocol {
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &TravelerMicInputState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let val = (0..Self::MIC_INPUT_COUNT).fold(0u32, |val, i| {
            let mut v = TRAVELER_MIC_CHANGE_FLAG;
            if params.pad[i] {
                v |= TRAVELER_MIC_PAD_FLAG;
            }
            v |= params.gain[i] & TRAVELER_MIC_GAIN_MASK;
            val | ((v as u32) << (i * 8))
        });
        write_quad(req, node, TRAVELER_MIC_PARAM_OFFSET as u32, val, timeout_ms)
    }
}

/// The protocol implementation for Ultralite.
#[derive(Default)]
pub struct UltraliteProtocol;

impl MotuPortAssignSpecification for UltraliteProtocol {
    const ASSIGN_PORT_TARGETS: &'static [TargetPort] = &[
        TargetPort::PhonePair,     // Stream-0/1
        TargetPort::AnalogPair(0), // Stream-2/3
        TargetPort::AnalogPair(1), // Stream-4/5
        TargetPort::AnalogPair(2), // Stream-6/7
        TargetPort::AnalogPair(3), // Stream-8/9
        TargetPort::MainPair,      // Stream-10/11
        TargetPort::SpdifPair,     // Stream-12/13
    ];

    const ASSIGN_PORT_VALS: &'static [u8] = &[
        0x01, // Stream-0/1
        0x02, // Stream-2/3
        0x03, // Stream-4/5
        0x04, // Stream-6/7
        0x05, // Stream-8/9
        0x06, // Stream-10/11
        0x07, // Stream-12/13
    ];
}

impl MotuClockNameDisplaySpecification for UltraliteProtocol {}

impl MotuVersion2ClockSpecification for UltraliteProtocol {
    const CLK_RATES: &'static [ClkRate] = &[
        ClkRate::R44100,
        ClkRate::R48000,
        ClkRate::R88200,
        ClkRate::R96000,
    ];
    const CLK_RATE_VALS: &'static [u8] = &[0x00, 0x01, 0x02, 0x03];

    const CLK_SRCS: &'static [V2ClkSrc] = &[V2ClkSrc::Internal, V2ClkSrc::SpdifCoax];
    const CLK_SRC_VALS: &'static [u8] = &[0x00, 0x02];
}

impl MotuRegisterDspSpecification for UltraliteProtocol {
    const MIXER_OUTPUT_DESTINATIONS: &'static [TargetPort] = &[
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

impl MotuRegisterDspMixerMonauralSourceSpecification for UltraliteProtocol {
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

impl MotuRegisterDspMonauralInputSpecification for UltraliteProtocol {}

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

/// The parameter of assignment to main output pair in Ultralite.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct UltraliteMainAssign(pub TargetPort);

const ULTRALITE_MAIN_ASSIGN_MASK: u32 = 0x000f0000;
const ULTRALITE_MAIN_ASSIGN_SHIFT: usize = 16;
const ULTRALITE_MAIN_ASSIGN_LABEL: &str = "ultralite-main-assign";

impl UltraliteProtocol {
    /// Notification mask for main assignment, and phone assignment. The change of phone assignment
    /// is also notified in message delivered by the sequence of isochronous packets.
    pub const NOTIFY_PORT_CHANGE: u32 = 0x40000000;

    /// The target of knob control.
    pub const KNOB_TARGETS: &'static [TargetPort] = &[
        TargetPort::MainPair,
        TargetPort::Analog6Pairs,
        TargetPort::Analog8Pairs,
        TargetPort::SpdifPair,
    ];

    /// The number of inputs.
    pub const INPUT_COUNT: usize = 10;

    /// The minimum value of input.
    pub const INPUT_GAIN_MIN: u8 = 0x00;
    /// The maximum value of input.
    pub const INPUT_GAIN_MAX: u8 = 0x18;
    /// The step value of input.
    pub const INPUT_GAIN_STEP: u8 = 0x01;
}

const KNOB_TARGET_VALS: &[u8] = &[0x00, 0x01, 0x02, 0x03];

impl MotuWhollyCacheableParamsOperation<UltraliteMainAssign> for UltraliteProtocol {
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut UltraliteMainAssign,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let quad = read_quad(req, node, OFFSET_PORT, timeout_ms)?;
        deserialize_flag(
            &mut params.0,
            &quad,
            ULTRALITE_MAIN_ASSIGN_MASK,
            ULTRALITE_MAIN_ASSIGN_SHIFT,
            Self::KNOB_TARGETS,
            KNOB_TARGET_VALS,
            ULTRALITE_MAIN_ASSIGN_LABEL,
        )
    }
}

impl MotuWhollyUpdatableParamsOperation<UltraliteMainAssign> for UltraliteProtocol {
    /// Update whole parameters.
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &UltraliteMainAssign,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quad = read_quad(req, node, OFFSET_PORT, timeout_ms)?;
        serialize_flag(
            &params.0,
            &mut quad,
            ULTRALITE_MAIN_ASSIGN_MASK,
            ULTRALITE_MAIN_ASSIGN_SHIFT,
            Self::KNOB_TARGETS,
            KNOB_TARGET_VALS,
            ULTRALITE_MAIN_ASSIGN_LABEL,
        )?;
        write_quad(req, node, OFFSET_PORT, quad, timeout_ms)
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

impl MotuPortAssignSpecification for F896hdProtocol {
    const ASSIGN_PORT_TARGETS: &'static [TargetPort] = &[
        TargetPort::PhonePair,     // Stream-0/1
        TargetPort::AnalogPair(0), // Stream-2/3
        TargetPort::AnalogPair(1), // Stream-4/5
        TargetPort::AnalogPair(2), // Stream-6/7
        TargetPort::AnalogPair(3), // Stream-8/9
        TargetPort::MainPair,      // Stream-10/11
        TargetPort::AesEbuPair,    // Stream-12/13
        TargetPort::AdatPair(0),   // Stream-14/15
        TargetPort::AdatPair(1),   // Stream-16/17
        TargetPort::AdatPair(2),   // Stream-18/19
        TargetPort::AdatPair(3),   // Stream-20/21
    ];

    const ASSIGN_PORT_VALS: &'static [u8] = &[
        0x01, // Stream-0/1
        0x02, // Stream-2/3
        0x03, // Stream-4/5
        0x04, // Stream-6/7
        0x05, // Stream-8/9
        0x06, // Stream-10/11
        0x07, // Stream-12/13
        0x08, // Stream-14/15
        0x09, // Stream-16/17
        0x0a, // Stream-18/19
        0x0b, // Stream-20/21
    ];
}

impl MotuWordClockOutputSpecification for F896hdProtocol {}

impl MotuAesebuRateConvertSpecification for F896hdProtocol {
    const AESEBU_RATE_CONVERT_MASK: u32 = 0x00000300;
    const AESEBU_RATE_CONVERT_SHIFT: usize = 8;
}

impl MotuLevelMetersSpecification for F896hdProtocol {}

impl MotuVersion2ClockSpecification for F896hdProtocol {
    const CLK_RATES: &'static [ClkRate] = &[
        ClkRate::R44100,
        ClkRate::R48000,
        ClkRate::R88200,
        ClkRate::R96000,
        ClkRate::R176400,
        ClkRate::R192000,
    ];
    const CLK_RATE_VALS: &'static [u8] = &[0x00, 0x01, 0x02, 0x03, 0x04, 0x05];

    const CLK_SRCS: &'static [V2ClkSrc] = &[
        V2ClkSrc::Internal,
        V2ClkSrc::AdatOpt,
        V2ClkSrc::AesebuXlr,
        V2ClkSrc::WordClk,
        V2ClkSrc::AdatDsub,
    ];
    const CLK_SRC_VALS: &'static [u8] = &[0x00, 0x01, 0x02, 0x04, 0x05];
}

impl MotuVersion2OpticalIfaceSpecification for F896hdProtocol {
    const OPT_IFACE_MODES: &'static [V2OptIfaceMode] =
        &[V2OptIfaceMode::None, V2OptIfaceMode::Adat];
}

impl MotuRegisterDspSpecification for F896hdProtocol {
    const MIXER_OUTPUT_DESTINATIONS: &'static [TargetPort] = &[
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

impl MotuRegisterDspMixerMonauralSourceSpecification for F896hdProtocol {
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
