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
