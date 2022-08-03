// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for M-Audio FireWire 1814 and ProjectMix I/O.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by M-Audio FireWire 1814 and ProjectMix I/O. The configuration for these models
//! is write-only, thus the implementaion includes caching mechanism for the configuration.
//!
//! DM1000 is used for M-Audio FireWire 1814.
//!
//! ## Diagram of internal signal flow for FireWire 1814 and ProjectMix I/O.
//!
//! ```text
//! analog-input-1/2 ---+-------------------------+--------------------------> stream-output-1/2
//! analog-input-3/4 ---|-+-----------------------|-+------------------------> stream-output-3/4
//! analog-input-5/6 ---|-|-+---------------------|-|-+----------------------> stream-output-5/6
//! analog-input-7/8 ---|-|-|-+-------------------|-|-|-+-----------------+
//! spdif-input-1/2 ----|-|-|-|-+-----------------|-|-|-|-+---------------+--> stream-output-7/8
//! adat-input-1/2 -----|-|-|-|-|-+---------------|-|-|-|-|-+----------------> stream-output-9/10
//! adat-input-3/4 -----|-|-|-|-|-|-+-------------|-|-|-|-|-|-+--------------> stream-output-11/12
//! adat-input-5/6 -----|-|-|-|-|-|-|-+-----------|-|-|-|-|-|-|-+------------> stream-output-13/14
//! adat-input-7/8 -----|-|-|-|-|-|-|-|-+---------|-|-|-|-|-|-|-|-+----------> stream-output-15/16
//!                     | | | | | | | | |         | | | | | | | | |
//!                     | | | | | | | | |         v v v v v v v v v
//!                     | | | | | | | | |       ++=================++
//!  stream-input-1/2 --|-|-|-|-|-|-|-|-|-+---> ||      22x2       ||
//!  stream-input-3/4 --|-|-|-|-|-|-|-|-|-|-+-> ||    aux mixer    || --+
//!                     | | | | | | | | | | |   ++=================++   |
//!                     | | | | | | | | | | |                           |
//!                     v v v v v v v v v v v                     aux-output-1/2
//!                   ++=====================++                       | | |
//!                   ||        22x4         || -- mixer-output-1/2 --+-|-|--> analog-output-1/2
//!                   ||        mixer        || -- mixer-output-3/4 --|-+-|--> analog-output-1/2
//!                   ++=====================++                       +-+-+--> headphone-1/2
//!
//!  stream-input-5/7 -------------------------------------------------------> digital-output-1/2
//!  stream-input-7/8 -------------------------------------------------------> digital-output-3/4
//!  stream-input-9/10 ------------------------------------------------------> digital-output-5/6
//!  stream-input-11/12 -----------------------------------------------------> digital-output-7/8
//! ```
//!
//! The protocol implementation for M-Audio FireWire 1814 was written with firmware version
//! below:
//!
//! ```sh
//! $ cargo run --bin bco-bootloader-info -- /dev/fw1
//! protocol:
//!   version: 1
//! bootloader:
//!   timestamp: 2004-03-30T02:59:09+0000
//!   version: 0.0.0
//! hardware:
//!   GUID: 0x007feef8000d6c04
//!   model ID: 0x000083
//!   revision: 0.0.1
//! software:
//!   timestamp: 2007-07-13T08:04:40+0000
//!   ID: 0x00000000
//!   revision: 0.0.0
//! image:
//!   base address: 0x20080000
//!   maximum size: 0x180000
//! ```

use super::*;

/// The protocol implementation for media clock of FireWire 1814.
#[derive(Default)]
pub struct Fw1814ClkProtocol;

impl MediaClockFrequencyOperation for Fw1814ClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000, 176400, 192000];

    fn read_clk_freq(avc: &BebobAvc, timeout_ms: u32) -> Result<usize, Error> {
        read_clk_freq(avc, Self::FREQ_LIST, timeout_ms)
    }
}

/// The protocol implementation for media clock of ProjectMix I/O.
#[derive(Default)]
pub struct ProjectMixClkProtocol;

impl MediaClockFrequencyOperation for ProjectMixClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];

    fn read_clk_freq(avc: &BebobAvc, timeout_ms: u32) -> Result<usize, Error> {
        read_clk_freq(avc, Self::FREQ_LIST, timeout_ms)
    }
}

// NOTE: Special models doesn't support any bridgeco extension.
fn read_clk_freq(avc: &BebobAvc, freq_list: &[u32], timeout_ms: u32) -> Result<usize, Error> {
    let mut op = OutputPlugSignalFormat::new(0);
    avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
    let fdf = AmdtpFdf::from(&op.0.fdf[..]);
    freq_list
        .iter()
        .position(|&freq| freq == fdf.freq)
        .ok_or_else(|| {
            let msg = format!("Unexpected value of FDF: {:?}", fdf);
            Error::new(FileError::Io, &msg)
        })
}

/// AV/C vendor-dependent command for specific LED switch.
pub struct MaudioSpecialLedSwitch {
    state: bool,
    op: VendorDependent,
}

// NOTE: Unknown OUI.
const SPECIAL_OUI_A: [u8; 3] = [0x03, 0x00, 0x01];

impl Default for MaudioSpecialLedSwitch {
    fn default() -> Self {
        Self {
            state: Default::default(),
            op: VendorDependent {
                company_id: SPECIAL_OUI_A,
                data: vec![0xff, 0xff],
            },
        }
    }
}

impl MaudioSpecialLedSwitch {
    pub fn new(state: bool) -> Self {
        Self {
            state,
            ..Default::default()
        }
    }
}

impl AvcOp for MaudioSpecialLedSwitch {
    const OPCODE: u8 = VendorDependent::OPCODE;
}

impl AvcControl for MaudioSpecialLedSwitch {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        self.op.data[0] = self.state.into();
        AvcControl::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AvcControl::parse_operands(&mut self.op, addr, operands)
    }
}

/// The protocol implementation of meter information.
#[derive(Default)]
pub struct MaudioSpecialMeterProtocol;

const METER_SIZE: usize = 84;

/// Information of hardware metering.
#[derive(Debug)]
pub struct MaudioSpecialMeterState {
    pub analog_inputs: [i16; 8],
    pub spdif_inputs: [i16; 2],
    pub adat_inputs: [i16; 8],
    pub analog_outputs: [i16; 4],
    pub spdif_outputs: [i16; 2],
    pub adat_outputs: [i16; 8],
    pub headphone: [i16; 4],
    pub aux_outputs: [i16; 2],
    pub switch: bool,
    pub rotaries: [i16; 3],
    pub sync_status: bool,
    cache: [u8; METER_SIZE],
}

impl Default for MaudioSpecialMeterState {
    fn default() -> Self {
        Self {
            analog_inputs: Default::default(),
            spdif_inputs: Default::default(),
            adat_inputs: Default::default(),
            analog_outputs: Default::default(),
            spdif_outputs: Default::default(),
            adat_outputs: Default::default(),
            headphone: Default::default(),
            aux_outputs: Default::default(),
            switch: Default::default(),
            rotaries: Default::default(),
            sync_status: Default::default(),
            cache: [0; METER_SIZE],
        }
    }
}

impl MaudioSpecialMeterProtocol {
    pub const LEVEL_MIN: i16 = 0;
    pub const LEVEL_MAX: i16 = i16::MAX;
    pub const LEVEL_STEP: i16 = 0x100;

    pub const ROTARY_MIN: i16 = i16::MIN;
    pub const ROTARY_MAX: i16 = 0;
    pub const ROTARY_STEP: i16 = 0x400;

    pub fn read_state(
        req: &FwReq,
        node: &FwNode,
        meter: &mut MaudioSpecialMeterState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let frame = &mut meter.cache;

        let mut bitmap0 = [0; 4];
        bitmap0.copy_from_slice(&frame[..4]);

        let mut bitmap1 = [0; 4];
        bitmap1.copy_from_slice(&frame[(METER_SIZE - 4)..]);

        req.transaction_sync(
            node,
            FwTcode::ReadBlockRequest,
            DM_APPL_METER_OFFSET,
            frame.len(),
            frame,
            timeout_ms,
        )?;

        let mut doublet = [0; 2];

        meter
            .analog_inputs
            .iter_mut()
            .chain(&mut meter.spdif_inputs)
            .chain(&mut meter.adat_inputs)
            .chain(&mut meter.analog_outputs)
            .chain(&mut meter.spdif_outputs)
            .chain(&mut meter.adat_outputs)
            .chain(&mut meter.headphone)
            .chain(&mut meter.aux_outputs)
            .enumerate()
            .for_each(|(i, m)| {
                let pos = 2 + (1 + i) * 2;
                doublet.copy_from_slice(&frame[pos..(pos + 2)]);
                *m = i16::from_be_bytes(doublet);
            });

        if bitmap0[0] ^ frame[0] > 0 {
            if frame[0] == 0x01 {
                meter.switch = !meter.switch;
            }
        }

        meter.rotaries.iter_mut().enumerate().for_each(|(i, r)| {
            let pos = i + 1;

            if bitmap0[pos] ^ frame[pos] > 0 {
                if frame[pos] == 0x01 {
                    if *r <= Self::ROTARY_MAX - Self::ROTARY_STEP {
                        *r += Self::ROTARY_STEP;
                    } else {
                        *r = Self::ROTARY_MAX;
                    }
                } else if frame[pos] == 0x02 {
                    if *r >= Self::ROTARY_MIN + Self::ROTARY_STEP {
                        *r -= Self::ROTARY_STEP;
                    } else {
                        *r = Self::ROTARY_MIN;
                    }
                }
            }
        });

        meter.sync_status = bitmap1[3] ^ frame[METER_SIZE - 1] > 0;

        Ok(())
    }
}

const CACHE_SIZE: usize = 160;

// 0x0000 - 0x0008: stream input gains
// 0x0008 - 0x0010: analog output volumes
// 0x0010 - 0x0020: analog input gains
// 0x0020 - 0x0024: spdif input gains
// 0x0024 - 0x0034: adat input gains
// 0x0034 - 0x0038: aux output volumes
// 0x0038 - 0x0040: headphone volumes
// 0x0040 - 0x0050: analog input balances
// 0x0050 - 0x0054: spdif input balances
// 0x0054 - 0x0064: adat input balances
// 0x0064 - 0x006c: aux stream input gains
// 0x006c - 0x007c: aux analog input gains
// 0x007c - 0x0080: aux spdif input gains
// 0x0080 - 0x0090: aux adat input gains
// 0x0090 - 0x0094: analog/spdif/adat sources to mixer
// 0x0094 - 0x0098: stream sources to mixer
// 0x0098 - 0x009c: source of headphone pair
// 0x009c - 0x00a0: source of analog output pair
const STREAM_INPUT_GAIN_POS: usize = 0x0000;
const ANALOG_OUTPUT_VOLUME_POS: usize = 0x0008;
const ANALOG_INPUT_GAIN_POS: usize = 0x0010;
const SPDIF_INPUT_GAIN_POS: usize = 0x0020;
const ADAT_INPUT_GAIN_POS: usize = 0x0024;
const AUX_OUTPUT_VOLUME_POS: usize = 0x0034;
const HEADPHONE_VOLUME_POS: usize = 0x0038;
const ANALOG_INPUT_BALANCE_POS: usize = 0x0040;
const SPDIF_INPUT_BALANCE_POS: usize = 0x0050;
const ADAT_INPUT_BALANCE_POS: usize = 0x0054;
const AUX_STREAM_INPUT_GAIN_POS: usize = 0x0064;
const AUX_ANALOG_INPUT_GAIN_POS: usize = 0x006c;
const AUX_SPDIF_INPUT_GAIN_POS: usize = 0x007c;
const AUX_ADAT_INPUT_GAIN_POS: usize = 0x0080;
const MIXER_PHYS_SOURCE_POS: usize = 0x0090;
const MIXER_STREAM_SOURCE_POS: usize = 0x0094;
const HEADPHONE_PAIR_SOURCE_POS: usize = 0x0098;
const ANALOG_OUTPUT_PAIR_SOURCE_POS: usize = 0x009c;

/// Cache of state.
#[derive(Debug)]
pub struct MaudioSpecialStateCache(pub [u8; CACHE_SIZE]);

impl Default for MaudioSpecialStateCache {
    fn default() -> Self {
        Self([0; CACHE_SIZE])
    }
}

impl MaudioSpecialStateCache {
    pub fn download(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        (0..CACHE_SIZE).step_by(4).try_for_each(|pos| {
            req.transaction_sync(
                node,
                FwTcode::WriteQuadletRequest,
                DM_APPL_PARAM_OFFSET + pos as u64,
                4,
                &mut self.0[pos..(pos + 4)],
                timeout_ms,
            )
        })
    }
}

/// Parameters of input.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MaudioSpecialInputParameters {
    pub stream_gains: [i16; 4],

    pub analog_gains: [i16; 8],
    pub spdif_gains: [i16; 2],
    pub adat_gains: [i16; 8],

    pub analog_balances: [i16; 8],
    pub spdif_balances: [i16; 2],
    pub adat_balances: [i16; 8],
}

impl Default for MaudioSpecialInputParameters {
    fn default() -> Self {
        Self {
            stream_gains: [MaudioSpecialInputProtocol::GAIN_MAX; 4],
            analog_gains: [MaudioSpecialInputProtocol::GAIN_MAX; 8],
            spdif_gains: [MaudioSpecialInputProtocol::GAIN_MAX; 2],
            adat_gains: [MaudioSpecialInputProtocol::GAIN_MAX; 8],
            analog_balances: [
                MaudioSpecialInputProtocol::BALANCE_MIN,
                MaudioSpecialInputProtocol::BALANCE_MAX,
                MaudioSpecialInputProtocol::BALANCE_MIN,
                MaudioSpecialInputProtocol::BALANCE_MAX,
                MaudioSpecialInputProtocol::BALANCE_MIN,
                MaudioSpecialInputProtocol::BALANCE_MAX,
                MaudioSpecialInputProtocol::BALANCE_MIN,
                MaudioSpecialInputProtocol::BALANCE_MAX,
            ],
            spdif_balances: [
                MaudioSpecialInputProtocol::BALANCE_MIN,
                MaudioSpecialInputProtocol::BALANCE_MAX,
            ],
            adat_balances: [
                MaudioSpecialInputProtocol::BALANCE_MIN,
                MaudioSpecialInputProtocol::BALANCE_MAX,
                MaudioSpecialInputProtocol::BALANCE_MIN,
                MaudioSpecialInputProtocol::BALANCE_MAX,
                MaudioSpecialInputProtocol::BALANCE_MIN,
                MaudioSpecialInputProtocol::BALANCE_MAX,
                MaudioSpecialInputProtocol::BALANCE_MIN,
                MaudioSpecialInputProtocol::BALANCE_MAX,
            ],
        }
    }
}

impl MaudioSpecialParameterOperation for MaudioSpecialInputParameters {
    fn write_to_cache(&self, cache: &mut [u8; CACHE_SIZE]) {
        self.stream_gains.iter().enumerate().for_each(|(i, &gain)| {
            let pos = STREAM_INPUT_GAIN_POS + i * 2;
            cache[pos..(pos + 2)].copy_from_slice(&gain.to_be_bytes());
        });

        self.analog_gains.iter().enumerate().for_each(|(i, &gain)| {
            let pos = ANALOG_INPUT_GAIN_POS + i * 2;
            cache[pos..(pos + 2)].copy_from_slice(&gain.to_be_bytes());
        });

        self.spdif_gains.iter().enumerate().for_each(|(i, &gain)| {
            let pos = SPDIF_INPUT_GAIN_POS + i * 2;
            cache[pos..(pos + 2)].copy_from_slice(&gain.to_be_bytes());
        });

        self.adat_gains.iter().enumerate().for_each(|(i, &gain)| {
            let pos = ADAT_INPUT_GAIN_POS + i * 2;
            cache[pos..(pos + 2)].copy_from_slice(&gain.to_be_bytes());
        });

        self.analog_balances
            .iter()
            .enumerate()
            .for_each(|(i, &gain)| {
                let pos = ANALOG_INPUT_BALANCE_POS + i * 2;
                cache[pos..(pos + 2)].copy_from_slice(&gain.to_be_bytes());
            });

        self.spdif_balances
            .iter()
            .enumerate()
            .for_each(|(i, &gain)| {
                let pos = SPDIF_INPUT_BALANCE_POS + i * 2;
                cache[pos..(pos + 2)].copy_from_slice(&gain.to_be_bytes());
            });

        self.adat_balances
            .iter()
            .enumerate()
            .for_each(|(i, &gain)| {
                let pos = ADAT_INPUT_BALANCE_POS + i * 2;
                cache[pos..(pos + 2)].copy_from_slice(&gain.to_be_bytes());
            });
    }
}

/// The protocol implementation to operate analog inputs.
#[derive(Default)]
pub struct MaudioSpecialInputProtocol;

impl MaudioSpecialInputProtocol {
    pub const GAIN_MIN: i16 = i16::MIN;
    pub const GAIN_MAX: i16 = 0;
    pub const GAIN_STEP: i16 = 0x100;

    pub const BALANCE_MIN: i16 = i16::MIN;
    pub const BALANCE_MAX: i16 = i16::MAX;
    pub const BALANCE_STEP: i16 = 0x100;
}

impl MaudioSpecialParameterProtocol<MaudioSpecialInputParameters> for MaudioSpecialInputProtocol {}

/// Source of analog output.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OutputSource {
    MixerOutputPair,
    AuxOutputPair0,
}

impl Default for OutputSource {
    fn default() -> Self {
        Self::MixerOutputPair
    }
}

/// Source of headphone.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HeadphoneSource {
    MixerOutputPair0,
    MixerOutputPair1,
    AuxOutputPair0,
}

impl Default for HeadphoneSource {
    fn default() -> Self {
        Self::AuxOutputPair0
    }
}

/// Parameters of output.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MaudioSpecialOutputParameters {
    pub analog_volumes: [i16; 4],
    pub analog_pair_sources: [OutputSource; 2],
    pub headphone_volumes: [i16; 4],
    pub headphone_pair_sources: [HeadphoneSource; 2],
}

impl Default for MaudioSpecialOutputParameters {
    fn default() -> Self {
        Self {
            analog_volumes: [MaudioSpecialOutputProtocol::VOLUME_MAX; 4],
            analog_pair_sources: [OutputSource::MixerOutputPair; 2],
            headphone_volumes: [MaudioSpecialOutputProtocol::VOLUME_MAX; 4],
            headphone_pair_sources: [
                HeadphoneSource::MixerOutputPair0,
                HeadphoneSource::MixerOutputPair1,
            ],
        }
    }
}

impl MaudioSpecialParameterOperation for MaudioSpecialOutputParameters {
    fn write_to_cache(&self, cache: &mut [u8; CACHE_SIZE]) {
        self.analog_volumes
            .iter()
            .enumerate()
            .for_each(|(i, &vol)| {
                let pos = ANALOG_OUTPUT_VOLUME_POS + i * 2;
                cache[pos..(pos + 2)].copy_from_slice(&vol.to_be_bytes());
            });

        self.headphone_volumes
            .iter()
            .enumerate()
            .for_each(|(i, &vol)| {
                let pos = HEADPHONE_VOLUME_POS + i * 2;
                cache[pos..(pos + 2)].copy_from_slice(&vol.to_be_bytes());
            });

        let mut quadlet = [0; 4];
        let pos = HEADPHONE_PAIR_SOURCE_POS;
        quadlet.copy_from_slice(&cache[pos..(pos + 4)]);
        let mut val = u32::from_be_bytes(quadlet);

        self.headphone_pair_sources
            .iter()
            .enumerate()
            .for_each(|(i, &src)| {
                let shift = i * 16;
                let mask = 0x07;
                let flag = match src {
                    HeadphoneSource::MixerOutputPair0 => 0x01,
                    HeadphoneSource::MixerOutputPair1 => 0x02,
                    HeadphoneSource::AuxOutputPair0 => 0x04,
                };

                val &= !(mask << shift);
                val |= flag << shift;
            });

        cache[pos..(pos + 4)].copy_from_slice(&val.to_be_bytes());

        let pos = ANALOG_OUTPUT_PAIR_SOURCE_POS;
        quadlet.copy_from_slice(&cache[pos..(pos + 4)]);
        let mut val = u32::from_be_bytes(quadlet);

        self.analog_pair_sources
            .iter()
            .enumerate()
            .for_each(|(i, &src)| {
                let shift = i;
                let mask = 0x01;
                let flag = match src {
                    OutputSource::MixerOutputPair => 0x00,
                    OutputSource::AuxOutputPair0 => 0x01,
                };

                val &= !(mask << shift);
                val |= flag << shift;
            });

        cache[pos..(pos + 4)].copy_from_slice(&val.to_be_bytes());
    }
}

/// The protocol implementation for physical output.
#[derive(Default)]
pub struct MaudioSpecialOutputProtocol;

impl MaudioSpecialOutputProtocol {
    pub const VOLUME_MIN: i16 = i16::MIN;
    pub const VOLUME_MAX: i16 = 0;
    pub const VOLUME_STEP: i16 = 0x100;
}

impl MaudioSpecialParameterProtocol<MaudioSpecialOutputParameters> for MaudioSpecialOutputProtocol {}

/// Parameters of aux signal multiplexer.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MaudioSpecialAuxParameters {
    pub output_volumes: [i16; 2],
    pub stream_gains: [i16; 4],
    pub analog_gains: [i16; 8],
    pub spdif_gains: [i16; 2],
    pub adat_gains: [i16; 8],
}

impl Default for MaudioSpecialAuxParameters {
    fn default() -> Self {
        Self {
            output_volumes: [MaudioSpecialAuxProtocol::VOLUME_MAX; 2],
            stream_gains: [0; 4],
            analog_gains: [
                MaudioSpecialAuxProtocol::GAIN_MAX,
                MaudioSpecialAuxProtocol::GAIN_MAX,
                MaudioSpecialAuxProtocol::GAIN_MIN,
                MaudioSpecialAuxProtocol::GAIN_MIN,
                MaudioSpecialAuxProtocol::GAIN_MIN,
                MaudioSpecialAuxProtocol::GAIN_MIN,
                MaudioSpecialAuxProtocol::GAIN_MIN,
                MaudioSpecialAuxProtocol::GAIN_MIN,
            ],
            spdif_gains: [MaudioSpecialAuxProtocol::GAIN_MIN; 2],
            adat_gains: [MaudioSpecialAuxProtocol::GAIN_MIN; 8],
        }
    }
}

impl MaudioSpecialParameterOperation for MaudioSpecialAuxParameters {
    fn write_to_cache(&self, cache: &mut [u8; CACHE_SIZE]) {
        self.output_volumes
            .iter()
            .enumerate()
            .for_each(|(i, &vol)| {
                let pos = AUX_OUTPUT_VOLUME_POS + i * 2;
                cache[pos..(pos + 2)].copy_from_slice(&vol.to_be_bytes());
            });

        self.stream_gains.iter().enumerate().for_each(|(i, &gain)| {
            let pos = AUX_STREAM_INPUT_GAIN_POS + i * 2;
            cache[pos..(pos + 2)].copy_from_slice(&gain.to_be_bytes());
        });

        self.analog_gains.iter().enumerate().for_each(|(i, &gain)| {
            let pos = AUX_ANALOG_INPUT_GAIN_POS + i * 2;
            cache[pos..(pos + 2)].copy_from_slice(&gain.to_be_bytes());
        });

        self.spdif_gains.iter().enumerate().for_each(|(i, &gain)| {
            let pos = AUX_SPDIF_INPUT_GAIN_POS + i * 2;
            cache[pos..(pos + 2)].copy_from_slice(&gain.to_be_bytes());
        });

        self.adat_gains.iter().enumerate().for_each(|(i, &gain)| {
            let pos = AUX_ADAT_INPUT_GAIN_POS + i * 2;
            cache[pos..(pos + 2)].copy_from_slice(&gain.to_be_bytes());
        });
    }
}

/// The protocol implementation to operate input and output of AUX mixer.
#[derive(Default)]
pub struct MaudioSpecialAuxProtocol;

impl MaudioSpecialAuxProtocol {
    pub const GAIN_MIN: i16 = i16::MIN;
    pub const GAIN_MAX: i16 = 0;
    pub const GAIN_STEP: i16 = 0x100;

    pub const VOLUME_MIN: i16 = i16::MIN;
    pub const VOLUME_MAX: i16 = 0;
    pub const VOLUME_STEP: i16 = 0x100;
}

impl MaudioSpecialParameterProtocol<MaudioSpecialAuxParameters> for MaudioSpecialAuxProtocol {}

/// Parameters of signal multiplexer.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MaudioSpecialMixerParameters {
    pub analog_pairs: [[bool; 4]; 2],
    pub spdif_pairs: [bool; 2],
    pub adat_pairs: [[bool; 4]; 2],
    pub stream_pairs: [[bool; 2]; 2],
}

impl Default for MaudioSpecialMixerParameters {
    fn default() -> Self {
        Self {
            analog_pairs: [[false; 4]; 2],
            spdif_pairs: [false; 2],
            adat_pairs: [[false; 4]; 2],
            stream_pairs: [[true, false], [false, true]],
        }
    }
}

impl MaudioSpecialParameterOperation for MaudioSpecialMixerParameters {
    fn write_to_cache(&self, cache: &mut [u8; CACHE_SIZE]) {
        let mut quadlet = [0; 4];

        let pos = MIXER_PHYS_SOURCE_POS;
        quadlet.copy_from_slice(&cache[pos..(pos + 4)]);
        let mut val = u32::from_be_bytes(quadlet);

        self.analog_pairs.iter().enumerate().for_each(|(i, pairs)| {
            pairs.iter().enumerate().for_each(|(j, &enabled)| {
                let flag = 1u32 << (i * 4 + j);
                val &= !flag;
                if enabled {
                    val |= flag;
                }
            });
        });

        self.spdif_pairs
            .iter()
            .enumerate()
            .for_each(|(i, &enabled)| {
                let flag = 1u32 << (16 + i);
                val &= !flag;
                if enabled {
                    val |= flag;
                }
            });

        self.adat_pairs.iter().enumerate().for_each(|(i, pairs)| {
            pairs.iter().enumerate().for_each(|(j, &enabled)| {
                let flag = 1u32 << (8 + i * 4 + j);
                val &= !flag;
                if enabled {
                    val |= flag;
                }
            });
        });

        cache[pos..(pos + 4)].copy_from_slice(&val.to_be_bytes());

        let pos = MIXER_STREAM_SOURCE_POS;
        quadlet.copy_from_slice(&cache[pos..(pos + 4)]);
        let mut val = u32::from_be_bytes(quadlet);

        self.stream_pairs.iter().enumerate().for_each(|(i, pairs)| {
            pairs.iter().enumerate().for_each(|(j, &enabled)| {
                let flag = 1u32 << (i * 2 + j);

                val &= !flag;
                if enabled {
                    val |= flag;
                }
            });
        });

        cache[pos..(pos + 4)].copy_from_slice(&val.to_be_bytes());
    }
}

/// The protocol implementation for input and output of mixer.
#[derive(Default)]
pub struct MaudioSpecialMixerProtocol;

impl MaudioSpecialParameterProtocol<MaudioSpecialMixerParameters> for MaudioSpecialMixerProtocol {}

/// The trait for operation about parameters.
pub trait MaudioSpecialParameterOperation {
    fn write_to_cache(&self, cache: &mut [u8; CACHE_SIZE]);
}

/// The trait for protocol of parameters.
pub trait MaudioSpecialParameterProtocol<T: MaudioSpecialParameterOperation + Copy> {
    fn update_params(
        req: &FwReq,
        node: &FwNode,
        params: &T,
        cache: &mut MaudioSpecialStateCache,
        old: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut new = [0; CACHE_SIZE];
        new.copy_from_slice(&cache.0);
        params.write_to_cache(&mut new);
        (0..CACHE_SIZE).step_by(4).try_for_each(|pos| {
            if new[pos..(pos + 4)] != cache.0[pos..(pos + 4)] {
                req.transaction_sync(
                    node,
                    FwTcode::WriteQuadletRequest,
                    DM_APPL_PARAM_OFFSET + pos as u64,
                    4,
                    &mut new[pos..(pos + 4)],
                    timeout_ms,
                )
                .map(|_| {
                    cache.0[pos..(pos + 4)].copy_from_slice(&new[pos..(pos + 4)]);
                    *old = *params;
                })
            } else {
                Ok(())
            }
        })
    }
}
