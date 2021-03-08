// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for Fireface 800.
use hinawa::{FwNode, FwReq};

use super::*;
use crate::*;

/// The structure to represent unique protocol for Fireface 800.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Ff800Protocol(FwReq);

impl AsRef<FwReq> for Ff800Protocol {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}


const MIXER_OFFSET: usize       = 0x000080080000;
const OUTPUT_OFFSET: usize      = 0x000080081f80;
const METER_OFFSET: usize       = 0x000080100000;
const STATUS_OFFSET: usize      = 0x0000801c0000;

const ANALOG_INPUT_COUNT: usize = 10;
const SPDIF_INPUT_COUNT: usize = 2;
const ADAT_INPUT_COUNT: usize = 16;
const STREAM_INPUT_COUNT: usize = 28;

const ANALOG_OUTPUT_COUNT: usize = 10;
const SPDIF_OUTPUT_COUNT: usize = 2;
const ADAT_OUTPUT_COUNT: usize = 16;

/// The structure to represent state of hardware meter for Fireface 400.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ff800MeterState(FormerMeterState);

impl AsRef<FormerMeterState> for Ff800MeterState {
    fn as_ref(&self) -> &FormerMeterState {
        &self.0
    }
}

impl AsMut<FormerMeterState> for Ff800MeterState {
    fn as_mut(&mut self) -> &mut FormerMeterState {
        &mut self.0
    }
}

impl FormerMeterSpec for Ff800MeterState {
    const ANALOG_INPUT_COUNT: usize = ANALOG_INPUT_COUNT;
    const SPDIF_INPUT_COUNT: usize = SPDIF_INPUT_COUNT;
    const ADAT_INPUT_COUNT: usize = ADAT_INPUT_COUNT;
    const STREAM_INPUT_COUNT: usize = STREAM_INPUT_COUNT;

    const ANALOG_OUTPUT_COUNT: usize = ANALOG_OUTPUT_COUNT;
    const SPDIF_OUTPUT_COUNT: usize = SPDIF_OUTPUT_COUNT;
    const ADAT_OUTPUT_COUNT: usize = ADAT_OUTPUT_COUNT;
}

impl Default for Ff800MeterState {
    fn default() -> Self {
        Self(Self::create_meter_state())
    }
}

impl<T, O> RmeFfFormerMeterProtocol<T, Ff800MeterState> for O
    where T: AsRef<FwNode>,
          O: AsRef<FwReq>,
{
    const METER_OFFSET: usize = METER_OFFSET;
}

/// The structure to represent volume of outputs for Fireface 800.
///
/// The value for volume is between 0x00000000 and 0x00010000 through 0x00000001 and 0x00080000 to
/// represent the range from negative infinite to 6.00 dB through -90.30 dB and 0.00 dB.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Ff800OutputVolumeState([i32;ANALOG_OUTPUT_COUNT + SPDIF_OUTPUT_COUNT + ADAT_OUTPUT_COUNT]);

impl AsRef<[i32]> for Ff800OutputVolumeState {
    fn as_ref(&self) -> &[i32] {
        &self.0
    }
}

impl AsMut<[i32]> for Ff800OutputVolumeState {
    fn as_mut(&mut self) -> &mut [i32] {
        &mut self.0
    }
}

impl<T: AsRef<FwNode>> RmeFormerOutputProtocol<T, Ff800OutputVolumeState> for Ff800Protocol {
    fn write_output_vol(&self, node: &T, ch: usize, vol: i32, timeout_ms: u32) -> Result<(), Error> {
        let mut raw = [0;4];
        raw.copy_from_slice(&vol.to_le_bytes());
        self.as_ref().transaction_sync(node.as_ref(), FwTcode::WriteBlockRequest,
                                       (OUTPUT_OFFSET + ch * 4) as u64, raw.len(), &mut raw,
                                       timeout_ms)
    }
}

/// The structure to represent state of mixer for RME Fireface 800.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ff800MixerState(pub Vec<FormerMixerSrc>);

impl AsRef<[FormerMixerSrc]> for Ff800MixerState {
    fn as_ref(&self) -> &[FormerMixerSrc] {
        &self.0
    }
}

impl AsMut<[FormerMixerSrc]> for Ff800MixerState {
    fn as_mut(&mut self) -> &mut [FormerMixerSrc] {
        &mut self.0
    }
}

impl RmeFormerMixerSpec for Ff800MixerState {
    const ANALOG_INPUT_COUNT: usize = ANALOG_INPUT_COUNT;
    const SPDIF_INPUT_COUNT: usize = SPDIF_INPUT_COUNT;
    const ADAT_INPUT_COUNT: usize = ADAT_INPUT_COUNT;
    const STREAM_INPUT_COUNT: usize = STREAM_INPUT_COUNT;

    const ANALOG_OUTPUT_COUNT: usize = ANALOG_OUTPUT_COUNT;
    const SPDIF_OUTPUT_COUNT: usize = SPDIF_OUTPUT_COUNT;
    const ADAT_OUTPUT_COUNT: usize = ADAT_OUTPUT_COUNT;
}

impl Default for Ff800MixerState {
    fn default() -> Self {
        Self(Self::create_mixer_state())
    }
}

impl<T, U> RmeFormerMixerProtocol<T, U> for Ff800Protocol
    where T: AsRef<FwNode>,
          U: RmeFormerMixerSpec + AsRef<[FormerMixerSrc]> + AsMut<[FormerMixerSrc]>,
{
    const MIXER_OFFSET: usize = MIXER_OFFSET;
    const AVAIL_COUNT: usize = 32;
}

/// The enumeration to represent source of sampling clock.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Ff800ClkSrc{
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
const Q0_SYNC_WORD_CLOCK_MASK: u32          = 0x40000000;
const Q0_LOCK_WORD_CLOCK_MASK: u32          = 0x20000000;
const Q0_EXT_CLK_RATE_MASK: u32             = 0x1e000000;
const  Q0_EXT_CLK_RATE_192000_FLAGS: u32    = 0x12000000;
const  Q0_EXT_CLK_RATE_176400_FLAGS: u32    = 0x10000000;
const  Q0_EXT_CLK_RATE_128000_FLAGS: u32    = 0x0c000000;
const  Q0_EXT_CLK_RATE_96000_FLAGS: u32     = 0x0e000000;
const  Q0_EXT_CLK_RATE_88200_FLAGS: u32     = 0x0a000000;
const  Q0_EXT_CLK_RATE_64000_FLAGS: u32     = 0x08000000;
const  Q0_EXT_CLK_RATE_48000_FLAGS: u32     = 0x06000000;
const  Q0_EXT_CLK_RATE_44100_FLAGS: u32     = 0x04000000;
const  Q0_EXT_CLK_RATE_32000_FLAGS: u32     = 0x02000000;
const Q0_ACTIVE_CLK_SRC_MASK: u32           = 0x01c00000;
const  Q0_ACTIVE_CLK_SRC_INTERNAL_FLAGS: u32= 0x01c00000;
const  Q0_ACTIVE_CLK_SRC_TCO_FLAGS: u32     = 0x01800000;
const  Q0_ACTIVE_CLK_SRC_WORD_CLK_FLAGS: u32= 0x01000000;
const  Q0_ACTIVE_CLK_SRC_SPDIF_FLAGS: u32   = 0x00c00000;
const  Q0_ACTIVE_CLK_SRC_ADAT_B_FLAGS: u32  = 0x00400000;
const  Q0_ACTIVE_CLK_SRC_ADAT_A_FLAGS: u32  = 0x00000000;
const Q0_SYNC_SPDIF_MASK: u32               = 0x00100000;
const Q0_LOCK_SPDIF_MASK: u32               = 0x00040000;
const Q0_SPDIF_RATE_MASK: u32               = 0x0003c000;
const  Q0_SPDIF_RATE_192000_FLAGS: u32      = 0x00024000;
const  Q0_SPDIF_RATE_176400_FLAGS: u32      = 0x00020000;
const  Q0_SPDIF_RATE_128000_FLAGS: u32      = 0x0001c000;
const  Q0_SPDIF_RATE_96000_FLAGS: u32       = 0x00018000;
const  Q0_SPDIF_RATE_88200_FLAGS: u32       = 0x00014000;
const  Q0_SPDIF_RATE_64000_FLAGS: u32       = 0x00010000;
const  Q0_SPDIF_RATE_48000_FLAGS: u32       = 0x0000c000;
const  Q0_SPDIF_RATE_44100_FLAGS: u32       = 0x00008000;
const  Q0_SPDIF_RATE_32000_FLAGS: u32       = 0x00004000;
const Q0_LOCK_ADAT_B_MASK: u32              = 0x00002000;
const Q0_LOCK_ADAT_A_MASK: u32              = 0x00001000;
const Q0_SYNC_ADAT_B_MASK: u32              = 0x00000800;
const Q0_SYNC_ADAT_A_MASK: u32              = 0x00000400;

// NOTE: for second quadlet of status quadlets.
const Q1_SYNC_TCO_MASK: u32                 = 0x00800000;
const Q1_LOCK_TCO_MASK: u32                 = 0x00400000;
const Q1_WORD_OUT_SINGLE_MASK: u32          = 0x00002000;
const Q1_CONF_CLK_SRC_MASK: u32             = 0x00001c01;
const  Q1_CONF_CLK_SRC_TCO_FLAGS: u32       = 0x00001800;
const  Q1_CONF_CLK_SRC_WORD_CLK_FLAGS: u32  = 0x00001000;
const  Q1_CONF_CLK_SRC_SPDIF_FLAGS: u32     = 0x00000c00;
const  Q1_CONF_CLK_SRC_ADAT_B_FLAGS: u32    = 0x00000400;
const  Q1_CONF_CLK_SRC_INTERNAL_FLAGS: u32  = 0x00000001;
const  Q1_CONF_CLK_SRC_ADAT_A_FLAGS: u32    = 0x00000000;
const Q1_SPDIF_IN_IFACE_MASK: u32           = 0x00000200;
const Q1_OPT_OUT_SIGNAL_MASK: u32           = 0x00000100;
const Q1_SPDIF_OUT_EMPHASIS_MASK: u32       = 0x00000040;
const Q1_SPDIF_OUT_FMT_MASK: u32            = 0x00000020;
const Q1_CONF_CLK_RATE_MASK: u32            = 0x0000001e;
const  Q1_CONF_CLK_RATE_192000_FLAGS: u32   = 0x00000016;
const  Q1_CONF_CLK_RATE_176400_FLAGS: u32   = 0x00000010;
const  Q1_CONF_CLK_RATE_128000_FLAGS: u32   = 0x00000012;
const  Q1_CONF_CLK_RATE_96000_FLAGS: u32    = 0x0000000e;
const  Q1_CONF_CLK_RATE_88200_FLAGS: u32    = 0x00000008;
const  Q1_CONF_CLK_RATE_64000_FLAGS: u32    = 0x0000000a;
const  Q1_CONF_CLK_RATE_48000_FLAGS: u32    = 0x00000006;
const  Q1_CONF_CLK_RATE_44100_FLAGS: u32    = 0x00000000;
const  Q1_CONF_CLK_RATE_32000_FLAGS: u32    = 0x00000002;

/// The structure to represent status of clock locking.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Ff800ClkLockStatus {
    pub adat_a: bool,
    pub adat_b: bool,
    pub spdif: bool,
    pub word_clock: bool,
    pub tco: bool,
}

impl Ff800ClkLockStatus {
    fn parse(&mut self, quads: &[u32]) {
        self.adat_a = quads[0] & Q0_LOCK_ADAT_A_MASK > 0;
        self.adat_b = quads[0] & Q0_LOCK_ADAT_B_MASK > 0;
        self.spdif = quads[0] & Q0_LOCK_SPDIF_MASK > 0;
        self.word_clock = quads[0] & Q0_LOCK_WORD_CLOCK_MASK > 0;
        self.tco = quads[1] & Q1_LOCK_TCO_MASK > 0;

    }
}

/// The structure to represent status of clock synchronization.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Ff800ClkSyncStatus {
    pub adat_a: bool,
    pub adat_b: bool,
    pub spdif: bool,
    pub word_clock: bool,
    pub tco: bool,
}

impl Ff800ClkSyncStatus {
    fn parse(&mut self, quads: &[u32]) {
        self.adat_a = quads[0] & Q0_SYNC_ADAT_A_MASK > 0;
        self.adat_b = quads[0] & Q0_SYNC_ADAT_B_MASK > 0;
        self.spdif = quads[0] & Q0_SYNC_SPDIF_MASK > 0;
        self.word_clock = quads[0] & Q0_SYNC_WORD_CLOCK_MASK > 0;
        self.tco = quads[1] & Q1_SYNC_TCO_MASK > 0;
    }
}

/// The structure to represent status of clock synchronization.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
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
    const QUADLET_COUNT: usize = 2;

    fn parse(&mut self, quads: &[u32]) {
        assert_eq!(quads.len(), Self::QUADLET_COUNT);

        self.lock.parse(&quads);
        self.sync.parse(&quads);

        self.spdif_rate = match quads[0] & Q0_SPDIF_RATE_MASK {
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

        self.active_clk_src = match quads[0] & Q0_ACTIVE_CLK_SRC_MASK {
            Q0_ACTIVE_CLK_SRC_ADAT_A_FLAGS => Ff800ClkSrc::AdatA,
            Q0_ACTIVE_CLK_SRC_ADAT_B_FLAGS => Ff800ClkSrc::AdatB,
            Q0_ACTIVE_CLK_SRC_SPDIF_FLAGS => Ff800ClkSrc::Spdif,
            Q0_ACTIVE_CLK_SRC_WORD_CLK_FLAGS => Ff800ClkSrc::WordClock,
            Q0_ACTIVE_CLK_SRC_TCO_FLAGS => Ff800ClkSrc::Tco,
            Q0_ACTIVE_CLK_SRC_INTERNAL_FLAGS => Ff800ClkSrc::Internal,
            _ => unreachable!(),
        };

        self.external_clk_rate = match quads[0] & Q0_EXT_CLK_RATE_MASK {
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

        self.spdif_in.iface = if quads[1] & Q1_SPDIF_IN_IFACE_MASK > 0 {
            SpdifIface::Optical
        } else {
            SpdifIface::Coaxial
        };

        self.spdif_out.format = if quads[1] & Q1_SPDIF_OUT_FMT_MASK > 0 {
            SpdifFormat::Professional
        } else {
            SpdifFormat::Consumer
        };

        self.spdif_out.emphasis = quads[1] & Q1_SPDIF_OUT_EMPHASIS_MASK > 0;

        self.opt_out_signal = if quads[1] & Q1_OPT_OUT_SIGNAL_MASK > 0 {
            OpticalOutputSignal::Spdif
        } else {
            OpticalOutputSignal::Adat
        };

        self.word_out_single = quads[1] & Q1_WORD_OUT_SINGLE_MASK > 0;

        self.configured_clk_src = match quads[1] & Q1_CONF_CLK_SRC_MASK {
            Q1_CONF_CLK_SRC_INTERNAL_FLAGS => Ff800ClkSrc::Internal,
            Q1_CONF_CLK_SRC_ADAT_B_FLAGS => Ff800ClkSrc::AdatB,
            Q1_CONF_CLK_SRC_SPDIF_FLAGS => Ff800ClkSrc::Spdif,
            Q1_CONF_CLK_SRC_WORD_CLK_FLAGS => Ff800ClkSrc::WordClock,
            Q1_CONF_CLK_SRC_TCO_FLAGS => Ff800ClkSrc::Tco,
            Q1_CONF_CLK_SRC_ADAT_A_FLAGS | _ => Ff800ClkSrc::AdatA,
        };

        self.configured_clk_rate = match quads[1] & Q1_CONF_CLK_RATE_MASK {
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

/// The trait to represent status protocol specific to RME Fireface 800.
pub trait RmeFf800StatusProtocol<T: AsRef<FwNode>> : AsRef<FwReq> {
    fn read_status(&self, node: &T, status: &mut Ff800Status, timeout_ms: u32) -> Result<(), Error> {
        let mut raw = [0;8];
        self.as_ref().transaction_sync(node.as_ref(), FwTcode::ReadBlockRequest, STATUS_OFFSET as u64,
                                       raw.len(), &mut raw, timeout_ms)
            .map(|_| {
                let mut quadlet = [0;4];
                let mut quads = [0u32;2];
                quads.iter_mut()
                    .enumerate()
                    .for_each(|(i, quad)| {
                        let pos = i * 4;
                        quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                        *quad = u32::from_le_bytes(quadlet);
                    });
                status.parse(&quads)
            })
    }
}

impl<T: AsRef<FwNode>> RmeFf800StatusProtocol<T> for Ff800Protocol {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_status() {
        let mut status = Ff800Status::default();

        let quads = [0x02001000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.lock.adat_a, true);

        let quads = [0x02002000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.lock.adat_b, true);

        let quads = [0x02040000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.lock.spdif, true);

        let quads = [0x22000000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.lock.word_clock, true);

        let quads = [0x02000400, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.sync.adat_a, true);

        let quads = [0x02000800, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.sync.adat_b, true);

        let quads = [0x02100800, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.sync.spdif, true);

        let quads = [0x42000000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.sync.word_clock, true);

        let quads = [0x02000000, 0x00800000];
        status.parse(&quads);
        assert_eq!(status.sync.tco, true);

        let quads = [0x02004000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.spdif_rate, Some(ClkNominalRate::R32000));

        let quads = [0x02008000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.spdif_rate, Some(ClkNominalRate::R44100));

        let quads = [0x0200c000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.spdif_rate, Some(ClkNominalRate::R48000));

        let quads = [0x02010000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.spdif_rate, Some(ClkNominalRate::R64000));

        let quads = [0x02014000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.spdif_rate, Some(ClkNominalRate::R88200));

        let quads = [0x02018000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.spdif_rate, Some(ClkNominalRate::R96000));

        let quads = [0x0201c000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.spdif_rate, Some(ClkNominalRate::R128000));

        let quads = [0x02020000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.spdif_rate, Some(ClkNominalRate::R176400));

        let quads = [0x02024000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.spdif_rate, Some(ClkNominalRate::R192000));

        let quads = [0x02000000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.active_clk_src, Ff800ClkSrc::AdatA);

        let quads = [0x02400000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.active_clk_src, Ff800ClkSrc::AdatB);

        let quads = [0x02c00000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.active_clk_src, Ff800ClkSrc::Spdif);

        let quads = [0x03000000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.active_clk_src, Ff800ClkSrc::WordClock);

        let quads = [0x03800000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.active_clk_src, Ff800ClkSrc::Tco);

        let quads = [0x03c00000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.active_clk_src, Ff800ClkSrc::Internal);

        let quads = [0x02000000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.external_clk_rate, Some(ClkNominalRate::R32000));

        let quads = [0x04000000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.external_clk_rate, Some(ClkNominalRate::R44100));

        let quads = [0x06000000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.external_clk_rate, Some(ClkNominalRate::R48000));

        let quads = [0x08000000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.external_clk_rate, Some(ClkNominalRate::R64000));

        let quads = [0x0a000000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.external_clk_rate, Some(ClkNominalRate::R88200));

        let quads = [0x0e000000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.external_clk_rate, Some(ClkNominalRate::R96000));

        let quads = [0x0c000000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.external_clk_rate, Some(ClkNominalRate::R128000));

        let quads = [0x10000000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.external_clk_rate, Some(ClkNominalRate::R176400));

        let quads = [0x12000000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.external_clk_rate, Some(ClkNominalRate::R192000));

        let quads = [0x02000000, 0x00400000];
        status.parse(&quads);
        assert_eq!(status.lock.tco, true);

        let quads = [0x02000000, 0x00800000];
        status.parse(&quads);
        assert_eq!(status.sync.tco, true);

        let quads = [0x02000000, 0x00002000];
        status.parse(&quads);
        assert_eq!(status.word_out_single, true);

        let quads = [0x02000000, 0x00000200];
        status.parse(&quads);
        assert_eq!(status.spdif_in.iface, SpdifIface::Optical);

        let quads = [0x02000000, 0x00000100];
        status.parse(&quads);
        assert_eq!(status.opt_out_signal, OpticalOutputSignal::Spdif);

        let quads = [0x02000000, 0x00000040];
        status.parse(&quads);
        assert_eq!(status.spdif_out.emphasis, true);

        let quads = [0x02000000, 0x00000020];
        status.parse(&quads);
        assert_eq!(status.spdif_out.format, SpdifFormat::Professional);

        let quads = [0x02000000, 0x00000002];
        status.parse(&quads);
        assert_eq!(status.configured_clk_rate, ClkNominalRate::R32000);

        let quads = [0x02000000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.configured_clk_rate, ClkNominalRate::R44100);

        let quads = [0x02000000, 0x00000006];
        status.parse(&quads);
        assert_eq!(status.configured_clk_rate, ClkNominalRate::R48000);

        let quads = [0x02000000, 0x0000000a];
        status.parse(&quads);
        assert_eq!(status.configured_clk_rate, ClkNominalRate::R64000);

        let quads = [0x02000000, 0x00000008];
        status.parse(&quads);
        assert_eq!(status.configured_clk_rate, ClkNominalRate::R88200);

        let quads = [0x02000000, 0x0000000e];
        status.parse(&quads);
        assert_eq!(status.configured_clk_rate, ClkNominalRate::R96000);

        let quads = [0x02000000, 0x00000012];
        status.parse(&quads);
        assert_eq!(status.configured_clk_rate, ClkNominalRate::R128000);

        let quads = [0x02000000, 0x00000010];
        status.parse(&quads);
        assert_eq!(status.configured_clk_rate, ClkNominalRate::R176400);

        let quads = [0x02000000, 0x00000016];
        status.parse(&quads);
        assert_eq!(status.configured_clk_rate, ClkNominalRate::R192000);

        let quads = [0x02000000, 0x00001000];
        status.parse(&quads);
        assert_eq!(status.configured_clk_src, Ff800ClkSrc::WordClock);

        let quads = [0x02000000, 0x00001800];
        status.parse(&quads);
        assert_eq!(status.configured_clk_src, Ff800ClkSrc::Tco);

        let quads = [0x02000000, 0x00000c00];
        status.parse(&quads);
        assert_eq!(status.configured_clk_src, Ff800ClkSrc::Spdif);

        let quads = [0x02000000, 0x00000400];
        status.parse(&quads);
        assert_eq!(status.configured_clk_src, Ff800ClkSrc::AdatB);

        let quads = [0x02000000, 0x00000001];
        status.parse(&quads);
        assert_eq!(status.configured_clk_src, Ff800ClkSrc::Internal);

        let quads = [0x02000000, 0x00000000];
        status.parse(&quads);
        assert_eq!(status.configured_clk_src, Ff800ClkSrc::AdatA);
    }
}
