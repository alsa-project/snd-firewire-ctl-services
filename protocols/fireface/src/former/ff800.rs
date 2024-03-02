// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for Fireface 800.

use super::*;

/// Unique protocol for Fireface 800.
#[derive(Default, Debug)]
pub struct Ff800Protocol;

const MIXER_OFFSET: u64 = 0x000080080000;
const OUTPUT_OFFSET: u64 = 0x000080081f80;
const METER_OFFSET: u64 = 0x000080100000;
const STATUS_OFFSET: u64 = 0x0000801c0000;
const CFG_OFFSET: u64 = 0x0000fc88f014;

// TODO: 4 quadlets are read at once.
#[allow(dead_code)]
const TCO_STATUS_OFFSET: usize = 0x0000801f0000;

// TODO; 4 quadlets are written at once.
#[allow(dead_code)]
const TCO_CFG_OFFSET: usize = 0x0000810f0020;

impl RmeFfFormerSpecification for Ff800Protocol {
    const ANALOG_INPUT_COUNT: usize = 10;
    const SPDIF_INPUT_COUNT: usize = 2;
    const ADAT_INPUT_COUNT: usize = 16;
    const STREAM_INPUT_COUNT: usize = 28;

    const ANALOG_OUTPUT_COUNT: usize = 10;
    const SPDIF_OUTPUT_COUNT: usize = 2;
    const ADAT_OUTPUT_COUNT: usize = 16;
}

impl RmeFfFormerMeterSpecification for Ff800Protocol {
    const METER_OFFSET: u64 = METER_OFFSET;
}

impl RmeFfWhollyUpdatableParamsOperation<FormerOutputVolumeState> for Ff800Protocol {
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &FormerOutputVolumeState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = Self::serialize_offsets(params);
        req.transaction(
            node,
            FwTcode::WriteBlockRequest,
            OUTPUT_OFFSET,
            raw.len(),
            &mut raw,
            timeout_ms,
        )
    }
}

impl RmeFfPartiallyUpdatableParamsOperation<FormerOutputVolumeState> for Ff800Protocol {
    fn update_partially(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut FormerOutputVolumeState,
        update: FormerOutputVolumeState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let old = Self::serialize_offsets(params);
        let mut new = Self::serialize_offsets(&update);

        (0..(new.len() / 4))
            .try_for_each(|i| {
                let pos = i * 4;
                if new[pos..(pos + 4)] != old[pos..(pos + 4)] {
                    req.transaction(
                        node,
                        FwTcode::WriteBlockRequest,
                        OUTPUT_OFFSET + pos as u64,
                        4,
                        &mut new[pos..(pos + 4)],
                        timeout_ms,
                    )
                } else {
                    Ok(())
                }
            })
            .map(|_| *params = update)
    }
}

impl RmeFormerMixerSpecification for Ff800Protocol {
    const MIXER_OFFSET: u64 = MIXER_OFFSET;
    const AVAIL_COUNT: usize = 32;
}

/// Signal source of sampling clock.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Ff800ClkSrc {
    Internal,
    WordClock,
    AdatA,
    AdatB,
    Spdif,
    Tco,
}

impl Default for Ff800ClkSrc {
    fn default() -> Self {
        Self::AdatA
    }
}

// NOTE: for first quadlet of status quadlets.
const Q0_SYNC_WORD_CLOCK_MASK: u32 = 0x40000000;
const Q0_LOCK_WORD_CLOCK_MASK: u32 = 0x20000000;
const Q0_EXT_CLK_RATE_MASK: u32 = 0x1e000000;
const Q0_EXT_CLK_RATE_192000_FLAGS: u32 = 0x12000000;
const Q0_EXT_CLK_RATE_176400_FLAGS: u32 = 0x10000000;
const Q0_EXT_CLK_RATE_128000_FLAGS: u32 = 0x0c000000;
const Q0_EXT_CLK_RATE_96000_FLAGS: u32 = 0x0e000000;
const Q0_EXT_CLK_RATE_88200_FLAGS: u32 = 0x0a000000;
const Q0_EXT_CLK_RATE_64000_FLAGS: u32 = 0x08000000;
const Q0_EXT_CLK_RATE_48000_FLAGS: u32 = 0x06000000;
const Q0_EXT_CLK_RATE_44100_FLAGS: u32 = 0x04000000;
const Q0_EXT_CLK_RATE_32000_FLAGS: u32 = 0x02000000;
const Q0_ACTIVE_CLK_SRC_MASK: u32 = 0x01c00000;
const Q0_ACTIVE_CLK_SRC_INTERNAL_FLAGS: u32 = 0x01c00000;
const Q0_ACTIVE_CLK_SRC_TCO_FLAGS: u32 = 0x01800000;
const Q0_ACTIVE_CLK_SRC_WORD_CLK_FLAGS: u32 = 0x01000000;
const Q0_ACTIVE_CLK_SRC_SPDIF_FLAGS: u32 = 0x00c00000;
const Q0_ACTIVE_CLK_SRC_ADAT_B_FLAGS: u32 = 0x00400000;
const Q0_ACTIVE_CLK_SRC_ADAT_A_FLAGS: u32 = 0x00000000;
const Q0_SYNC_SPDIF_MASK: u32 = 0x00100000;
const Q0_LOCK_SPDIF_MASK: u32 = 0x00040000;
const Q0_SPDIF_RATE_MASK: u32 = 0x0003c000;
const Q0_SPDIF_RATE_192000_FLAGS: u32 = 0x00024000;
const Q0_SPDIF_RATE_176400_FLAGS: u32 = 0x00020000;
const Q0_SPDIF_RATE_128000_FLAGS: u32 = 0x0001c000;
const Q0_SPDIF_RATE_96000_FLAGS: u32 = 0x00018000;
const Q0_SPDIF_RATE_88200_FLAGS: u32 = 0x00014000;
const Q0_SPDIF_RATE_64000_FLAGS: u32 = 0x00010000;
const Q0_SPDIF_RATE_48000_FLAGS: u32 = 0x0000c000;
const Q0_SPDIF_RATE_44100_FLAGS: u32 = 0x00008000;
const Q0_SPDIF_RATE_32000_FLAGS: u32 = 0x00004000;
const Q0_LOCK_ADAT_B_MASK: u32 = 0x00002000;
const Q0_LOCK_ADAT_A_MASK: u32 = 0x00001000;
const Q0_SYNC_ADAT_B_MASK: u32 = 0x00000800;
const Q0_SYNC_ADAT_A_MASK: u32 = 0x00000400;

// NOTE: for second quadlet of status quadlets.
const Q1_SYNC_TCO_MASK: u32 = 0x00800000;
const Q1_LOCK_TCO_MASK: u32 = 0x00400000;
const Q1_WORD_OUT_SINGLE_MASK: u32 = 0x00002000;
const Q1_CONF_CLK_SRC_MASK: u32 = 0x00001c01;
const Q1_CONF_CLK_SRC_TCO_FLAGS: u32 = 0x00001800;
const Q1_CONF_CLK_SRC_WORD_CLK_FLAGS: u32 = 0x00001000;
const Q1_CONF_CLK_SRC_SPDIF_FLAGS: u32 = 0x00000c00;
const Q1_CONF_CLK_SRC_ADAT_B_FLAGS: u32 = 0x00000400;
const Q1_CONF_CLK_SRC_INTERNAL_FLAGS: u32 = 0x00000001;
const Q1_CONF_CLK_SRC_ADAT_A_FLAGS: u32 = 0x00000000;
const Q1_SPDIF_IN_IFACE_MASK: u32 = 0x00000200;
const Q1_OPT_OUT_SIGNAL_MASK: u32 = 0x00000100;
const Q1_SPDIF_OUT_EMPHASIS_MASK: u32 = 0x00000040;
const Q1_SPDIF_OUT_FMT_MASK: u32 = 0x00000020;
const Q1_CONF_CLK_RATE_MASK: u32 = 0x0000001e;
const Q1_CONF_CLK_RATE_192000_FLAGS: u32 = 0x00000016;
const Q1_CONF_CLK_RATE_176400_FLAGS: u32 = 0x00000010;
const Q1_CONF_CLK_RATE_128000_FLAGS: u32 = 0x00000012;
const Q1_CONF_CLK_RATE_96000_FLAGS: u32 = 0x0000000e;
const Q1_CONF_CLK_RATE_88200_FLAGS: u32 = 0x00000008;
const Q1_CONF_CLK_RATE_64000_FLAGS: u32 = 0x0000000a;
const Q1_CONF_CLK_RATE_48000_FLAGS: u32 = 0x00000006;
const Q1_CONF_CLK_RATE_44100_FLAGS: u32 = 0x00000000;
const Q1_CONF_CLK_RATE_32000_FLAGS: u32 = 0x00000002;

/// Status of clock locking.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Ff800ClkLockStatus {
    pub adat_a: bool,
    pub adat_b: bool,
    pub spdif: bool,
    pub word_clock: bool,
    pub tco: bool,
}

impl Ff800ClkLockStatus {
    const QUADLET_COUNT: usize = 2;
}

fn serialize_lock_status(status: &Ff800ClkLockStatus, quads: &mut [u32]) {
    assert!(quads.len() >= Ff800ClkLockStatus::QUADLET_COUNT);

    quads[0] &= !Q0_LOCK_ADAT_A_MASK;
    if status.adat_a {
        quads[0] |= Q0_LOCK_ADAT_A_MASK;
    }

    quads[0] &= !Q0_LOCK_ADAT_B_MASK;
    if status.adat_b {
        quads[0] |= Q0_LOCK_ADAT_B_MASK;
    }

    quads[0] &= !Q0_LOCK_SPDIF_MASK;
    if status.spdif {
        quads[0] |= Q0_LOCK_SPDIF_MASK;
    }

    quads[0] &= !Q0_LOCK_WORD_CLOCK_MASK;
    if status.word_clock {
        quads[0] |= Q0_LOCK_WORD_CLOCK_MASK;
    }

    quads[1] &= !Q1_LOCK_TCO_MASK;
    if status.tco {
        quads[1] |= Q1_LOCK_TCO_MASK;
    }
}

fn deserialize_lock_status(status: &mut Ff800ClkLockStatus, quads: &[u32]) {
    assert!(quads.len() >= Ff800ClkLockStatus::QUADLET_COUNT);

    status.adat_a = quads[0] & Q0_LOCK_ADAT_A_MASK > 0;
    status.adat_b = quads[0] & Q0_LOCK_ADAT_B_MASK > 0;
    status.spdif = quads[0] & Q0_LOCK_SPDIF_MASK > 0;
    status.word_clock = quads[0] & Q0_LOCK_WORD_CLOCK_MASK > 0;
    status.tco = quads[1] & Q1_LOCK_TCO_MASK > 0;
}

/// Status of clock synchronization.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Ff800ClkSyncStatus {
    pub adat_a: bool,
    pub adat_b: bool,
    pub spdif: bool,
    pub word_clock: bool,
    pub tco: bool,
}

impl Ff800ClkSyncStatus {
    const QUADLET_COUNT: usize = 2;
}

fn serialize_sync_status(status: &Ff800ClkSyncStatus, quads: &mut [u32]) {
    assert!(quads.len() >= Ff800ClkSyncStatus::QUADLET_COUNT);

    quads[0] &= !Q0_SYNC_ADAT_A_MASK;
    if status.adat_a {
        quads[0] |= Q0_SYNC_ADAT_A_MASK;
    }

    quads[0] &= !Q0_SYNC_ADAT_B_MASK;
    if status.adat_b {
        quads[0] |= Q0_SYNC_ADAT_B_MASK;
    }

    quads[0] &= !Q0_SYNC_SPDIF_MASK;
    if status.spdif {
        quads[0] |= Q0_SYNC_SPDIF_MASK;
    }

    quads[0] &= !Q0_SYNC_WORD_CLOCK_MASK;
    if status.word_clock {
        quads[0] |= Q0_SYNC_WORD_CLOCK_MASK;
    }

    quads[1] &= !Q1_SYNC_TCO_MASK;
    if status.tco {
        quads[1] |= Q1_SYNC_TCO_MASK;
    }
}

fn deserialize_sync_status(status: &mut Ff800ClkSyncStatus, quads: &[u32]) {
    assert!(quads.len() >= Ff800ClkSyncStatus::QUADLET_COUNT);

    status.adat_a = quads[0] & Q0_SYNC_ADAT_A_MASK > 0;
    status.adat_b = quads[0] & Q0_SYNC_ADAT_B_MASK > 0;
    status.spdif = quads[0] & Q0_SYNC_SPDIF_MASK > 0;
    status.word_clock = quads[0] & Q0_SYNC_WORD_CLOCK_MASK > 0;
    status.tco = quads[1] & Q1_SYNC_TCO_MASK > 0;
}

/// Status of clock synchronization.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Ff800Status {
    /// For S/PDIF input.
    pub spdif_in: SpdifInput,
    /// For S/PDIF output.
    pub spdif_out: FormerSpdifOutput,
    /// The type of signal to optical output interface.
    pub opt_out_signal: OpticalOutputSignal,
    /// Whether to fix speed to single even if at double/quadruple rate.
    pub word_out_single: bool,
    /// For status of synchronization to external clocks.
    pub sync: Ff800ClkSyncStatus,
    /// For status of locking to external clocks.
    pub lock: Ff800ClkLockStatus,

    pub spdif_rate: Option<ClkNominalRate>,
    pub active_clk_src: Ff800ClkSrc,
    pub external_clk_rate: Option<ClkNominalRate>,
    pub configured_clk_src: Ff800ClkSrc,
    pub configured_clk_rate: ClkNominalRate,
}

impl Ff800Status {
    const QUADLET_COUNT: usize = FORMER_STATUS_SIZE / 4;
}

impl RmeFfOffsetParamsSerialize<Ff800Status> for Ff800Protocol {
    fn serialize_offsets(params: &Ff800Status) -> Vec<u8> {
        let mut quads = [0; Ff800Status::QUADLET_COUNT];

        serialize_lock_status(&params.lock, &mut quads);
        serialize_sync_status(&params.sync, &mut quads);

        quads[0] &= !Q0_SPDIF_RATE_MASK;
        if let Some(rate) = &params.spdif_rate {
            let flag = match rate {
                ClkNominalRate::R32000 => Q0_SPDIF_RATE_32000_FLAGS,
                ClkNominalRate::R44100 => Q0_SPDIF_RATE_44100_FLAGS,
                ClkNominalRate::R48000 => Q0_SPDIF_RATE_48000_FLAGS,
                ClkNominalRate::R64000 => Q0_SPDIF_RATE_64000_FLAGS,
                ClkNominalRate::R88200 => Q0_SPDIF_RATE_88200_FLAGS,
                ClkNominalRate::R96000 => Q0_SPDIF_RATE_96000_FLAGS,
                ClkNominalRate::R128000 => Q0_SPDIF_RATE_128000_FLAGS,
                ClkNominalRate::R176400 => Q0_SPDIF_RATE_176400_FLAGS,
                ClkNominalRate::R192000 => Q0_SPDIF_RATE_192000_FLAGS,
            };
            quads[0] |= flag;
        }

        quads[0] &= !Q0_ACTIVE_CLK_SRC_MASK;
        let flag = match &params.active_clk_src {
            Ff800ClkSrc::AdatA => Q0_ACTIVE_CLK_SRC_ADAT_A_FLAGS,
            Ff800ClkSrc::AdatB => Q0_ACTIVE_CLK_SRC_ADAT_B_FLAGS,
            Ff800ClkSrc::Spdif => Q0_ACTIVE_CLK_SRC_SPDIF_FLAGS,
            Ff800ClkSrc::WordClock => Q0_ACTIVE_CLK_SRC_WORD_CLK_FLAGS,
            Ff800ClkSrc::Tco => Q0_ACTIVE_CLK_SRC_TCO_FLAGS,
            Ff800ClkSrc::Internal => Q0_ACTIVE_CLK_SRC_INTERNAL_FLAGS,
        };
        quads[0] |= flag;

        quads[0] &= !Q0_EXT_CLK_RATE_MASK;
        if let Some(rate) = &params.external_clk_rate {
            let flag = match rate {
                ClkNominalRate::R32000 => Q0_EXT_CLK_RATE_32000_FLAGS,
                ClkNominalRate::R44100 => Q0_EXT_CLK_RATE_44100_FLAGS,
                ClkNominalRate::R48000 => Q0_EXT_CLK_RATE_48000_FLAGS,
                ClkNominalRate::R64000 => Q0_EXT_CLK_RATE_64000_FLAGS,
                ClkNominalRate::R88200 => Q0_EXT_CLK_RATE_88200_FLAGS,
                ClkNominalRate::R96000 => Q0_EXT_CLK_RATE_96000_FLAGS,
                ClkNominalRate::R128000 => Q0_EXT_CLK_RATE_128000_FLAGS,
                ClkNominalRate::R176400 => Q0_EXT_CLK_RATE_176400_FLAGS,
                ClkNominalRate::R192000 => Q0_EXT_CLK_RATE_192000_FLAGS,
            };
            quads[0] |= flag;
        }

        quads[1] &= !Q1_SPDIF_IN_IFACE_MASK;
        if params.spdif_in.iface == SpdifIface::Optical {
            quads[1] |= Q1_SPDIF_IN_IFACE_MASK;
        }

        quads[1] &= !Q1_SPDIF_OUT_FMT_MASK;
        if params.spdif_out.format == SpdifFormat::Professional {
            quads[1] |= Q1_SPDIF_OUT_FMT_MASK;
        }

        quads[1] &= !Q1_SPDIF_OUT_EMPHASIS_MASK;
        if params.spdif_out.emphasis {
            quads[1] |= Q1_SPDIF_OUT_EMPHASIS_MASK;
        }

        quads[1] &= !Q1_OPT_OUT_SIGNAL_MASK;
        if params.opt_out_signal == OpticalOutputSignal::Spdif {
            quads[1] |= Q1_OPT_OUT_SIGNAL_MASK;
        }

        quads[1] &= !Q1_WORD_OUT_SINGLE_MASK;
        if params.word_out_single {
            quads[1] |= Q1_WORD_OUT_SINGLE_MASK;
        }

        quads[1] &= !Q1_CONF_CLK_SRC_MASK;
        let flag = match &params.configured_clk_src {
            Ff800ClkSrc::Internal => Q1_CONF_CLK_SRC_INTERNAL_FLAGS,
            Ff800ClkSrc::AdatB => Q1_CONF_CLK_SRC_ADAT_B_FLAGS,
            Ff800ClkSrc::Spdif => Q1_CONF_CLK_SRC_SPDIF_FLAGS,
            Ff800ClkSrc::WordClock => Q1_CONF_CLK_SRC_WORD_CLK_FLAGS,
            Ff800ClkSrc::Tco => Q1_CONF_CLK_SRC_TCO_FLAGS,
            Ff800ClkSrc::AdatA => Q1_CONF_CLK_SRC_ADAT_A_FLAGS,
        };
        quads[1] |= flag;

        quads[1] &= !Q1_CONF_CLK_RATE_MASK;
        let flag = match &params.configured_clk_rate {
            ClkNominalRate::R32000 => Q1_CONF_CLK_RATE_32000_FLAGS,
            ClkNominalRate::R44100 => Q1_CONF_CLK_RATE_44100_FLAGS,
            ClkNominalRate::R48000 => Q1_CONF_CLK_RATE_48000_FLAGS,
            ClkNominalRate::R64000 => Q1_CONF_CLK_RATE_64000_FLAGS,
            ClkNominalRate::R88200 => Q1_CONF_CLK_RATE_88200_FLAGS,
            ClkNominalRate::R96000 => Q1_CONF_CLK_RATE_96000_FLAGS,
            ClkNominalRate::R128000 => Q1_CONF_CLK_RATE_128000_FLAGS,
            ClkNominalRate::R176400 => Q1_CONF_CLK_RATE_176400_FLAGS,
            ClkNominalRate::R192000 => Q1_CONF_CLK_RATE_192000_FLAGS,
        };
        quads[1] |= flag;

        quads.iter().flat_map(|quad| quad.to_le_bytes()).collect()
    }
}

impl RmeFfOffsetParamsDeserialize<Ff800Status> for Ff800Protocol {
    fn deserialize_offsets(params: &mut Ff800Status, raw: &[u8]) {
        assert!(raw.len() >= FORMER_STATUS_SIZE);

        let mut quads = [0; Ff800Status::QUADLET_COUNT];
        let mut quadlet = [0; 4];
        quads.iter_mut().enumerate().for_each(|(i, quad)| {
            let pos = i * 4;
            quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
            *quad = u32::from_le_bytes(quadlet);
        });

        deserialize_lock_status(&mut params.lock, &quads);
        deserialize_sync_status(&mut params.sync, &quads);

        params.spdif_rate = match quads[0] & Q0_SPDIF_RATE_MASK {
            Q0_SPDIF_RATE_32000_FLAGS => Some(ClkNominalRate::R32000),
            Q0_SPDIF_RATE_44100_FLAGS => Some(ClkNominalRate::R44100),
            Q0_SPDIF_RATE_48000_FLAGS => Some(ClkNominalRate::R48000),
            Q0_SPDIF_RATE_64000_FLAGS => Some(ClkNominalRate::R64000),
            Q0_SPDIF_RATE_88200_FLAGS => Some(ClkNominalRate::R88200),
            Q0_SPDIF_RATE_96000_FLAGS => Some(ClkNominalRate::R96000),
            Q0_SPDIF_RATE_128000_FLAGS => Some(ClkNominalRate::R128000),
            Q0_SPDIF_RATE_176400_FLAGS => Some(ClkNominalRate::R176400),
            Q0_SPDIF_RATE_192000_FLAGS => Some(ClkNominalRate::R192000),
            _ => None,
        };

        params.active_clk_src = match quads[0] & Q0_ACTIVE_CLK_SRC_MASK {
            Q0_ACTIVE_CLK_SRC_ADAT_A_FLAGS => Ff800ClkSrc::AdatA,
            Q0_ACTIVE_CLK_SRC_ADAT_B_FLAGS => Ff800ClkSrc::AdatB,
            Q0_ACTIVE_CLK_SRC_SPDIF_FLAGS => Ff800ClkSrc::Spdif,
            Q0_ACTIVE_CLK_SRC_WORD_CLK_FLAGS => Ff800ClkSrc::WordClock,
            Q0_ACTIVE_CLK_SRC_TCO_FLAGS => Ff800ClkSrc::Tco,
            Q0_ACTIVE_CLK_SRC_INTERNAL_FLAGS => Ff800ClkSrc::Internal,
            _ => unreachable!(),
        };

        params.external_clk_rate = match quads[0] & Q0_EXT_CLK_RATE_MASK {
            Q0_EXT_CLK_RATE_32000_FLAGS => Some(ClkNominalRate::R32000),
            Q0_EXT_CLK_RATE_44100_FLAGS => Some(ClkNominalRate::R44100),
            Q0_EXT_CLK_RATE_48000_FLAGS => Some(ClkNominalRate::R48000),
            Q0_EXT_CLK_RATE_64000_FLAGS => Some(ClkNominalRate::R64000),
            Q0_EXT_CLK_RATE_88200_FLAGS => Some(ClkNominalRate::R88200),
            Q0_EXT_CLK_RATE_96000_FLAGS => Some(ClkNominalRate::R96000),
            Q0_EXT_CLK_RATE_128000_FLAGS => Some(ClkNominalRate::R128000),
            Q0_EXT_CLK_RATE_176400_FLAGS => Some(ClkNominalRate::R176400),
            Q0_EXT_CLK_RATE_192000_FLAGS => Some(ClkNominalRate::R192000),
            _ => None,
        };

        params.spdif_in.iface = if quads[1] & Q1_SPDIF_IN_IFACE_MASK > 0 {
            SpdifIface::Optical
        } else {
            SpdifIface::Coaxial
        };

        params.spdif_out.format = if quads[1] & Q1_SPDIF_OUT_FMT_MASK > 0 {
            SpdifFormat::Professional
        } else {
            SpdifFormat::Consumer
        };

        params.spdif_out.emphasis = quads[1] & Q1_SPDIF_OUT_EMPHASIS_MASK > 0;

        params.opt_out_signal = if quads[1] & Q1_OPT_OUT_SIGNAL_MASK > 0 {
            OpticalOutputSignal::Spdif
        } else {
            OpticalOutputSignal::Adat
        };

        params.word_out_single = quads[1] & Q1_WORD_OUT_SINGLE_MASK > 0;

        params.configured_clk_src = match quads[1] & Q1_CONF_CLK_SRC_MASK {
            Q1_CONF_CLK_SRC_INTERNAL_FLAGS => Ff800ClkSrc::Internal,
            Q1_CONF_CLK_SRC_ADAT_B_FLAGS => Ff800ClkSrc::AdatB,
            Q1_CONF_CLK_SRC_SPDIF_FLAGS => Ff800ClkSrc::Spdif,
            Q1_CONF_CLK_SRC_WORD_CLK_FLAGS => Ff800ClkSrc::WordClock,
            Q1_CONF_CLK_SRC_TCO_FLAGS => Ff800ClkSrc::Tco,
            Q1_CONF_CLK_SRC_ADAT_A_FLAGS | _ => Ff800ClkSrc::AdatA,
        };

        params.configured_clk_rate = match quads[1] & Q1_CONF_CLK_RATE_MASK {
            Q1_CONF_CLK_RATE_32000_FLAGS => ClkNominalRate::R32000,
            Q1_CONF_CLK_RATE_48000_FLAGS => ClkNominalRate::R48000,
            Q1_CONF_CLK_RATE_64000_FLAGS => ClkNominalRate::R64000,
            Q1_CONF_CLK_RATE_88200_FLAGS => ClkNominalRate::R88200,
            Q1_CONF_CLK_RATE_96000_FLAGS => ClkNominalRate::R96000,
            Q1_CONF_CLK_RATE_128000_FLAGS => ClkNominalRate::R128000,
            Q1_CONF_CLK_RATE_176400_FLAGS => ClkNominalRate::R176400,
            Q1_CONF_CLK_RATE_192000_FLAGS => ClkNominalRate::R192000,
            Q1_CONF_CLK_RATE_44100_FLAGS | _ => ClkNominalRate::R44100,
        };
    }
}

impl RmeFfCacheableParamsOperation<Ff800Status> for Ff800Protocol {
    fn cache_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &mut Ff800Status,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        read_status::<Ff800Protocol, Ff800Status>(req, node, STATUS_OFFSET, params, timeout_ms)
    }
}

// NOTE: for first quadlet of configuration quadlets.
const Q0_LINE_OUT_LEVEL_MASK: u32 = 0x00001c00;
const Q0_LINE_OUT_LEVEL_CON_FLAG: u32 = 0x00001000;
const Q0_LINE_OUT_LEVEL_PRO_FLAG: u32 = 0x00000800;
const Q0_LINE_OUT_LEVEL_HIGH_FLAG: u32 = 0x00000400;
const Q0_INPUT_0_INST_DRIVE_MASK: u32 = 0x00000200;
const Q0_INPUT_9_POWERING_MASK: u32 = 0x00000100;
const Q0_INPUT_7_POWERING_MASK: u32 = 0x00000080;
const Q0_LINE_IN_LEVEL_MASK: u32 = 0x00000038;
const Q0_LINE_IN_LEVEL_PRO_FLAG: u32 = 0x00000010;
const Q0_LINE_IN_LEVEL_CON_FLAG: u32 = 0x00000020;
const Q0_LINE_IN_LEVEL_LOW_FLAG: u32 = 0x00000008;
const Q0_INPUT_0_INST_SPKR_EMU_MASK: u32 = 0x00000004;
const Q0_INPUT_8_POWERING_MASK: u32 = 0x00000002;
const Q0_INPUT_6_POWERING_MASK: u32 = 0x00000001;

// NOTE: for second quadlet of configuration quadlets.
const Q1_INPUT_0_FRONT_JACK_MASK: u32 = 0x00000800;
const Q1_INPUT_0_INST_DRIVE_MASK: u32 = 0x00000200;
const Q1_INPUT_7_REAR_JACK_MASK: u32 = 0x00000100;
const Q1_INPUT_7_FRONT_JACK_MASK: u32 = 0x00000080;
const Q1_INPUT_6_REAR_JACK_MASK: u32 = 0x00000040;
const Q1_INPUT_6_FRONT_JACK_MASK: u32 = 0x00000020;
const Q1_LINE_OUT_LEVEL_MASK: u32 = 0x00000018;
const Q1_LINE_OUT_LEVEL_PRO_FLAG: u32 = 0x00000018;
const Q1_LINE_OUT_LEVEL_HIGH_FLAG: u32 = 0x00000010;
const Q1_LINE_OUT_LEVEL_CON_FLAG: u32 = 0x00000008;
const Q1_INPUT_0_REAR_JACK_MASK: u32 = 0x00000004;
const Q1_LINE_IN_LEVEL_MASK: u32 = 0x00000003;
const Q1_LINE_IN_LEVEL_CON_FLAG: u32 = 0x00000003;
const Q1_LINE_IN_LEVEL_PRO_FLAG: u32 = 0x00000002;
const Q1_LINE_IN_LEVEL_LOW_FLAG: u32 = 0x00000000;

// NOTE: for third quadlet of configuration quadlets.
const Q2_SPDIF_IN_USE_PREEMBLE: u32 = 0x40000000;
const Q2_INPUT_0_INST_LIMITTER_MASK: u32 = 0x00010000;
const Q2_WORD_OUT_SINGLE_SPEED_MASK: u32 = 0x00002000;
const Q2_CLK_SRC_MASK: u32 = 0x00001c01;
const Q2_CLK_SRC_TCO_FLAG: u32 = 0x00001c00;
const Q2_CLK_SRC_WORD_CLK_FLAG: u32 = 0x00001400;
const Q2_CLK_SRC_SPDIF_FLAG: u32 = 0x00000c00;
const Q2_CLK_SRC_ADAT_B_FLAG: u32 = 0x00000400;
const Q2_CLK_SRC_INTERNAL_FLAG: u32 = 0x00000001;
const Q2_CLK_SRC_ADAT_A_FLAG: u32 = 0x00000000;
const Q2_SPDIF_IN_IFACE_OPT_MASK: u32 = 0x00000200;
const Q2_OPT_OUT_SIGNAL_MASK: u32 = 0x00000100;
const Q2_SPDIF_OUT_NON_AUDIO_MASK: u32 = 0x00000080;
const Q2_SPDIF_OUT_EMPHASIS_MASK: u32 = 0x00000040;
const Q2_SPDIF_OUT_FMT_PRO_MASK: u32 = 0x00000020;
const Q2_CLK_AVAIL_RATE_QUADRUPLE_MASK: u32 = 0x00000010;
const Q2_CLK_AVAIL_RATE_DOUBLE_MASK: u32 = 0x00000008;
const Q2_CLK_AVAIL_RATE_BASE_48000_MASK: u32 = 0x00000004;
const Q2_CLK_AVAIL_RATE_BASE_44100_MASK: u32 = 0x00000002;
const Q2_CONTINUE_AT_ERRORS: u32 = 0x80000000;

/// Configurations of sampling clock.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Ff800ClkConfig {
    pub primary_src: Ff800ClkSrc,
    avail_rate_44100: bool,
    avail_rate_48000: bool,
    avail_rate_double: bool,
    avail_rate_quadruple: bool,
}

impl Default for Ff800ClkConfig {
    fn default() -> Self {
        Self {
            primary_src: Ff800ClkSrc::default(),
            avail_rate_44100: true,
            avail_rate_48000: true,
            avail_rate_double: true,
            avail_rate_quadruple: true,
        }
    }
}

impl Ff800ClkConfig {
    const QUADLET_COUNT: usize = 3;
}

fn serialize_clock_config(config: &Ff800ClkConfig, quads: &mut [u32]) {
    assert!(quads.len() >= Ff800ClkConfig::QUADLET_COUNT);

    quads[2] &= !Q2_CLK_SRC_MASK;
    let flag = match config.primary_src {
        Ff800ClkSrc::Internal => Q2_CLK_SRC_INTERNAL_FLAG,
        Ff800ClkSrc::WordClock => Q2_CLK_SRC_WORD_CLK_FLAG,
        Ff800ClkSrc::AdatA => Q2_CLK_SRC_ADAT_A_FLAG,
        Ff800ClkSrc::AdatB => Q2_CLK_SRC_ADAT_B_FLAG,
        Ff800ClkSrc::Spdif => Q2_CLK_SRC_SPDIF_FLAG,
        Ff800ClkSrc::Tco => Q2_CLK_SRC_TCO_FLAG,
    };
    quads[2] |= flag;

    quads[2] &= !Q2_CLK_AVAIL_RATE_BASE_44100_MASK;
    if config.avail_rate_44100 {
        quads[2] |= Q2_CLK_AVAIL_RATE_BASE_44100_MASK;
    }

    quads[2] &= !Q2_CLK_AVAIL_RATE_BASE_48000_MASK;
    if config.avail_rate_48000 {
        quads[2] |= Q2_CLK_AVAIL_RATE_BASE_48000_MASK;
    }

    quads[2] &= !Q2_CLK_AVAIL_RATE_DOUBLE_MASK;
    if config.avail_rate_double {
        quads[2] |= Q2_CLK_AVAIL_RATE_DOUBLE_MASK;
    }

    quads[2] &= !Q2_CLK_AVAIL_RATE_QUADRUPLE_MASK;
    if config.avail_rate_quadruple {
        quads[2] |= Q2_CLK_AVAIL_RATE_QUADRUPLE_MASK;
    }
}

fn deserialize_clock_config(config: &mut Ff800ClkConfig, quads: &[u32]) {
    assert!(quads.len() >= Ff800ClkConfig::QUADLET_COUNT);

    config.primary_src = match quads[2] & Q2_CLK_SRC_MASK {
        Q2_CLK_SRC_INTERNAL_FLAG => Ff800ClkSrc::Internal,
        Q2_CLK_SRC_WORD_CLK_FLAG => Ff800ClkSrc::WordClock,
        Q2_CLK_SRC_ADAT_B_FLAG => Ff800ClkSrc::AdatB,
        Q2_CLK_SRC_SPDIF_FLAG => Ff800ClkSrc::Spdif,
        Q2_CLK_SRC_TCO_FLAG => Ff800ClkSrc::Tco,
        Q2_CLK_SRC_ADAT_A_FLAG | _ => Ff800ClkSrc::AdatA,
    };

    config.avail_rate_44100 = quads[2] & Q2_CLK_AVAIL_RATE_BASE_44100_MASK > 0;
    config.avail_rate_48000 = quads[2] & Q2_CLK_AVAIL_RATE_BASE_48000_MASK > 0;
    config.avail_rate_double = quads[2] & Q2_CLK_AVAIL_RATE_DOUBLE_MASK > 0;
    config.avail_rate_quadruple = quads[2] & Q2_CLK_AVAIL_RATE_QUADRUPLE_MASK > 0;
}

/// Configurations for instrument.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Ff800InstConfig {
    /// Whether to add extra gain by 25 dB.
    pub drive: bool,
    /// Whether to soft limitter to reduce by -10 dB.
    pub limitter: bool,
    /// Whether to enable low pass and high pass filter.
    pub speaker_emulation: bool,
}

impl Ff800InstConfig {
    const QUADLET_COUNT: usize = 3;
}

fn serialize_instrument_config(config: &Ff800InstConfig, quads: &mut [u32]) {
    assert!(quads.len() >= Ff800InstConfig::QUADLET_COUNT);

    quads[0] &= !Q0_INPUT_0_INST_DRIVE_MASK;
    quads[1] &= !Q1_INPUT_0_INST_DRIVE_MASK;
    if config.drive {
        quads[0] |= Q0_INPUT_0_INST_DRIVE_MASK;
        quads[1] |= Q1_INPUT_0_INST_DRIVE_MASK;
    }

    quads[2] &= !Q2_INPUT_0_INST_LIMITTER_MASK;
    if config.limitter {
        quads[2] |= Q2_INPUT_0_INST_LIMITTER_MASK;
    }

    quads[0] &= !Q0_INPUT_0_INST_SPKR_EMU_MASK;
    if config.speaker_emulation {
        quads[0] |= Q0_INPUT_0_INST_SPKR_EMU_MASK;
    }
}

fn deserialize_instrument_config(config: &mut Ff800InstConfig, quads: &[u32]) {
    assert!(quads.len() >= Ff800InstConfig::QUADLET_COUNT);

    config.drive =
        quads[0] & Q0_INPUT_0_INST_DRIVE_MASK > 0 && quads[0] & Q1_INPUT_0_INST_DRIVE_MASK > 0;
    config.limitter = quads[2] & Q2_INPUT_0_INST_LIMITTER_MASK > 0;
    config.speaker_emulation = quads[0] & Q0_INPUT_0_INST_SPKR_EMU_MASK > 0;
}

/// Jack of analog inputs.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Ff800AnalogInputJack {
    Front,
    Rear,
    FrontRear,
}

impl Default for Ff800AnalogInputJack {
    fn default() -> Self {
        Self::Front
    }
}

/// Configuration for analog inputs.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Ff800AnalogInConfig {
    /// Whether to use rear jack instead of front jack for input 1, 7, and 8.
    pub jacks: [Ff800AnalogInputJack; 3],
    /// The nominal level of audio signal for line input.
    pub line_level: FormerLineInNominalLevel,
    /// Whether to deliver +48 V powering for mic 7, 8, 9, 10.
    pub phantom_powering: [bool; 4],
    /// The configurations for instrument input.
    pub inst: Ff800InstConfig,
}

impl Ff800AnalogInConfig {
    const QUADLET_COUNT: usize = 2;
}

fn serialize_analog_input_config(config: &Ff800AnalogInConfig, quads: &mut [u32]) {
    assert!(quads.len() >= Ff800AnalogInConfig::QUADLET_COUNT);

    [
        (Q1_INPUT_0_REAR_JACK_MASK, Q1_INPUT_0_FRONT_JACK_MASK),
        (Q1_INPUT_6_REAR_JACK_MASK, Q1_INPUT_6_FRONT_JACK_MASK),
        (Q1_INPUT_7_REAR_JACK_MASK, Q1_INPUT_7_FRONT_JACK_MASK),
    ]
    .iter()
    .zip(&config.jacks)
    .for_each(|(&(rear_mask, front_mask), &jack)| {
        if jack != Ff800AnalogInputJack::Front {
            quads[1] |= rear_mask;
        }
        if jack != Ff800AnalogInputJack::Rear {
            quads[1] |= front_mask;
        }
    });

    quads[0] &= !Q0_LINE_IN_LEVEL_MASK;
    quads[1] &= !Q1_LINE_IN_LEVEL_MASK;
    match config.line_level {
        FormerLineInNominalLevel::Low => {
            quads[0] |= Q0_LINE_IN_LEVEL_LOW_FLAG;
            quads[1] |= Q1_LINE_IN_LEVEL_LOW_FLAG;
        }
        FormerLineInNominalLevel::Consumer => {
            quads[0] |= Q0_LINE_IN_LEVEL_CON_FLAG;
            quads[1] |= Q1_LINE_IN_LEVEL_CON_FLAG;
        }
        FormerLineInNominalLevel::Professional => {
            quads[0] |= Q0_LINE_IN_LEVEL_PRO_FLAG;
            quads[1] |= Q1_LINE_IN_LEVEL_PRO_FLAG;
        }
    }

    if config.phantom_powering[0] {
        quads[0] |= Q0_INPUT_6_POWERING_MASK;
    }
    if config.phantom_powering[1] {
        quads[0] |= Q0_INPUT_7_POWERING_MASK;
    }
    if config.phantom_powering[2] {
        quads[0] |= Q0_INPUT_8_POWERING_MASK;
    }
    if config.phantom_powering[3] {
        quads[0] |= Q0_INPUT_9_POWERING_MASK;
    }

    serialize_instrument_config(&config.inst, quads);
}

fn deserialize_analog_input_config(config: &mut Ff800AnalogInConfig, quads: &[u32]) {
    assert!(quads.len() >= Ff800AnalogInConfig::QUADLET_COUNT);

    [
        (Q1_INPUT_0_REAR_JACK_MASK, Q1_INPUT_0_FRONT_JACK_MASK),
        (Q1_INPUT_6_REAR_JACK_MASK, Q1_INPUT_6_FRONT_JACK_MASK),
        (Q1_INPUT_7_REAR_JACK_MASK, Q1_INPUT_7_FRONT_JACK_MASK),
    ]
    .iter()
    .zip(&mut config.jacks)
    .for_each(|(&(rear_mask, front_mask), jack)| {
        *jack = match (quads[1] & rear_mask > 0, quads[1] & front_mask > 0) {
            (true, true) => Ff800AnalogInputJack::FrontRear,
            (true, false) => Ff800AnalogInputJack::Rear,
            (false, true) => Ff800AnalogInputJack::Front,
            _ => unreachable!(),
        };
    });

    let pair = (
        quads[0] & Q0_LINE_IN_LEVEL_MASK,
        quads[1] & Q1_LINE_IN_LEVEL_MASK,
    );
    config.line_level = match pair {
        (Q0_LINE_IN_LEVEL_LOW_FLAG, Q1_LINE_IN_LEVEL_LOW_FLAG) => FormerLineInNominalLevel::Low,
        (Q0_LINE_IN_LEVEL_CON_FLAG, Q1_LINE_IN_LEVEL_CON_FLAG) => {
            FormerLineInNominalLevel::Consumer
        }
        (Q0_LINE_IN_LEVEL_PRO_FLAG, Q1_LINE_IN_LEVEL_PRO_FLAG) => {
            FormerLineInNominalLevel::Professional
        }
        _ => unreachable!(),
    };

    config.phantom_powering[0] = quads[0] & Q0_INPUT_6_POWERING_MASK > 0;
    config.phantom_powering[1] = quads[0] & Q0_INPUT_7_POWERING_MASK > 0;
    config.phantom_powering[2] = quads[0] & Q0_INPUT_8_POWERING_MASK > 0;
    config.phantom_powering[3] = quads[0] & Q0_INPUT_9_POWERING_MASK > 0;

    deserialize_instrument_config(&mut config.inst, quads);
}

/// Configurations for Fireface 800.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Ff800Config {
    /// For sampling clock.
    pub clk: Ff800ClkConfig,
    /// For analog inputs.
    pub analog_in: Ff800AnalogInConfig,
    /// The nominal level of audio signal for line output.
    pub line_out_level: LineOutNominalLevel,
    /// For S/PDIF input.
    pub spdif_in: SpdifInput,
    /// For S/PDIF output.
    pub spdif_out: FormerSpdifOutput,
    /// The type of signal to optical output interface.
    pub opt_out_signal: OpticalOutputSignal,
    /// Whether to fix speed to single even if at double/quadruple rate.
    pub word_out_single: bool,
    /// Whether to continue audio processing against any synchronization corruption.
    continue_at_errors: bool,
}

impl Default for Ff800Config {
    fn default() -> Self {
        Self {
            clk: Default::default(),
            analog_in: Default::default(),
            line_out_level: Default::default(),
            spdif_in: Default::default(),
            spdif_out: Default::default(),
            opt_out_signal: Default::default(),
            word_out_single: Default::default(),
            continue_at_errors: true,
        }
    }
}

impl Ff800Config {
    const QUADLET_COUNT: usize = FORMER_CONFIG_SIZE / 4;

    /// Although the configuration registers are write-only, some of them are available in status
    /// registers.
    pub fn init(&mut self, status: &Ff800Status) {
        self.clk.primary_src = status.configured_clk_src;
        self.spdif_in = status.spdif_in;
        self.spdif_out = status.spdif_out;
        self.opt_out_signal = status.opt_out_signal;
        self.word_out_single = status.word_out_single;
    }
}

impl RmeFfOffsetParamsSerialize<Ff800Config> for Ff800Protocol {
    fn serialize_offsets(params: &Ff800Config) -> Vec<u8> {
        let mut quads = [0; Ff800Config::QUADLET_COUNT];

        serialize_clock_config(&params.clk, &mut quads);
        serialize_analog_input_config(&params.analog_in, &mut quads);

        quads[0] &= !Q0_LINE_OUT_LEVEL_MASK;
        quads[1] &= !Q1_LINE_OUT_LEVEL_MASK;
        match params.line_out_level {
            LineOutNominalLevel::High => {
                quads[0] |= Q0_LINE_OUT_LEVEL_HIGH_FLAG;
                quads[1] |= Q1_LINE_OUT_LEVEL_HIGH_FLAG;
            }
            LineOutNominalLevel::Consumer => {
                quads[0] |= Q0_LINE_OUT_LEVEL_CON_FLAG;
                quads[1] |= Q1_LINE_OUT_LEVEL_CON_FLAG;
            }
            LineOutNominalLevel::Professional => {
                quads[0] |= Q0_LINE_OUT_LEVEL_PRO_FLAG;
                quads[1] |= Q1_LINE_OUT_LEVEL_PRO_FLAG;
            }
        }

        quads[2] &= !Q2_SPDIF_IN_IFACE_OPT_MASK;
        if params.spdif_in.iface == SpdifIface::Optical {
            quads[2] |= Q2_SPDIF_IN_IFACE_OPT_MASK;
        }

        quads[2] &= !Q2_SPDIF_IN_USE_PREEMBLE;
        if params.spdif_in.use_preemble {
            quads[2] |= Q2_SPDIF_IN_USE_PREEMBLE;
        }

        quads[2] &= !Q2_OPT_OUT_SIGNAL_MASK;
        if params.opt_out_signal == OpticalOutputSignal::Spdif {
            quads[2] |= Q2_OPT_OUT_SIGNAL_MASK;
        }

        quads[2] &= !Q2_SPDIF_OUT_FMT_PRO_MASK;
        if params.spdif_out.format == SpdifFormat::Professional {
            quads[2] |= Q2_SPDIF_OUT_FMT_PRO_MASK;
        }

        quads[2] &= !Q2_SPDIF_OUT_EMPHASIS_MASK;
        if params.spdif_out.emphasis {
            quads[2] |= Q2_SPDIF_OUT_EMPHASIS_MASK;
        }

        quads[2] &= !Q2_SPDIF_OUT_NON_AUDIO_MASK;
        if params.spdif_out.non_audio {
            quads[2] |= Q2_SPDIF_OUT_NON_AUDIO_MASK;
        }

        quads[2] &= !Q2_WORD_OUT_SINGLE_SPEED_MASK;
        if params.word_out_single {
            quads[2] |= Q2_WORD_OUT_SINGLE_SPEED_MASK;
        }

        quads[2] &= !Q2_CONTINUE_AT_ERRORS;
        if params.continue_at_errors {
            quads[2] |= Q2_CONTINUE_AT_ERRORS;
        }

        quads.iter().flat_map(|quad| quad.to_le_bytes()).collect()
    }
}

impl RmeFfOffsetParamsDeserialize<Ff800Config> for Ff800Protocol {
    fn deserialize_offsets(params: &mut Ff800Config, raw: &[u8]) {
        let mut quads = [0; Ff800Config::QUADLET_COUNT];

        let mut quadlet = [0; 4];
        quads.iter_mut().enumerate().for_each(|(i, quad)| {
            let pos = i * 4;
            quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
            *quad = u32::from_le_bytes(quadlet);
        });

        deserialize_clock_config(&mut params.clk, &quads);
        deserialize_analog_input_config(&mut params.analog_in, &quads);

        let pair = (
            quads[0] & Q0_LINE_OUT_LEVEL_MASK,
            quads[1] & Q1_LINE_OUT_LEVEL_MASK,
        );
        params.line_out_level = match pair {
            (Q0_LINE_OUT_LEVEL_HIGH_FLAG, Q1_LINE_OUT_LEVEL_HIGH_FLAG) => LineOutNominalLevel::High,
            (Q0_LINE_OUT_LEVEL_CON_FLAG, Q1_LINE_OUT_LEVEL_CON_FLAG) => {
                LineOutNominalLevel::Consumer
            }
            (Q0_LINE_OUT_LEVEL_PRO_FLAG, Q1_LINE_OUT_LEVEL_PRO_FLAG) => {
                LineOutNominalLevel::Professional
            }
            _ => unreachable!(),
        };

        params.spdif_in.iface = if quads[2] & Q2_SPDIF_IN_IFACE_OPT_MASK > 0 {
            SpdifIface::Optical
        } else {
            SpdifIface::Coaxial
        };
        params.spdif_in.use_preemble = quads[2] & Q2_SPDIF_IN_USE_PREEMBLE > 0;

        params.spdif_out.format = if quads[2] & Q2_SPDIF_OUT_FMT_PRO_MASK > 0 {
            SpdifFormat::Professional
        } else {
            SpdifFormat::Consumer
        };
        params.spdif_out.emphasis = quads[2] & Q2_SPDIF_OUT_EMPHASIS_MASK > 0;
        params.spdif_out.non_audio = quads[2] & Q2_SPDIF_OUT_NON_AUDIO_MASK > 0;

        params.opt_out_signal = if quads[2] & Q2_OPT_OUT_SIGNAL_MASK > 0 {
            OpticalOutputSignal::Spdif
        } else {
            OpticalOutputSignal::Adat
        };

        params.word_out_single = quads[2] & Q2_WORD_OUT_SINGLE_SPEED_MASK > 0;
        params.continue_at_errors = quads[2] & Q2_CONTINUE_AT_ERRORS > 0;
    }
}

impl RmeFfWhollyUpdatableParamsOperation<Ff800Config> for Ff800Protocol {
    fn update_wholly(
        req: &mut FwReq,
        node: &mut FwNode,
        params: &Ff800Config,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        write_config::<Ff800Protocol, Ff800Config>(req, node, CFG_OFFSET, params, timeout_ms)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lock_status_serdes() {
        let orig = Ff800ClkLockStatus {
            adat_a: true,
            adat_b: true,
            spdif: true,
            word_clock: true,
            tco: true,
        };
        let mut quads = [0; Ff800ClkLockStatus::QUADLET_COUNT];
        serialize_lock_status(&orig, &mut quads);
        let mut target = Ff800ClkLockStatus::default();
        deserialize_lock_status(&mut target, &quads);

        assert_eq!(target, orig);
    }

    #[test]
    fn sync_status_serdes() {
        let orig = Ff800ClkSyncStatus {
            adat_a: true,
            adat_b: true,
            spdif: true,
            word_clock: true,
            tco: true,
        };
        let mut quads = [0; Ff800ClkSyncStatus::QUADLET_COUNT];
        serialize_sync_status(&orig, &mut quads);
        let mut target = Ff800ClkSyncStatus::default();
        deserialize_sync_status(&mut target, &quads);

        assert_eq!(target, orig);
    }

    fn quads_to_bytes(quads: &[u32]) -> Vec<u8> {
        quads.iter().flat_map(|quad| quad.to_le_bytes()).collect()
    }

    #[test]
    fn status_serdes() {
        let mut status = Ff800Status::default();

        let quads = [0x02001000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.lock.adat_a, true);

        let quads = [0x02002000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.lock.adat_b, true);

        let quads = [0x02040000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.lock.spdif, true);

        let quads = [0x22000000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.lock.word_clock, true);

        let quads = [0x02000400, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.sync.adat_a, true);

        let quads = [0x02000800, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.sync.adat_b, true);

        let quads = [0x02100800, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.sync.spdif, true);

        let quads = [0x42000000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.sync.word_clock, true);

        let quads = [0x02000000, 0x00800000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.sync.tco, true);

        let quads = [0x02004000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.spdif_rate, Some(ClkNominalRate::R32000));

        let quads = [0x02008000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.spdif_rate, Some(ClkNominalRate::R44100));

        let quads = [0x0200c000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.spdif_rate, Some(ClkNominalRate::R48000));

        let quads = [0x02010000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.spdif_rate, Some(ClkNominalRate::R64000));

        let quads = [0x02014000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.spdif_rate, Some(ClkNominalRate::R88200));

        let quads = [0x02018000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.spdif_rate, Some(ClkNominalRate::R96000));

        let quads = [0x0201c000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.spdif_rate, Some(ClkNominalRate::R128000));

        let quads = [0x02020000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.spdif_rate, Some(ClkNominalRate::R176400));

        let quads = [0x02024000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.spdif_rate, Some(ClkNominalRate::R192000));

        let quads = [0x02000000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.active_clk_src, Ff800ClkSrc::AdatA);

        let quads = [0x02400000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.active_clk_src, Ff800ClkSrc::AdatB);

        let quads = [0x02c00000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.active_clk_src, Ff800ClkSrc::Spdif);

        let quads = [0x03000000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.active_clk_src, Ff800ClkSrc::WordClock);

        let quads = [0x03800000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.active_clk_src, Ff800ClkSrc::Tco);

        let quads = [0x03c00000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.active_clk_src, Ff800ClkSrc::Internal);

        let quads = [0x02000000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.external_clk_rate, Some(ClkNominalRate::R32000));

        let quads = [0x04000000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.external_clk_rate, Some(ClkNominalRate::R44100));

        let quads = [0x06000000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.external_clk_rate, Some(ClkNominalRate::R48000));

        let quads = [0x08000000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.external_clk_rate, Some(ClkNominalRate::R64000));

        let quads = [0x0a000000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.external_clk_rate, Some(ClkNominalRate::R88200));

        let quads = [0x0e000000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.external_clk_rate, Some(ClkNominalRate::R96000));

        let quads = [0x0c000000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.external_clk_rate, Some(ClkNominalRate::R128000));

        let quads = [0x10000000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.external_clk_rate, Some(ClkNominalRate::R176400));

        let quads = [0x12000000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.external_clk_rate, Some(ClkNominalRate::R192000));

        let quads = [0x02000000, 0x00400000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.lock.tco, true);

        let quads = [0x02000000, 0x00800000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.sync.tco, true);

        let quads = [0x02000000, 0x00002000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.word_out_single, true);

        let quads = [0x02000000, 0x00000200];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.spdif_in.iface, SpdifIface::Optical);

        let quads = [0x02000000, 0x00000100];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.opt_out_signal, OpticalOutputSignal::Spdif);

        let quads = [0x02000000, 0x00000040];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.spdif_out.emphasis, true);

        let quads = [0x02000000, 0x00000020];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.spdif_out.format, SpdifFormat::Professional);

        let quads = [0x02000000, 0x00000002];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.configured_clk_rate, ClkNominalRate::R32000);

        let quads = [0x02000000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.configured_clk_rate, ClkNominalRate::R44100);

        let quads = [0x02000000, 0x00000006];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.configured_clk_rate, ClkNominalRate::R48000);

        let quads = [0x02000000, 0x0000000a];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.configured_clk_rate, ClkNominalRate::R64000);

        let quads = [0x02000000, 0x00000008];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.configured_clk_rate, ClkNominalRate::R88200);

        let quads = [0x02000000, 0x0000000e];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.configured_clk_rate, ClkNominalRate::R96000);

        let quads = [0x02000000, 0x00000012];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.configured_clk_rate, ClkNominalRate::R128000);

        let quads = [0x02000000, 0x00000010];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.configured_clk_rate, ClkNominalRate::R176400);

        let quads = [0x02000000, 0x00000016];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.configured_clk_rate, ClkNominalRate::R192000);

        let quads = [0x02000000, 0x00001000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.configured_clk_src, Ff800ClkSrc::WordClock);

        let quads = [0x02000000, 0x00001800];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.configured_clk_src, Ff800ClkSrc::Tco);

        let quads = [0x02000000, 0x00000c00];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.configured_clk_src, Ff800ClkSrc::Spdif);

        let quads = [0x02000000, 0x00000400];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.configured_clk_src, Ff800ClkSrc::AdatB);

        let quads = [0x02000000, 0x00000001];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.configured_clk_src, Ff800ClkSrc::Internal);

        let quads = [0x02000000, 0x00000000];
        Ff800Protocol::deserialize_offsets(&mut status, &quads_to_bytes(&quads));
        assert_eq!(status.configured_clk_src, Ff800ClkSrc::AdatA);

        let raw = Ff800Protocol::serialize_offsets(&status);
        let mut target = Ff800Status::default();
        Ff800Protocol::deserialize_offsets(&mut target, &raw);
        assert_eq!(target, status);
    }

    #[test]
    fn clock_config_serdes() {
        let mut orig = Ff800ClkConfig::default();
        let mut cfg = Ff800ClkConfig::default();

        orig.primary_src = Ff800ClkSrc::Internal;
        let mut quads = [0u32; 3];
        serialize_clock_config(&orig, &mut quads);
        assert_eq!(quads[2], 0x0000001f);
        deserialize_clock_config(&mut cfg, &quads);
        assert_eq!(cfg, orig);

        orig.primary_src = Ff800ClkSrc::WordClock;
        let mut quads = [0u32; 3];
        serialize_clock_config(&orig, &mut quads);
        assert_eq!(quads[2], 0x0000141e);
        deserialize_clock_config(&mut cfg, &quads);
        assert_eq!(cfg, orig);

        orig.primary_src = Ff800ClkSrc::AdatA;
        let mut quads = [0u32; 3];
        serialize_clock_config(&orig, &mut quads);
        assert_eq!(quads[2], 0x0000001e);
        deserialize_clock_config(&mut cfg, &quads);
        assert_eq!(cfg, orig);

        orig.primary_src = Ff800ClkSrc::AdatB;
        let mut quads = [0u32; 3];
        serialize_clock_config(&orig, &mut quads);
        assert_eq!(quads[2], 0x0000041e);
        deserialize_clock_config(&mut cfg, &quads);
        assert_eq!(cfg, orig);

        orig.primary_src = Ff800ClkSrc::Spdif;
        let mut quads = [0u32; 3];
        serialize_clock_config(&orig, &mut quads);
        assert_eq!(quads[2], 0x00000c1e);
        deserialize_clock_config(&mut cfg, &quads);
        assert_eq!(cfg, orig);

        orig.primary_src = Ff800ClkSrc::Tco;
        let mut quads = [0u32; 3];
        serialize_clock_config(&orig, &mut quads);
        assert_eq!(quads[2], 0x00001c1e);
        deserialize_clock_config(&mut cfg, &quads);
        assert_eq!(cfg, orig);
    }

    #[test]
    fn instrument_config_serdes() {
        let mut orig = Ff800InstConfig::default();
        let mut cfg = Ff800InstConfig::default();

        orig.drive = false;
        orig.limitter = false;
        orig.speaker_emulation = false;
        let mut quads = [0u32; 3];
        serialize_instrument_config(&orig, &mut quads);
        assert_eq!(&quads[..], &[0x00000000, 0x00000000, 0x00000000]);
        deserialize_instrument_config(&mut cfg, &quads);
        assert_eq!(cfg, orig);

        orig.drive = true;
        let mut quads = [0u32; 3];
        serialize_instrument_config(&orig, &mut quads);
        assert_eq!(&quads[..], &[0x00000200, 0x00000200, 0x00000000]);
        deserialize_instrument_config(&mut cfg, &quads);
        assert_eq!(cfg, orig);

        orig.limitter = true;
        let mut quads = [0u32; 3];
        serialize_instrument_config(&orig, &mut quads);
        assert_eq!(&quads[..], &[0x00000200, 0x00000200, 0x00010000]);
        deserialize_instrument_config(&mut cfg, &quads);
        assert_eq!(cfg, orig);

        orig.speaker_emulation = true;
        let mut quads = [0u32; 3];
        serialize_instrument_config(&orig, &mut quads);
        assert_eq!(&quads[..], &[0x00000204, 0x00000200, 0x00010000]);
        deserialize_instrument_config(&mut cfg, &quads);
        assert_eq!(cfg, orig);
    }

    #[test]
    fn analog_input_config_serdes() {
        let mut orig = Ff800AnalogInConfig::default();
        let mut cfg = Ff800AnalogInConfig::default();

        orig.jacks[0] = Ff800AnalogInputJack::FrontRear;
        let mut quads = [0u32; 3];
        serialize_analog_input_config(&orig, &mut quads);
        assert_eq!(&quads[..], &[0x00000008, 0x000008a4, 0x00000000]);
        deserialize_analog_input_config(&mut cfg, &quads);
        assert_eq!(cfg, orig);

        orig.jacks[1] = Ff800AnalogInputJack::FrontRear;
        let mut quads = [0u32; 3];
        serialize_analog_input_config(&orig, &mut quads);
        assert_eq!(&quads[..], &[0x00000008, 0x000008e4, 0x00000000]);
        deserialize_analog_input_config(&mut cfg, &quads);
        assert_eq!(cfg, orig);

        orig.jacks[2] = Ff800AnalogInputJack::FrontRear;
        let mut quads = [0u32; 3];
        serialize_analog_input_config(&orig, &mut quads);
        assert_eq!(&quads[..], &[0x00000008, 0x000009e4, 0x00000000]);
        deserialize_analog_input_config(&mut cfg, &quads);
        assert_eq!(cfg, orig);

        orig.line_level = FormerLineInNominalLevel::Consumer;
        let mut quads = [0u32; 3];
        serialize_analog_input_config(&orig, &mut quads);
        assert_eq!(&quads[..], &[0x00000020, 0x000009e7, 0x00000000]);
        deserialize_analog_input_config(&mut cfg, &quads);
        assert_eq!(cfg, orig);

        orig.line_level = FormerLineInNominalLevel::Professional;
        let mut quads = [0u32; 3];
        serialize_analog_input_config(&orig, &mut quads);
        assert_eq!(&quads[..], &[0x00000010, 0x000009e6, 0x00000000]);
        deserialize_analog_input_config(&mut cfg, &quads);
        assert_eq!(cfg, orig);

        orig.phantom_powering[0] = true;
        let mut quads = [0u32; 3];
        serialize_analog_input_config(&orig, &mut quads);
        assert_eq!(&quads[..], &[0x00000011, 0x000009e6, 0x00000000]);
        deserialize_analog_input_config(&mut cfg, &quads);
        assert_eq!(cfg, orig);

        orig.phantom_powering[1] = true;
        let mut quads = [0u32; 3];
        serialize_analog_input_config(&orig, &mut quads);
        assert_eq!(&quads[..], &[0x00000091, 0x000009e6, 0x00000000]);
        deserialize_analog_input_config(&mut cfg, &quads);
        assert_eq!(cfg, orig);

        orig.phantom_powering[2] = true;
        let mut quads = [0u32; 3];
        serialize_analog_input_config(&orig, &mut quads);
        assert_eq!(&quads[..], &[0x00000093, 0x000009e6, 0x00000000]);
        deserialize_analog_input_config(&mut cfg, &quads);
        assert_eq!(cfg, orig);

        orig.phantom_powering[3] = true;
        let mut quads = [0u32; 3];
        serialize_analog_input_config(&orig, &mut quads);
        assert_eq!(&quads[..], &[0x00000193, 0x000009e6, 0x00000000]);
        deserialize_analog_input_config(&mut cfg, &quads);
        assert_eq!(cfg, orig);
    }

    fn config_to_quads(raw: &[u8]) -> Vec<u32> {
        let mut quadlet = [0; 4];
        (0..Ff800Config::QUADLET_COUNT)
            .map(|i| {
                let pos = i * 4;
                quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                u32::from_le_bytes(quadlet)
            })
            .collect()
    }

    #[test]
    fn config_serdes() {
        let mut orig = Ff800Config::default();
        let mut cfg = Ff800Config::default();

        orig.line_out_level = LineOutNominalLevel::High;
        let raw = Ff800Protocol::serialize_offsets(&orig);
        Ff800Protocol::deserialize_offsets(&mut cfg, &raw);
        assert_eq!(cfg, orig);
        let quads = config_to_quads(&raw);
        assert_eq!(&quads[..], &[0x00000408, 0x000008b0, 0x8000001e]);

        orig.line_out_level = LineOutNominalLevel::Consumer;
        let raw = Ff800Protocol::serialize_offsets(&orig);
        Ff800Protocol::deserialize_offsets(&mut cfg, &raw);
        assert_eq!(cfg, orig);
        let quads = config_to_quads(&raw);
        assert_eq!(&quads[..], &[0x00001008, 0x000008a8, 0x8000001e]);

        orig.line_out_level = LineOutNominalLevel::Professional;
        let raw = Ff800Protocol::serialize_offsets(&orig);
        Ff800Protocol::deserialize_offsets(&mut cfg, &raw);
        assert_eq!(cfg, orig);
        let quads = config_to_quads(&raw);
        assert_eq!(&quads[..], &[0x00000808, 0x000008b8, 0x8000001e]);

        orig.spdif_in.iface = SpdifIface::Optical;
        let raw = Ff800Protocol::serialize_offsets(&orig);
        Ff800Protocol::deserialize_offsets(&mut cfg, &raw);
        assert_eq!(cfg, orig);
        let quads = config_to_quads(&raw);
        assert_eq!(&quads[..], &[0x00000808, 0x000008b8, 0x8000021e]);

        orig.spdif_in.use_preemble = true;
        let raw = Ff800Protocol::serialize_offsets(&orig);
        Ff800Protocol::deserialize_offsets(&mut cfg, &raw);
        assert_eq!(cfg, orig);
        let quads = config_to_quads(&raw);
        assert_eq!(&quads[..], &[0x00000808, 0x000008b8, 0xc000021e]);

        orig.opt_out_signal = OpticalOutputSignal::Spdif;
        let raw = Ff800Protocol::serialize_offsets(&orig);
        Ff800Protocol::deserialize_offsets(&mut cfg, &raw);
        assert_eq!(cfg, orig);
        let quads = config_to_quads(&raw);
        assert_eq!(&quads[..], &[0x00000808, 0x000008b8, 0xc000031e]);

        orig.spdif_out.format = SpdifFormat::Professional;
        let raw = Ff800Protocol::serialize_offsets(&orig);
        Ff800Protocol::deserialize_offsets(&mut cfg, &raw);
        assert_eq!(cfg, orig);
        let quads = config_to_quads(&raw);
        assert_eq!(&quads[..], &[0x00000808, 0x000008b8, 0xc000033e]);

        orig.spdif_out.emphasis = true;
        let raw = Ff800Protocol::serialize_offsets(&orig);
        Ff800Protocol::deserialize_offsets(&mut cfg, &raw);
        assert_eq!(cfg, orig);
        let quads = config_to_quads(&raw);
        assert_eq!(&quads[..], &[0x00000808, 0x000008b8, 0xc000037e]);

        orig.spdif_out.non_audio = true;
        let raw = Ff800Protocol::serialize_offsets(&orig);
        Ff800Protocol::deserialize_offsets(&mut cfg, &raw);
        assert_eq!(cfg, orig);
        let quads = config_to_quads(&raw);
        assert_eq!(&quads[..], &[0x00000808, 0x000008b8, 0xc00003fe]);

        orig.word_out_single = true;
        let raw = Ff800Protocol::serialize_offsets(&orig);
        Ff800Protocol::deserialize_offsets(&mut cfg, &raw);
        assert_eq!(cfg, orig);
        let quads = config_to_quads(&raw);
        assert_eq!(&quads[..], &[0x00000808, 0x000008b8, 0xc00023fe]);
    }
}
