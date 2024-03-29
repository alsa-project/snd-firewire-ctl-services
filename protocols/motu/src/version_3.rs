// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol used in version 3 devices of MOTU FireWire series.
//!
//! The modules includes structure, enumeration, and trait and its implementation for protocol
//! used in version 3 devices of Mark of the Unicorn FireWire series.

use super::{command_dsp::*, register_dsp::*, *};

/// Signal source of sampling clock.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

impl Default for V3ClkSrc {
    fn default() -> Self {
        Self::Internal
    }
}

/// The parameters of media and sampling clock.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Version3ClockParameters {
    /// The rate of media clock.
    pub rate: ClkRate,
    /// The source of sampling clock.
    pub source: V3ClkSrc,
}

const CLK_RATE_LABEL: &str = "clock-rate-v3";
const CLK_RATE_MASK: u32 = 0x0000ff00;
const CLK_RATE_SHIFT: usize = 8;

const CLK_SRC_LABEL: &str = "clock-source-v3";
const CLK_SRC_MASK: u32 = 0x000000ff;
const CLK_SRC_SHIFT: usize = 0;

/// The trait for specification of sampling and media clocks.
pub trait MotuVersion3ClockSpecification {
    const CLOCK_RATES: &'static [ClkRate];
    const CLOCK_RATE_VALS: &'static [u8];

    const CLOCK_SRCS: &'static [V3ClkSrc];
    const CLOCK_SRC_VALS: &'static [u8];
}

impl<O> MotuWhollyCacheableParamsOperation<Version3ClockParameters> for O
where
    O: MotuVersion3ClockSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut Version3ClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let quad = read_quad(req, node, OFFSET_CLK, timeout_ms)?;

        deserialize_flag(
            &mut params.rate,
            &quad,
            CLK_RATE_MASK,
            CLK_RATE_SHIFT,
            Self::CLOCK_RATES,
            Self::CLOCK_RATE_VALS,
            CLK_RATE_LABEL,
        )?;

        deserialize_flag(
            &mut params.source,
            &quad,
            CLK_SRC_MASK,
            CLK_SRC_SHIFT,
            Self::CLOCK_SRCS,
            Self::CLOCK_SRC_VALS,
            CLK_SRC_LABEL,
        )?;

        Ok(())
    }
}

impl<O> MotuWhollyUpdatableParamsOperation<Version3ClockParameters> for O
where
    O: MotuVersion3ClockSpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &Version3ClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quad = read_quad(req, node, OFFSET_CLK, timeout_ms)?;

        serialize_flag(
            &params.rate,
            &mut quad,
            CLK_RATE_MASK,
            CLK_RATE_SHIFT,
            Self::CLOCK_RATES,
            Self::CLOCK_RATE_VALS,
            CLK_RATE_LABEL,
        )?;

        serialize_flag(
            &params.source,
            &mut quad,
            CLK_SRC_MASK,
            CLK_SRC_SHIFT,
            Self::CLOCK_SRCS,
            Self::CLOCK_SRC_VALS,
            CLK_SRC_LABEL,
        )?;

        write_quad(req, node, OFFSET_CLK, quad, timeout_ms)
    }
}

/// The parameters of port assignment.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct V3PortAssignParameters {
    /// The main assignment.
    pub main: TargetPort,
    /// The mixer return assignment.
    pub mixer_return: TargetPort,
}

const PORT_MAIN_LABEL: &str = "main-out-assign-v3";
const PORT_MAIN_MASK: u32 = 0x000000f0;
const PORT_MAIN_SHIFT: usize = 4;

const PORT_RETURN_LABEL: &str = "return-assign-v3";
const PORT_RETURN_MASK: u32 = 0x00000f00;
const PORT_RETURN_SHIFT: usize = 8;

impl<O> MotuWhollyCacheableParamsOperation<V3PortAssignParameters> for O
where
    O: MotuPortAssignSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut V3PortAssignParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let quad = read_quad(req, node, OFFSET_PORT, timeout_ms)?;

        deserialize_flag(
            &mut params.main,
            &quad,
            PORT_MAIN_MASK,
            PORT_MAIN_SHIFT,
            Self::ASSIGN_PORT_TARGETS,
            Self::ASSIGN_PORT_VALS,
            PORT_MAIN_LABEL,
        )?;

        deserialize_flag(
            &mut params.mixer_return,
            &quad,
            PORT_RETURN_MASK,
            PORT_RETURN_SHIFT,
            Self::ASSIGN_PORT_TARGETS,
            Self::ASSIGN_PORT_VALS,
            PORT_RETURN_LABEL,
        )
    }
}

impl<O> MotuWhollyUpdatableParamsOperation<V3PortAssignParameters> for O
where
    O: MotuPortAssignSpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &V3PortAssignParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quad = read_quad(req, node, OFFSET_PORT, timeout_ms)?;

        serialize_flag(
            &params.main,
            &mut quad,
            PORT_MAIN_MASK,
            PORT_MAIN_SHIFT,
            Self::ASSIGN_PORT_TARGETS,
            Self::ASSIGN_PORT_VALS,
            PORT_MAIN_LABEL,
        )?;

        serialize_flag(
            &params.mixer_return,
            &mut quad,
            PORT_RETURN_MASK,
            PORT_RETURN_SHIFT,
            Self::ASSIGN_PORT_TARGETS,
            Self::ASSIGN_PORT_VALS,
            PORT_RETURN_LABEL,
        )?;

        write_quad(req, node, OFFSET_PORT, quad, timeout_ms)
    }
}

/// Mode of optical interface.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

/// The parameters of optical input and output interfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V3OpticalIfaceParameters {
    /// The mode of input interfaces.
    pub input_modes: Vec<V3OptIfaceMode>,
    /// The mode of output interfaces.
    pub output_modes: Vec<V3OptIfaceMode>,
}

fn get_opt_iface_masks(is_b: bool, is_out: bool) -> (u32, u32) {
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

const OFFSET_OPT: u32 = 0x0c94;

/// The trait for specification of optical input and output interfaces.
pub trait MotuVersion3OpticalIfaceSpecification {
    const OPT_IFACE_COUNT: usize;

    const OPT_IFACE_MODES: &'static [V3OptIfaceMode; 3] = &[
        V3OptIfaceMode::Disabled,
        V3OptIfaceMode::Adat,
        V3OptIfaceMode::Spdif,
    ];

    /// Instantiate parameters of optical input and output interfaces.
    fn create_optical_iface_parameters() -> V3OpticalIfaceParameters {
        V3OpticalIfaceParameters {
            input_modes: vec![Default::default(); Self::OPT_IFACE_COUNT],
            output_modes: vec![Default::default(); Self::OPT_IFACE_COUNT],
        }
    }
}

fn serialize_opt_iface_mode(mode: &V3OptIfaceMode, quad: &mut u32, is_b: bool, is_out: bool) {
    let (enabled_mask, no_adat_mask) = get_opt_iface_masks(is_b, is_out);
    *quad &= !(enabled_mask | no_adat_mask);
    match *mode {
        V3OptIfaceMode::Disabled => {}
        V3OptIfaceMode::Adat => *quad |= enabled_mask,
        V3OptIfaceMode::Spdif => *quad |= enabled_mask | no_adat_mask,
    }
}

fn deserialize_opt_iface_mode(mode: &mut V3OptIfaceMode, quad: &u32, is_b: bool, is_out: bool) {
    let (enabled_mask, no_adat_mask) = get_opt_iface_masks(is_b, is_out);
    *mode = match (*quad & enabled_mask > 0, *quad & no_adat_mask > 0) {
        (false, false) | (false, true) => V3OptIfaceMode::Disabled,
        (true, false) => V3OptIfaceMode::Adat,
        (true, true) => V3OptIfaceMode::Spdif,
    };
}

impl<O> MotuWhollyCacheableParamsOperation<V3OpticalIfaceParameters> for O
where
    O: MotuVersion3OpticalIfaceSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut V3OpticalIfaceParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(params.input_modes.len(), Self::OPT_IFACE_COUNT);
        assert_eq!(params.output_modes.len(), Self::OPT_IFACE_COUNT);

        let quad = read_quad(req, node, OFFSET_OPT, timeout_ms)?;
        params
            .input_modes
            .iter_mut()
            .enumerate()
            .for_each(|(i, mode)| deserialize_opt_iface_mode(mode, &quad, i > 0, false));
        params
            .output_modes
            .iter_mut()
            .enumerate()
            .for_each(|(i, mode)| deserialize_opt_iface_mode(mode, &quad, i > 0, true));
        Ok(())
    }
}

impl<O> MotuWhollyUpdatableParamsOperation<V3OpticalIfaceParameters> for O
where
    O: MotuVersion3OpticalIfaceSpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &V3OpticalIfaceParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quad = read_quad(req, node, OFFSET_OPT, timeout_ms)?;

        params
            .input_modes
            .iter()
            .enumerate()
            .for_each(|(i, mode)| serialize_opt_iface_mode(mode, &mut quad, i > 0, false));

        params
            .output_modes
            .iter()
            .enumerate()
            .for_each(|(i, mode)| serialize_opt_iface_mode(mode, &mut quad, i > 0, true));

        write_quad(req, node, OFFSET_OPT, quad, timeout_ms)
    }
}

/// The protocol implementation for Audio Express.
#[derive(Default)]
pub struct AudioExpressProtocol;

impl MotuPortAssignSpecification for AudioExpressProtocol {
    const ASSIGN_PORT_TARGETS: &'static [TargetPort] = &[
        TargetPort::PhonePair,     // = Stream-1/2
        TargetPort::MainPair,      // = Stream-5/6
        TargetPort::AnalogPair(0), // = Stream-3/4
        TargetPort::SpdifPair,     // = Stream-7/8
                                   // Blank for Stream-9/10
    ];
    const ASSIGN_PORT_VALS: &'static [u8] = &[
        0x01, // = Stream-1/2
        0x02, // = Stream-5/6
        0x06, // = Stream-3/4
        0x07, // = Stream-7/8
              // Blank for Stream-9/10
    ];
}

impl MotuVersion3ClockSpecification for AudioExpressProtocol {
    const CLOCK_RATES: &'static [ClkRate] = &[
        ClkRate::R44100,
        ClkRate::R48000,
        ClkRate::R88200,
        ClkRate::R96000,
    ];
    const CLOCK_RATE_VALS: &'static [u8] = &[0x00, 0x01, 0x02, 0x03];

    const CLOCK_SRCS: &'static [V3ClkSrc] = &[V3ClkSrc::Internal, V3ClkSrc::SpdifCoax];
    const CLOCK_SRC_VALS: &'static [u8] = &[0x00, 0x01];
}

impl MotuRegisterDspSpecification for AudioExpressProtocol {
    const MIXER_OUTPUT_DESTINATIONS: &'static [TargetPort] = &[];
}

impl MotuRegisterDspMixerStereoSourceSpecification for AudioExpressProtocol {}

impl MotuRegisterDspStereoInputSpecification for AudioExpressProtocol {
    const MIC_COUNT: usize = 2;
}

impl MotuRegisterDspMeterSpecification for AudioExpressProtocol {
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
    const OUTPUT_PORT_PAIRS: &'static [TargetPort] = &[
        TargetPort::PhonePair,
        TargetPort::MainPair,
        TargetPort::AnalogPair(0),
        TargetPort::SpdifPair,
    ];
    const OUTPUT_PORT_PAIR_POS: &'static [[usize; 2]] = &[[0, 1], [2, 3], [10, 11], [12, 13]];
}

impl AudioExpressProtocol {
    /// Notification mask for footswitch.
    pub const NOTIFY_FOOTSWITCH_MASK: u32 = 0x01000000;
}

/// The protocol implementation for 828mk3 (FireWire only).
#[derive(Default, Debug)]
pub struct F828mk3Protocol;

const F828MK3_ASSIGN_PORT_TARGETS: &[TargetPort] = &[
    TargetPort::MainPair,        // = Stream-10/13
    TargetPort::AnalogPair(0),   // = Stream-2/3
    TargetPort::AnalogPair(1),   // = Stream-4/5
    TargetPort::AnalogPair(2),   // = Stream-6/7
    TargetPort::AnalogPair(3),   // = Stream-8/9
    TargetPort::SpdifPair,       // = Stream-12/13
    TargetPort::PhonePair,       // = Stream-0/1
    TargetPort::OpticalAPair(0), // = Stream-14/15
    TargetPort::OpticalAPair(1), // = Stream-16/17
    TargetPort::OpticalAPair(2), // = Stream-18/19
    TargetPort::OpticalAPair(3), // = Stream-20/21
    TargetPort::OpticalBPair(0), // = Stream-22/23
    TargetPort::OpticalBPair(1), // = Stream-24/25
    TargetPort::OpticalBPair(2), // = Stream-26/27
    TargetPort::OpticalBPair(3), // = Stream-28/29
];

const F828MK3_ASSIGN_PORT_VALS: &[u8] = &[
    0x00, // = Stream-10/13
    0x01, // = Stream-2/3
    0x02, // = Stream-4/5
    0x03, // = Stream-6/7
    0x04, // = Stream-8/9
    0x05, // = Stream-12/13
    0x06, // = Stream-0/1
    0x07, // = Stream-14/15
    0x08, // = Stream-16/17
    0x09, // = Stream-18/19
    0x0a, // = Stream-20/21
    0x0b, // = Stream-22/23
    0x0c, // = Stream-24/25
    0x0d, // = Stream-26/27
    0x0e, // = Stream-28/29
];

const F828MK3_CLOCK_RATES: &[ClkRate] = &[
    ClkRate::R44100,
    ClkRate::R48000,
    ClkRate::R88200,
    ClkRate::R96000,
    ClkRate::R176400,
    ClkRate::R192000,
];

const F828MK3_CLOCK_RATE_VALS: &[u8] = &[0x00, 0x01, 0x02, 0x03, 0x04, 0x05];

const F828MK3_CLOCK_SRCS: &[V3ClkSrc] = &[
    V3ClkSrc::Internal,
    V3ClkSrc::WordClk,
    V3ClkSrc::SpdifCoax,
    V3ClkSrc::SignalOptA,
    V3ClkSrc::SignalOptB,
];

const F828MK3_CLOCK_SRC_VALS: &[u8] = &[0x00, 0x01, 0x10, 0x18, 0x19];

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

impl MotuPortAssignSpecification for F828mk3Protocol {
    const ASSIGN_PORT_TARGETS: &'static [TargetPort] = F828MK3_ASSIGN_PORT_TARGETS;
    const ASSIGN_PORT_VALS: &'static [u8] = F828MK3_ASSIGN_PORT_VALS;
}

impl MotuWordClockOutputSpecification for F828mk3Protocol {}

impl MotuClockNameDisplaySpecification for F828mk3Protocol {}

impl MotuVersion3ClockSpecification for F828mk3Protocol {
    const CLOCK_RATES: &'static [ClkRate] = F828MK3_CLOCK_RATES;
    const CLOCK_RATE_VALS: &'static [u8] = F828MK3_CLOCK_RATE_VALS;

    const CLOCK_SRCS: &'static [V3ClkSrc] = F828MK3_CLOCK_SRCS;
    const CLOCK_SRC_VALS: &'static [u8] = F828MK3_CLOCK_SRC_VALS;
}

impl MotuVersion3OpticalIfaceSpecification for F828mk3Protocol {
    const OPT_IFACE_COUNT: usize = 2;
}

impl CommandDspOperation for F828mk3Protocol {}

impl MotuCommandDspReverbSpecification for F828mk3Protocol {}

impl MotuCommandDspMonitorSpecification for F828mk3Protocol {
    const RETURN_ASSIGN_TARGETS: &'static [TargetPort] = F828MK3_RETURN_ASSIGN_TARGETS;
}

impl MotuCommandDspMixerSpecification for F828mk3Protocol {
    const SOURCE_PORTS: &'static [TargetPort] = F828MK3_MIXER_SOURCE_PORTS;
    const OUTPUT_PORTS: &'static [TargetPort] = F828MK3_MIXER_OUTPUT_PORTS;
}

impl MotuCommandDspEqualizerSpecification for F828mk3Protocol {}

impl MotuCommandDspDynamicsSpecification for F828mk3Protocol {}

impl MotuCommandDspInputSpecification for F828mk3Protocol {
    const INPUT_PORTS: &'static [TargetPort] = F828MK3_INPUT_PORTS;
    const MIC_COUNT: usize = 0;
    const LINE_INPUT_COUNT: usize = 0;
}

impl MotuCommandDspOutputSpecification for F828mk3Protocol {
    const OUTPUT_PORTS: &'static [TargetPort] = F828MK3_OUTPUT_PORTS;
}

impl MotuCommandDspMeterSpecification for F828mk3Protocol {
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
#[derive(Default, Debug)]
pub struct F828mk3HybridProtocol;

impl MotuPortAssignSpecification for F828mk3HybridProtocol {
    const ASSIGN_PORT_TARGETS: &'static [TargetPort] = F828MK3_ASSIGN_PORT_TARGETS;
    const ASSIGN_PORT_VALS: &'static [u8] = F828MK3_ASSIGN_PORT_VALS;
}

impl MotuWordClockOutputSpecification for F828mk3HybridProtocol {}

impl MotuClockNameDisplaySpecification for F828mk3HybridProtocol {}

impl MotuVersion3ClockSpecification for F828mk3HybridProtocol {
    const CLOCK_RATES: &'static [ClkRate] = F828MK3_CLOCK_RATES;
    const CLOCK_RATE_VALS: &'static [u8] = F828MK3_CLOCK_RATE_VALS;

    const CLOCK_SRCS: &'static [V3ClkSrc] = F828MK3_CLOCK_SRCS;
    const CLOCK_SRC_VALS: &'static [u8] = F828MK3_CLOCK_SRC_VALS;
}

impl MotuVersion3OpticalIfaceSpecification for F828mk3HybridProtocol {
    const OPT_IFACE_COUNT: usize = 2;
}

impl CommandDspOperation for F828mk3HybridProtocol {}

impl MotuCommandDspReverbSpecification for F828mk3HybridProtocol {}

impl MotuCommandDspMonitorSpecification for F828mk3HybridProtocol {
    const RETURN_ASSIGN_TARGETS: &'static [TargetPort] = F828MK3_RETURN_ASSIGN_TARGETS;
}

impl MotuCommandDspEqualizerSpecification for F828mk3HybridProtocol {}

impl MotuCommandDspDynamicsSpecification for F828mk3HybridProtocol {}

impl MotuCommandDspMixerSpecification for F828mk3HybridProtocol {
    const SOURCE_PORTS: &'static [TargetPort] = F828MK3_MIXER_SOURCE_PORTS;
    const OUTPUT_PORTS: &'static [TargetPort] = F828MK3_MIXER_OUTPUT_PORTS;
}

impl MotuCommandDspInputSpecification for F828mk3HybridProtocol {
    const INPUT_PORTS: &'static [TargetPort] = F828MK3_INPUT_PORTS;
    // The mic functions are not configureble by command. They are just hard-wired.
    const MIC_COUNT: usize = 0;
    const LINE_INPUT_COUNT: usize = 0;
}

impl MotuCommandDspOutputSpecification for F828mk3HybridProtocol {
    const OUTPUT_PORTS: &'static [TargetPort] = F828MK3_OUTPUT_PORTS;
}

impl MotuCommandDspMeterSpecification for F828mk3HybridProtocol {
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

impl MotuPortAssignSpecification for H4preProtocol {
    const ASSIGN_PORT_TARGETS: &'static [TargetPort] = &[
        TargetPort::PhonePair,     // = Stream-1/2
        TargetPort::MainPair,      // = Stream-5/6
        TargetPort::AnalogPair(0), // = Stream-3/4
        TargetPort::SpdifPair,     // = Stream-7/8
                                   // Blank for Stream-9/10
    ];

    const ASSIGN_PORT_VALS: &'static [u8] = &[
        0x01, // = Stream-1/2
        0x02, // = Stream-5/6
        0x06, // = Stream-3/4
        0x07, // = Stream-7/8
              // Blank for Stream-9/10
    ];
}

impl MotuVersion3ClockSpecification for H4preProtocol {
    const CLOCK_RATES: &'static [ClkRate] = &[
        ClkRate::R44100,
        ClkRate::R48000,
        ClkRate::R88200,
        ClkRate::R96000,
    ];
    const CLOCK_RATE_VALS: &'static [u8] = &[0x00, 0x01, 0x02, 0x03];

    const CLOCK_SRCS: &'static [V3ClkSrc] = &[V3ClkSrc::Internal, V3ClkSrc::SpdifCoax];
    const CLOCK_SRC_VALS: &'static [u8] = &[0x00, 0x01];
}

impl MotuRegisterDspSpecification for H4preProtocol {
    const MIXER_OUTPUT_DESTINATIONS: &'static [TargetPort] = &[];
}

impl MotuRegisterDspMixerStereoSourceSpecification for H4preProtocol {}

impl MotuRegisterDspStereoInputSpecification for H4preProtocol {
    const MIC_COUNT: usize = 4;
}

impl MotuRegisterDspMeterSpecification for H4preProtocol {
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
    const OUTPUT_PORT_PAIRS: &'static [TargetPort] = &[
        TargetPort::PhonePair,
        TargetPort::MainPair,
        TargetPort::AnalogPair(0),
        TargetPort::SpdifPair,
    ];
    const OUTPUT_PORT_PAIR_POS: &'static [[usize; 2]] = &[[0, 1], [2, 3], [10, 11], [12, 13]];
}

const F896_MK3_ASSIGN_PORT_TARGETS: &[TargetPort] = &[
    TargetPort::MainPair,      // = Stream-0/1
    TargetPort::AnalogPair(0), // = Stream-2/3
    TargetPort::AnalogPair(1), // = Stream-4/5
    TargetPort::AnalogPair(2), // = Stream-6/7
    TargetPort::AnalogPair(3), // = Stream-8/9
    TargetPort::AesEbuPair,    // = Stream-10/11
    TargetPort::SpdifPair,     // = Stream-12/13
    TargetPort::PhonePair,     // = Stream-14/15
    // = Stream-16/17 for dummy
    TargetPort::OpticalAPair(0), // = Stream-18/19
    TargetPort::OpticalAPair(1), // = Stream-20/21
    TargetPort::OpticalAPair(2), // = Stream-22/23
    TargetPort::OpticalAPair(3), // = Stream-24/25
    TargetPort::OpticalBPair(0), // = Stream-26/27
    TargetPort::OpticalBPair(1), // = Stream-28/29
    TargetPort::OpticalBPair(2), // = Stream-30/31
    TargetPort::OpticalBPair(3), // = Stream-32/33
];

const F896_MK3_ASSIGN_PORT_VALS: &[u8] = &[
    0x00, // = Stream-0/1
    0x01, // = Stream-2/3
    0x02, // = Stream-4/5
    0x03, // = Stream-6/7
    0x04, // = Stream-8/9
    0x05, // = Stream-10/11
    0x06, // = Stream-12/13
    0x07, // = Stream-14/15
    0x08, // = Stream-18/19
    0x09, // = Stream-20/21
    0x0a, // = Stream-22/23
    0x0b, // = Stream-24/25
    0x0c, // = Stream-26/27
    0x0d, // = Stream-28/29
    0x0e, // = Stream-30/31
    0x0f, // = Stream-32/33
];

const F896_MK3_CLOCK_RATES: &[ClkRate] = &[
    ClkRate::R44100,
    ClkRate::R48000,
    ClkRate::R88200,
    ClkRate::R96000,
    ClkRate::R176400,
    ClkRate::R192000,
];

const F896_MK3_CLOCK_RATE_VALS: &[u8] = &[0x00, 0x01, 0x02, 0x03, 0x04, 0x05];

const F896_MK3_CLOCK_SRCS: &[V3ClkSrc] = &[
    V3ClkSrc::Internal,
    V3ClkSrc::WordClk,
    V3ClkSrc::AesEbuXlr,
    V3ClkSrc::SpdifCoax,
    V3ClkSrc::SignalOptA,
    V3ClkSrc::SignalOptB,
];

const F896_MK3_CLOCK_SRC_VALS: &[u8] = &[0x00, 0x01, 0x08, 0x10, 0x18, 0x19];

const F896_MK3_RETURN_ASSIGN_TARGETS: &[TargetPort] = &[
    TargetPort::MainPair,
    TargetPort::AnalogPair(0),
    TargetPort::AnalogPair(1),
    TargetPort::AnalogPair(2),
    TargetPort::AnalogPair(3),
    TargetPort::AesEbuPair,
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

const F896_MK3_MIXER_SOURCE_PORTS: &[TargetPort] = &[
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
    TargetPort::AesEbu(0),
    TargetPort::AesEbu(1),
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

const F896_MK3_MIXER_OUTPUT_PORTS: &[TargetPort] = &[
    TargetPort::Disabled,
    TargetPort::MainPair,
    TargetPort::AnalogPair(0),
    TargetPort::AnalogPair(1),
    TargetPort::AnalogPair(2),
    TargetPort::AnalogPair(3),
    TargetPort::AesEbuPair,
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

const F896_MK3_INPUT_PORTS: &[TargetPort] = &[
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
    TargetPort::AesEbu(0),
    TargetPort::AesEbu(1),
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

const F896_MK3_OUTPUT_PORTS: &[TargetPort] = &[
    TargetPort::MainPair,
    TargetPort::AnalogPair(0),
    TargetPort::AnalogPair(1),
    TargetPort::AnalogPair(2),
    TargetPort::AnalogPair(3),
    TargetPort::AesEbuPair,
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

const F896_MK3_METER_INPUT_PORTS: &[(TargetPort, usize)] = &[
    (TargetPort::Analog(0), 8),
    (TargetPort::Analog(1), 9),
    (TargetPort::Analog(2), 10),
    (TargetPort::Analog(3), 11),
    (TargetPort::Analog(4), 12),
    (TargetPort::Analog(5), 13),
    (TargetPort::Analog(6), 14),
    (TargetPort::Analog(7), 15),
    (TargetPort::AesEbu(0), 18),
    (TargetPort::AesEbu(1), 19),
    (TargetPort::Spdif(0), 16),
    (TargetPort::Spdif(1), 17),
    (TargetPort::OpticalA(0), 20),
    (TargetPort::OpticalA(1), 21),
    (TargetPort::OpticalA(2), 22),
    (TargetPort::OpticalA(3), 23),
    (TargetPort::OpticalA(4), 24),
    (TargetPort::OpticalA(5), 25),
    (TargetPort::OpticalA(6), 26),
    (TargetPort::OpticalA(7), 27),
    (TargetPort::OpticalB(0), 28),
    (TargetPort::OpticalB(1), 29),
    (TargetPort::OpticalB(2), 30),
    (TargetPort::OpticalB(3), 31),
    (TargetPort::OpticalB(4), 32),
    (TargetPort::OpticalB(5), 33),
    (TargetPort::OpticalB(6), 34),
    (TargetPort::OpticalB(7), 35),
];

const F896_MK3_METER_OUTPUT_PORTS: &[(TargetPort, usize)] = &[
    (TargetPort::Main(0), 82),
    (TargetPort::Main(1), 83),
    (TargetPort::Analog(0), 84),
    (TargetPort::Analog(1), 85),
    (TargetPort::Analog(2), 86),
    (TargetPort::Analog(3), 87),
    (TargetPort::Analog(4), 88),
    (TargetPort::Analog(5), 89),
    (TargetPort::Analog(6), 90),
    (TargetPort::Analog(7), 91),
    (TargetPort::AesEbu(0), 92),
    (TargetPort::AesEbu(1), 93),
    (TargetPort::Spdif(0), 80),
    (TargetPort::Spdif(1), 81),
    (TargetPort::Phone(0), 94),
    (TargetPort::Phone(1), 95),
    (TargetPort::OpticalA(0), 96),
    (TargetPort::OpticalA(1), 97),
    (TargetPort::OpticalA(2), 98),
    (TargetPort::OpticalA(3), 99),
    (TargetPort::OpticalA(4), 100),
    (TargetPort::OpticalA(5), 101),
    (TargetPort::OpticalA(6), 102),
    (TargetPort::OpticalA(7), 103),
    (TargetPort::OpticalB(0), 104),
    (TargetPort::OpticalB(1), 105),
    (TargetPort::OpticalB(2), 106),
    (TargetPort::OpticalB(3), 107),
    (TargetPort::OpticalB(4), 108),
    (TargetPort::OpticalB(5), 109),
    (TargetPort::OpticalB(6), 110),
    (TargetPort::OpticalB(7), 111),
];

const F896_MK3_LEVEL_METERS_PROGRAMMABLE_MODES: &[LevelMetersProgrammableMode] = &[
    LevelMetersProgrammableMode::AnalogOutput,
    LevelMetersProgrammableMode::AdatAInput,
    LevelMetersProgrammableMode::AdatAOutput,
    LevelMetersProgrammableMode::AdatBInput,
    LevelMetersProgrammableMode::AdatBOutput,
    LevelMetersProgrammableMode::AesEbuInputOutput,
];

const F896_MK3_OFFSET_AES_EBU_RATE_CONVERTER: u32 = 0x0c90;

const F896_MK3_NOTIFY_PORT_CHANGE_MASK: u32 = 0x40000000;

const F896_MK3_NOTIFY_FOOTSWITCH_MASK: u32 = 0x01000000;

/// Mode of rate convert for AES/EBU input/output signals.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum F896mk3AesebuRateConvertMode {
    /// Not available.
    None,
    /// The rate of input signal is converted to system rate.
    InputToSystem,
    /// The rate of output signal is slave to input, ignoring system rate.
    OutputDependsInput,
    /// The rate of output signal is at 44.1 kHz.
    Output441,
    /// The rate of output signal is at 48.0 kHz.
    Output480,
    /// The rate of output signal is at 88.2 kHz.
    Output882,
    /// The rate of output signal is at 96.0 kHz.
    Output960,
}

impl Default for F896mk3AesebuRateConvertMode {
    fn default() -> Self {
        Self::None
    }
}

/// The trait for specification of AES/EBU sampling rate converter in F896mk3.
pub trait F896mk3AesebuRateConvertSpecification {}

fn serialize_f896mk3_aes_ebu_rate_converter_mode(
    mode: &F896mk3AesebuRateConvertMode,
    quad: &mut u32,
) {
    *quad = match mode {
        F896mk3AesebuRateConvertMode::Output960 => 6,
        F896mk3AesebuRateConvertMode::Output882 => 5,
        F896mk3AesebuRateConvertMode::Output480 => 4,
        F896mk3AesebuRateConvertMode::Output441 => 3,
        F896mk3AesebuRateConvertMode::OutputDependsInput => 2,
        F896mk3AesebuRateConvertMode::InputToSystem => 1,
        F896mk3AesebuRateConvertMode::None => 0,
    };
}

fn deserialize_f896mk3_aes_ebu_rate_converter_mode(
    mode: &mut F896mk3AesebuRateConvertMode,
    quad: &u32,
) {
    *mode = match quad {
        6 => F896mk3AesebuRateConvertMode::Output960,
        5 => F896mk3AesebuRateConvertMode::Output882,
        4 => F896mk3AesebuRateConvertMode::Output480,
        3 => F896mk3AesebuRateConvertMode::Output441,
        2 => F896mk3AesebuRateConvertMode::OutputDependsInput,
        1 => F896mk3AesebuRateConvertMode::InputToSystem,
        _ => F896mk3AesebuRateConvertMode::None,
    }
}

impl<O> MotuWhollyCacheableParamsOperation<F896mk3AesebuRateConvertMode> for O
where
    O: F896mk3AesebuRateConvertSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        mode: &mut F896mk3AesebuRateConvertMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let quad = read_quad(
            req,
            node,
            F896_MK3_OFFSET_AES_EBU_RATE_CONVERTER,
            timeout_ms,
        )?;
        deserialize_f896mk3_aes_ebu_rate_converter_mode(mode, &quad);
        Ok(())
    }
}

impl<O> MotuWhollyUpdatableParamsOperation<F896mk3AesebuRateConvertMode> for O
where
    O: F896mk3AesebuRateConvertSpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        mode: &F896mk3AesebuRateConvertMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quad = read_quad(
            req,
            node,
            F896_MK3_OFFSET_AES_EBU_RATE_CONVERTER,
            timeout_ms,
        )?;
        serialize_f896mk3_aes_ebu_rate_converter_mode(mode, &mut quad);
        write_quad(
            req,
            node,
            F896_MK3_OFFSET_AES_EBU_RATE_CONVERTER,
            quad,
            timeout_ms,
        )
    }
}

/// The protocol implementation for 896 mk3 (FireWire only).
#[derive(Default, Debug)]
pub struct F896mk3Protocol;

impl MotuPortAssignSpecification for F896mk3Protocol {
    const ASSIGN_PORT_TARGETS: &'static [TargetPort] = F896_MK3_ASSIGN_PORT_TARGETS;
    const ASSIGN_PORT_VALS: &'static [u8] = F896_MK3_ASSIGN_PORT_VALS;
}

impl MotuWordClockOutputSpecification for F896mk3Protocol {}

impl MotuClockNameDisplaySpecification for F896mk3Protocol {}

impl MotuVersion3ClockSpecification for F896mk3Protocol {
    const CLOCK_RATES: &'static [ClkRate] = F896_MK3_CLOCK_RATES;
    const CLOCK_RATE_VALS: &'static [u8] = F896_MK3_CLOCK_RATE_VALS;

    const CLOCK_SRCS: &'static [V3ClkSrc] = F896_MK3_CLOCK_SRCS;
    const CLOCK_SRC_VALS: &'static [u8] = F896_MK3_CLOCK_SRC_VALS;
}

impl MotuVersion3OpticalIfaceSpecification for F896mk3Protocol {
    const OPT_IFACE_COUNT: usize = 2;
}

impl CommandDspOperation for F896mk3Protocol {}

impl MotuCommandDspReverbSpecification for F896mk3Protocol {}

impl MotuCommandDspMonitorSpecification for F896mk3Protocol {
    const RETURN_ASSIGN_TARGETS: &'static [TargetPort] = F896_MK3_RETURN_ASSIGN_TARGETS;
}

impl MotuCommandDspMixerSpecification for F896mk3Protocol {
    const SOURCE_PORTS: &'static [TargetPort] = F896_MK3_MIXER_SOURCE_PORTS;
    const OUTPUT_PORTS: &'static [TargetPort] = F896_MK3_MIXER_OUTPUT_PORTS;
}

impl MotuCommandDspEqualizerSpecification for F896mk3Protocol {}

impl MotuCommandDspDynamicsSpecification for F896mk3Protocol {}

impl MotuCommandDspInputSpecification for F896mk3Protocol {
    const INPUT_PORTS: &'static [TargetPort] = F896_MK3_INPUT_PORTS;
    const MIC_COUNT: usize = 0;
    const LINE_INPUT_COUNT: usize = 0;
}

impl MotuCommandDspOutputSpecification for F896mk3Protocol {
    const OUTPUT_PORTS: &'static [TargetPort] = F896_MK3_OUTPUT_PORTS;
}

impl MotuCommandDspMeterSpecification for F896mk3Protocol {
    const INPUT_PORTS: &'static [(TargetPort, usize)] = F896_MK3_METER_INPUT_PORTS;
    const OUTPUT_PORTS: &'static [(TargetPort, usize)] = F896_MK3_METER_OUTPUT_PORTS;
}

impl MotuLevelMetersSpecification for F896mk3Protocol {
    const LEVEL_METERS_PROGRAMMABLE_MODES: &'static [LevelMetersProgrammableMode] =
        F896_MK3_LEVEL_METERS_PROGRAMMABLE_MODES;
}

impl F896mk3AesebuRateConvertSpecification for F896mk3Protocol {}

impl F896mk3Protocol {
    /// Notification mask for main assignment, return assignment, and phone assignment, as well as
    /// programmable level meter. The change of phone assignment is also notified in command
    /// message.
    pub const NOTIFY_PORT_CHANGE_MASK: u32 = F896_MK3_NOTIFY_PORT_CHANGE_MASK;

    /// Notification mask for footswitch.
    pub const NOTIFY_FOOTSWITCH_MASK: u32 = F896_MK3_NOTIFY_FOOTSWITCH_MASK;
}

/// The protocol implementation for 896 mk3 (Hybrid).
#[derive(Default, Debug)]
pub struct F896mk3HybridProtocol;

impl MotuPortAssignSpecification for F896mk3HybridProtocol {
    const ASSIGN_PORT_TARGETS: &'static [TargetPort] = F896_MK3_ASSIGN_PORT_TARGETS;
    const ASSIGN_PORT_VALS: &'static [u8] = F896_MK3_ASSIGN_PORT_VALS;
}

impl MotuWordClockOutputSpecification for F896mk3HybridProtocol {}

impl MotuClockNameDisplaySpecification for F896mk3HybridProtocol {}

impl MotuVersion3ClockSpecification for F896mk3HybridProtocol {
    const CLOCK_RATES: &'static [ClkRate] = F896_MK3_CLOCK_RATES;
    const CLOCK_RATE_VALS: &'static [u8] = F896_MK3_CLOCK_RATE_VALS;

    const CLOCK_SRCS: &'static [V3ClkSrc] = F896_MK3_CLOCK_SRCS;
    const CLOCK_SRC_VALS: &'static [u8] = F896_MK3_CLOCK_SRC_VALS;
}

impl MotuVersion3OpticalIfaceSpecification for F896mk3HybridProtocol {
    const OPT_IFACE_COUNT: usize = 2;
}

impl CommandDspOperation for F896mk3HybridProtocol {}

impl MotuCommandDspReverbSpecification for F896mk3HybridProtocol {}

impl MotuCommandDspMonitorSpecification for F896mk3HybridProtocol {
    const RETURN_ASSIGN_TARGETS: &'static [TargetPort] = F896_MK3_RETURN_ASSIGN_TARGETS;
}

impl MotuCommandDspMixerSpecification for F896mk3HybridProtocol {
    const SOURCE_PORTS: &'static [TargetPort] = F896_MK3_MIXER_SOURCE_PORTS;
    const OUTPUT_PORTS: &'static [TargetPort] = F896_MK3_MIXER_OUTPUT_PORTS;
}

impl MotuCommandDspEqualizerSpecification for F896mk3HybridProtocol {}

impl MotuCommandDspDynamicsSpecification for F896mk3HybridProtocol {}

impl MotuCommandDspInputSpecification for F896mk3HybridProtocol {
    const INPUT_PORTS: &'static [TargetPort] = F896_MK3_INPUT_PORTS;
    const MIC_COUNT: usize = 0;
    const LINE_INPUT_COUNT: usize = 0;
}

impl MotuCommandDspOutputSpecification for F896mk3HybridProtocol {
    const OUTPUT_PORTS: &'static [TargetPort] = F896_MK3_OUTPUT_PORTS;
}

impl MotuCommandDspMeterSpecification for F896mk3HybridProtocol {
    const INPUT_PORTS: &'static [(TargetPort, usize)] = F896_MK3_METER_INPUT_PORTS;
    const OUTPUT_PORTS: &'static [(TargetPort, usize)] = F896_MK3_METER_OUTPUT_PORTS;
}

impl MotuLevelMetersSpecification for F896mk3HybridProtocol {
    const LEVEL_METERS_PROGRAMMABLE_MODES: &'static [LevelMetersProgrammableMode] =
        F896_MK3_LEVEL_METERS_PROGRAMMABLE_MODES;
}

impl F896mk3AesebuRateConvertSpecification for F896mk3HybridProtocol {}

impl F896mk3HybridProtocol {
    /// Notification mask for main assignment, return assignment, and phone assignment, as well as
    /// programmable level meter. The change of phone assignment is also notified in command
    /// message.
    pub const NOTIFY_PORT_CHANGE_MASK: u32 = F896_MK3_NOTIFY_PORT_CHANGE_MASK;

    /// Notification mask for footswitch.
    pub const NOTIFY_FOOTSWITCH_MASK: u32 = F896_MK3_NOTIFY_FOOTSWITCH_MASK;
}

/// The protocol implementation for Ultralite mk3 (FireWire only).
#[derive(Default, Debug)]
pub struct UltraliteMk3Protocol;

const ULTRALITE_MK3_ASSIGN_PORT_TARGETS: &[TargetPort] = &[
    TargetPort::MainPair,      // = Stream-0/1
    TargetPort::AnalogPair(0), // = Stream-2/3
    TargetPort::AnalogPair(1), // = Stream-4/5
    TargetPort::AnalogPair(2), // = Stream-6/7
    TargetPort::AnalogPair(3), // = Stream-8/9
    TargetPort::SpdifPair,     // = Stream-12/13
    TargetPort::PhonePair,     // = Stream-10/11
];

const ULTRALITE_MK3_ASSIGN_PORT_VALS: &[u8] = &[
    0x00, // = Stream-0/1
    0x01, // = Stream-2/3
    0x02, // = Stream-4/5
    0x03, // = Stream-6/7
    0x04, // = Stream-8/9
    0x05, // = Stream-12/13
    0x06, // = Stream-10/11
];

const ULTRALITE_MK3_CLOCK_RATES: &[ClkRate] = &[
    ClkRate::R44100,
    ClkRate::R48000,
    ClkRate::R88200,
    ClkRate::R96000,
];

const ULTRALITE_MK3_CLOCK_RATE_VALS: &[u8] = &[0x00, 0x01, 0x02, 0x03];

const ULTRALITE_MK3_CLOCK_SRCS: &[V3ClkSrc] = &[V3ClkSrc::Internal, V3ClkSrc::SpdifCoax];

const ULTRALITE_MK3_CLOCK_SRC_VALS: &[u8] = &[0x00, 0x01];

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

impl MotuPortAssignSpecification for UltraliteMk3Protocol {
    const ASSIGN_PORT_TARGETS: &'static [TargetPort] = ULTRALITE_MK3_ASSIGN_PORT_TARGETS;
    const ASSIGN_PORT_VALS: &'static [u8] = ULTRALITE_MK3_ASSIGN_PORT_VALS;
}

impl MotuClockNameDisplaySpecification for UltraliteMk3Protocol {}

impl MotuVersion3ClockSpecification for UltraliteMk3Protocol {
    const CLOCK_RATES: &'static [ClkRate] = ULTRALITE_MK3_CLOCK_RATES;
    const CLOCK_RATE_VALS: &'static [u8] = ULTRALITE_MK3_CLOCK_RATE_VALS;

    const CLOCK_SRCS: &'static [V3ClkSrc] = ULTRALITE_MK3_CLOCK_SRCS;
    const CLOCK_SRC_VALS: &'static [u8] = ULTRALITE_MK3_CLOCK_SRC_VALS;
}

impl CommandDspOperation for UltraliteMk3Protocol {}

impl MotuCommandDspReverbSpecification for UltraliteMk3Protocol {}

impl MotuCommandDspMonitorSpecification for UltraliteMk3Protocol {
    const RETURN_ASSIGN_TARGETS: &'static [TargetPort] = ULTRALITE_MK3_RETURN_ASSIGN_TARGETS;
}

impl MotuCommandDspEqualizerSpecification for UltraliteMk3Protocol {}

impl MotuCommandDspDynamicsSpecification for UltraliteMk3Protocol {}

impl MotuCommandDspMixerSpecification for UltraliteMk3Protocol {
    const SOURCE_PORTS: &'static [TargetPort] = ULTRALITE_MK3_MIXER_SOURCE_PORTS;
    const OUTPUT_PORTS: &'static [TargetPort] = ULTRALITE_MK3_MIXER_OUTPUT_PORTS;
}

impl MotuCommandDspInputSpecification for UltraliteMk3Protocol {
    const INPUT_PORTS: &'static [TargetPort] = ULTRALITE_MK3_INPUT_PORTS;
    // The mic functions are not configureble by command. They are just hard-wired.
    const MIC_COUNT: usize = 0;
    const LINE_INPUT_COUNT: usize = 0;
}

impl MotuCommandDspOutputSpecification for UltraliteMk3Protocol {
    const OUTPUT_PORTS: &'static [TargetPort] = ULTRALITE_MK3_OUTPUT_PORTS;
}

impl UltraliteMk3Protocol {
    /// Notification mask for main assignment, return assignment, and phone assignment. The change
    /// of phone assignment is also notified in command message.
    pub const NOTIFY_PORT_CHANGE: u32 = 0x40000000;
}

impl MotuCommandDspMeterSpecification for UltraliteMk3Protocol {
    const INPUT_PORTS: &'static [(TargetPort, usize)] = ULTRALITEMK3_METER_INPUT_PORTS;
    const OUTPUT_PORTS: &'static [(TargetPort, usize)] = ULTRALITEMK3_METER_OUTPUT_PORTS;
}

/// The protocol implementation for Ultralite mk3 Hybrid.
#[derive(Default, Debug)]
pub struct UltraliteMk3HybridProtocol;

impl MotuPortAssignSpecification for UltraliteMk3HybridProtocol {
    const ASSIGN_PORT_TARGETS: &'static [TargetPort] = ULTRALITE_MK3_ASSIGN_PORT_TARGETS;
    const ASSIGN_PORT_VALS: &'static [u8] = ULTRALITE_MK3_ASSIGN_PORT_VALS;
}

impl MotuClockNameDisplaySpecification for UltraliteMk3HybridProtocol {}

impl MotuVersion3ClockSpecification for UltraliteMk3HybridProtocol {
    const CLOCK_RATES: &'static [ClkRate] = ULTRALITE_MK3_CLOCK_RATES;
    const CLOCK_RATE_VALS: &'static [u8] = ULTRALITE_MK3_CLOCK_RATE_VALS;

    const CLOCK_SRCS: &'static [V3ClkSrc] = ULTRALITE_MK3_CLOCK_SRCS;
    const CLOCK_SRC_VALS: &'static [u8] = ULTRALITE_MK3_CLOCK_SRC_VALS;
}

impl CommandDspOperation for UltraliteMk3HybridProtocol {}

impl MotuCommandDspReverbSpecification for UltraliteMk3HybridProtocol {}

impl MotuCommandDspMonitorSpecification for UltraliteMk3HybridProtocol {
    const RETURN_ASSIGN_TARGETS: &'static [TargetPort] = ULTRALITE_MK3_RETURN_ASSIGN_TARGETS;
}

impl MotuCommandDspEqualizerSpecification for UltraliteMk3HybridProtocol {}

impl MotuCommandDspDynamicsSpecification for UltraliteMk3HybridProtocol {}

impl MotuCommandDspMixerSpecification for UltraliteMk3HybridProtocol {
    const SOURCE_PORTS: &'static [TargetPort] = ULTRALITE_MK3_MIXER_SOURCE_PORTS;
    const OUTPUT_PORTS: &'static [TargetPort] = ULTRALITE_MK3_MIXER_OUTPUT_PORTS;
}

impl MotuCommandDspInputSpecification for UltraliteMk3HybridProtocol {
    const INPUT_PORTS: &'static [TargetPort] = ULTRALITE_MK3_INPUT_PORTS;
    const MIC_COUNT: usize = 2;
    const LINE_INPUT_COUNT: usize = 0;
}

impl MotuCommandDspOutputSpecification for UltraliteMk3HybridProtocol {
    const OUTPUT_PORTS: &'static [TargetPort] = ULTRALITE_MK3_OUTPUT_PORTS;
}

impl MotuCommandDspMeterSpecification for UltraliteMk3HybridProtocol {
    const INPUT_PORTS: &'static [(TargetPort, usize)] = ULTRALITEMK3_METER_INPUT_PORTS;
    const OUTPUT_PORTS: &'static [(TargetPort, usize)] = ULTRALITEMK3_METER_OUTPUT_PORTS;
}

impl UltraliteMk3HybridProtocol {
    /// Notification mask for main assignment, return assignment, and phone assignment. The change
    /// of phone assignment is also notified in command message.
    pub const NOTIFY_PORT_CHANGE: u32 = 0x40000000;
}

/// The protocol implementation for Traveler mk3.
#[derive(Default, Debug)]
pub struct TravelerMk3Protocol;

impl MotuPortAssignSpecification for TravelerMk3Protocol {
    const ASSIGN_PORT_TARGETS: &'static [TargetPort] = &[
        TargetPort::AnalogPair(0),   // = Stream-2/3
        TargetPort::AnalogPair(1),   // = Stream-4/5
        TargetPort::AnalogPair(2),   // = Stream-6/7
        TargetPort::AnalogPair(3),   // = Stream-8/9
        TargetPort::AesEbuPair,      // = Stream-10/11
        TargetPort::SpdifPair,       // = Stream-12/13
        TargetPort::PhonePair,       // = Stream-0/1
        TargetPort::OpticalAPair(0), // = Stream-14/15
        TargetPort::OpticalAPair(1), // = Stream-16/17
        TargetPort::OpticalAPair(2), // = Stream-18/19
        TargetPort::OpticalAPair(3), // = Stream-20/21
        TargetPort::OpticalBPair(0), // = Stream-22/23
        TargetPort::OpticalBPair(1), // = Stream-24/25
        TargetPort::OpticalBPair(2), // = Stream-26/27
        TargetPort::OpticalBPair(3), // = Stream-28/29
    ];
    const ASSIGN_PORT_VALS: &'static [u8] = &[
        0x00, // = Stream-2/3
        0x01, // = Stream-4/5
        0x02, // = Stream-6/7
        0x03, // = Stream-8/9
        0x04, // = Stream-10/11
        0x05, // = Stream-12/13
        0x06, // = Stream-0/1
        0x07, // = Stream-14/15
        0x08, // = Stream-16/17
        0x09, // = Stream-18/19
        0x0a, // = Stream-20/21
        0x0b, // = Stream-22/23
        0x0c, // = Stream-24/25
        0x0d, // = Stream-26/27
        0x0e, // = Stream-28/29
    ];
}

impl MotuClockNameDisplaySpecification for TravelerMk3Protocol {}

impl MotuVersion3ClockSpecification for TravelerMk3Protocol {
    const CLOCK_RATES: &'static [ClkRate] = &[
        ClkRate::R44100,
        ClkRate::R48000,
        ClkRate::R88200,
        ClkRate::R96000,
        ClkRate::R176400,
        ClkRate::R192000,
    ];
    const CLOCK_RATE_VALS: &'static [u8] = &[0x00, 0x01, 0x02, 0x03, 0x04, 0x05];

    const CLOCK_SRCS: &'static [V3ClkSrc] = &[
        V3ClkSrc::Internal,
        V3ClkSrc::WordClk,
        V3ClkSrc::AesEbuXlr,
        V3ClkSrc::SpdifCoax,
        V3ClkSrc::SignalOptA,
        V3ClkSrc::SignalOptB,
    ];
    const CLOCK_SRC_VALS: &'static [u8] = &[0x00, 0x01, 0x08, 0x10, 0x18, 0x19];
}

impl MotuVersion3OpticalIfaceSpecification for TravelerMk3Protocol {
    const OPT_IFACE_COUNT: usize = 2;
}

impl MotuWordClockOutputSpecification for TravelerMk3Protocol {}

impl CommandDspOperation for TravelerMk3Protocol {}

impl MotuCommandDspReverbSpecification for TravelerMk3Protocol {}

impl MotuCommandDspMonitorSpecification for TravelerMk3Protocol {
    const RETURN_ASSIGN_TARGETS: &'static [TargetPort] = &[
        TargetPort::AnalogPair(0),
        TargetPort::AnalogPair(1),
        TargetPort::AnalogPair(2),
        TargetPort::AnalogPair(3),
        TargetPort::AesEbuPair,
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
}

impl MotuCommandDspMixerSpecification for TravelerMk3Protocol {
    const SOURCE_PORTS: &'static [TargetPort] = &[
        TargetPort::Analog(0), // Mic-0
        TargetPort::Analog(1), // Mic-1
        TargetPort::Analog(2), // Mic-2
        TargetPort::Analog(3), // Mic-3
        TargetPort::Analog(4),
        TargetPort::Analog(5),
        TargetPort::Analog(6),
        TargetPort::Analog(7),
        TargetPort::AesEbu(0),
        TargetPort::AesEbu(1),
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
    const OUTPUT_PORTS: &'static [TargetPort] = &[
        TargetPort::Disabled,
        TargetPort::AnalogPair(0),
        TargetPort::AnalogPair(1),
        TargetPort::AnalogPair(2),
        TargetPort::AnalogPair(3),
        TargetPort::AesEbuPair,
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
}

impl MotuCommandDspEqualizerSpecification for TravelerMk3Protocol {}

impl MotuCommandDspDynamicsSpecification for TravelerMk3Protocol {}

impl MotuCommandDspInputSpecification for TravelerMk3Protocol {
    const INPUT_PORTS: &'static [TargetPort] = &[
        TargetPort::Analog(0), // Mic-0
        TargetPort::Analog(1), // Mic-1
        TargetPort::Analog(2), // Mic-2
        TargetPort::Analog(3), // Mic-3
        TargetPort::Analog(4),
        TargetPort::Analog(5),
        TargetPort::Analog(6),
        TargetPort::Analog(7),
        TargetPort::AesEbu(0),
        TargetPort::AesEbu(1),
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
    const MIC_COUNT: usize = 4;
    const LINE_INPUT_COUNT: usize = 4;
}

impl MotuCommandDspOutputSpecification for TravelerMk3Protocol {
    const OUTPUT_PORTS: &'static [TargetPort] = &[
        TargetPort::AnalogPair(0),
        TargetPort::AnalogPair(1),
        TargetPort::AnalogPair(2),
        TargetPort::AnalogPair(3),
        TargetPort::AesEbuPair,
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
}

impl MotuCommandDspMeterSpecification for TravelerMk3Protocol {
    const INPUT_PORTS: &'static [(TargetPort, usize)] = &[
        (TargetPort::Analog(0), 4),
        (TargetPort::Analog(1), 5),
        (TargetPort::Analog(2), 6),
        (TargetPort::Analog(3), 7),
        (TargetPort::Analog(4), 8),
        (TargetPort::Analog(5), 9),
        (TargetPort::Analog(6), 10),
        (TargetPort::Analog(7), 11),
        (TargetPort::AesEbu(0), 14),
        (TargetPort::AesEbu(1), 15),
        (TargetPort::Spdif(0), 12),
        (TargetPort::Spdif(1), 13),
        (TargetPort::OpticalA(0), 16),
        (TargetPort::OpticalA(1), 17),
        (TargetPort::OpticalA(2), 18),
        (TargetPort::OpticalA(3), 19),
        (TargetPort::OpticalA(4), 20),
        (TargetPort::OpticalA(5), 21),
        (TargetPort::OpticalA(6), 22),
        (TargetPort::OpticalA(7), 23),
        (TargetPort::OpticalB(0), 24),
        (TargetPort::OpticalB(1), 25),
        (TargetPort::OpticalB(2), 26),
        (TargetPort::OpticalB(3), 27),
        (TargetPort::OpticalB(4), 28),
        (TargetPort::OpticalB(5), 29),
        (TargetPort::OpticalB(6), 30),
        (TargetPort::OpticalB(7), 31),
        (TargetPort::Analog(0), 48),
        (TargetPort::Analog(1), 49),
        (TargetPort::Analog(2), 50),
        (TargetPort::Analog(3), 51),
        (TargetPort::Analog(4), 52),
        (TargetPort::Analog(5), 53),
        (TargetPort::Analog(6), 54),
        (TargetPort::Analog(7), 55),
        (TargetPort::AesEbu(0), 58),
        (TargetPort::AesEbu(1), 59),
        (TargetPort::Spdif(0), 56),
        (TargetPort::Spdif(1), 57),
        (TargetPort::OpticalA(0), 60),
        (TargetPort::OpticalA(1), 61),
        (TargetPort::OpticalA(2), 62),
        (TargetPort::OpticalA(3), 63),
        (TargetPort::OpticalA(4), 64),
        (TargetPort::OpticalA(5), 65),
        (TargetPort::OpticalA(6), 66),
        (TargetPort::OpticalA(7), 67),
        (TargetPort::OpticalB(0), 68),
        (TargetPort::OpticalB(1), 69),
        (TargetPort::OpticalB(2), 70),
        (TargetPort::OpticalB(3), 71),
        (TargetPort::OpticalB(4), 72),
        (TargetPort::OpticalB(5), 73),
        (TargetPort::OpticalB(6), 74),
        (TargetPort::OpticalB(7), 75),
    ];
    const OUTPUT_PORTS: &'static [(TargetPort, usize)] = &[
        (TargetPort::Phone(0), 88),
        (TargetPort::Phone(1), 89),
        (TargetPort::Analog(0), 78),
        (TargetPort::Analog(1), 79),
        (TargetPort::Analog(2), 80),
        (TargetPort::Analog(3), 81),
        (TargetPort::Analog(4), 82),
        (TargetPort::Analog(5), 83),
        (TargetPort::Analog(6), 84),
        (TargetPort::Analog(7), 85),
        (TargetPort::AesEbu(0), 86),
        (TargetPort::AesEbu(1), 87),
        (TargetPort::Spdif(0), 76),
        (TargetPort::Spdif(1), 77),
        (TargetPort::OpticalA(0), 90),
        (TargetPort::OpticalA(1), 91),
        (TargetPort::OpticalA(2), 92),
        (TargetPort::OpticalA(3), 93),
        (TargetPort::OpticalA(4), 94),
        (TargetPort::OpticalA(5), 95),
        (TargetPort::OpticalA(6), 96),
        (TargetPort::OpticalA(7), 97),
        (TargetPort::OpticalB(0), 98),
        (TargetPort::OpticalB(1), 99),
        (TargetPort::OpticalB(2), 100),
        (TargetPort::OpticalB(3), 101),
        (TargetPort::OpticalB(4), 102),
        (TargetPort::OpticalB(5), 103),
        (TargetPort::OpticalB(6), 104),
        (TargetPort::OpticalB(7), 105),
    ];
}

impl TravelerMk3Protocol {
    /// Notification mask for main assignment, return assignment, and phone assignment. The change
    /// of phone assignment is also notified in command message.
    pub const NOTIFY_PORT_CHANGE: u32 = 0x40000000;
}

/// The protocol implementation for Track 16.
#[derive(Default, Debug)]
pub struct Track16Protocol;

impl MotuPortAssignSpecification for Track16Protocol {
    const ASSIGN_PORT_TARGETS: &'static [TargetPort] = &[
        TargetPort::AnalogPair(0),   // = Stream-2/3
        TargetPort::AnalogPair(1),   // = Stream-4/5
        TargetPort::PhonePair,       // = Stream-0/1
        TargetPort::OpticalAPair(0), // = Stream-14/15
        TargetPort::OpticalAPair(1), // = Stream-16/17
        TargetPort::OpticalAPair(2), // = Stream-18/19
        TargetPort::OpticalAPair(3), // = Stream-20/21
    ];

    const ASSIGN_PORT_VALS: &'static [u8] = &[
        0x00, // = Stream-2/3
        0x01, // = Stream-4/5
        0x02, // = Stream-0/1
        0x07, // = Stream-14/15
        0x08, // = Stream-16/17
        0x09, // = Stream-18/19
        0x0a, // = Stream-20/21
    ];
}

impl MotuVersion3ClockSpecification for Track16Protocol {
    const CLOCK_RATES: &'static [ClkRate] = &[
        ClkRate::R44100,
        ClkRate::R48000,
        ClkRate::R88200,
        ClkRate::R96000,
        ClkRate::R176400,
        ClkRate::R192000,
    ];
    const CLOCK_RATE_VALS: &'static [u8] = &[0x00, 0x01, 0x02, 0x03, 0x04, 0x05];

    const CLOCK_SRCS: &'static [V3ClkSrc] = &[V3ClkSrc::Internal, V3ClkSrc::SignalOptA];
    const CLOCK_SRC_VALS: &'static [u8] = &[0x00, 0x18];
}

impl MotuVersion3OpticalIfaceSpecification for Track16Protocol {
    const OPT_IFACE_COUNT: usize = 1;
}

impl MotuWordClockOutputSpecification for Track16Protocol {}

impl CommandDspOperation for Track16Protocol {}

impl MotuCommandDspReverbSpecification for Track16Protocol {}

impl MotuCommandDspMonitorSpecification for Track16Protocol {
    const RETURN_ASSIGN_TARGETS: &'static [TargetPort] = &[
        TargetPort::AnalogPair(0),
        TargetPort::AnalogPair(1),
        TargetPort::PhonePair,
        TargetPort::OpticalAPair(0),
        TargetPort::OpticalAPair(1),
        TargetPort::OpticalAPair(2),
        TargetPort::OpticalAPair(3),
    ];
}

impl MotuCommandDspMixerSpecification for Track16Protocol {
    const SOURCE_PORTS: &'static [TargetPort] = &[
        TargetPort::Analog(0), // Mic-0
        TargetPort::Analog(1), // Mic-1
        TargetPort::Analog(2),
        TargetPort::Analog(3),
        TargetPort::Analog(4),
        TargetPort::Analog(5),
        TargetPort::Analog(6),
        TargetPort::Analog(7),
        TargetPort::OpticalA(0),
        TargetPort::OpticalA(1),
        TargetPort::OpticalA(2),
        TargetPort::OpticalA(3),
        TargetPort::OpticalA(4),
        TargetPort::OpticalA(5),
        TargetPort::OpticalA(6),
        TargetPort::OpticalA(7),
    ];
    const OUTPUT_PORTS: &'static [TargetPort] = &[
        TargetPort::Disabled,
        TargetPort::AnalogPair(0),
        TargetPort::AnalogPair(1),
        TargetPort::AnalogPair(2),
        TargetPort::AnalogPair(3),
        TargetPort::PhonePair,
        TargetPort::OpticalAPair(0),
        TargetPort::OpticalAPair(1),
        TargetPort::OpticalAPair(2),
        TargetPort::OpticalAPair(3),
    ];
}

impl MotuCommandDspEqualizerSpecification for Track16Protocol {}

impl MotuCommandDspDynamicsSpecification for Track16Protocol {}

impl MotuCommandDspInputSpecification for Track16Protocol {
    const INPUT_PORTS: &'static [TargetPort] = &[
        TargetPort::Analog(0), // Mic-0
        TargetPort::Analog(1), // Mic-1
        TargetPort::Analog(2),
        TargetPort::Analog(3),
        TargetPort::Analog(4),
        TargetPort::Analog(5),
        TargetPort::Analog(6),
        TargetPort::Analog(7),
        TargetPort::OpticalA(0),
        TargetPort::OpticalA(1),
        TargetPort::OpticalA(2),
        TargetPort::OpticalA(3),
        TargetPort::OpticalA(4),
        TargetPort::OpticalA(5),
        TargetPort::OpticalA(6),
        TargetPort::OpticalA(7),
    ];
    const MIC_COUNT: usize = 2;
    const LINE_INPUT_COUNT: usize = 0;
}

impl MotuCommandDspOutputSpecification for Track16Protocol {
    const OUTPUT_PORTS: &'static [TargetPort] = &[
        TargetPort::AnalogPair(0),
        TargetPort::AnalogPair(1),
        TargetPort::PhonePair,
        TargetPort::OpticalAPair(0),
        TargetPort::OpticalAPair(1),
        TargetPort::OpticalAPair(2),
        TargetPort::OpticalAPair(3),
    ];
}

impl MotuCommandDspMeterSpecification for Track16Protocol {
    const INPUT_PORTS: &'static [(TargetPort, usize)] = &[
        (TargetPort::Analog(0), 2),
        (TargetPort::Analog(1), 3),
        (TargetPort::Analog(2), 4),
        (TargetPort::Analog(3), 5),
        (TargetPort::Analog(4), 6),
        (TargetPort::Analog(5), 7),
        (TargetPort::Adat(0), 10),
        (TargetPort::Adat(1), 11),
        (TargetPort::Adat(2), 12),
        (TargetPort::Adat(3), 13),
        (TargetPort::Adat(4), 14),
        (TargetPort::Adat(5), 15),
        (TargetPort::Adat(6), 16),
        (TargetPort::Adat(7), 17),
    ];
    const OUTPUT_PORTS: &'static [(TargetPort, usize)] = &[
        (TargetPort::Main(0), 50),
        (TargetPort::Main(1), 51),
        (TargetPort::Analog(0), 52),
        (TargetPort::Analog(1), 53),
        (TargetPort::Phone(0), 54),
        (TargetPort::Adat(0), 55),
        (TargetPort::Adat(1), 56),
        (TargetPort::Adat(2), 57),
        (TargetPort::Adat(3), 58),
        (TargetPort::Adat(4), 59),
        (TargetPort::Adat(5), 60),
        (TargetPort::Adat(6), 61),
        (TargetPort::Adat(7), 62),
    ];
}

impl Track16Protocol {
    /// Notification mask for main assignment, return assignment, and phone assignment. The change
    /// of phone assignment is also notified in command message.
    pub const NOTIFY_PORT_CHANGE: u32 = 0x40000000;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn opt_iface_mode_serdes() {
        [
            // For input A.
            (V3OptIfaceMode::Disabled, false, false, 0x00000000),
            (V3OptIfaceMode::Adat, false, false, 0x00000001),
            (V3OptIfaceMode::Spdif, false, false, 0x00010001),
            // For input B.
            (V3OptIfaceMode::Disabled, true, false, 0x00000000),
            (V3OptIfaceMode::Adat, true, false, 0x00000002),
            (V3OptIfaceMode::Spdif, true, false, 0x00100002),
            // For output A.
            (V3OptIfaceMode::Disabled, false, true, 0x00000000),
            (V3OptIfaceMode::Adat, false, true, 0x00000100),
            (V3OptIfaceMode::Spdif, false, true, 0x00040100),
            // For output B.
            (V3OptIfaceMode::Disabled, true, true, 0x00000000),
            (V3OptIfaceMode::Adat, true, true, 0x00000200),
            (V3OptIfaceMode::Spdif, true, true, 0x00400200),
        ]
        .iter()
        .for_each(|&(mode, is_b, is_out, val)| {
            let mut target = V3OptIfaceMode::default();
            deserialize_opt_iface_mode(&mut target, &val, is_b, is_out);
            assert_eq!(target, mode, "{:?},0x{:08x},{},{}", mode, val, is_b, is_out);

            let mut quad = 0;
            serialize_opt_iface_mode(&mode, &mut quad, is_b, is_out);
            assert_eq!(quad, val, "{:?},0x{:08x},{},{}", mode, val, is_b, is_out);
        });
    }

    #[test]
    fn f896mk3_aesebu_rate_convert_mode_serdes() {
        [
            F896mk3AesebuRateConvertMode::None,
            F896mk3AesebuRateConvertMode::InputToSystem,
            F896mk3AesebuRateConvertMode::OutputDependsInput,
            F896mk3AesebuRateConvertMode::Output441,
            F896mk3AesebuRateConvertMode::Output480,
            F896mk3AesebuRateConvertMode::Output882,
            F896mk3AesebuRateConvertMode::Output960,
        ]
        .iter()
        .for_each(|mode| {
            let mut quad = 0;
            serialize_f896mk3_aes_ebu_rate_converter_mode(mode, &mut quad);

            let mut target = F896mk3AesebuRateConvertMode::default();
            deserialize_f896mk3_aes_ebu_rate_converter_mode(&mut target, &quad);

            assert_eq!(&target, mode);
        });
    }

    #[test]
    fn common_assign_port_specification() {
        assert_eq!(
            F828mk3Protocol::ASSIGN_PORT_TARGETS.len(),
            F828mk3Protocol::ASSIGN_PORT_VALS.len()
        );

        assert_eq!(
            F828mk3HybridProtocol::ASSIGN_PORT_TARGETS.len(),
            F828mk3HybridProtocol::ASSIGN_PORT_VALS.len()
        );

        assert_eq!(
            F896mk3Protocol::ASSIGN_PORT_TARGETS.len(),
            F896mk3Protocol::ASSIGN_PORT_VALS.len()
        );

        assert_eq!(
            F896mk3HybridProtocol::ASSIGN_PORT_TARGETS.len(),
            F896mk3HybridProtocol::ASSIGN_PORT_VALS.len()
        );

        assert_eq!(
            H4preProtocol::ASSIGN_PORT_TARGETS.len(),
            H4preProtocol::ASSIGN_PORT_VALS.len()
        );

        assert_eq!(
            UltraliteMk3Protocol::ASSIGN_PORT_TARGETS.len(),
            UltraliteMk3Protocol::ASSIGN_PORT_VALS.len()
        );

        assert_eq!(
            UltraliteMk3HybridProtocol::ASSIGN_PORT_TARGETS.len(),
            UltraliteMk3HybridProtocol::ASSIGN_PORT_VALS.len()
        );

        assert_eq!(
            TravelerMk3Protocol::ASSIGN_PORT_TARGETS.len(),
            TravelerMk3Protocol::ASSIGN_PORT_VALS.len()
        );

        assert_eq!(
            Track16Protocol::ASSIGN_PORT_TARGETS.len(),
            Track16Protocol::ASSIGN_PORT_VALS.len()
        );
    }

    #[test]
    fn v3_clock_specification() {
        assert_eq!(
            AudioExpressProtocol::CLOCK_RATES.len(),
            AudioExpressProtocol::CLOCK_RATE_VALS.len(),
        );
        assert_eq!(
            AudioExpressProtocol::CLOCK_SRCS.len(),
            AudioExpressProtocol::CLOCK_SRC_VALS.len(),
        );

        assert_eq!(
            F828mk3Protocol::CLOCK_RATES.len(),
            F828mk3Protocol::CLOCK_RATE_VALS.len(),
        );
        assert_eq!(
            F828mk3Protocol::CLOCK_SRCS.len(),
            F828mk3Protocol::CLOCK_SRC_VALS.len(),
        );

        assert_eq!(
            F828mk3HybridProtocol::CLOCK_RATES.len(),
            F828mk3HybridProtocol::CLOCK_RATE_VALS.len(),
        );
        assert_eq!(
            F828mk3HybridProtocol::CLOCK_SRCS.len(),
            F828mk3HybridProtocol::CLOCK_SRC_VALS.len(),
        );

        assert_eq!(
            F896mk3Protocol::CLOCK_RATES.len(),
            F896mk3Protocol::CLOCK_RATE_VALS.len(),
        );
        assert_eq!(
            F828mk3Protocol::CLOCK_SRCS.len(),
            F828mk3Protocol::CLOCK_SRC_VALS.len(),
        );

        assert_eq!(
            F896mk3HybridProtocol::CLOCK_RATES.len(),
            F896mk3HybridProtocol::CLOCK_RATE_VALS.len(),
        );
        assert_eq!(
            F828mk3HybridProtocol::CLOCK_SRCS.len(),
            F828mk3HybridProtocol::CLOCK_SRC_VALS.len(),
        );

        assert_eq!(
            H4preProtocol::CLOCK_RATES.len(),
            H4preProtocol::CLOCK_RATE_VALS.len(),
        );
        assert_eq!(
            H4preProtocol::CLOCK_SRCS.len(),
            H4preProtocol::CLOCK_SRC_VALS.len(),
        );

        assert_eq!(
            UltraliteMk3Protocol::CLOCK_RATES.len(),
            UltraliteMk3Protocol::CLOCK_RATE_VALS.len(),
        );
        assert_eq!(
            UltraliteMk3Protocol::CLOCK_SRCS.len(),
            UltraliteMk3Protocol::CLOCK_SRC_VALS.len(),
        );

        assert_eq!(
            UltraliteMk3HybridProtocol::CLOCK_RATES.len(),
            UltraliteMk3HybridProtocol::CLOCK_RATE_VALS.len(),
        );
        assert_eq!(
            UltraliteMk3HybridProtocol::CLOCK_SRCS.len(),
            UltraliteMk3HybridProtocol::CLOCK_SRC_VALS.len(),
        );

        assert_eq!(
            TravelerMk3Protocol::CLOCK_RATES.len(),
            TravelerMk3Protocol::CLOCK_RATE_VALS.len(),
        );
        assert_eq!(
            TravelerMk3Protocol::CLOCK_SRCS.len(),
            TravelerMk3Protocol::CLOCK_SRC_VALS.len(),
        );

        assert_eq!(
            Track16Protocol::CLOCK_RATES.len(),
            Track16Protocol::CLOCK_RATE_VALS.len(),
        );
        assert_eq!(
            Track16Protocol::CLOCK_SRCS.len(),
            Track16Protocol::CLOCK_SRC_VALS.len(),
        );
    }

    #[test]
    fn register_dsp_meter_specification() {
        assert_eq!(
            AudioExpressProtocol::OUTPUT_PORT_PAIRS.len(),
            AudioExpressProtocol::OUTPUT_PORT_PAIR_POS.len()
        );

        assert_eq!(
            H4preProtocol::OUTPUT_PORT_PAIRS.len(),
            H4preProtocol::OUTPUT_PORT_PAIR_POS.len()
        );
    }
}
