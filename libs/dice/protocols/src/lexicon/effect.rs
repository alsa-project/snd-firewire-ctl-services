// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Effect protocol specific to Lexicon I-ONIX FW810s.
//!
//! The module includes structure, enumeration, and trait and its implementation for effect protocol
//! defined by Lexicon for I-ONIX FW810s.

use glib::Error;
use hinawa::FwNode;

use super::*;

pub trait IonixEffectProtocol<T: AsRef<FwNode>> : IonixProtocol<T> {
    // NOTE: states of all effect are available with structured data by read block request with 512 bytes.
    const EFFECT_OFFSET: usize = 0x00004000;

    const DATA_PREFIX: [u8;5] = [0x06, 0x00, 0x1b, 0x01, 0x41];

    const SYSEX_MSG_PREFIX: u8 = 0xf0;
    const SYSEX_MSG_SUFFIX: u8 = 0xf7;

    fn write_data(&self, node: &T, data: &[u8], timeout_ms: u32) -> Result<(), Error> {
        // NOTE: The data has prefix.
        let mut msgs = Self::DATA_PREFIX.to_vec();
        msgs.extend_from_slice(&data);

        // NOTE: Append checksum calculated by XOR for all the data.
        let checksum = msgs.iter()
            .fold(0u8, |val, &msg| val | msg);
        msgs.push(checksum);

        // NOTE: Construct MIDI system exclusive message.
        msgs.insert(0, 0xf0);
        msgs.push(0xf7);

        // NOTE: One quadlet deliver one byte of message.
        let mut raw = Vec::<u8>::new();
        msgs.iter()
            .for_each(|&msg| raw.extend_from_slice(&(msg as u32).to_be_bytes()));

        IonixProtocol::write(self, node, Self::EFFECT_OFFSET, &mut raw, timeout_ms)
    }
}

impl<O, T> IonixEffectProtocol<T> for O
    where T: AsRef<FwNode>,
          O: IonixProtocol<T>,
{}
