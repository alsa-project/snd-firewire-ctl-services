// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by RME GmbH for Fireface 400.

use glib::Error;

use hinawa::{FwNode, FwTcode, FwReq, FwReqExtManual};

use super::*;

/// The structure to represent unique protocol for Fireface 400.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Ff400Protocol(FwReq);

impl AsRef<FwReq> for Ff400Protocol {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

const MIXER_OFFSET: usize       = 0x000080080000;
const OUTPUT_OFFSET: usize      = 0x000080080f80;
const METER_OFFSET: usize       = 0x000080100000;
const STATUS_OFFSET: usize      = 0x0000801c0000;
const AMP_OFFSET: usize         = 0x0000801c0180;

const ANALOG_INPUT_COUNT: usize = 8;
const SPDIF_INPUT_COUNT: usize = 2;
const ADAT_INPUT_COUNT: usize = 8;
const STREAM_INPUT_COUNT: usize = 18;

const ANALOG_OUTPUT_COUNT: usize = 8;
const SPDIF_OUTPUT_COUNT: usize = 2;
const ADAT_OUTPUT_COUNT: usize = 8;

const AMP_MIC_IN_CH_OFFSET: u8 = 0;
const AMP_LINE_IN_CH_OFFSET: u8 = 2;
const AMP_OUT_CH_OFFSET: u8 = 4;

/// The structure to represent state of hardware meter for Fireface 400.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ff400MeterState(FormerMeterState);

impl AsRef<FormerMeterState> for Ff400MeterState {
    fn as_ref(&self) -> &FormerMeterState {
        &self.0
    }
}

impl AsMut<FormerMeterState> for Ff400MeterState {
    fn as_mut(&mut self) -> &mut FormerMeterState {
        &mut self.0
    }
}

impl FormerMeterSpec for Ff400MeterState {
    const ANALOG_INPUT_COUNT: usize = ANALOG_INPUT_COUNT;
    const SPDIF_INPUT_COUNT: usize = SPDIF_INPUT_COUNT;
    const ADAT_INPUT_COUNT: usize = ADAT_INPUT_COUNT;
    const STREAM_INPUT_COUNT: usize = STREAM_INPUT_COUNT;

    const ANALOG_OUTPUT_COUNT: usize = ANALOG_OUTPUT_COUNT;
    const SPDIF_OUTPUT_COUNT: usize = SPDIF_OUTPUT_COUNT;
    const ADAT_OUTPUT_COUNT: usize = ADAT_OUTPUT_COUNT;
}

impl Default for Ff400MeterState {
    fn default() -> Self {
        Self(Self::create_meter_state())
    }
}

impl<T, O> RmeFfFormerMeterProtocol<T, Ff400MeterState> for O
    where T: AsRef<FwNode>,
          O: AsRef<FwReq>,
{
    const METER_OFFSET: usize = METER_OFFSET;
}

/// The trait to represent amplifier protocol of Fireface 400.
pub trait RmeFf400AmpProtocol<T: AsRef<FwNode>> : AsRef<FwReq> {
    fn write_amp_cmd(&self, node: &T, ch: u8, level: i8, timeout_ms: u32) -> Result<(), Error> {
        let cmd = ((ch as u32) << 16) | ((level as u32) & 0xff);
        let mut raw = [0;4];
        raw.copy_from_slice(&cmd.to_le_bytes());
        self.as_ref().transaction_sync(node.as_ref(), FwTcode::WriteQuadletRequest,
                                       AMP_OFFSET as u64, raw.len(), &mut raw, timeout_ms)
    }
}

impl<T: AsRef<FwNode>> RmeFf400AmpProtocol<T> for Ff400Protocol {}

/// The structure to represent status of input gains of Fireface 400.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Ff400InputGainStatus{
    /// The level of gain for input 1 and 2. The value is between 0 and 65 by step 1 to represent
    /// the range from 0 to 65 dB.
    pub mic: [i8;2],
    /// The level of gain for input 3 and 4. The value is between 0 and 36 by step 1 to represent
    /// the range from 0 to 18 dB.
    pub line: [i8;2],
}

/// The trait to represent amplifier protocol of Fireface 400.
pub trait RmeFf400InputGainProtocol<T: AsRef<FwNode>> : RmeFf400AmpProtocol<T> {
    fn write_input_mic_gain(&self, node: &T, ch: usize, gain: i8, timeout_ms: u32)
        -> Result<(), Error>
    {
        self.write_amp_cmd(node, AMP_MIC_IN_CH_OFFSET + ch as u8, gain, timeout_ms)
    }

    fn write_input_line_gain(&self, node: &T, ch: usize, gain: i8, timeout_ms: u32)
        -> Result<(), Error>
    {
        self.write_amp_cmd(node, AMP_LINE_IN_CH_OFFSET + ch as u8, gain, timeout_ms)
    }

    fn init_input_gains(&self, node: &T, status: &Ff400InputGainStatus, timeout_ms: u32)
        -> Result<(), Error>
    {
        status.mic.iter()
            .enumerate()
            .try_for_each(|(i, gain)| self.write_input_mic_gain(node, i, *gain, timeout_ms))?;

        status.line.iter()
            .enumerate()
            .try_for_each(|(i, gain)| self.write_input_line_gain(node, i, *gain, timeout_ms))?;

        Ok(())
    }

    fn write_input_mic_gains(&self, node: &T, status: &mut Ff400InputGainStatus, gains: &[i8],
                             timeout_ms: u32)
        -> Result<(), Error>
    {
        status.mic.iter_mut()
            .zip(gains.iter())
            .enumerate()
            .filter(|(_, (o, n))| !o.eq(n))
            .try_for_each(|(i, (o, n))| {
                self.write_input_mic_gain(node, i, *n, timeout_ms)
                    .map(|_| *o = *n)
            })
    }

    fn write_input_line_gains(&self, node: &T, status: &mut Ff400InputGainStatus, gains: &[i8],
                             timeout_ms: u32)
        -> Result<(), Error>
    {
        status.line.iter_mut()
            .zip(gains.iter())
            .enumerate()
            .filter(|(_, (o, n))| !o.eq(n))
            .try_for_each(|(i, (o, n))| {
                self.write_input_line_gain(node, i, *n, timeout_ms)
                    .map(|_| *o = *n)
            })
    }
}

impl<T: AsRef<FwNode>, O: RmeFf400AmpProtocol<T>> RmeFf400InputGainProtocol<T> for O {}

/// The structure to represent volume of outputs for Fireface 400.
///
/// The value is between 0x00000000, 0x00010000 through 0x00000001 and 0x00008000 by step 1 to
/// represent the range from negative infinite to + 6dB through -90.30 dB and 0 dB.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Ff400OutputVolumeState([i32;ANALOG_OUTPUT_COUNT + SPDIF_OUTPUT_COUNT + ADAT_OUTPUT_COUNT]);

impl AsMut<[i32]> for Ff400OutputVolumeState {
    fn as_mut(&mut self) -> &mut [i32] {
        &mut self.0
    }
}

impl AsRef<[i32]> for Ff400OutputVolumeState {
    fn as_ref(&self) -> &[i32] {
        &self.0
    }
}

impl<T: AsRef<FwNode>> RmeFormerOutputProtocol<T, Ff400OutputVolumeState> for Ff400Protocol {
    fn write_output_vol(&self, node: &T, ch: usize, vol: i32, timeout_ms: u32) -> Result<(), Error> {
        let mut raw = [0;4];
        raw.copy_from_slice(&vol.to_le_bytes());
        self.as_ref().transaction_sync(node.as_ref(), FwTcode::WriteBlockRequest,
                                       (OUTPUT_OFFSET + ch * 4) as u64, raw.len(), &mut raw,
                                       timeout_ms)
            .and_then(|_| {
                // The value for level is between 0x3f to 0x00 by step 1 to represent -57 dB
                // (=mute) to +6 dB.
                let level = (0x3f * (vol as i64) / (0x00010000 as i64)) as i8;
                let amp_offset = AMP_OUT_CH_OFFSET + ch as u8;
                self.write_amp_cmd(node, amp_offset, level, timeout_ms)
            })
    }
}


/// The structure to represent state of mixer for RME Fireface 400.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ff400MixerState(pub Vec<FormerMixerSrc>);

impl AsRef<[FormerMixerSrc]> for Ff400MixerState {
    fn as_ref(&self) -> &[FormerMixerSrc] {
        &self.0
    }
}

impl AsMut<[FormerMixerSrc]> for Ff400MixerState {
    fn as_mut(&mut self) -> &mut [FormerMixerSrc] {
        &mut self.0
    }
}

impl RmeFormerMixerSpec for Ff400MixerState {
    const ANALOG_INPUT_COUNT: usize = ANALOG_INPUT_COUNT;
    const SPDIF_INPUT_COUNT: usize = SPDIF_INPUT_COUNT;
    const ADAT_INPUT_COUNT: usize = ADAT_INPUT_COUNT;
    const STREAM_INPUT_COUNT: usize = STREAM_INPUT_COUNT;

    const ANALOG_OUTPUT_COUNT: usize = ANALOG_OUTPUT_COUNT;
    const SPDIF_OUTPUT_COUNT: usize = SPDIF_OUTPUT_COUNT;
    const ADAT_OUTPUT_COUNT: usize = ADAT_OUTPUT_COUNT;
}

impl Default for Ff400MixerState {
    fn default() -> Self {
        Self(Self::create_mixer_state())
    }
}

impl<T, U> RmeFormerMixerProtocol<T, U> for Ff400Protocol
    where T: AsRef<FwNode>,
          U: RmeFormerMixerSpec + AsRef<[FormerMixerSrc]> + AsMut<[FormerMixerSrc]>,
{
    const MIXER_OFFSET: usize = MIXER_OFFSET as usize;
    const AVAIL_COUNT: usize = 18;
}

/// The enumeration to represent source of sampling clock.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Ff400ClkSrc{
    Internal,
    WordClock,
    Adat,
    Spdif,
    Ltc,
}

impl Default for Ff400ClkSrc {
    fn default() -> Self {
        Self::Adat
    }
}

// NOTE: for first quadlet of status quadlets.
const Q0_SYNC_WORD_CLOCK_MASK: u32          = 0x40000000;
const Q0_LOCK_WORD_CLOCK_MASK: u32          = 0x20000000;
const Q0_EXT_CLK_RATE_MASK: u32             = 0x1e000000;
const  Q0_EXT_CLK_RATE_192000_FLAG: u32     = 0x12000000;
const  Q0_EXT_CLK_RATE_176400_FLAG: u32     = 0x10000000;
const  Q0_EXT_CLK_RATE_128000_FLAG: u32     = 0x0c000000;
const  Q0_EXT_CLK_RATE_96000_FLAG: u32      = 0x0e000000;
const  Q0_EXT_CLK_RATE_88200_FLAG: u32      = 0x0a000000;
const  Q0_EXT_CLK_RATE_64000_FLAG: u32      = 0x08000000;
const  Q0_EXT_CLK_RATE_48000_FLAG: u32      = 0x06000000;
const  Q0_EXT_CLK_RATE_44100_FLAG: u32      = 0x04000000;
const  Q0_EXT_CLK_RATE_32000_FLAG: u32      = 0x02000000;
const Q0_ACTIVE_CLK_SRC_MASK: u32           = 0x01c00000;
const  Q0_ACTIVE_CLK_SRC_INTERNAL_FLAG: u32 = 0x01c00000;
const  Q0_ACTIVE_CLK_SRC_LTC_FLAG: u32      = 0x01400000;
const  Q0_ACTIVE_CLK_SRC_WORD_CLK_FLAG: u32 = 0x01000000;
const  Q0_ACTIVE_CLK_SRC_SPDIF_FLAG: u32    = 0x00c00000;
const  Q0_ACTIVE_CLK_SRC_ADAT_FLAG: u32     = 0x00000000;
const Q0_SYNC_SPDIF_MASK: u32               = 0x00100000;
const Q0_LOCK_SPDIF_MASK: u32               = 0x00040000;
const Q0_SPDIF_RATE_MASK: u32               = 0x0003c000;
const  Q0_SPDIF_RATE_192000_FLAG: u32       = 0x00024000;
const  Q0_SPDIF_RATE_176400_FLAG: u32       = 0x00020000;
const  Q0_SPDIF_RATE_128000_FLAG: u32       = 0x0001c000;
const  Q0_SPDIF_RATE_96000_FLAG: u32        = 0x00018000;
const  Q0_SPDIF_RATE_88200_FLAG: u32        = 0x00014000;
const  Q0_SPDIF_RATE_64000_FLAG: u32        = 0x00010000;
const  Q0_SPDIF_RATE_48000_FLAG: u32        = 0x0000c000;
const  Q0_SPDIF_RATE_44100_FLAG: u32        = 0x00008000;
const  Q0_SPDIF_RATE_32000_FLAG: u32        = 0x00004000;
const Q0_LOCK_ADAT_MASK: u32                = 0x00001000;
const Q0_SYNC_ADAT_MASK: u32                = 0x00000400;

// NOTE: for second quadlet of status quadlets.
const Q1_WORD_OUT_SINGLE_MASK: u32          = 0x00002000;
const Q1_CONF_CLK_SRC_MASK: u32             = 0x00001c01;
const  Q1_CONF_CLK_SRC_LTC_FLAG: u32        = 0x00001400;
const  Q1_CONF_CLK_SRC_WORD_CLK_FLAG: u32   = 0x00001000;
const  Q1_CONF_CLK_SRC_SPDIF_FLAG: u32      = 0x00000c00;
const  Q1_CONF_CLK_SRC_INTERNAL_FLAG: u32   = 0x00000001;
const  Q1_CONF_CLK_SRC_ADAT_FLAG: u32       = 0x00000000;
const Q1_SPDIF_IN_IFACE_MASK: u32           = 0x00000200;
const Q1_OPT_OUT_SIGNAL_MASK: u32           = 0x00000100;
const Q1_SPDIF_OUT_EMPHASIS_MASK: u32       = 0x00000040;
const Q1_SPDIF_OUT_FMT_MASK: u32            = 0x00000020;
const Q1_CONF_CLK_RATE_MASK: u32            = 0x0000001e;
const  Q1_CONF_CLK_RATE_192000_FLAG: u32    = 0x00000016;
const  Q1_CONF_CLK_RATE_176400_FLAG: u32    = 0x00000010;
const  Q1_CONF_CLK_RATE_128000_FLAG: u32    = 0x00000012;
const  Q1_CONF_CLK_RATE_96000_FLAG: u32     = 0x0000000e;
const  Q1_CONF_CLK_RATE_88200_FLAG: u32     = 0x00000008;
const  Q1_CONF_CLK_RATE_64000_FLAG: u32     = 0x0000000a;
const  Q1_CONF_CLK_RATE_48000_FLAG: u32     = 0x00000006;
const  Q1_CONF_CLK_RATE_44100_FLAG: u32     = 0x00000000;
const  Q1_CONF_CLK_RATE_32000_FLAG: u32     = 0x00000002;

/// The structure to represent status of clock locking.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Ff400ClkLockStatus {
    pub adat: bool,
    pub spdif: bool,
    pub word_clock: bool,
}

impl Ff400ClkLockStatus {
    fn parse(&mut self, quads: &[u32]) {
        self.adat = quads[0] & Q0_LOCK_ADAT_MASK > 0;
        self.spdif = quads[0] & Q0_LOCK_SPDIF_MASK > 0;
        self.word_clock = quads[0] & Q0_LOCK_WORD_CLOCK_MASK > 0;

    }
}

/// The structure to represent status of clock synchronization.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Ff400ClkSyncStatus {
    pub adat: bool,
    pub spdif: bool,
    pub word_clock: bool,
}

impl Ff400ClkSyncStatus {
    fn parse(&mut self, quads: &[u32]) {
        self.adat = quads[0] & Q0_SYNC_ADAT_MASK > 0;
        self.spdif = quads[0] & Q0_SYNC_SPDIF_MASK > 0;
        self.word_clock = quads[0] & Q0_SYNC_WORD_CLOCK_MASK > 0;
    }
}

/// The structure to represent status of clock synchronization.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Ff400Status {
    /// For S/PDIF input.
    pub spdif_in: SpdifInput,
    /// For S/PDIF output.
    pub spdif_out: FormerSpdifOutput,
    /// The type of signal to optical output interface.
    pub opt_out_signal: OpticalOutputSignal,
    /// Whether to fix speed to single even if at double/quadruple rate.
    pub word_out_single: bool,
    /// For status of synchronization to external clocks.
    pub sync: Ff400ClkSyncStatus,
    /// For status of locking to external clocks.
    pub lock: Ff400ClkLockStatus,

    pub spdif_rate: Option<ClkNominalRate>,
    pub active_clk_src: Ff400ClkSrc,
    pub external_clk_rate: Option<ClkNominalRate>,
    pub configured_clk_src: Ff400ClkSrc,
    pub configured_clk_rate: ClkNominalRate,
}

impl Ff400Status {
    const QUADLET_COUNT: usize = 2;

    fn parse(&mut self, quads: &[u32]) {
        assert_eq!(quads.len(), Self::QUADLET_COUNT);

        self.lock.parse(&quads);
        self.sync.parse(&quads);

        self.spdif_rate = match quads[0] & Q0_SPDIF_RATE_MASK {
            Q0_SPDIF_RATE_32000_FLAG => Some(ClkNominalRate::R32000),
            Q0_SPDIF_RATE_44100_FLAG => Some(ClkNominalRate::R44100),
            Q0_SPDIF_RATE_48000_FLAG => Some(ClkNominalRate::R48000),
            Q0_SPDIF_RATE_64000_FLAG => Some(ClkNominalRate::R64000),
            Q0_SPDIF_RATE_88200_FLAG => Some(ClkNominalRate::R88200),
            Q0_SPDIF_RATE_96000_FLAG => Some(ClkNominalRate::R96000),
            Q0_SPDIF_RATE_128000_FLAG => Some(ClkNominalRate::R128000),
            Q0_SPDIF_RATE_176400_FLAG => Some(ClkNominalRate::R176400),
            Q0_SPDIF_RATE_192000_FLAG => Some(ClkNominalRate::R192000),
            _ => None,
        };

        self.active_clk_src = match quads[0] & Q0_ACTIVE_CLK_SRC_MASK {
            Q0_ACTIVE_CLK_SRC_ADAT_FLAG => Ff400ClkSrc::Adat,
            Q0_ACTIVE_CLK_SRC_SPDIF_FLAG => Ff400ClkSrc::Spdif,
            Q0_ACTIVE_CLK_SRC_WORD_CLK_FLAG => Ff400ClkSrc::WordClock,
            Q0_ACTIVE_CLK_SRC_LTC_FLAG => Ff400ClkSrc::Ltc,
            Q0_ACTIVE_CLK_SRC_INTERNAL_FLAG => Ff400ClkSrc::Internal,
            _ => unreachable!(),
        };

        self.external_clk_rate = match quads[0] & Q0_EXT_CLK_RATE_MASK {
            Q0_EXT_CLK_RATE_32000_FLAG => Some(ClkNominalRate::R32000),
            Q0_EXT_CLK_RATE_44100_FLAG => Some(ClkNominalRate::R44100),
            Q0_EXT_CLK_RATE_48000_FLAG => Some(ClkNominalRate::R48000),
            Q0_EXT_CLK_RATE_64000_FLAG => Some(ClkNominalRate::R64000),
            Q0_EXT_CLK_RATE_88200_FLAG => Some(ClkNominalRate::R88200),
            Q0_EXT_CLK_RATE_96000_FLAG => Some(ClkNominalRate::R96000),
            Q0_EXT_CLK_RATE_128000_FLAG => Some(ClkNominalRate::R128000),
            Q0_EXT_CLK_RATE_176400_FLAG => Some(ClkNominalRate::R176400),
            Q0_EXT_CLK_RATE_192000_FLAG => Some(ClkNominalRate::R192000),
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
            Q1_CONF_CLK_SRC_INTERNAL_FLAG => Ff400ClkSrc::Internal,
            Q1_CONF_CLK_SRC_SPDIF_FLAG => Ff400ClkSrc::Spdif,
            Q1_CONF_CLK_SRC_WORD_CLK_FLAG => Ff400ClkSrc::WordClock,
            Q1_CONF_CLK_SRC_LTC_FLAG => Ff400ClkSrc::Ltc,
            Q1_CONF_CLK_SRC_ADAT_FLAG | _ => Ff400ClkSrc::Adat,
        };

        self.configured_clk_rate = match quads[1] & Q1_CONF_CLK_RATE_MASK {
            Q1_CONF_CLK_RATE_32000_FLAG => ClkNominalRate::R32000,
            Q1_CONF_CLK_RATE_48000_FLAG => ClkNominalRate::R48000,
            Q1_CONF_CLK_RATE_64000_FLAG => ClkNominalRate::R64000,
            Q1_CONF_CLK_RATE_88200_FLAG => ClkNominalRate::R88200,
            Q1_CONF_CLK_RATE_96000_FLAG => ClkNominalRate::R96000,
            Q1_CONF_CLK_RATE_128000_FLAG => ClkNominalRate::R128000,
            Q1_CONF_CLK_RATE_176400_FLAG => ClkNominalRate::R176400,
            Q1_CONF_CLK_RATE_192000_FLAG => ClkNominalRate::R192000,
            Q1_CONF_CLK_RATE_44100_FLAG | _ => ClkNominalRate::R44100,
        };

    }
}

/// The trait to represent status protocol specific to RME Fireface 800.
pub trait RmeFf400StatusProtocol<T: AsRef<FwNode>> : AsRef<FwReq> {
    fn read_status(&self, node: &T, status: &mut Ff400Status, timeout_ms: u32) -> Result<(), Error> {
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

impl<T: AsRef<FwNode>> RmeFf400StatusProtocol<T> for Ff400Protocol {}
