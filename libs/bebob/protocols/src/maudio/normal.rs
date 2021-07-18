// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for M-Audio FireWire series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by M-Audio normal FireWire series.

use crate::*;

use ta1394::ccm::{SignalAddr, SignalSubunitAddr, SignalUnitAddr};
use ta1394::MUSIC_SUBUNIT_0;

use super::*;

/// The protocol implementation for media and sampling clock of FireWire 410.
#[derive(Default)]
pub struct Fw410ClkProtocol;

impl MediaClockFrequencyOperation for Fw410ClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000, 176400, 192000];
}

impl SamplingClockSourceOperation for Fw410ClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x01,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x01,
        }),
        // S/PDIF
        SignalAddr::Unit(SignalUnitAddr::Ext(0x02)),
    ];
}

/// The protocol implementation for meter in FireWire 410.
#[derive(Default)]
pub struct Fw410MeterProtocol;

impl MaudioNormalMeterProtocol for Fw410MeterProtocol {
    const PHYS_INPUT_COUNT: usize = 4;
    const STREAM_INPUT_COUNT: usize = 0;
    const PHYS_OUTPUT_COUNT: usize = 10;
    const ROTARY_COUNT: usize = 1;
    const HAS_SWITCH: bool = false;
    const HAS_SYNC_STATUS: bool = true;
}

/// The protocol implementation for media and sampling clock of FireWire Solo.
#[derive(Default)]
pub struct SoloClkProtocol;

impl MediaClockFrequencyOperation for SoloClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SamplingClockSourceOperation for SoloClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x01,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x01,
        }),
        // S/PDIF
        SignalAddr::Unit(SignalUnitAddr::Ext(0x01)),
    ];
}

/// The protocol implementation for meter in FireWire Solo.
#[derive(Default)]
pub struct SoloMeterProtocol;

impl MaudioNormalMeterProtocol for SoloMeterProtocol {
    const PHYS_INPUT_COUNT: usize = 4;
    const STREAM_INPUT_COUNT: usize = 4;
    const PHYS_OUTPUT_COUNT: usize = 4;
    const ROTARY_COUNT: usize = 0;
    const HAS_SWITCH: bool = false;
    const HAS_SYNC_STATUS: bool = true;
}

/// The protocol implementation for media and sampling clock of FireWire Audiophile.
#[derive(Default)]
pub struct AudiophileClkProtocol;

impl MediaClockFrequencyOperation for AudiophileClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SamplingClockSourceOperation for AudiophileClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x01,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x01,
        }),
        // S/PDIF
        SignalAddr::Unit(SignalUnitAddr::Ext(0x02)),
    ];
}

/// The protocol implementation for meter in FireWire Audiophile.
#[derive(Default)]
pub struct AudiophileMeterProtocol;

impl MaudioNormalMeterProtocol for AudiophileMeterProtocol {
    const PHYS_INPUT_COUNT: usize = 4;
    const STREAM_INPUT_COUNT: usize = 0;
    const PHYS_OUTPUT_COUNT: usize = 6;
    const ROTARY_COUNT: usize = 2;
    const HAS_SWITCH: bool = true;
    const HAS_SYNC_STATUS: bool = true;
}

/// The protocol implementation for media and sampling clock of Ozonic.
#[derive(Default)]
pub struct OzonicClkProtocol;

impl MediaClockFrequencyOperation for OzonicClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SamplingClockSourceOperation for OzonicClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x05,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x05,
        }),
    ];
}

/// The protocol implementation for meter in Ozonic.
#[derive(Default)]
pub struct OzonicMeterProtocol;

impl MaudioNormalMeterProtocol for OzonicMeterProtocol {
    const PHYS_INPUT_COUNT: usize = 4;
    const STREAM_INPUT_COUNT: usize = 4;
    const PHYS_OUTPUT_COUNT: usize = 4;
    const ROTARY_COUNT: usize = 0;
    const HAS_SWITCH: bool = false;
    const HAS_SYNC_STATUS: bool = false;
}

/// The state of switch with LED specific to FireWire Audiophile.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AudiophileSwitchState {
    Off,
    A,
    B,
}

impl Default for AudiophileSwitchState {
    fn default() -> Self {
        Self::Off
    }
}

impl From<AudiophileSwitchState> for u8 {
    fn from(state: AudiophileSwitchState) -> Self {
        match state {
            AudiophileSwitchState::Off => 0x00,
            AudiophileSwitchState::A => 0x01,
            AudiophileSwitchState::B => 0x02,
        }
    }
}

impl From<u8> for AudiophileSwitchState {
    fn from(val: u8) -> Self {
        match val {
            0x01 => AudiophileSwitchState::A,
            0x02 => AudiophileSwitchState::B,
            _ => AudiophileSwitchState::Off,
        }
    }
}

/// The structure to express AV/C vendor-dependent command for LED switch specific to FireWire
/// Audiophile.
pub struct AudiophileLedSwitch {
    state: AudiophileSwitchState,
    op: VendorDependent,
}

impl AudiophileLedSwitch {
    pub fn new(switch_state: AudiophileSwitchState) -> Self {
        let mut instance = Self::default();
        instance.state = switch_state;
        instance
    }
}

impl Default for AudiophileLedSwitch {
    fn default() -> Self {
        Self {
            state: Default::default(),
            op: VendorDependent {
                company_id: MAUDIO_OUI,
                data: vec![0x02, 0x00, 0x01, 0xff, 0xff, 0xff],
            },
        }
    }
}

impl AvcOp for AudiophileLedSwitch {
    const OPCODE: u8 = VendorDependent::OPCODE;
}

impl AvcControl for AudiophileLedSwitch {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        self.op.data[3] = self.state.into();
        AvcControl::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcControl::parse_operands(&mut self.op, addr, operands)
    }
}

/// The structure to express metering information.
pub struct MaudioNormalMeter {
    pub phys_inputs: Vec<i32>,
    pub stream_inputs: Option<Vec<i32>>,
    pub phys_outputs: Vec<i32>,
    pub headphone: Option<[i32; 2]>,
    pub aux_output: Option<[i32; 2]>,
    pub rotaries: Option<[i32; 2]>,
    pub switch: Option<AudiophileSwitchState>,
    pub sync_status: Option<bool>,

    cache: Vec<u8>,
}

/// The trait for meter protocol specific to M-Audio FireWire series.
pub trait MaudioNormalMeterProtocol {
    const PHYS_INPUT_COUNT: usize;
    const STREAM_INPUT_COUNT: usize;
    const PHYS_OUTPUT_COUNT: usize;
    const ROTARY_COUNT: usize;
    const HAS_SWITCH: bool;
    const HAS_SYNC_STATUS: bool;

    const LEVEL_MIN: i32 = 0;
    const LEVEL_MAX: i32 = i32::MAX;
    const LEVEL_STEP: i32 = 0x100;

    const ROTARY_MIN: i32 = 0;
    const ROTARY_MAX: i32 = i16::MAX as i32;
    const ROTARY_STEP: i32 = 0x200;

    fn create_meter() -> MaudioNormalMeter {
        let mut meter = MaudioNormalMeter {
            phys_inputs: vec![0; Self::PHYS_INPUT_COUNT],
            stream_inputs: Default::default(),
            phys_outputs: vec![0; Self::PHYS_OUTPUT_COUNT],
            headphone: Default::default(),
            aux_output: Default::default(),
            rotaries: Default::default(),
            switch: Default::default(),
            sync_status: Default::default(),
            cache: vec![0; Self::calculate_meter_frame_size()],
        };

        if Self::STREAM_INPUT_COUNT > 0 {
            meter.stream_inputs = Some(vec![0; Self::STREAM_INPUT_COUNT]);
        } else {
            meter.headphone = Some(Default::default());
            meter.aux_output = Some(Default::default());
        }

        if Self::ROTARY_COUNT > 0 {
            meter.rotaries = Some(Default::default());
        }

        if Self::HAS_SWITCH {
            meter.switch = Some(Default::default());
        }

        if Self::HAS_SYNC_STATUS {
            meter.sync_status = Some(Default::default());
        }

        meter
    }

    fn calculate_meter_frame_size() -> usize {
        let mut size = Self::PHYS_INPUT_COUNT + Self::PHYS_OUTPUT_COUNT;

        if Self::STREAM_INPUT_COUNT > 0 {
            size += Self::STREAM_INPUT_COUNT;
        } else {
            // Plus headphone-1 and -2, aux-1 and -2.
            size += 4;
        }

        if Self::ROTARY_COUNT > 0 || Self::HAS_SWITCH || Self::HAS_SYNC_STATUS {
            size += 1;
        }

        size * 4
    }

    fn read_meter(
        req: &FwReq,
        node: &FwNode,
        meter: &mut MaudioNormalMeter,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let frame = &mut meter.cache;

        // For rotaries, switch, and sync_status if available.
        let mut bitmap = [0; 4];
        let pos = frame.len() - 4;
        bitmap.copy_from_slice(&frame[pos..]);

        read_block(req, node, METER_OFFSET, frame, timeout_ms)?;

        let mut quadlet = [0; 4];

        meter.phys_inputs.iter_mut().enumerate().for_each(|(i, m)| {
            let pos = i * 4;
            quadlet.copy_from_slice(&frame[pos..(pos + 4)]);
            *m = i32::from_be_bytes(quadlet);
        });

        if let Some(stream_inputs) = &mut meter.stream_inputs {
            stream_inputs.iter_mut().enumerate().for_each(|(i, m)| {
                let pos = (Self::PHYS_INPUT_COUNT + i) * 4;
                quadlet.copy_from_slice(&frame[pos..(pos + 4)]);
                *m = i32::from_be_bytes(quadlet);
            });
        }

        meter
            .phys_outputs
            .iter_mut()
            .enumerate()
            .for_each(|(i, m)| {
                let pos = (Self::PHYS_INPUT_COUNT + Self::STREAM_INPUT_COUNT + i) * 4;
                quadlet.copy_from_slice(&frame[pos..(pos + 4)]);
                *m = i32::from_be_bytes(quadlet);
            });

        if let Some(headphone) = &mut meter.headphone {
            headphone.iter_mut().enumerate().for_each(|(i, m)| {
                let pos = (Self::PHYS_INPUT_COUNT + Self::PHYS_OUTPUT_COUNT + i) * 4;
                quadlet.copy_from_slice(&frame[pos..(pos + 4)]);
                *m = i32::from_be_bytes(quadlet);
            });
        }

        if let Some(aux_output) = &mut meter.aux_output {
            aux_output.iter_mut().enumerate().for_each(|(i, m)| {
                let pos = (Self::PHYS_INPUT_COUNT + Self::PHYS_OUTPUT_COUNT + 2 + i) * 4;
                quadlet.copy_from_slice(&frame[pos..(pos + 4)]);
                *m = i32::from_be_bytes(quadlet);
            });
        }

        if let Some(rotaries) = &mut meter.rotaries {
            rotaries.iter_mut().enumerate().for_each(|(i, r)| {
                let shift = i * 4;
                if (bitmap[1] ^ frame[frame.len() - 3]) & (0x0f << shift) > 0 {
                    let flag = (frame[frame.len() - 3] >> shift) & 0x0f;
                    if flag == 0x01 {
                        if *r <= Self::ROTARY_MAX - Self::ROTARY_STEP {
                            *r += Self::ROTARY_STEP;
                        }
                    } else if flag == 0x02 {
                        if *r >= Self::ROTARY_MIN + Self::ROTARY_STEP {
                            *r -= Self::ROTARY_STEP;
                        }
                    }
                }
            });
        }

        if let Some(switch) = &mut meter.switch {
            if bitmap[0] ^ frame[frame.len() - 4] & 0xf0 > 0 {
                if bitmap[0] & 0xf0 > 0 {
                    *switch = match *switch {
                        AudiophileSwitchState::Off => AudiophileSwitchState::A,
                        AudiophileSwitchState::A => AudiophileSwitchState::B,
                        AudiophileSwitchState::B => AudiophileSwitchState::Off,
                    };
                }
            }
        }

        if let Some(sync_status) = &mut meter.sync_status {
            *sync_status = frame[frame.len() - 1] > 0;
        }

        Ok(())
    }
}
