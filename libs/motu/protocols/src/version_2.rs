// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol used in version 2 devices of MOTU FireWire series.
//!
//! The modules includes structure, enumeration, and trait and its implementation for protocol
//! used in version 2 devices of Mark of the Unicorn FireWire series.

use glib::Error;

use hinawa::FwReq;

use super::{register_dsp::*, *};

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
pub trait V2ClkOperation {
    const CLK_RATES: &'static [(ClkRate, u8)];
    const CLK_SRCS: &'static [(V2ClkSrc, u8)];

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

const MAIN_VOL_LABEL: &str = "main-vol-target-v2";
const MAIN_VOL_MASK: u32 = 0x000f0000;
const MAIN_VOL_SHIFT: usize = 16;

/// The trait for main volume knob assignment in version 2.
pub trait V2MainAssignOperation {
    const KNOB_TARGETS: &'static [(TargetPort, u8)];

    fn get_main_vol_assign(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32
    ) -> Result<usize, Error> {
        let vals: Vec<u8> = Self::KNOB_TARGETS.iter().map(|e| e.1).collect();
        get_idx_from_val(
            OFFSET_PORT,
            MAIN_VOL_MASK,
            MAIN_VOL_SHIFT,
            MAIN_VOL_LABEL,
            req,
            node,
            &vals,
            timeout_ms,
        )
    }

    fn set_main_vol_assign(
        req: &mut FwReq,
        node: &mut FwNode,
        idx: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let vals: Vec<u8> = Self::KNOB_TARGETS.iter().map(|e| e.1).collect();
        set_idx_to_val(
            OFFSET_PORT,
            MAIN_VOL_MASK,
            MAIN_VOL_SHIFT,
            MAIN_VOL_LABEL,
            req,
            node,
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
pub trait V2OptIfaceOperation {
    const OPT_IFACE_MODES: &'static [(V2OptIfaceMode, u8)];

    fn get_opt_in_iface_mode(
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32
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
        timeout_ms: u32
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
        (TargetPort::PhonePair0, 0x01), // = Stream-1/2
        (TargetPort::AnalogPair0, 0x02), // = Stream-3/4
        (TargetPort::AnalogPair1, 0x03), // = Stream-5/6
        (TargetPort::AnalogPair2, 0x04), // = Stream-7/8
        (TargetPort::AnalogPair3, 0x05), // = Stream-9/10
        (TargetPort::MainPair0, 0x06), // = Stream-11/12
        (TargetPort::SpdifPair0, 0x07), // = Stream-13/14
        (TargetPort::AdatPair0, 0x08), // = Stream-15/16
        (TargetPort::AdatPair1, 0x09), // = Stream-17/18
        (TargetPort::AdatPair2, 0x0a), // = Stream-19/20
        (TargetPort::AdatPair3, 0x0b), // = Stream-21/22
    ];
}

impl WordClkOperation for F828mk2Protocol {}

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
        TargetPort::PhonePair0,
        TargetPort::AnalogPair0,
        TargetPort::AnalogPair1,
        TargetPort::AnalogPair2,
        TargetPort::AnalogPair3,
        TargetPort::MainPair0,
        TargetPort::SpdifPair0,
        TargetPort::AdatPair0,
        TargetPort::AdatPair1,
        TargetPort::AdatPair2,
        TargetPort::AdatPair3,
    ];
}

impl RegisterDspMixerReturnOperation for F828mk2Protocol {
    const RETURN_SOURCES: &'static [TargetPort] = &[
        TargetPort::Disabled,
        TargetPort::PhonePair0,
        TargetPort::AnalogPair0,
        TargetPort::AnalogPair1,
        TargetPort::AnalogPair2,
        TargetPort::AnalogPair3,
        TargetPort::MainPair0,
        TargetPort::SpdifPair0,
        TargetPort::AdatPair0,
        TargetPort::AdatPair1,
        TargetPort::AdatPair2,
        TargetPort::AdatPair3,
    ];
}

impl RegisterDspMixerMonauralSourceOperation for F828mk2Protocol {
    const MIXER_SOURCES: &'static [TargetPort] = &[
        TargetPort::Analog0,
        TargetPort::Analog1,
        TargetPort::Analog2,
        TargetPort::Analog3,
        TargetPort::Analog4,
        TargetPort::Analog5,
        TargetPort::Analog6,
        TargetPort::Analog7,
        TargetPort::Mic0,
        TargetPort::Mic1,
        TargetPort::Spdif0,
        TargetPort::Spdif1,
        TargetPort::Adat0,
        TargetPort::Adat1,
        TargetPort::Adat2,
        TargetPort::Adat3,
        TargetPort::Adat4,
        TargetPort::Adat5,
        TargetPort::Adat6,
        TargetPort::Adat7,
    ];
}

impl RegisterDspOutputOperation for F828mk2Protocol {}

impl Traveler828mk2LineInputOperation for F828mk2Protocol {
    const LINE_INPUT_COUNT: usize = 8;
    const CH_OFFSET: usize = 0;
}

/// The protocol implementation for 8pre.
#[derive(Default)]
pub struct F8preProtocol;

impl AssignOperation for F8preProtocol {
    const ASSIGN_PORTS: &'static [(TargetPort, u8)] = &[
        (TargetPort::PhonePair0, 0x01),
        (TargetPort::MainPair0, 0x02),
    ];
}

impl V2ClkOperation for F8preProtocol {
    const CLK_RATES: &'static [(ClkRate, u8)] = &[
        (ClkRate::R44100, 0x00),
        (ClkRate::R48000, 0x01),
        (ClkRate::R88200, 0x02),
        (ClkRate::R96000, 0x03),
    ];

    const CLK_SRCS: &'static [(V2ClkSrc, u8)] = &[(V2ClkSrc::Internal, 0x00), (V2ClkSrc::AdatOpt, 0x01)];

    const HAS_LCD: bool = false;
}

impl V2OptIfaceOperation for F8preProtocol {
    const OPT_IFACE_MODES: &'static [(V2OptIfaceMode, u8)] =
        &[(V2OptIfaceMode::None, 0x00), (V2OptIfaceMode::Adat, 0x01)];
}

impl RegisterDspMixerOutputOperation for F8preProtocol {
    const OUTPUT_DESTINATIONS: &'static [TargetPort] = &[
        TargetPort::Disabled,
        TargetPort::PhonePair0,
        TargetPort::MainPair0,
        TargetPort::AdatPair0,
        TargetPort::AdatPair1,
        TargetPort::AdatPair2,
        TargetPort::AdatPair3,
    ];
}

impl RegisterDspMixerReturnOperation for F8preProtocol {
    const RETURN_SOURCES: &'static [TargetPort] = &[
        TargetPort::Disabled,
        TargetPort::PhonePair0,
        TargetPort::MainPair0,
        TargetPort::AdatPair0,
        TargetPort::AdatPair1,
        TargetPort::AdatPair2,
        TargetPort::AdatPair3,
    ];
}

impl RegisterDspMixerMonauralSourceOperation for F8preProtocol {
    const MIXER_SOURCES: &'static [TargetPort] = &[
        TargetPort::Analog0,
        TargetPort::Analog1,
        TargetPort::Analog2,
        TargetPort::Analog3,
        TargetPort::Analog4,
        TargetPort::Analog5,
        TargetPort::Analog6,
        TargetPort::Analog7,
        TargetPort::Adat0,
        TargetPort::Adat1,
        TargetPort::Adat2,
        TargetPort::Adat3,
        TargetPort::Adat4,
        TargetPort::Adat5,
        TargetPort::Adat6,
        TargetPort::Adat7,
    ];
}

impl RegisterDspOutputOperation for F8preProtocol {}

/// The protocol implementation for Traveler.
#[derive(Default)]
pub struct TravelerProtocol;

impl AssignOperation for TravelerProtocol {
    const ASSIGN_PORTS: &'static [(TargetPort, u8)] = &[
        (TargetPort::PhonePair0, 0x01), // = Stream-1/2
        (TargetPort::AnalogPair0, 0x02), // = Stream-3/4
        (TargetPort::AnalogPair1, 0x03), // = Stream-5/6
        (TargetPort::AnalogPair2, 0x04), // = Stream-7/8
        (TargetPort::AnalogPair3, 0x05), // = Stream-9/10
        (TargetPort::AesEbuPair0, 0x06), // = Stream-11/12
        (TargetPort::SpdifPair0, 0x07), // = Stream-13/14
        (TargetPort::AdatPair0, 0x08), // = Stream-15/16
        (TargetPort::AdatPair1, 0x09), // = Stream-17/18
        (TargetPort::AdatPair2, 0x0a), // = Stream-19/20
        (TargetPort::AdatPair3, 0x0b), // = Stream-21/22
    ];
}

impl WordClkOperation for TravelerProtocol {}

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
        TargetPort::PhonePair0,
        TargetPort::AnalogPair0,
        TargetPort::AnalogPair1,
        TargetPort::AnalogPair2,
        TargetPort::AnalogPair3,
        TargetPort::AesEbuPair0,
        TargetPort::SpdifPair0,
        TargetPort::AdatPair0,
        TargetPort::AdatPair1,
        TargetPort::AdatPair2,
        TargetPort::AdatPair3,
    ];
}

impl RegisterDspMixerReturnOperation for TravelerProtocol {
    const RETURN_SOURCES: &'static [TargetPort] = &[
        TargetPort::Disabled,
        TargetPort::PhonePair0,
        TargetPort::AnalogPair0,
        TargetPort::AnalogPair1,
        TargetPort::AnalogPair2,
        TargetPort::AnalogPair3,
        TargetPort::AesEbuPair0,
        TargetPort::SpdifPair0,
        TargetPort::AdatPair0,
        TargetPort::AdatPair1,
        TargetPort::AdatPair2,
        TargetPort::AdatPair3,
    ];
}

impl RegisterDspMixerMonauralSourceOperation for TravelerProtocol {
    const MIXER_SOURCES: &'static [TargetPort] = &[
        TargetPort::Analog0,
        TargetPort::Analog1,
        TargetPort::Analog2,
        TargetPort::Analog3,
        TargetPort::Analog4,
        TargetPort::Analog5,
        TargetPort::Analog6,
        TargetPort::Analog7,
        TargetPort::AesEbu0,
        TargetPort::AesEbu1,
        TargetPort::Spdif0,
        TargetPort::Spdif1,
        TargetPort::Adat0,
        TargetPort::Adat1,
        TargetPort::Adat2,
        TargetPort::Adat3,
        TargetPort::Adat4,
        TargetPort::Adat5,
        TargetPort::Adat6,
        TargetPort::Adat7,
    ];
}

impl RegisterDspOutputOperation for TravelerProtocol {}

impl Traveler828mk2LineInputOperation for TravelerProtocol {
    const LINE_INPUT_COUNT: usize = 4;
    const CH_OFFSET: usize = 4;
}

/// The structure for state of inputs in Traveler.
#[derive(Default)]
pub struct TravelerMicInputState {
    pub gain: [u8; TravelerProtocol::MIC_INPUT_COUNT],
    pub pad: [bool; TravelerProtocol::MIC_INPUT_COUNT],
}

const TRAVELER_MIC_PARAM_OFFSET: usize = 0x0c1c;
const   TRAVELER_MIC_GAIN_MASK: u8 = 0x3f;
const   TRAVELER_MIC_PAD_FLAG: u8 = 0x40;
const   TRAVELER_MIC_CHANGE_FLAG: u8 = 0x80;

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
        timeout_ms: u32
    ) -> Result<(), Error> {
        read_quad(req, node, TRAVELER_MIC_PARAM_OFFSET as u32, timeout_ms).map(|val| {
            (0..Self::MIC_INPUT_COUNT)
                .for_each(|i| {
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
        timeout_ms: u32
    ) -> Result<(), Error> {
        assert_eq!(gain.len(), Self::MIC_INPUT_COUNT);

        let val = gain.iter()
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
        write_quad(req, node, TRAVELER_MIC_PARAM_OFFSET as u32, val, timeout_ms).map(|_| {
            state.gain.copy_from_slice(gain)
        })
    }

    pub fn write_mic_pad(
        req: &mut FwReq,
        node: &mut FwNode,
        pad: &[bool],
        state: &mut TravelerMicInputState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        assert_eq!(pad.len(), Self::MIC_INPUT_COUNT);

        let val = pad.iter()
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
        write_quad(req, node, TRAVELER_MIC_PARAM_OFFSET as u32, val, timeout_ms).map(|_| {
            state.pad.copy_from_slice(pad)
        })
    }
}

/// The protocol implementation for Ultralite.
#[derive(Default)]
pub struct UltraliteProtocol;

impl AssignOperation for UltraliteProtocol {
    const ASSIGN_PORTS: &'static [(TargetPort, u8)] = &[
        (TargetPort::PhonePair0, 0x01), // Stream-1/2
        (TargetPort::AnalogPair0, 0x02), // Stream-3/4
        (TargetPort::AnalogPair1, 0x03), // Stream-5/6
        (TargetPort::AnalogPair2, 0x04), // Stream-7/8
        (TargetPort::AnalogPair3, 0x05), // Stream-9/10
        (TargetPort::MainPair0, 0x06), // Stream-11/12
        (TargetPort::SpdifPair0, 0x07), // Stream-13/14
    ];
}

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

impl V2MainAssignOperation for UltraliteProtocol {
    const KNOB_TARGETS: &'static [(TargetPort, u8)] = &[
        (TargetPort::MainPair0, 0x00),
        (TargetPort::Analog6Pairs, 0x01),
        (TargetPort::Analog8Pairs, 0x02),
        (TargetPort::SpdifPair0, 0x03),
    ];
}

impl RegisterDspMixerOutputOperation for UltraliteProtocol {
    const OUTPUT_DESTINATIONS: &'static [TargetPort] = &[
        TargetPort::Disabled,
        TargetPort::PhonePair0,
        TargetPort::AnalogPair0,
        TargetPort::AnalogPair1,
        TargetPort::AnalogPair2,
        TargetPort::AnalogPair3,
        TargetPort::MainPair0,
        TargetPort::SpdifPair0,
    ];
}

impl RegisterDspMixerReturnOperation for UltraliteProtocol {
    const RETURN_SOURCES: &'static [TargetPort] = &[
        TargetPort::Disabled,
        TargetPort::PhonePair0,
        TargetPort::AnalogPair0,
        TargetPort::AnalogPair1,
        TargetPort::AnalogPair2,
        TargetPort::AnalogPair3,
        TargetPort::MainPair0,
        TargetPort::SpdifPair0,
    ];
}

impl RegisterDspMixerMonauralSourceOperation for UltraliteProtocol {
    const MIXER_SOURCES: &'static [TargetPort] = &[
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
}

impl RegisterDspOutputOperation for UltraliteProtocol {}

const ULTRALITE_INPUT_OFFSETS: [usize; 3] = [0x0c70, 0x0c74, 0x0c78];
const   ULTRALITE_INPUT_GAIN_MASK: u8 = 0x18;
const   ULTRALITE_INPUT_INVERT_FLAG: u8 = 0x20;
const   ULTRALITE_INPUT_CHANGE_FLAG: u8 = 0x80;

/// The structure for state of input in Ultralite.
#[derive(Default)]
pub struct UltraliteInputState {
    pub gain: [u8; UltraliteProtocol::INPUT_COUNT],
    pub invert: [bool; UltraliteProtocol::INPUT_COUNT],
}

impl UltraliteProtocol {
    pub const INPUT_COUNT: usize = 10;

    pub const INPUT_GAIN_MIN: u8 = 0x00;
    pub const INPUT_GAIN_MAX: u8 = 0x18;
    pub const INPUT_GAIN_STEP: u8 = 0x01;

    const CH_TABLE: [(usize, usize); 3] = [(0, 4), (4, 8), (8, 10)];

    pub fn read_input_state(
        req: &mut FwReq,
        node: &mut FwNode,
        state: &mut UltraliteInputState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        ULTRALITE_INPUT_OFFSETS
            .iter()
            .zip(Self::CH_TABLE.iter())
            .try_for_each(|(&offset, &(begin, end))| {
                read_quad(req, node, offset as u32, timeout_ms).map(|val| {
                    (begin..end)
                        .for_each(|i| {
                            let pos = i % 4;
                            let v = ((val >> (pos * 8)) & 0xff) as u8;
                            state.gain[i] = v & ULTRALITE_INPUT_GAIN_MASK;
                            state.invert[i] = v & ULTRALITE_INPUT_INVERT_FLAG > 0;
                        });
                })
            })
    }

    pub fn write_input_gain(
        req: &mut FwReq,
        node: &mut FwNode,
        gain: &[u8],
        state: &mut UltraliteInputState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        assert_eq!(gain.len(), Self::INPUT_COUNT);

        let mut vals = [0u32; 3];
        gain
            .iter()
            .enumerate()
            .filter(|&(i, g)| !state.gain[i].eq(g))
            .for_each(|(i, &g)| {
                let mut v = ULTRALITE_INPUT_CHANGE_FLAG;
                if state.invert[i] {
                    v |= ULTRALITE_INPUT_INVERT_FLAG;
                }
                v |= g;
                let pos = i % 4;
                vals[i / 4] |= (v as u32) << (pos * 8);
            });

        ULTRALITE_INPUT_OFFSETS
            .iter()
            .zip(vals.iter())
            .filter(|(_, &val)| val > 0)
            .try_for_each(|(&offset, &val)| {
                write_quad(req, node, offset as u32, val, timeout_ms)
            })
            .map(|_| state.gain.copy_from_slice(gain))
    }

    pub fn write_input_invert(
        req: &mut FwReq,
        node: &mut FwNode,
        invert: &[bool],
        state: &mut UltraliteInputState,
        timeout_ms: u32
    ) -> Result<(), Error> {
        assert_eq!(invert.len(), Self::INPUT_COUNT);

        let mut vals = [0u32; 3];
        invert
            .iter()
            .enumerate()
            .filter(|&(i, v)| !state.invert[i].eq(v))
            .for_each(|(i, &inv)| {
                let mut v = ULTRALITE_INPUT_CHANGE_FLAG;
                if inv {
                    v |= ULTRALITE_INPUT_INVERT_FLAG;
                }
                v |= state.gain[i];
                let pos = i % 4;
                vals[i / 4] |= (v as u32) << (pos * 8);
            });

        ULTRALITE_INPUT_OFFSETS
            .iter()
            .zip(vals.iter())
            .filter(|(_, &val)| val > 0)
            .try_for_each(|(&offset, &val)| {
                write_quad(req, node, offset as u32, val, timeout_ms)
            })
            .map(|_| state.invert.copy_from_slice(invert))
    }
}

/// The protocol implementation for 896HD.
#[derive(Default)]
pub struct F896hdProtocol;

impl AssignOperation for F896hdProtocol {
    const ASSIGN_PORTS: &'static [(TargetPort, u8)] = &[
        (TargetPort::PhonePair0, 0x01),
        (TargetPort::AnalogPair0, 0x02), // Stream-1/2
        (TargetPort::AnalogPair1, 0x03), // Stream-3/4
        (TargetPort::AnalogPair2, 0x04), // Stream-5/6
        (TargetPort::AnalogPair3, 0x05), // Stream-7/8
        (TargetPort::MainPair0, 0x06), // Stream-9/10
        (TargetPort::AesEbuPair0, 0x07), // Stream-11/12
        (TargetPort::AdatPair0, 0x08), // Stream-13/14
        (TargetPort::AdatPair1, 0x09), // Stream-15/16
        (TargetPort::AdatPair2, 0x0a), // Stream-17/18
        (TargetPort::AdatPair3, 0x0b), // Stream-19/20
    ];
}

impl WordClkOperation for F896hdProtocol {}

impl AesebuRateConvertOperation for F896hdProtocol {
    const AESEBU_RATE_CONVERT_MASK: u32 = 0x00000300;
    const AESEBU_RATE_CONVERT_SHIFT: usize = 8;
}

impl LevelMetersOperation for F896hdProtocol {}

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
        TargetPort::PhonePair0,
        TargetPort::AnalogPair0,
        TargetPort::AnalogPair1,
        TargetPort::AnalogPair2,
        TargetPort::AnalogPair3,
        TargetPort::MainPair0,
        TargetPort::AesEbuPair0,
        TargetPort::AdatPair0,
        TargetPort::AdatPair1,
        TargetPort::AdatPair2,
        TargetPort::AdatPair3,
    ];
}

impl RegisterDspMixerReturnOperation for F896hdProtocol {
    const RETURN_SOURCES: &'static [TargetPort] = &[
        TargetPort::Disabled,
        TargetPort::PhonePair0,
        TargetPort::AnalogPair0,
        TargetPort::AnalogPair1,
        TargetPort::AnalogPair2,
        TargetPort::AnalogPair3,
        TargetPort::MainPair0,
        TargetPort::AesEbuPair0,
        TargetPort::AdatPair0,
        TargetPort::AdatPair1,
        TargetPort::AdatPair2,
        TargetPort::AdatPair3,
    ];
}

impl RegisterDspMixerMonauralSourceOperation for F896hdProtocol {
    const MIXER_SOURCES: &'static [TargetPort] = &[
        TargetPort::Analog0,
        TargetPort::Analog1,
        TargetPort::Analog2,
        TargetPort::Analog3,
        TargetPort::Analog4,
        TargetPort::Analog5,
        TargetPort::Analog6,
        TargetPort::Analog7,
        TargetPort::AesEbu0,
        TargetPort::AesEbu1,
        TargetPort::Adat0,
        TargetPort::Adat1,
        TargetPort::Adat2,
        TargetPort::Adat3,
        TargetPort::Adat4,
        TargetPort::Adat5,
        TargetPort::Adat6,
        TargetPort::Adat7,
    ];
}

impl RegisterDspOutputOperation for F896hdProtocol {}
