// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! External synchronization section in general protocol defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for extended
//! synchronization section in general protocol defined by TCAT for ASICs of DICE.

use super::{global_section::*, *};

/// Parameters in extended synchronization section.
#[derive(Default, Copy, Clone, Debug, Eq, PartialEq)]
pub struct ExtendedSyncParameters {
    /// Current clock source; read-only.
    pub clk_src: ClockSource,
    /// Clock source is locked (boolean); read-only.
    pub clk_src_locked: bool,
    /// Current sample rate (CLOCK_RATE_* >> CLOCK_RATE_SHIFT), _32000-_192000 or _NONE; read-only.
    pub clk_rate: ClockRate,
    /// ADAT user data bits; read-only.
    pub adat_user_data: Option<u8>,
}

impl<O: TcatOperation> TcatSectionSerdes<ExtendedSyncParameters> for O {
    const MIN_SIZE: usize = 16;

    const ERROR_TYPE: GeneralProtocolError = GeneralProtocolError::ExtendedSync;

    fn serialize(_: &ExtendedSyncParameters, _: &mut [u8]) -> Result<(), String> {
        // All of fields are read-only.
        Ok(())
    }

    fn deserialize(params: &mut ExtendedSyncParameters, raw: &[u8]) -> Result<(), String> {
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&raw[..4]);
        params.clk_src = ClockSource::from(u32::from_be_bytes(quadlet) as u8);

        params.clk_src_locked = u32::from_be_bytes(quadlet) > 0;

        quadlet.copy_from_slice(&raw[8..12]);
        params.clk_rate = ClockRate::from(u32::from_be_bytes(quadlet) as u8);

        quadlet.copy_from_slice(&raw[12..16]);
        let val = u32::from_be_bytes(quadlet);
        params.adat_user_data = if val & ExtSyncBlock::ADAT_USER_DATA_UNAVAIL > 0 {
            None
        } else {
            Some((val & ExtSyncBlock::ADAT_USER_DATA_MASK) as u8)
        };

        Ok(())
    }
}

impl<O: TcatOperation> TcatSectionOperation<ExtendedSyncParameters> for O {}

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
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&self.0[Self::SYNC_SRC_OFFSET..(Self::SYNC_SRC_OFFSET + 4)]);
        ClockSource::from(u32::from_be_bytes(quadlet) as u8)
    }

    pub fn get_sync_src_locked(&self) -> bool {
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&self.0[Self::SYNC_LOCKED_OFFSET..(Self::SYNC_LOCKED_OFFSET + 4)]);
        u32::from_be_bytes(quadlet) > 0
    }

    pub fn get_sync_src_rate(&self) -> ClockRate {
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&self.0[Self::SYNC_RATE_OFFSET..(Self::SYNC_RATE_OFFSET + 4)]);
        ClockRate::from(u32::from_be_bytes(quadlet) as u8)
    }

    pub fn get_sync_src_adat_user_data(&self) -> Option<u8> {
        let mut quadlet = [0; 4];
        quadlet
            .copy_from_slice(&self.0[Self::SYNC_ADAT_DATA_BITS..(Self::SYNC_ADAT_DATA_BITS + 4)]);
        let val = u32::from_be_bytes(quadlet);
        if val & Self::ADAT_USER_DATA_UNAVAIL > 0 {
            None
        } else {
            Some((val & Self::ADAT_USER_DATA_MASK) as u8)
        }
    }
}

/// Protocol implementaion of external synchronization section.
#[derive(Default)]
pub struct ExtSyncSectionProtocol;

impl ExtSyncSectionProtocol {
    pub fn read_block(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &GeneralSections,
        timeout_ms: u32,
    ) -> Result<ExtSyncBlock, Error> {
        if sections.ext_sync.size < ExtSyncBlock::SIZE {
            let msg = format!(
                "Ext sync section has {} less size than {} expected",
                sections.ext_sync.size,
                ExtSyncBlock::SIZE
            );
            Err(Error::new(FileError::Nxio, &msg))
        } else {
            let mut data = vec![0; sections.ext_sync.size];
            GeneralProtocol::read(req, node, sections.ext_sync.offset, &mut data, timeout_ms)
                .map(|_| ExtSyncBlock(data))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct Protocol;

    impl TcatOperation for Protocol {}

    #[test]
    fn ext_sync_params_serdes() {
        let raw = [0, 0, 0, 0xa, 0, 0, 0, 1, 0, 0, 0, 5, 0, 0, 0, 7];
        let mut params = ExtendedSyncParameters::default();
        Protocol::deserialize(&mut params, &raw).unwrap();

        assert_eq!(params.clk_src, ClockSource::Arx3);
        assert_eq!(params.clk_src_locked, true);
        assert_eq!(params.clk_rate, ClockRate::R176400);
        assert_eq!(params.adat_user_data, Some(0x7));
    }
}
