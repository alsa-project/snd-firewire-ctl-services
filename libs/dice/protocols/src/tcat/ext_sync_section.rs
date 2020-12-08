// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! External synchronization section in general protocol defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for external
//! synchronization section in general protocol defined by TCAT for ASICs of DICE.
use glib::{Error, FileError};

use super::{*, global_section::*};

pub struct ExtSyncBlock(Vec<u8>);

impl ExtSyncBlock {
    const SIZE: usize = 0x10;

    const SYNC_SRC_OFFSET: usize = 0x00;
    const SYNC_LOCKED_OFFSET: usize = 0x04;
    const SYNC_RATE_OFFSET: usize = 0x08;
    const SYNC_ADAT_DATA_BITS: usize = 0x0c;

    const ADAT_USER_DATA_MASK: u32 = 0x0f;
    const ADAT_USER_DATA_UNAVAIL: u32 = 0x10;

    pub fn get_sync_src(&self) -> ClockSource {
        let mut quadlet = [0;4];
        quadlet.copy_from_slice(&self.0[Self::SYNC_SRC_OFFSET..(Self::SYNC_SRC_OFFSET + 4)]);
        ClockSource::from(u32::from_be_bytes(quadlet) as u8)
    }

    pub fn get_sync_src_locked(&self) -> bool {
        let mut quadlet = [0;4];
        quadlet.copy_from_slice(&self.0[Self::SYNC_LOCKED_OFFSET..(Self::SYNC_LOCKED_OFFSET + 4)]);
        u32::from_be_bytes(quadlet) > 0
    }

    pub fn get_sync_src_rate(&self) -> ClockRate {
        let mut quadlet = [0;4];
        quadlet.copy_from_slice(&self.0[Self::SYNC_RATE_OFFSET..(Self::SYNC_RATE_OFFSET + 4)]);
        ClockRate::from(u32::from_be_bytes(quadlet) as u8)
    }

    pub fn get_sync_src_adat_user_data(&self) -> Option<u8> {
        let mut quadlet = [0;4];
        quadlet.copy_from_slice(&self.0[Self::SYNC_ADAT_DATA_BITS..(Self::SYNC_ADAT_DATA_BITS+ 4)]);
        let val = u32::from_be_bytes(quadlet);
        if val & Self::ADAT_USER_DATA_UNAVAIL > 0 {
            None
        } else {
            Some((val & Self::ADAT_USER_DATA_MASK) as u8)
        }
    }
}

pub trait ExtSyncSectionProtocol<T> : GeneralProtocol<T>
    where T: AsRef<FwNode>,
{
    fn read_ext_sync_block(&self, node: &T, sections: &GeneralSections, timeout_ms: u32) -> Result<ExtSyncBlock, Error> {
        if sections.ext_sync.size < ExtSyncBlock::SIZE {
            let msg = format!("Ext sync section has {} less size than {} expected",
                              sections.ext_sync.size, ExtSyncBlock::SIZE);
            Err(Error::new(FileError::Nxio, &msg))
        } else {
            let mut data = vec![0;sections.ext_sync.size];
            self.read(node, sections.ext_sync.offset, &mut data, timeout_ms)
                .map(|_| ExtSyncBlock(data))
        }
    }
}

impl<O: AsRef<FwReq>, T: AsRef<FwNode>> ExtSyncSectionProtocol<T> for O {}
