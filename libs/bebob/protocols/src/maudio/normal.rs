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

/// The protocol implementation for physical input of FireWire 410.
#[derive(Default)]
pub struct Fw410PhysInputProtocol;

impl AvcLevelOperation for Fw410PhysInputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x03, AudioCh::Each(0)), // analog-input-1
        (0x03, AudioCh::Each(1)), // analog-input-2
        (0x04, AudioCh::Each(0)), // digital-input-1
        (0x04, AudioCh::Each(1)), // digital-input-2
    ];
}

impl AvcLrBalanceOperation for Fw410PhysInputProtocol {}

/// The protocol implementation for physical output of FireWire 410.
#[derive(Default)]
pub struct Fw410PhysOutputProtocol;

impl AvcLevelOperation for Fw410PhysOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x0a, AudioCh::Each(0)), // analog-output-1
        (0x0a, AudioCh::Each(1)), // analog-output-2
        (0x0b, AudioCh::Each(0)), // analog-output-3
        (0x0b, AudioCh::Each(1)), // analog-output-4
        (0x0c, AudioCh::Each(0)), // analog-output-5
        (0x0c, AudioCh::Each(1)), // analog-output-6
        (0x0d, AudioCh::Each(0)), // analog-output-7
        (0x0d, AudioCh::Each(1)), // analog-output-8
        (0x0e, AudioCh::Each(0)), // digital-output-1
        (0x0e, AudioCh::Each(1)), // digital-output-2
    ];
}

/// The protocol implementation for source of aux mixer in FireWire 410.
#[derive(Default)]
pub struct Fw410AuxSourceProtocol;

impl AvcLevelOperation for Fw410AuxSourceProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x07, AudioCh::Each(0)), // analog-input-1
        (0x07, AudioCh::Each(1)), // analog-input-2
        (0x08, AudioCh::Each(0)), // digital-input-1
        (0x08, AudioCh::Each(1)), // digital-input-2
        (0x06, AudioCh::Each(0)), // stream-input-1
        (0x06, AudioCh::Each(1)), // stream-input-2
        (0x05, AudioCh::Each(0)), // stream-input-3
        (0x05, AudioCh::Each(1)), // stream-input-4
        (0x05, AudioCh::Each(2)), // stream-input-5
        (0x05, AudioCh::Each(3)), // stream-input-6
        (0x05, AudioCh::Each(4)), // stream-input-7
        (0x05, AudioCh::Each(5)), // stream-input-8
        (0x05, AudioCh::Each(6)), // stream-input-9
        (0x05, AudioCh::Each(7)), // stream-input-10
    ];
}

/// The protocol implementation for output of aux mixer in FireWire 410.
#[derive(Default)]
pub struct Fw410AuxOutputProtocol;

impl AvcLevelOperation for Fw410AuxOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x09, AudioCh::Each(0)), // aux-output-1
        (0x09, AudioCh::Each(1)), // aux-output-2
    ];
}

/// The protocol implementation for output of headphone in FireWire 410.
#[derive(Default)]
pub struct Fw410HeadphoneProtocol;

impl AvcLevelOperation for Fw410HeadphoneProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x0f, AudioCh::Each(0)), // headphone-1
        (0x0f, AudioCh::Each(1)), // headphone-2
    ];
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

/// The protocol implementation for physical input of FireWire Solo.
#[derive(Default)]
pub struct SoloPhysInputProtocol;

impl AvcLevelOperation for SoloPhysInputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x03, AudioCh::Each(0)), // analog-input-1
        (0x03, AudioCh::Each(1)), // analog-input-2
        (0x04, AudioCh::Each(0)), // digital-input-1
        (0x04, AudioCh::Each(1)), // digital-input-2
    ];
}

impl AvcLrBalanceOperation for SoloPhysInputProtocol {}

/// The protocol implementation for stream input of FireWire Solo.
#[derive(Default)]
pub struct SoloStreamInputProtocol;

impl AvcLevelOperation for SoloStreamInputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x01, AudioCh::Each(0)), // stream-input-1
        (0x01, AudioCh::Each(1)), // stream-input-2
        (0x02, AudioCh::Each(0)), // stream-input-3
        (0x02, AudioCh::Each(1)), // stream-input-4
    ];
}

// NOTE: outputs are not configurable, connected to hardware dial directly.

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

/// The protocol implementation for physical input in FireWire Audiophile.
#[derive(Default)]
pub struct AudiophilePhysInputProtocol;

impl AvcLevelOperation for AudiophilePhysInputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x04, AudioCh::Each(0)), // analog-input-1
        (0x04, AudioCh::Each(1)), // analog-input-2
        (0x05, AudioCh::Each(0)), // digital-input-1
        (0x05, AudioCh::Each(1)), // digital-input-2
    ];
}

impl AvcLrBalanceOperation for AudiophilePhysInputProtocol {}

/// The protocol implementation for physical output in FireWire Audiophile.
#[derive(Default)]
pub struct AudiophilePhysOutputProtocol;

impl AvcLevelOperation for AudiophilePhysOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x0c, AudioCh::Each(0)), // analog-output-1
        (0x0c, AudioCh::Each(1)), // analog-output-2
        (0x0d, AudioCh::Each(0)), // analog-output-3
        (0x0d, AudioCh::Each(1)), // analog-output-4
        (0x0e, AudioCh::Each(0)), // digital-output-1
        (0x0e, AudioCh::Each(1)), // digital-output-2
    ];
}

/// The protocol implementation for source of aux mixer in FireWire Audiophile.
#[derive(Default)]
pub struct AudiophileAuxSourceProtocol;

impl AvcLevelOperation for AudiophileAuxSourceProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x09, AudioCh::Each(0)), // analog-input-1
        (0x09, AudioCh::Each(1)), // analog-input-2
        (0x0a, AudioCh::Each(0)), // digital-input-1
        (0x0a, AudioCh::Each(1)), // digital-input-2
        (0x06, AudioCh::Each(0)), // stream-input-1
        (0x06, AudioCh::Each(1)), // stream-input-2
        (0x07, AudioCh::Each(0)), // stream-input-3
        (0x07, AudioCh::Each(1)), // stream-input-4
        (0x08, AudioCh::Each(0)), // stream-input-5
        (0x08, AudioCh::Each(1)), // stream-input-6
    ];
}

/// The protocol implementation for output of aux mixer in FireWire Audiophile.
#[derive(Default)]
pub struct AudiophileAuxOutputProtocol;

impl AvcLevelOperation for AudiophileAuxOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x0b, AudioCh::Each(0)), // aux-output-1
        (0x0b, AudioCh::Each(1)), // aux-output-2
    ];
}

/// The protocol implementation for output of headphone in FireWire Audiophile.
#[derive(Default)]
pub struct AudiophileHeadphoneProtocol;

impl AvcLevelOperation for AudiophileHeadphoneProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x0f, AudioCh::Each(0)), // headphone-1
        (0x0f, AudioCh::Each(1)), // headphone-2
    ];
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

/// The protocol implementation for physical input of FireWire Audiophile.
#[derive(Default)]
pub struct OzonicPhysInputProtocol;

impl AvcLevelOperation for OzonicPhysInputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x03, AudioCh::Each(0)), // analog-input-1
        (0x03, AudioCh::Each(1)), // analog-input-2
        (0x04, AudioCh::Each(0)), // analog-input-3
        (0x04, AudioCh::Each(1)), // analog-input-4
    ];
}

impl AvcLrBalanceOperation for OzonicPhysInputProtocol {}

/// The protocol implementation for stream input of FireWire Audiophile.
#[derive(Default)]
pub struct OzonicStreamInputProtocol;

impl AvcLevelOperation for OzonicStreamInputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x01, AudioCh::Each(0)), // stream-input-1
        (0x01, AudioCh::Each(1)), // stream-input-2
        (0x02, AudioCh::Each(0)), // stream-input-3
        (0x02, AudioCh::Each(1)), // stream-input-4
    ];
}

// NOTE: outputs are not configurable, connected to hardware dial directly.

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
