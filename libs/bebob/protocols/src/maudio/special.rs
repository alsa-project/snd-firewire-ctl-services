// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for M-Audio FireWire 1814 and ProjectMix I/O.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by M-Audio FireWire 1814 and ProjectMix I/O.
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

use hinawa::{FwNode, FwReq, FwReqExtManual, FwTcode};

use crate::*;

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
    let fdf = AmdtpFdf::from(&op.fdf[..]);
    freq_list
        .iter()
        .position(|&freq| freq == fdf.freq)
        .ok_or_else(|| {
            let msg = format!("Unexpected value of FDF: {:?}", fdf);
            Error::new(FileError::Io, &msg)
        })
}

/// The structure of AV/C vendor-dependent command for specific LED switch.
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
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        self.op.data[0] = self.state.into();
        AvcControl::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcControl::parse_operands(&mut self.op, addr, operands)
    }
}

/// The protocol implementation of meter information.
#[derive(Default)]
pub struct MaudioSpecialMeterProtocol;

const METER_SIZE: usize = 84;

/// The structure for meter information.
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

        read_block(req, node, METER_OFFSET, frame, timeout_ms)?;

        let mut doublet = [0; 2];

        meter
            .analog_inputs
            .iter_mut()
            .chain(meter.spdif_inputs.iter_mut())
            .chain(meter.adat_inputs.iter_mut())
            .chain(meter.analog_outputs.iter_mut())
            .chain(meter.spdif_outputs.iter_mut())
            .chain(meter.adat_outputs.iter_mut())
            .chain(meter.headphone.iter_mut())
            .chain(meter.aux_outputs.iter_mut())
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

const PARAM_OFFSET: u64 = 0x00700000;

const CACHE_SIZE: usize = 160;

// 0x0000 - 0x0008: stream input gains
// 0x0010 - 0x0020: analog input gains
// 0x0020 - 0x0024: spdif input gains
// 0x0024 - 0x0034: adat input gains
// 0x0040 - 0x0050: analog input balances
// 0x0050 - 0x0054: spdif input balances
// 0x0054 - 0x0064: adat input balances
const STREAM_INPUT_GAIN_POS: usize = 0x0000;
const ANALOG_INPUT_GAIN_POS: usize = 0x0010;
const SPDIF_INPUT_GAIN_POS: usize = 0x0020;
const ADAT_INPUT_GAIN_POS: usize = 0x0024;
const ANALOG_INPUT_BALANCE_POS: usize = 0x0040;
const SPDIF_INPUT_BALANCE_POS: usize = 0x0050;
const ADAT_INPUT_BALANCE_POS: usize = 0x0054;

/// The structure for input parameters.
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
        self.stream_gains.iter()
            .enumerate()
            .for_each(|(i, &gain)| {
                let pos = STREAM_INPUT_GAIN_POS + i * 2;
                cache[pos..(pos + 2)].copy_from_slice(&gain.to_be_bytes());
            });

        self.analog_gains.iter()
            .enumerate()
            .for_each(|(i, &gain)| {
                let pos = ANALOG_INPUT_GAIN_POS + i * 2;
                cache[pos..(pos + 2)].copy_from_slice(&gain.to_be_bytes());
            });

        self.spdif_gains.iter()
            .enumerate()
            .for_each(|(i, &gain)| {
                let pos = SPDIF_INPUT_GAIN_POS + i * 2;
                cache[pos..(pos + 2)].copy_from_slice(&gain.to_be_bytes());
            });

        self.adat_gains.iter()
            .enumerate()
            .for_each(|(i, &gain)| {
                let pos = ADAT_INPUT_GAIN_POS + i * 2;
                cache[pos..(pos + 2)].copy_from_slice(&gain.to_be_bytes());
            });

        self.analog_balances.iter()
            .enumerate()
            .for_each(|(i, &gain)| {
                let pos = ANALOG_INPUT_BALANCE_POS + i * 2;
                cache[pos..(pos + 2)].copy_from_slice(&gain.to_be_bytes());
            });

        self.spdif_balances.iter()
            .enumerate()
            .for_each(|(i, &gain)| {
                let pos = SPDIF_INPUT_BALANCE_POS + i * 2;
                cache[pos..(pos + 2)].copy_from_slice(&gain.to_be_bytes());
            });

        self.adat_balances.iter()
            .enumerate()
            .for_each(|(i, &gain)| {
                let pos = ADAT_INPUT_BALANCE_POS + i * 2;
                cache[pos..(pos + 2)].copy_from_slice(&gain.to_be_bytes());
            });
    }
}

#[derive(Default)]
pub struct MaudioSpecialInputProtocol;

impl MaudioSpecialInputProtocol{
    pub const GAIN_MIN: i16 = i16::MIN;
    pub const GAIN_MAX: i16 = 0;
    pub const GAIN_STEP: i16 = 0x100;

    pub const BALANCE_MIN: i16 = i16::MIN;
    pub const BALANCE_MAX: i16 = i16::MAX;
    pub const BALANCE_STEP: i16 = 0x100;
}

impl MaudioSpecialParameterProtocol<MaudioSpecialInputParameters> for MaudioSpecialInputProtocol {}

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
        cache: &mut [u8; CACHE_SIZE],
        old: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut new = [0; CACHE_SIZE];
        new.copy_from_slice(cache);
        params.write_to_cache(&mut new);
        (0..CACHE_SIZE).step_by(4)
            .try_for_each(|pos| {
                if new[pos..(pos + 4)] != cache[pos..(pos + 4)] {
                    req.transaction_sync(
                        node,
                        FwTcode::WriteQuadletRequest,
                        BASE_OFFSET + PARAM_OFFSET + pos as u64,
                        4,
                        &mut new[pos..(pos + 4)],
                        timeout_ms,
                    )
                    .map(|_| {
                        cache[pos..(pos + 4)].copy_from_slice(&new[pos..(pos + 4)]);
                        *old = *params;
                    })
                } else {
                    Ok(())
                }
            })
    }
}
