// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol used in version 1 devices of MOTU FireWire series.
//!
//! The modules includes structure, enumeration, and trait and its implementation for protocol
//! used in version 1 devices of Mark of the Unicorn FireWire series.

use super::*;

/// Signal source of sampling clock.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum V1ClkSrc {
    /// Internal.
    Internal,
    /// S/PDIF on coaxial or optical interface.
    Spdif,
    /// Word clock on BNC interface.
    WordClk,
    /// ADAT on optical interface.
    AdatOpt,
    /// ADAT on D-Sub interface.
    AdatDsub,
    /// AES/EBU on XLR interface.
    AesebuXlr,
}

impl Default for V1ClkSrc {
    fn default() -> Self {
        Self::Internal
    }
}

/// Mode of optical interface.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum V1OptIfaceMode {
    Adat,
    Spdif,
}

impl Default for V1OptIfaceMode {
    fn default() -> Self {
        Self::Adat
    }
}

// 828 registers:
//
// 0x'ffff'f000'0b00: configuration for sampling clock and digital interfaces.
//
//  0xffff0000: communication control. ALSA firewire-motu driver changes it.
//  0x00008000: mode of optical input interface.
//    0x00008000: for S/PDIF signal.
//    0x00000000: disabled or for ADAT signal.
//  0x00004000: mode of optical output interface.
//    0x00004000: for S/PDIF signal.
//    0x00000000: disabled or for ADAT signal.
//  0x00003f00: monitor input mode.
//    0x00000800: analog-1/2
//    0x00001a00: analog-3/4
//    0x00002c00: analog-5/6
//    0x00003e00: analog-7/8
//    0x00000000: analog-1
//    0x00000900: analog-2
//    0x00001200: analog-3
//    0x00001b00: analog-4
//    0x00002400: analog-5
//    0x00002d00: analog-6
//    0x00003600: analog-7
//    0x00003f00: analog-8
//  0x00000080: enable stream input.
//  0x00000040: disable monitor input.
//  0x00000008: enable main out.
//  0x00000004: rate of sampling clock.
//    0x00000004: 48.0 kHz
//    0x00000000: 44.1 kHz
//  0x00000023: source of sampling clock.
//    0x00000003: source packet header (SPH)
//    0x00000002: S/PDIF on optical/coaxial interface.
//    0x00000021: ADAT on optical interface
//    0x00000001: ADAT on Dsub 9pin
//    0x00000000: internal

const CONF_828_OFFSET: u32 = 0x00000b00;

const CONF_828_OPT_IN_IFACE_MASK: u32 = 0x00008000;
const CONF_828_OPT_IN_IFACE_SHIFT: usize = 15;

const CONF_828_OPT_OUT_IFACE_MASK: u32 = 0x00004000;
const CONF_828_OPT_OUT_IFACE_SHIFT: usize = 14;

const CONF_828_OPT_IFACE_VALS: [u8; 2] = [0x00, 0x01];

const CONF_828_MONITOR_INPUT_CH_MASK: u32 = 0x00003f00;
const CONF_828_MONITOR_INPUT_CH_SHIFT: usize = 8;
const CONF_828_MONITOR_INPUT_CH_VALS: &[u8] = &[
    0x08, // 0/1
    0x1a, // 2/3
    0x2c, // 4/5
    0x3e, // 6/7
    0x00, // 0
    0x09, // 1
    0x12, // 2
    0x1b, // 3
    0x24, // 4
    0x2d, // 5
    0x36, // 6
    0x3f, // 7
];

const CONF_828_STREAM_INPUT_ENABLE_MASK: u32 = 0x00000080;

const CONF_828_MONITOR_INPUT_DISABLE_MASK: u32 = 0x00000040;

const CONF_828_OUTPUT_ENABLE_MASK: u32 = 0x00000008;

const CONF_828_CLK_RATE_MASK: u32 = 0x00000004;
const CONF_828_CLK_RATE_SHIFT: usize = 2;

const CONF_828_CLK_SRC_MASK: u32 = 0x00000023;
const CONF_828_CLK_SRC_SHIFT: usize = 0;

//
// 896 registers:
//
// 0x'ffff'f000'0b14: configuration for sampling clock and input source for main output.
//  0xf0000000: enable physical and stream input to DAC.
//    0x80000000: disable
//    0x40000000: disable
//    0x20000000: enable (prior to the other bits)
//    0x10000000: disable
//    0x00000000: disable
//  0x08000000: speed of word clock signal output on BNC interface.
//    0x00000000: force to low rate (44.1/48.0 kHz).
//    0x08000000: follow to system clock.
//  0x04000000: something relevant to clock.
//  0x03000000: enable output.
//   0x02000000: enabled irreversibly once standing
//   0x01000000: enabled irreversibly once standing
//  0x00ffff00: input to monitor.
//    0x00000000: none
//    0x00004800: analog-1/2
//    0x00005a00: analog-3/4
//    0x00006c00: analog-5/6
//    0x00007e00: analog-7/8
//    0x00104800: AES/EBU-1/2
//    0x00004000: analog-1
//    0x00004900: analog-2
//    0x00005200: analog-3
//    0x00005b00: analog-4
//    0x00006400: analog-5
//    0x00006d00: analog-6
//    0x00007600: analog-7
//    0x00007f00: analog-8
//    0x00104000: AES/EBU-1
//    0x00104900: AES/EBU-2
//  0x00000060: sample rate conversion for AES/EBU input/output.
//    0x00000000: None
//    0x00000020: input signal is converted to system rate
//    0x00000040: output is slave to input, ignoring system rate
//    0x00000060: output is double rate than system rate
//  0x00000018: nominal rate of sampling clock.
//    0x00000000: 44.1 kHz
//    0x00000008: 48.0 kHz
//    0x00000010: 88.2 kHz
//    0x00000018: 96.0 kHz
//  0x00000007: source of sampling clock.
//    0x00000000: internal
//    0x00000001: ADAT on optical interface
//    0x00000002: AES/EBU on XLR
//    0x00000003: source packet header (SPH)
//    0x00000004: word clock on BNC
//    0x00000005: ADAT on Dsub 9pin
//
// 0x'ffff'f000'0b24: configuration for meter and stream source for main output.
//  0x00004000: LED carnival.
//  0x00003800: peak hold time.
//   0x00003800: infinite
//   0x00003000: 480 sec
//   0x00002800: 300 sec
//   0x00002000: 60 sec
//   0x00001800: 10 sec
//   0x00001000: 4 sec
//   0x00000800: 2 sec
//   0x00000000: disabled
//  0x00000700: clip hold time.
//   0x00000700: infinite
//   0x00000600: 480 sec
//   0x00000500: 300 sec
//   0x00000400: 60 sec
//   0x00000300: 10 sec
//   0x00000200: 4 sec
//   0x00000100: 2 sec
//   0x00000000: disabled
//  0x000000f0: stream source to main output.
//   0x00000080: Stream-16/17
//   0x00000080: Stream-16/17
//   0x00000070: (mute)
//   0x00000060: (mute)
//   0x00000050: (mute)
//   0x00000040: (mute)
//   0x00000030: Stream-6/7
//   0x00000020: Stream-4/5
//   0x00000010: Stream-2/3
//   0x00000000: Stream-0/1
//  0x00000004: The target of AES/EBU meter.
//   0x00000001: AES/EBU input
//   0x00000000: AES/EBU output.
//  0x00000003: The target of programmable meter.
//   0x00000002: ADAT output
//   0x00000001: ADAT input
//   0x00000000: Analog output

const CONF_896_MONITOR_INPUT_AESEBU_MASK: u32 = 0x00100000;
const CONF_896_MONITOR_INPUT_AESEBU_SHIFT: usize = 20;
const CONF_896_MONITOR_INPUT_CH_VALS: &[u8] = &[
    0x00, // disabled
    0x48, // 1/2
    0x5a, // 3/4
    0x6c, // 5/6
    0x7e, // 7/8
    0x40, // 1
    0x49, // 2
    0x52, // 3
    0x5b, // 4
    0x64, // 5
    0x6d, // 6
    0x76, // 7
    0x7f, // 8
];

const CONF_896_MONITOR_INPUT_CH_MASK: u32 = 0x0000ff00;
const CONF_896_MONITOR_INPUT_CH_SHIFT: usize = 8;
const CONF_896_MONITOR_INPUT_VALS: &[(usize, usize)] = &[
    (0, 0),
    (1, 0),
    (2, 0),
    (3, 0),
    (4, 0),
    (1, 1),
    (5, 0),
    (6, 0),
    (7, 0),
    (8, 0),
    (9, 0),
    (10, 0),
    (11, 0),
    (12, 0),
    (5, 1),
];

const CONF_896_CLK_RATE_MASK: u32 = 0x00000018;
const CONF_896_CLK_RATE_SHIFT: usize = 3;

const CONF_896_CLK_SRC_MASK: u32 = 0x00000007;
const CONF_896_CLK_SRC_SHIFT: usize = 0;

const CLK_RATE_LABEL: &str = "clock-rate-v1";
const CLK_SRC_LABEL: &str = "clock-source-v1";

/// The parameters of media and sampling clock.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Version1ClockParameters {
    /// The rate of media clock.
    pub rate: ClkRate,
    /// The source of sampling clock.
    pub source: V1ClkSrc,
}

/// The trait for specification of sampling and media clock in version 1 protocol.
pub trait MotuVersion1ClockSpecification {
    const CLK_OFFSET: u32;

    const CLK_RATE_MASK: u32;
    const CLK_RATE_SHIFT: usize;
    const CLK_RATE_VALS: &'static [u8];
    const CLK_RATES: &'static [ClkRate];

    const CLK_SRC_MASK: u32;
    const CLK_SRC_SHIFT: usize;
    const CLK_SRC_VALS: &'static [u8];
    const CLK_SRCS: &'static [V1ClkSrc];
}

impl<O> MotuWhollyCacheableParamsOperation<Version1ClockParameters> for O
where
    O: MotuVersion1ClockSpecification,
{
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut Version1ClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let quad = read_quad(req, node, Self::CLK_OFFSET, timeout_ms)?;

        deserialize_flag(
            &mut params.rate,
            &quad,
            Self::CLK_RATE_MASK,
            Self::CLK_RATE_SHIFT,
            Self::CLK_RATES,
            Self::CLK_RATE_VALS,
            CLK_RATE_LABEL,
        )?;

        deserialize_flag(
            &mut params.source,
            &quad,
            Self::CLK_SRC_MASK,
            Self::CLK_SRC_SHIFT,
            Self::CLK_SRCS,
            Self::CLK_SRC_VALS,
            CLK_SRC_LABEL,
        )
    }
}

impl<O> MotuWhollyUpdatableParamsOperation<Version1ClockParameters> for O
where
    O: MotuVersion1ClockSpecification,
{
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &Version1ClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quad = read_quad(req, node, Self::CLK_OFFSET, timeout_ms)?;

        serialize_flag(
            &params.rate,
            &mut quad,
            Self::CLK_RATE_MASK,
            Self::CLK_RATE_SHIFT,
            Self::CLK_RATES,
            Self::CLK_RATE_VALS,
            CLK_RATE_LABEL,
        )?;

        serialize_flag(
            &params.source,
            &mut quad,
            Self::CLK_SRC_MASK,
            Self::CLK_SRC_SHIFT,
            Self::CLK_SRCS,
            Self::CLK_SRC_VALS,
            CLK_SRC_LABEL,
        )?;

        write_quad(req, node, Self::CLK_OFFSET, quad, timeout_ms)
    }
}

/// The parameters of monitor inputs.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Version1MonitorInputParameters(pub TargetPort);

/// The trait for specification of monitor input.
pub trait MotuVersion1MonitorInputSpecification {
    const MONITOR_INPUT_MODES: &'static [TargetPort];
}

const MONITOR_INPUT_CH_LABEL: &str = "monitor-input-ch-v1";

/// The protocol implementation for 828.
#[derive(Default)]
pub struct F828Protocol;

impl MotuVersion1ClockSpecification for F828Protocol {
    const CLK_OFFSET: u32 = CONF_828_OFFSET;

    const CLK_RATE_MASK: u32 = CONF_828_CLK_RATE_MASK;
    const CLK_RATE_SHIFT: usize = CONF_828_CLK_RATE_SHIFT;
    const CLK_RATE_VALS: &'static [u8] = &[0x00, 0x01];
    const CLK_RATES: &'static [ClkRate] = &[ClkRate::R44100, ClkRate::R48000];

    const CLK_SRC_MASK: u32 = CONF_828_CLK_SRC_MASK;
    const CLK_SRC_SHIFT: usize = CONF_828_CLK_SRC_SHIFT;
    const CLK_SRC_VALS: &'static [u8] = &[0x00, 0x01, 0x02, 0x21];
    const CLK_SRCS: &'static [V1ClkSrc] = &[
        V1ClkSrc::Internal,
        V1ClkSrc::AdatDsub,
        V1ClkSrc::Spdif,
        V1ClkSrc::AdatOpt,
    ];
}

impl MotuVersion1MonitorInputSpecification for F828Protocol {
    const MONITOR_INPUT_MODES: &'static [TargetPort] = &[
        TargetPort::Disabled,
        TargetPort::AnalogPair(0),
        TargetPort::AnalogPair(1),
        TargetPort::AnalogPair(2),
        TargetPort::AnalogPair(3),
        TargetPort::Analog(0),
        TargetPort::Analog(1),
        TargetPort::Analog(2),
        TargetPort::Analog(3),
        TargetPort::Analog(4),
        TargetPort::Analog(5),
        TargetPort::Analog(6),
        TargetPort::Analog(7),
    ];
}

impl MotuWhollyCacheableParamsOperation<Version1MonitorInputParameters> for F828Protocol {
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut Version1MonitorInputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let quad = read_quad(req, node, CONF_828_OFFSET, timeout_ms)?;

        if quad & CONF_828_MONITOR_INPUT_DISABLE_MASK > 0 {
            params.0 = TargetPort::Disabled;
            Ok(())
        } else {
            deserialize_flag(
                &mut params.0,
                &quad,
                CONF_828_MONITOR_INPUT_CH_MASK,
                CONF_828_MONITOR_INPUT_CH_SHIFT,
                &Self::MONITOR_INPUT_MODES[1..],
                CONF_828_MONITOR_INPUT_CH_VALS,
                MONITOR_INPUT_CH_LABEL,
            )
        }
    }
}

impl MotuWhollyUpdatableParamsOperation<Version1MonitorInputParameters> for F828Protocol {
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &Version1MonitorInputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if Self::MONITOR_INPUT_MODES
            .iter()
            .find(|m| params.0.eq(m))
            .is_none()
        {
            let msg = format!("{:?} is not supported for monitor input", params.0);
            Err(Error::new(FileError::Inval, &msg))?;
        }

        let mut quad = read_quad(req, node, CONF_828_OFFSET, timeout_ms)?;

        if params.0 == TargetPort::Disabled {
            quad |= CONF_828_MONITOR_INPUT_DISABLE_MASK;
        } else {
            quad &= !CONF_828_MONITOR_INPUT_DISABLE_MASK;
            serialize_flag(
                &params.0,
                &mut quad,
                CONF_828_MONITOR_INPUT_CH_MASK,
                CONF_828_MONITOR_INPUT_CH_SHIFT,
                &<F828Protocol as MotuVersion1MonitorInputSpecification>::MONITOR_INPUT_MODES[1..],
                CONF_828_MONITOR_INPUT_CH_VALS,
                MONITOR_INPUT_CH_LABEL,
            )?;
        }

        write_quad(req, node, CONF_828_OFFSET, quad, timeout_ms)
    }
}

/// The parameter of optical interface for 828.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct F828OpticalIfaceParameters {
    /// The mode of signal in optical input interface.
    pub input_mode: V1OptIfaceMode,
    /// The mode of signal in optical output interface.
    pub output_mode: V1OptIfaceMode,
}

const CONF_828_OPT_OUT_IFACE_LABEL: &str = "opt-out-iface-v1";
const CONF_828_OPT_IN_IFACE_LABEL: &str = "opt-in-iface-v1";

impl F828Protocol {
    /// The available modes of optical interface.
    pub const OPT_IFACE_MODES: &[V1OptIfaceMode] = &[V1OptIfaceMode::Adat, V1OptIfaceMode::Spdif];
}

impl MotuWhollyCacheableParamsOperation<F828OpticalIfaceParameters> for F828Protocol {
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut F828OpticalIfaceParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let quad = read_quad(req, node, CONF_828_OFFSET, timeout_ms)?;

        deserialize_flag(
            &mut params.input_mode,
            &quad,
            CONF_828_OPT_IN_IFACE_MASK,
            CONF_828_OPT_IN_IFACE_SHIFT,
            Self::OPT_IFACE_MODES,
            &CONF_828_OPT_IFACE_VALS,
            CONF_828_OPT_IN_IFACE_LABEL,
        )?;

        deserialize_flag(
            &mut params.output_mode,
            &quad,
            CONF_828_OPT_OUT_IFACE_MASK,
            CONF_828_OPT_OUT_IFACE_SHIFT,
            Self::OPT_IFACE_MODES,
            &CONF_828_OPT_IFACE_VALS,
            CONF_828_OPT_OUT_IFACE_LABEL,
        )
    }
}

impl MotuWhollyUpdatableParamsOperation<F828OpticalIfaceParameters> for F828Protocol {
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &F828OpticalIfaceParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quad = read_quad(req, node, CONF_828_OFFSET, timeout_ms)?;

        serialize_flag(
            &params.input_mode,
            &mut quad,
            CONF_828_OPT_IN_IFACE_MASK,
            CONF_828_OPT_IN_IFACE_SHIFT,
            Self::OPT_IFACE_MODES,
            &CONF_828_OPT_IFACE_VALS,
            CONF_828_OPT_IN_IFACE_LABEL,
        )?;

        serialize_flag(
            &params.output_mode,
            &mut quad,
            CONF_828_OPT_OUT_IFACE_MASK,
            CONF_828_OPT_OUT_IFACE_SHIFT,
            Self::OPT_IFACE_MODES,
            &CONF_828_OPT_IFACE_VALS,
            CONF_828_OPT_OUT_IFACE_LABEL,
        )?;

        write_quad(req, node, CONF_828_OFFSET, quad, timeout_ms)
    }
}

/// The parameter of stream input for 828.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct F828StreamInputParameters(pub bool);

impl MotuWhollyCacheableParamsOperation<F828StreamInputParameters> for F828Protocol {
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut F828StreamInputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let quad = read_quad(req, node, CONF_828_OFFSET, timeout_ms)?;

        params.0 = quad & CONF_828_STREAM_INPUT_ENABLE_MASK > 0;

        Ok(())
    }
}

impl MotuWhollyUpdatableParamsOperation<F828StreamInputParameters> for F828Protocol {
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &F828StreamInputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quad = read_quad(req, node, CONF_828_OFFSET, timeout_ms)?;

        quad &= !CONF_828_STREAM_INPUT_ENABLE_MASK;
        if params.0 {
            quad |= CONF_828_STREAM_INPUT_ENABLE_MASK;
        }

        write_quad(req, node, CONF_828_OFFSET, quad, timeout_ms)
    }
}

/// The parameter of output for 828.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct F828OutputParameters(pub bool);

impl MotuWhollyCacheableParamsOperation<F828OutputParameters> for F828Protocol {
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut F828OutputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let quad = read_quad(req, node, CONF_828_OFFSET, timeout_ms)?;

        params.0 = quad & CONF_828_OUTPUT_ENABLE_MASK > 0;

        Ok(())
    }
}

impl MotuWhollyUpdatableParamsOperation<F828OutputParameters> for F828Protocol {
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &F828OutputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut quad = read_quad(req, node, CONF_828_OFFSET, timeout_ms)?;

        quad &= !CONF_828_OUTPUT_ENABLE_MASK;
        if params.0 {
            quad |= CONF_828_OUTPUT_ENABLE_MASK;
        }

        write_quad(req, node, CONF_828_OFFSET, quad, timeout_ms)
    }
}

/// The protocol implementation for 896.
#[derive(Default)]
pub struct F896Protocol;

impl F896Protocol {
    /// Notification mask for footswitch.
    pub const NOTIFY_FOOTSWITCH_MASK: u32 = 0x01000000;
}

impl MotuWordClockOutputSpecification for F896Protocol {}

impl MotuVersion1ClockSpecification for F896Protocol {
    const CLK_OFFSET: u32 = OFFSET_CLK;

    const CLK_RATE_MASK: u32 = CONF_896_CLK_RATE_MASK;
    const CLK_RATE_SHIFT: usize = CONF_896_CLK_RATE_SHIFT;
    const CLK_RATE_VALS: &'static [u8] = &[0x00, 0x01, 0x02, 0x03];
    const CLK_RATES: &'static [ClkRate] = &[
        ClkRate::R44100,
        ClkRate::R48000,
        ClkRate::R88200,
        ClkRate::R96000,
    ];

    const CLK_SRC_MASK: u32 = CONF_896_CLK_SRC_MASK;
    const CLK_SRC_SHIFT: usize = CONF_896_CLK_SRC_SHIFT;
    const CLK_SRC_VALS: &'static [u8] = &[0x00, 0x01, 0x02, 0x04, 0x05];
    const CLK_SRCS: &'static [V1ClkSrc] = &[
        V1ClkSrc::Internal,
        V1ClkSrc::AdatOpt,
        V1ClkSrc::AesebuXlr,
        V1ClkSrc::WordClk,
        V1ClkSrc::AdatDsub,
    ];
}

impl MotuVersion1MonitorInputSpecification for F896Protocol {
    const MONITOR_INPUT_MODES: &'static [TargetPort] = &[
        TargetPort::Disabled,
        TargetPort::AnalogPair(0),
        TargetPort::AnalogPair(1),
        TargetPort::AnalogPair(2),
        TargetPort::AnalogPair(3),
        TargetPort::AesEbuPair,
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
    ];
}

impl MotuWhollyCacheableParamsOperation<Version1MonitorInputParameters> for F896Protocol {
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut Version1MonitorInputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let quad = read_quad(req, node, OFFSET_CLK, timeout_ms)?;

        let aesebu_idx = ((quad & CONF_896_MONITOR_INPUT_AESEBU_MASK)
            >> CONF_896_MONITOR_INPUT_AESEBU_SHIFT) as usize;
        let ch_idx =
            ((quad & CONF_896_MONITOR_INPUT_CH_MASK) >> CONF_896_MONITOR_INPUT_CH_SHIFT) as usize;

        Self::MONITOR_INPUT_MODES
            .iter()
            .zip(CONF_896_MONITOR_INPUT_VALS)
            .find(|(_, entry)| (ch_idx, aesebu_idx).eq(entry))
            .ok_or_else(|| {
                let label = "Detect invalid value for monitor input";
                Error::new(FileError::Io, &label)
            })
            .map(|(&mode, _)| params.0 = mode)
    }
}

impl MotuWhollyUpdatableParamsOperation<Version1MonitorInputParameters> for F896Protocol {
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &Version1MonitorInputParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let (ch_idx, aesebu_idx) = Self::MONITOR_INPUT_MODES
            .iter()
            .zip(CONF_896_MONITOR_INPUT_VALS)
            .find(|(m, _)| params.0.eq(m))
            .ok_or_else(|| {
                let msg = format!("{:?} is not supported for monitor input", params.0);
                Error::new(FileError::Io, &msg)
            })
            .map(|(_, &entry)| entry)?;

        let mut quad = read_quad(req, node, OFFSET_CLK, timeout_ms)?;

        quad &= !CONF_896_MONITOR_INPUT_AESEBU_MASK;
        if aesebu_idx > 0 {
            quad |= (aesebu_idx as u32) << CONF_896_MONITOR_INPUT_AESEBU_SHIFT;
        }

        quad &= !CONF_896_MONITOR_INPUT_CH_MASK;
        quad |= (CONF_896_MONITOR_INPUT_CH_VALS[ch_idx] as u32) << CONF_896_MONITOR_INPUT_CH_SHIFT;

        write_quad(req, node, OFFSET_CLK, quad, timeout_ms)
    }
}

impl MotuAesebuRateConvertSpecification for F896Protocol {
    const AESEBU_RATE_CONVERT_MASK: u32 = 0x00000060;
    const AESEBU_RATE_CONVERT_SHIFT: usize = 5;
}

impl MotuLevelMetersSpecification for F896Protocol {
    const LEVEL_METERS_PROGRAMMABLE_MODES: &'static [LevelMetersProgrammableMode] = &[
        LevelMetersProgrammableMode::AnalogOutput,
        LevelMetersProgrammableMode::AdatAInput,
        LevelMetersProgrammableMode::AdatAOutput,
    ];
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn v1_clock_specification() {
        assert_eq!(
            F828Protocol::CLK_RATE_VALS.len(),
            F828Protocol::CLK_RATES.len()
        );
        assert_eq!(
            F828Protocol::CLK_SRC_VALS.len(),
            F828Protocol::CLK_SRCS.len()
        );

        assert_eq!(
            F896Protocol::CLK_RATE_VALS.len(),
            F896Protocol::CLK_RATES.len()
        );
        assert_eq!(
            F896Protocol::CLK_SRC_VALS.len(),
            F896Protocol::CLK_SRCS.len()
        );
    }
}
