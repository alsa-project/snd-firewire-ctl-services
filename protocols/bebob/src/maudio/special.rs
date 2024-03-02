// SPDX-License-Identifier: LGPL-3.0-or-later
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

use {super::*, std::ops::Range};

/// The protocol implementation for media clock of FireWire 1814.
#[derive(Default, Debug)]
pub struct Fw1814ClkProtocol;

impl MediaClockFrequencyOperation for Fw1814ClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000, 176400, 192000];

    fn cache_freq(
        avc: &BebobAvc,
        params: &mut MediaClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        cache_freq(avc, params, Self::FREQ_LIST, timeout_ms)
    }
}

/// The protocol implementation for media clock of ProjectMix I/O.
#[derive(Default, Debug)]
pub struct ProjectMixClkProtocol;

impl MediaClockFrequencyOperation for ProjectMixClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];

    fn cache_freq(
        avc: &BebobAvc,
        params: &mut MediaClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        cache_freq(avc, params, Self::FREQ_LIST, timeout_ms)
    }
}

// NOTE: Special models doesn't support any bridgeco extension.
fn cache_freq(
    avc: &BebobAvc,
    params: &mut MediaClockParameters,
    freq_list: &[u32],
    timeout_ms: u32,
) -> Result<(), Error> {
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
        .map(|freq_idx| params.freq_idx = freq_idx)
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
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        self.op.data[0] = self.state.into();
        AvcControl::build_operands(&mut self.op, addr)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AvcControl::parse_operands(&mut self.op, addr, operands)
    }
}

/// The protocol implementation for hardware metering.
#[derive(Default, Debug)]
pub struct MaudioSpecialMeterProtocol;

const METER_SIZE: usize = 84;

/// Information of hardware metering.
#[derive(Debug)]
pub struct MaudioSpecialMeterState {
    /// Detected levels for analog inputs.
    pub analog_inputs: [i16; 8],
    /// Detected levels of S/PDIF inputs.
    pub spdif_inputs: [i16; 2],
    /// Detected levels of ADAT inputs.
    pub adat_inputs: [i16; 8],
    /// Detected levels of analog outputs.
    pub analog_outputs: [i16; 4],
    /// Detected levels of S/PDIF outputs.
    pub spdif_outputs: [i16; 2],
    /// Detected levels of ADAT outputs.
    pub adat_outputs: [i16; 8],

    /// Detected levels of headphone outputs.
    pub headphone: [i16; 4],
    /// Detected levels of outputs from auxiliary mixer.
    pub aux_outputs: [i16; 2],
    /// Detected state of hardware switch.
    pub switch: bool,
    /// Detected states of hardware rotary knobs.
    pub rotaries: [i16; 3],
    /// The status of sampling clock synchronization.
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
    /// The minimum value of detected level.
    pub const LEVEL_MIN: i16 = 0;
    /// The maximum value of detected level.
    pub const LEVEL_MAX: i16 = i16::MAX;
    /// The step value of detected level.
    pub const LEVEL_STEP: i16 = 0x100;

    /// The minimum value of hardware rotary.
    pub const ROTARY_MIN: i16 = i16::MIN;
    /// The maximum value of hardware rotary.
    pub const ROTARY_MAX: i16 = 0;
    /// The step value of hardware rotary.
    pub const ROTARY_STEP: i16 = 0x400;

    pub fn cache(
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

        req.transaction(
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
const STREAM_INPUT_GAIN_RANGE: Range<usize> = Range {
    start: 0x0000,
    end: 0x0008,
};
const ANALOG_OUTPUT_VOLUME_RANGE: Range<usize> = Range {
    start: 0x0008,
    end: 0x0010,
};
const ANALOG_INPUT_GAIN_RANGE: Range<usize> = Range {
    start: 0x0010,
    end: 0x0020,
};
const SPDIF_INPUT_GAIN_RANGE: Range<usize> = Range {
    start: 0x0020,
    end: 0x0024,
};
const ADAT_INPUT_GAIN_RANGE: Range<usize> = Range {
    start: 0x0024,
    end: 0x0034,
};
const AUX_OUTPUT_VOLUME_RANGE: Range<usize> = Range {
    start: 0x0034,
    end: 0x0038,
};
const HEADPHONE_VOLUME_RANGE: Range<usize> = Range {
    start: 0x0038,
    end: 0x0040,
};
const ANALOG_INPUT_BALANCE_RANGE: Range<usize> = Range {
    start: 0x0040,
    end: 0x0050,
};
const SPDIF_INPUT_BALANCE_RANGE: Range<usize> = Range {
    start: 0x0050,
    end: 0x0054,
};
const ADAT_INPUT_BALANCE_RANGE: Range<usize> = Range {
    start: 0x0054,
    end: 0x0064,
};
const AUX_STREAM_INPUT_GAIN_RANGE: Range<usize> = Range {
    start: 0x0064,
    end: 0x006c,
};
const AUX_ANALOG_INPUT_GAIN_RANGE: Range<usize> = Range {
    start: 0x006c,
    end: 0x007c,
};
const AUX_SPDIF_INPUT_GAIN_RANGE: Range<usize> = Range {
    start: 0x007c,
    end: 0x0080,
};
const AUX_ADAT_INPUT_GAIN_RANGE: Range<usize> = Range {
    start: 0x0080,
    end: 0x0090,
};
const MIXER_PHYS_SOURCE_RANGE: Range<usize> = Range {
    start: 0x0090,
    end: 0x0094,
};
const MIXER_STREAM_SOURCE_RANGE: Range<usize> = Range {
    start: 0x0094,
    end: 0x0098,
};
const HEADPHONE_PAIR_SOURCE_RANGE: Range<usize> = Range {
    start: 0x0098,
    end: 0x009c,
};
const ANALOG_OUTPUT_PAIR_SOURCE_RANGE: Range<usize> = Range {
    start: 0x009c,
    end: 0x00a0,
};

/// Cache of state.
#[derive(Debug)]
pub struct MaudioSpecialStateCache(pub [u8; CACHE_SIZE]);

impl Default for MaudioSpecialStateCache {
    fn default() -> Self {
        Self([0; CACHE_SIZE])
    }
}

/// Parameters of input.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MaudioSpecialInputParameters {
    /// The gains of stream inputs.
    pub stream_gains: [i16; 4],

    /// The gains of analog inputs.
    pub analog_gains: [i16; 8],
    /// The gains of S/PDIF inputs.
    pub spdif_gains: [i16; 2],
    /// The gains of ADAT inputs.
    pub adat_gains: [i16; 8],

    /// The L/R balance of analog inputs.
    pub analog_balances: [i16; 8],
    /// The L/R balance of S/PDIF inputs.
    pub spdif_balances: [i16; 2],
    /// The L/R balance of ADAT inputs.
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

/// The protocol implementation to operate analog inputs.
#[derive(Default, Debug)]
pub struct MaudioSpecialInputProtocol;

impl MaudioSpecialInputProtocol {
    /// The minimum value of gain.
    pub const GAIN_MIN: i16 = i16::MIN;
    /// The maximum value of gain.
    pub const GAIN_MAX: i16 = 0;
    /// The step value of gain.
    pub const GAIN_STEP: i16 = 0x100;

    /// The minimum value of L/R balance.
    pub const BALANCE_MIN: i16 = i16::MIN;
    /// The maximum value of L/R balance.
    pub const BALANCE_MAX: i16 = i16::MAX;
    /// The step value of L/R balance.
    pub const BALANCE_STEP: i16 = 0x100;
}

impl SpecialParametersSerdes<MaudioSpecialInputParameters> for MaudioSpecialInputProtocol {
    const OFFSET_RANGES: &'static [&'static Range<usize>] = &[
        &STREAM_INPUT_GAIN_RANGE,
        &ANALOG_INPUT_GAIN_RANGE,
        &SPDIF_INPUT_GAIN_RANGE,
        &ADAT_INPUT_GAIN_RANGE,
        &ANALOG_INPUT_BALANCE_RANGE,
        &SPDIF_INPUT_BALANCE_RANGE,
        &ADAT_INPUT_BALANCE_RANGE,
    ];

    fn serialize(params: &MaudioSpecialInputParameters, raw: &mut [u8]) {
        [
            (&params.stream_gains[..], &STREAM_INPUT_GAIN_RANGE),
            (&params.analog_gains[..], &ANALOG_INPUT_GAIN_RANGE),
            (&params.spdif_gains[..], &SPDIF_INPUT_GAIN_RANGE),
            (&params.adat_gains[..], &ADAT_INPUT_GAIN_RANGE),
            (&params.analog_balances[..], &ANALOG_INPUT_BALANCE_RANGE),
            (&params.spdif_balances[..], &SPDIF_INPUT_BALANCE_RANGE),
            (&params.adat_balances[..], &ADAT_INPUT_BALANCE_RANGE),
        ]
        .iter()
        .for_each(|(gains, range)| {
            gains.iter().enumerate().for_each(|(i, gain)| {
                let pos = range.start + i * 2;
                raw[pos..(pos + 2)].copy_from_slice(&gain.to_be_bytes());
            })
        });
    }

    fn deserialize(params: &mut MaudioSpecialInputParameters, raw: &[u8]) {
        let mut doublet = [0u8; 2];

        [
            (&mut params.stream_gains[..], &STREAM_INPUT_GAIN_RANGE),
            (&mut params.analog_gains[..], &ANALOG_INPUT_GAIN_RANGE),
            (&mut params.spdif_gains[..], &SPDIF_INPUT_GAIN_RANGE),
            (&mut params.adat_gains[..], &ADAT_INPUT_GAIN_RANGE),
            (&mut params.analog_balances[..], &ANALOG_INPUT_BALANCE_RANGE),
            (&mut params.spdif_balances[..], &SPDIF_INPUT_BALANCE_RANGE),
            (&mut params.adat_balances[..], &ADAT_INPUT_BALANCE_RANGE),
        ]
        .iter_mut()
        .for_each(|(gains, range)| {
            gains.iter_mut().enumerate().for_each(|(i, gain)| {
                let pos = range.start + i * 2;
                doublet.copy_from_slice(&raw[pos..(pos + 2)]);
                *gain = i16::from_be_bytes(doublet);
            })
        });
    }
}

/// Source of analog output.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OutputSource {
    /// The corresponding pair of mixer outputs.
    MixerOutputPair,
    /// The pair of auxiliary mixer outputs.
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
    /// The 1st pair of mixer outputs.
    MixerOutputPair0,
    /// The 2nd pair of mixer outputs.
    MixerOutputPair1,
    /// The pair of auxiliary mixer outputs.
    AuxOutputPair0,
}

impl Default for HeadphoneSource {
    fn default() -> Self {
        Self::AuxOutputPair0
    }
}

/// Parameters of output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MaudioSpecialOutputParameters {
    /// The volume of analog outputs.
    pub analog_volumes: [i16; 4],
    /// The source for pair of analog outputs.
    pub analog_pair_sources: [OutputSource; 2],
    /// The volume of headphone outputs.
    pub headphone_volumes: [i16; 4],
    /// The source for pair of headphone outputs.
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

/// The protocol implementation for physical output.
#[derive(Default, Debug)]
pub struct MaudioSpecialOutputProtocol;

impl MaudioSpecialOutputProtocol {
    /// The minimum value of volume.
    pub const VOLUME_MIN: i16 = i16::MIN;
    /// The maximum value of volume.
    pub const VOLUME_MAX: i16 = 0;
    /// The step value of volume.
    pub const VOLUME_STEP: i16 = 0x100;
}

impl SpecialParametersSerdes<MaudioSpecialOutputParameters> for MaudioSpecialOutputProtocol {
    const OFFSET_RANGES: &'static [&'static Range<usize>] = &[
        &ANALOG_OUTPUT_VOLUME_RANGE,
        &HEADPHONE_VOLUME_RANGE,
        &HEADPHONE_PAIR_SOURCE_RANGE,
        &ANALOG_OUTPUT_PAIR_SOURCE_RANGE,
    ];

    fn serialize(params: &MaudioSpecialOutputParameters, raw: &mut [u8]) {
        [
            (&params.analog_volumes[..], &ANALOG_OUTPUT_VOLUME_RANGE),
            (&params.headphone_volumes[..], &HEADPHONE_VOLUME_RANGE),
        ]
        .iter()
        .for_each(|(vols, range)| {
            vols.iter().enumerate().for_each(|(i, vol)| {
                let pos = range.start + i * 2;
                raw[pos..(pos + 2)].copy_from_slice(&vol.to_be_bytes());
            });
        });

        let mut quadlet = [0; 4];
        let pos = HEADPHONE_PAIR_SOURCE_RANGE.start;
        quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
        let mut val = u32::from_be_bytes(quadlet);

        params
            .headphone_pair_sources
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

        raw[pos..(pos + 4)].copy_from_slice(&val.to_be_bytes());

        let pos = ANALOG_OUTPUT_PAIR_SOURCE_RANGE.start;
        quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
        let mut val = u32::from_be_bytes(quadlet);

        params
            .analog_pair_sources
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

        raw[pos..(pos + 4)].copy_from_slice(&val.to_be_bytes());
    }

    fn deserialize(params: &mut MaudioSpecialOutputParameters, raw: &[u8]) {
        let mut doublet = [0u8; 2];

        [
            (&mut params.analog_volumes[..], &ANALOG_OUTPUT_VOLUME_RANGE),
            (&mut params.headphone_volumes[..], &HEADPHONE_VOLUME_RANGE),
        ]
        .iter_mut()
        .for_each(|(vols, range)| {
            vols.iter_mut().enumerate().for_each(|(i, vol)| {
                let pos = range.start + i * 2;
                doublet.copy_from_slice(&raw[pos..(pos + 2)]);
                *vol = i16::from_be_bytes(doublet);
            });
        });

        let mut quadlet = [0u8; 4];
        let pos = HEADPHONE_PAIR_SOURCE_RANGE.start;
        quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
        let val = u32::from_be_bytes(quadlet);

        params
            .headphone_pair_sources
            .iter_mut()
            .enumerate()
            .for_each(|(i, src)| {
                let shift = i * 16;
                let mask = 0x07;
                let flag = (val >> shift) & mask;
                *src = match flag {
                    0x04 => HeadphoneSource::AuxOutputPair0,
                    0x02 => HeadphoneSource::MixerOutputPair1,
                    _ => HeadphoneSource::MixerOutputPair0,
                };
            });

        let pos = ANALOG_OUTPUT_PAIR_SOURCE_RANGE.start;
        quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
        let val = u32::from_be_bytes(quadlet);

        params
            .analog_pair_sources
            .iter_mut()
            .enumerate()
            .for_each(|(i, src)| {
                let shift = i;
                let mask = 0x01;
                let flag = (val >> shift) & mask;
                *src = match flag {
                    0x01 => OutputSource::AuxOutputPair0,
                    _ => OutputSource::MixerOutputPair,
                };
            });
    }
}

/// Parameters of aux signal multiplexer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MaudioSpecialAuxParameters {
    /// The volume of outputs.
    pub output_volumes: [i16; 2],
    /// The gain of stream inputs.
    pub stream_gains: [i16; 4],
    /// The gain of analog inputs.
    pub analog_gains: [i16; 8],
    /// The gain of S/PDIF inputs.
    pub spdif_gains: [i16; 2],
    /// The gain of ADAT inputs.
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

/// The protocol implementation to operate input and output of AUX mixer.
#[derive(Default, Debug)]
pub struct MaudioSpecialAuxProtocol;

impl MaudioSpecialAuxProtocol {
    /// The minimum value of input gain.
    pub const GAIN_MIN: i16 = i16::MIN;
    /// The maximum value of input gain.
    pub const GAIN_MAX: i16 = 0;
    /// The step value of input gain.
    pub const GAIN_STEP: i16 = 0x100;

    /// The minimum value of output volume.
    pub const VOLUME_MIN: i16 = i16::MIN;
    /// The maximum value of output volume.
    pub const VOLUME_MAX: i16 = 0;
    /// The step value of output volume.
    pub const VOLUME_STEP: i16 = 0x100;
}

impl SpecialParametersSerdes<MaudioSpecialAuxParameters> for MaudioSpecialAuxProtocol {
    const OFFSET_RANGES: &'static [&'static Range<usize>] = &[
        &AUX_OUTPUT_VOLUME_RANGE,
        &AUX_STREAM_INPUT_GAIN_RANGE,
        &AUX_ANALOG_INPUT_GAIN_RANGE,
        &AUX_SPDIF_INPUT_GAIN_RANGE,
        &AUX_ADAT_INPUT_GAIN_RANGE,
    ];

    fn serialize(params: &MaudioSpecialAuxParameters, raw: &mut [u8]) {
        [
            (&params.output_volumes[..], &AUX_OUTPUT_VOLUME_RANGE),
            (&params.stream_gains[..], &AUX_STREAM_INPUT_GAIN_RANGE),
            (&params.analog_gains[..], &AUX_ANALOG_INPUT_GAIN_RANGE),
            (&params.spdif_gains[..], &AUX_SPDIF_INPUT_GAIN_RANGE),
            (&params.adat_gains[..], &AUX_ADAT_INPUT_GAIN_RANGE),
        ]
        .iter()
        .for_each(|(levels, range)| {
            levels.iter().enumerate().for_each(|(i, level)| {
                let pos = range.start + i * 2;
                raw[pos..(pos + 2)].copy_from_slice(&level.to_be_bytes());
            })
        });
    }

    fn deserialize(params: &mut MaudioSpecialAuxParameters, raw: &[u8]) {
        let mut doublet = [0u8; 2];

        [
            (&mut params.output_volumes[..], &AUX_OUTPUT_VOLUME_RANGE),
            (&mut params.stream_gains[..], &AUX_STREAM_INPUT_GAIN_RANGE),
            (&mut params.analog_gains[..], &AUX_ANALOG_INPUT_GAIN_RANGE),
            (&mut params.spdif_gains[..], &AUX_SPDIF_INPUT_GAIN_RANGE),
            (&mut params.adat_gains[..], &AUX_ADAT_INPUT_GAIN_RANGE),
        ]
        .iter_mut()
        .for_each(|(levels, range)| {
            levels.iter_mut().enumerate().for_each(|(i, level)| {
                let pos = range.start + i * 2;
                doublet.copy_from_slice(&raw[pos..(pos + 2)]);
                *level = i16::from_be_bytes(doublet);
            })
        });
    }
}

/// Parameters of signal multiplexer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MaudioSpecialMixerParameters {
    /// Enable/Disable the pairs of analog inputs.
    pub analog_pairs: [[bool; 4]; 2],
    /// Enable/Disable the pairs of S/PDIF inputs.
    pub spdif_pairs: [bool; 2],
    /// Enable/Disable the pairs of ADAT inputs.
    pub adat_pairs: [[bool; 4]; 2],
    /// Enable/Disable the pairs of stream inputs.
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

/// The protocol implementation for input and output of mixer.
#[derive(Default, Debug)]
pub struct MaudioSpecialMixerProtocol;

impl SpecialParametersSerdes<MaudioSpecialMixerParameters> for MaudioSpecialMixerProtocol {
    const OFFSET_RANGES: &'static [&'static Range<usize>] =
        &[&MIXER_PHYS_SOURCE_RANGE, &MIXER_STREAM_SOURCE_RANGE];

    fn serialize(params: &MaudioSpecialMixerParameters, raw: &mut [u8]) {
        let mut quadlet = [0; 4];

        quadlet.copy_from_slice(&raw[MIXER_PHYS_SOURCE_RANGE]);
        let mut val = u32::from_be_bytes(quadlet);

        params
            .analog_pairs
            .iter()
            .enumerate()
            .for_each(|(i, pairs)| {
                pairs.iter().enumerate().for_each(|(j, &enabled)| {
                    let flag = 1u32 << (i * 4 + j);
                    val &= !flag;
                    if enabled {
                        val |= flag;
                    }
                });
            });

        params
            .spdif_pairs
            .iter()
            .enumerate()
            .for_each(|(i, &enabled)| {
                let flag = 1u32 << (16 + i);
                val &= !flag;
                if enabled {
                    val |= flag;
                }
            });

        params.adat_pairs.iter().enumerate().for_each(|(i, pairs)| {
            pairs.iter().enumerate().for_each(|(j, &enabled)| {
                let flag = 1u32 << (8 + i * 4 + j);
                val &= !flag;
                if enabled {
                    val |= flag;
                }
            });
        });

        raw[MIXER_PHYS_SOURCE_RANGE].copy_from_slice(&val.to_be_bytes());

        quadlet.copy_from_slice(&raw[MIXER_STREAM_SOURCE_RANGE]);
        let mut val = u32::from_be_bytes(quadlet);

        params
            .stream_pairs
            .iter()
            .enumerate()
            .for_each(|(i, pairs)| {
                pairs.iter().enumerate().for_each(|(j, &enabled)| {
                    let flag = 1u32 << (j * 2 + i);

                    val &= !flag;
                    if enabled {
                        val |= flag;
                    }
                });
            });

        raw[MIXER_STREAM_SOURCE_RANGE].copy_from_slice(&val.to_be_bytes());
    }

    fn deserialize(params: &mut MaudioSpecialMixerParameters, raw: &[u8]) {
        let mut quadlet = [0; 4];

        quadlet.copy_from_slice(&raw[MIXER_PHYS_SOURCE_RANGE]);
        let val = u32::from_be_bytes(quadlet);

        params
            .analog_pairs
            .iter_mut()
            .enumerate()
            .for_each(|(i, pairs)| {
                pairs.iter_mut().enumerate().for_each(|(j, enabled)| {
                    let flag = 1u32 << (i * 4 + j);
                    *enabled = val & flag > 0;
                });
            });

        params
            .spdif_pairs
            .iter_mut()
            .enumerate()
            .for_each(|(i, enabled)| {
                let flag = 1u32 << (16 + i);
                *enabled = val & flag > 0;
            });

        params
            .adat_pairs
            .iter_mut()
            .enumerate()
            .for_each(|(i, pairs)| {
                pairs.iter_mut().enumerate().for_each(|(j, enabled)| {
                    let flag = 1u32 << (8 + i * 4 + j);
                    *enabled = val & flag > 0;
                });
            });

        quadlet.copy_from_slice(&raw[MIXER_STREAM_SOURCE_RANGE]);
        let val = u32::from_be_bytes(quadlet);

        params
            .stream_pairs
            .iter_mut()
            .enumerate()
            .for_each(|(i, pairs)| {
                pairs.iter_mut().enumerate().for_each(|(j, enabled)| {
                    let flag = 1u32 << (j * 2 + i);
                    *enabled = val & flag > 0;
                });
            });
    }
}

/// Protocol interface for each type of parameters.
pub trait SpecialParametersSerdes<T: Copy> {
    /// The set of offset ranges for the type of parameters.
    const OFFSET_RANGES: &'static [&'static Range<usize>];

    /// Change the content of cache by the given parameters.
    fn serialize(params: &T, raw: &mut [u8]);

    /// Decode the cache to change the given parameter
    fn deserialize(params: &mut T, raw: &[u8]);
}

/// The trait for protocol of parameters.
pub trait MaudioSpecialParameterProtocol<T: Copy>: SpecialParametersSerdes<T> {
    /// Update the hardware for the whole parameters.
    fn whole_update(
        req: &FwReq,
        node: &FwNode,
        params: &T,
        cache: &mut MaudioSpecialStateCache,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(Self::OFFSET_RANGES.len() > 0);

        Self::serialize(params, &mut cache.0);

        let mut peek_iter = Self::OFFSET_RANGES.iter().peekable();
        let mut prev_offset = Self::OFFSET_RANGES[0].start;

        // NOTE: detect range of continuous offsets or the end entry.
        while let Some(curr_range) = peek_iter.next() {
            let begin_offset = prev_offset;
            let mut next = peek_iter.peek();

            if let Some(next_range) = next {
                if curr_range.end != next_range.start {
                    prev_offset = next_range.start;
                    next = None;
                }
            }

            if next.is_none() {
                let raw = &mut cache.0[begin_offset..curr_range.end];
                if raw.len() == 4 {
                    FwTcode::WriteQuadletRequest
                } else {
                    FwTcode::WriteBlockRequest
                };

                req.transaction(
                    node,
                    FwTcode::WriteQuadletRequest,
                    DM_APPL_PARAM_OFFSET + begin_offset as u64,
                    raw.len(),
                    raw,
                    timeout_ms,
                )?;
            }
        }

        Ok(())
    }

    /// Update the hardware partially for any change of parameter.
    fn partial_update(
        req: &FwReq,
        node: &FwNode,
        params: &T,
        cache: &mut MaudioSpecialStateCache,
        old: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(Self::OFFSET_RANGES.len() > 0);

        let mut new = [0; CACHE_SIZE];
        new.copy_from_slice(&cache.0);
        Self::serialize(params, &mut new);

        Self::OFFSET_RANGES
            .iter()
            .try_for_each(|range| {
                let raw = &mut new[range.start..range.end];

                if raw != &cache.0[range.start..range.end] {
                    if raw.len() == 4 {
                        FwTcode::WriteQuadletRequest
                    } else {
                        FwTcode::WriteBlockRequest
                    };

                    req.transaction(
                        node,
                        FwTcode::WriteQuadletRequest,
                        DM_APPL_PARAM_OFFSET + range.start as u64,
                        raw.len(),
                        raw,
                        timeout_ms,
                    )
                    .map(|_| cache.0[range.start..range.end].copy_from_slice(raw))
                } else {
                    Ok(())
                }
            })
            .map(|_| *old = *params)
    }
}

impl<O: SpecialParametersSerdes<T>, T: Copy> MaudioSpecialParameterProtocol<T> for O {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn offset_ranges() {
        let ranges = [
            &STREAM_INPUT_GAIN_RANGE,
            &ANALOG_OUTPUT_VOLUME_RANGE,
            &ANALOG_INPUT_GAIN_RANGE,
            &SPDIF_INPUT_GAIN_RANGE,
            &ADAT_INPUT_GAIN_RANGE,
            &AUX_OUTPUT_VOLUME_RANGE,
            &HEADPHONE_VOLUME_RANGE,
            &ANALOG_INPUT_BALANCE_RANGE,
            &SPDIF_INPUT_BALANCE_RANGE,
            &ADAT_INPUT_BALANCE_RANGE,
            &AUX_STREAM_INPUT_GAIN_RANGE,
            &AUX_ANALOG_INPUT_GAIN_RANGE,
            &AUX_SPDIF_INPUT_GAIN_RANGE,
            &AUX_ADAT_INPUT_GAIN_RANGE,
            &MIXER_PHYS_SOURCE_RANGE,
            &MIXER_STREAM_SOURCE_RANGE,
            &HEADPHONE_PAIR_SOURCE_RANGE,
            &ANALOG_OUTPUT_PAIR_SOURCE_RANGE,
        ];

        (0..CACHE_SIZE).for_each(|pos| {
            let count = ranges.iter().filter(|range| range.contains(&pos)).count();
            assert_eq!(count, 1);
        });
    }

    #[test]
    fn input_params_serdes() {
        let expected = MaudioSpecialInputParameters {
            stream_gains: [-2, -1, 0, 1],
            analog_gains: [-3, -2, -1, 0, 1, 2, 3, 4],
            spdif_gains: [-1, 0],
            adat_gains: [-3, -2, -1, 0, 1, 2, 3, 4],
            analog_balances: [-3, -2, -1, 0, 1, 2, 3, 4],
            spdif_balances: [-1, 0],
            adat_balances: [-3, -2, -1, 0, 1, 2, 3, 4],
        };
        let mut cache = MaudioSpecialStateCache::default();
        MaudioSpecialInputProtocol::serialize(&expected, &mut cache.0);

        let mut params = MaudioSpecialInputParameters::default();
        MaudioSpecialInputProtocol::deserialize(&mut params, &cache.0);

        assert_eq!(expected, params);
    }

    #[test]
    fn output_params_serdes() {
        let expected = MaudioSpecialOutputParameters {
            analog_volumes: [-2, -1, 0, 1],
            analog_pair_sources: [OutputSource::MixerOutputPair, OutputSource::AuxOutputPair0],
            headphone_volumes: [-1, 0, 1, 2],
            headphone_pair_sources: [
                HeadphoneSource::AuxOutputPair0,
                HeadphoneSource::MixerOutputPair0,
            ],
        };
        let mut cache = MaudioSpecialStateCache::default();
        MaudioSpecialOutputProtocol::serialize(&expected, &mut cache.0);

        let mut params = MaudioSpecialOutputParameters::default();
        MaudioSpecialOutputProtocol::deserialize(&mut params, &cache.0);

        assert_eq!(expected, params);
    }

    #[test]
    fn aux_params_serdes() {
        let expected = MaudioSpecialAuxParameters {
            output_volumes: [0, 1],
            stream_gains: [-3, -2, -1, 0],
            analog_gains: [-3, -2, -1, 0, 1, 2, 3, 4],
            spdif_gains: [-1, 0],
            adat_gains: [-3, -2, -1, 0, 1, 2, 3, 4],
        };
        let mut cache = MaudioSpecialStateCache::default();
        MaudioSpecialAuxProtocol::serialize(&expected, &mut cache.0);

        let mut params = MaudioSpecialAuxParameters::default();
        MaudioSpecialAuxProtocol::deserialize(&mut params, &cache.0);

        assert_eq!(expected, params);
    }

    #[test]
    fn mixer_params_serdes() {
        let expected = MaudioSpecialMixerParameters {
            analog_pairs: [[false, true, false, true], [true, false, true, false]],
            spdif_pairs: [true, false],
            adat_pairs: [[false, true, false, true], [true, false, true, false]],
            stream_pairs: [[true, false], [false, true]],
        };
        let mut cache = MaudioSpecialStateCache::default();
        MaudioSpecialMixerProtocol::serialize(&expected, &mut cache.0);

        let mut params = MaudioSpecialMixerParameters::default();
        MaudioSpecialMixerProtocol::deserialize(&mut params, &cache.0);

        assert_eq!(expected, params);
    }
}
