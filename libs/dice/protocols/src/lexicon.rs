// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol specific to Lexicon I-ONIX FW810s.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Lexicon for I-ONIX FW810s.

use glib::Error;
use hinawa::FwNode;

use super::tcat::*;

/// The trait to represent protocol defined by Lexicon for I-ONIX FW810s.
pub trait IonixProtocol<T: AsRef<FwNode>> : GeneralProtocol<T> {
    const BASE_OFFSET: usize = 0x00200000;

    fn read(&self, node: &T, offset: usize, frame: &mut [u8], timeout_ms: u32) -> Result<(), Error> {
        GeneralProtocol::read(self, node, Self::BASE_OFFSET + offset, frame, timeout_ms)
    }

    fn write(&self, node: &T, offset: usize, frame: &mut [u8], timeout_ms: u32) -> Result<(), Error> {
        GeneralProtocol::write(self, node, Self::BASE_OFFSET + offset, frame, timeout_ms)
    }
}

impl<O, T> IonixProtocol<T> for O
    where T: AsRef<FwNode>,
          O: GeneralProtocol<T>,
{}
