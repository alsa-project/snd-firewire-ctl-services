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
