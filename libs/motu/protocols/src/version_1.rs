// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol used in version 1 devices of MOTU FireWire series.
//!
//! The modules includes structure, enumeration, and trait and its implementation for protocol
//! used in version 1 devices of Mark of the Unicorn FireWire series.

use super::*;

/// Signal source of sampling clock.
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

/// Mode of optical interface.
pub enum V1OptIfaceMode {
    Adat,
    Spdif,
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

const CONF_BOOL_VALS: [u8; 2] = [0x00, 0x01];

const CONF_828_OPT_IN_IFACE_MASK: u32 = 0x00008000;
const CONF_828_OPT_IN_IFACE_SHIFT: usize = 15;

const CONF_828_OPT_OUT_IFACE_MASK: u32 = 0x00004000;
const CONF_828_OPT_OUT_IFACE_SHIFT: usize = 14;

const CONF_828_OPT_IFACE_VALS: [u8; 2] = [0x00, 0x01];

const CONF_828_MONITOR_INPUT_CH_MASK: u32 = 0x00003f00;
const CONF_828_MONITOR_INPUT_CH_SHIFT: usize = 8;
const CONF_828_MONITOR_INPUT_CH_VALS: [u8; 12] = [
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
const CONF_828_STREAM_INPUT_ENABLE_SHIFT: usize = 7;

const CONF_828_MONITOR_INPUT_DISABLE_MASK: u32 = 0x00000040;
const CONF_828_MONITOR_INPUT_DISABLE_SHIFT: usize = 6;

const CONF_828_OUTPUT_ENABLE_MASK: u32 = 0x00000008;
const CONF_828_OUTPUT_ENABLE_SHIFT: usize = 3;

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
const CONF_896_MONITOR_INPUT_CH_VALS: [u8; 13] = [
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
const CONF_896_MONITOR_INPUT_VALS: [(usize, usize); 15] = [
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

/// The trait for configuration of sampling clock in version 1 protocol.
pub trait V1ClkOperation {
    const CLK_OFFSET: u32;

    const CLK_RATE_MASK: u32;
    const CLK_RATE_SHIFT: usize;
    const CLK_RATE_VALS: &'static [u8];
    const CLK_RATE_LABELS: &'static [ClkRate];

    const CLK_SRC_MASK: u32;
    const CLK_SRC_SHIFT: usize;
    const CLK_SRC_VALS: &'static [u8];
    const CLK_SRC_LABELS: &'static [V1ClkSrc];

    fn get_clk_rate(req: &mut FwReq, node: &mut FwNode, timeout_ms: u32) -> Result<usize, Error> {
        get_idx_from_val(
            Self::CLK_OFFSET,
            Self::CLK_RATE_MASK,
            Self::CLK_RATE_SHIFT,
            CLK_RATE_LABEL,
            req,
            node,
            Self::CLK_RATE_VALS,
            timeout_ms,
        )
    }

    fn set_clk_rate(
        req: &mut FwReq,
        node: &mut FwNode,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        set_idx_to_val(
            Self::CLK_OFFSET,
            Self::CLK_RATE_MASK,
            Self::CLK_RATE_SHIFT,
            CLK_RATE_LABEL,
            req,
            node,
            Self::CLK_RATE_VALS,
            idx,
            timeout_ms,
        )
    }

    fn get_clk_src(req: &mut FwReq, node: &mut FwNode, timeout_ms: u32) -> Result<usize, Error> {
        get_idx_from_val(
            Self::CLK_OFFSET,
            Self::CLK_SRC_MASK,
            Self::CLK_SRC_SHIFT,
            CLK_SRC_LABEL,
            req,
            node,
            Self::CLK_SRC_VALS,
            timeout_ms,
        )
    }

    fn set_clk_src(
        req: &mut FwReq,
        node: &mut FwNode,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        set_idx_to_val(
            Self::CLK_OFFSET,
            Self::CLK_SRC_MASK,
            Self::CLK_SRC_SHIFT,
            CLK_SRC_LABEL,
            req,
            node,
            Self::CLK_SRC_VALS,
            idx,
            timeout_ms,
        )
    }
}

const MONITOR_INPUT_CH_LABEL: &str = "monitor-input-ch-v1";
const MONITOR_INPUT_DISABLE_LABEL: &str = "monitor-input-enable-v1";
const MONITOR_INPUT_AESEBU_LABEL: &str = "monitor-input-aesebu-v1";

/// The trait for configuration of input to monitor in version 1 protocol.
pub trait V1MonitorInputOperation {
    const MONITOR_INPUT_MODES: &'static [TargetPort];

    fn set_monitor_input(
        req: &mut FwReq,
        node: &mut FwNode,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error>;
    fn get_monitor_input(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<usize, Error>;
}

/// The protocol implementation for 828.
#[derive(Default)]
pub struct F828Protocol;

impl V1ClkOperation for F828Protocol {
    const CLK_OFFSET: u32 = CONF_828_OFFSET;

    const CLK_RATE_MASK: u32 = CONF_828_CLK_RATE_MASK;
    const CLK_RATE_SHIFT: usize = CONF_828_CLK_RATE_SHIFT;
    const CLK_RATE_VALS: &'static [u8] = &[0x00, 0x01];
    const CLK_RATE_LABELS: &'static [ClkRate] = &[ClkRate::R44100, ClkRate::R48000];

    const CLK_SRC_MASK: u32 = CONF_828_CLK_SRC_MASK;
    const CLK_SRC_SHIFT: usize = CONF_828_CLK_SRC_SHIFT;
    const CLK_SRC_VALS: &'static [u8] = &[0x00, 0x01, 0x02, 0x21];
    const CLK_SRC_LABELS: &'static [V1ClkSrc] = &[
        V1ClkSrc::Internal,
        V1ClkSrc::AdatDsub,
        V1ClkSrc::Spdif,
        V1ClkSrc::AdatOpt,
    ];
}

impl V1MonitorInputOperation for F828Protocol {
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

    fn set_monitor_input(
        req: &mut FwReq,
        node: &mut FwNode,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let (disable_idx, ch_idx) = if idx == 0 { (1, 0) } else { (0, idx - 1) };

        set_idx_to_val(
            CONF_828_OFFSET,
            CONF_828_MONITOR_INPUT_DISABLE_MASK,
            CONF_828_MONITOR_INPUT_DISABLE_SHIFT,
            MONITOR_INPUT_DISABLE_LABEL,
            req,
            node,
            &CONF_BOOL_VALS,
            disable_idx,
            timeout_ms,
        )?;

        set_idx_to_val(
            CONF_828_OFFSET,
            CONF_828_MONITOR_INPUT_CH_MASK,
            CONF_828_MONITOR_INPUT_CH_SHIFT,
            MONITOR_INPUT_CH_LABEL,
            req,
            node,
            &CONF_828_MONITOR_INPUT_CH_VALS,
            ch_idx,
            timeout_ms,
        )
    }

    fn get_monitor_input(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<usize, Error> {
        let mut idx = get_idx_from_val(
            CONF_828_OFFSET,
            CONF_828_MONITOR_INPUT_DISABLE_MASK,
            CONF_828_MONITOR_INPUT_DISABLE_SHIFT,
            MONITOR_INPUT_DISABLE_LABEL,
            req,
            node,
            &CONF_BOOL_VALS,
            timeout_ms,
        )
        .map(|idx| if idx > 0 { 0 } else { 1 })?;
        if idx > 0 {
            idx += get_idx_from_val(
                CONF_828_OFFSET,
                CONF_828_MONITOR_INPUT_CH_MASK,
                CONF_828_MONITOR_INPUT_CH_SHIFT,
                MONITOR_INPUT_CH_LABEL,
                req,
                node,
                &CONF_828_MONITOR_INPUT_CH_VALS,
                timeout_ms,
            )?;
        }
        Ok(idx)
    }
}

const CONF_828_OPT_OUT_IFACE_LABEL: &str = "opt-out-iface-v1";
const CONF_828_OPT_IN_IFACE_LABEL: &str = "opt-in-iface-v1";
const CONF_828_STREAM_INPUT_ENABLE_LABEL: &str = "stream-input-enable-v1";
const CONF_828_OUTPUT_ENABLE_LABEL: &str = "output-enable-v1";

impl F828Protocol {
    pub const OPT_IFACE_MODES: [V1OptIfaceMode; 2] = [V1OptIfaceMode::Adat, V1OptIfaceMode::Spdif];

    fn get_opt_iface_mode(
        mask: u32,
        shift: usize,
        label: &str,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<usize, Error> {
        get_idx_from_val(
            CONF_828_OFFSET,
            mask,
            shift,
            label,
            req,
            node,
            &CONF_828_OPT_IFACE_VALS,
            timeout_ms,
        )
    }

    fn set_opt_iface_mode(
        mask: u32,
        shift: usize,
        label: &str,
        req: &mut FwReq,
        node: &mut FwNode,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        set_idx_to_val(
            CONF_828_OFFSET,
            mask,
            shift,
            label,
            req,
            node,
            &CONF_828_OPT_IFACE_VALS,
            idx,
            timeout_ms,
        )
    }

    pub fn get_optical_output_iface_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<usize, Error> {
        Self::get_opt_iface_mode(
            CONF_828_OPT_OUT_IFACE_MASK,
            CONF_828_OPT_OUT_IFACE_SHIFT,
            CONF_828_OPT_OUT_IFACE_LABEL,
            req,
            node,
            timeout_ms,
        )
    }

    pub fn set_optical_output_iface_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::set_opt_iface_mode(
            CONF_828_OPT_OUT_IFACE_MASK,
            CONF_828_OPT_OUT_IFACE_SHIFT,
            CONF_828_OPT_OUT_IFACE_LABEL,
            req,
            node,
            idx,
            timeout_ms,
        )
    }

    pub fn get_optical_input_iface_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<usize, Error> {
        Self::get_opt_iface_mode(
            CONF_828_OPT_IN_IFACE_MASK,
            CONF_828_OPT_IN_IFACE_SHIFT,
            CONF_828_OPT_IN_IFACE_LABEL,
            req,
            node,
            timeout_ms,
        )
    }

    pub fn set_optical_input_iface_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::set_opt_iface_mode(
            CONF_828_OPT_IN_IFACE_MASK,
            CONF_828_OPT_IN_IFACE_SHIFT,
            CONF_828_OPT_IN_IFACE_LABEL,
            req,
            node,
            idx,
            timeout_ms,
        )
    }

    pub fn get_stream_input_enable(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        get_idx_from_val(
            CONF_828_OFFSET,
            CONF_828_STREAM_INPUT_ENABLE_MASK,
            CONF_828_STREAM_INPUT_ENABLE_SHIFT,
            CONF_828_STREAM_INPUT_ENABLE_LABEL,
            req,
            node,
            &CONF_BOOL_VALS,
            timeout_ms,
        )
        .map(|val| val > 0)
    }

    pub fn set_stream_input_enable(
        req: &mut FwReq,
        node: &mut FwNode,
        enable: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let idx = match enable {
            false => 0x00,
            true => 0x01,
        };
        set_idx_to_val(
            CONF_828_OFFSET,
            CONF_828_STREAM_INPUT_ENABLE_MASK,
            CONF_828_STREAM_INPUT_ENABLE_SHIFT,
            CONF_828_STREAM_INPUT_ENABLE_LABEL,
            req,
            node,
            &CONF_BOOL_VALS,
            idx,
            timeout_ms,
        )
    }

    pub fn get_output_enable(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        get_idx_from_val(
            CONF_828_OFFSET,
            CONF_828_OUTPUT_ENABLE_MASK,
            CONF_828_OUTPUT_ENABLE_SHIFT,
            CONF_828_OUTPUT_ENABLE_LABEL,
            req,
            node,
            &CONF_BOOL_VALS,
            timeout_ms,
        )
        .map(|val| val > 0)
    }

    pub fn set_output_enable(
        req: &mut FwReq,
        node: &mut FwNode,
        enable: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let idx = match enable {
            false => 0x00,
            true => 0x01,
        };
        set_idx_to_val(
            CONF_828_OFFSET,
            CONF_828_OUTPUT_ENABLE_MASK,
            CONF_828_OUTPUT_ENABLE_SHIFT,
            CONF_828_OUTPUT_ENABLE_LABEL,
            req,
            node,
            &CONF_BOOL_VALS,
            idx,
            timeout_ms,
        )
    }
}

/// The protocol implementation for 896.
#[derive(Default)]
pub struct F896Protocol;

impl F896Protocol {
    /// Notification mask for footswitch.
    pub const NOTIFY_FOOTSWITCH_MASK: u32 = 0x01000000;
}

impl WordClkOperation for F896Protocol {}

impl V1ClkOperation for F896Protocol {
    const CLK_OFFSET: u32 = OFFSET_CLK;

    const CLK_RATE_MASK: u32 = CONF_896_CLK_RATE_MASK;
    const CLK_RATE_SHIFT: usize = CONF_896_CLK_RATE_SHIFT;
    const CLK_RATE_VALS: &'static [u8] = &[0x00, 0x01, 0x02, 0x03];
    const CLK_RATE_LABELS: &'static [ClkRate] = &[
        ClkRate::R44100,
        ClkRate::R48000,
        ClkRate::R88200,
        ClkRate::R96000,
    ];

    const CLK_SRC_MASK: u32 = CONF_896_CLK_SRC_MASK;
    const CLK_SRC_SHIFT: usize = CONF_896_CLK_SRC_SHIFT;
    const CLK_SRC_VALS: &'static [u8] = &[0x00, 0x01, 0x02, 0x04, 0x05];
    const CLK_SRC_LABELS: &'static [V1ClkSrc] = &[
        V1ClkSrc::Internal,
        V1ClkSrc::AdatOpt,
        V1ClkSrc::AesebuXlr,
        V1ClkSrc::WordClk,
        V1ClkSrc::AdatDsub,
    ];
}

impl V1MonitorInputOperation for F896Protocol {
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

    fn set_monitor_input(
        req: &mut FwReq,
        node: &mut FwNode,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let &(ch_idx, aesebu_idx) =
            CONF_896_MONITOR_INPUT_VALS.iter().nth(idx).ok_or_else(|| {
                let label = "Invalid argument for index of monitor input}";
                Error::new(FileError::Inval, &label)
            })?;
        set_idx_to_val(
            OFFSET_CLK,
            CONF_896_MONITOR_INPUT_CH_MASK,
            CONF_896_MONITOR_INPUT_CH_SHIFT,
            MONITOR_INPUT_CH_LABEL,
            req,
            node,
            &CONF_896_MONITOR_INPUT_CH_VALS,
            ch_idx,
            timeout_ms,
        )?;
        set_idx_to_val(
            OFFSET_CLK,
            CONF_896_MONITOR_INPUT_AESEBU_MASK,
            CONF_896_MONITOR_INPUT_AESEBU_SHIFT,
            MONITOR_INPUT_AESEBU_LABEL,
            req,
            node,
            &CONF_BOOL_VALS,
            aesebu_idx,
            timeout_ms,
        )
    }

    fn get_monitor_input(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<usize, Error> {
        let ch_idx = get_idx_from_val(
            OFFSET_CLK,
            CONF_896_MONITOR_INPUT_CH_MASK,
            CONF_896_MONITOR_INPUT_CH_SHIFT,
            MONITOR_INPUT_CH_LABEL,
            req,
            node,
            &CONF_896_MONITOR_INPUT_CH_VALS,
            timeout_ms,
        )?;
        let aesebu_idx = get_idx_from_val(
            OFFSET_CLK,
            CONF_896_MONITOR_INPUT_AESEBU_MASK,
            CONF_896_MONITOR_INPUT_AESEBU_SHIFT,
            MONITOR_INPUT_AESEBU_LABEL,
            req,
            node,
            &CONF_BOOL_VALS,
            timeout_ms,
        )?;
        CONF_896_MONITOR_INPUT_VALS
            .iter()
            .position(|&e| e == (ch_idx, aesebu_idx))
            .ok_or_else(|| {
                let label = "Detect invalid value for monitor input";
                Error::new(FileError::Io, &label)
            })
    }
}

impl AesebuRateConvertOperation for F896Protocol {
    const AESEBU_RATE_CONVERT_MASK: u32 = 0x00000060;
    const AESEBU_RATE_CONVERT_SHIFT: usize = 5;
}

impl LevelMetersOperation for F896Protocol {}
