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

const ADAT_USER_DATA_MASK: u32 = 0x0f;
const ADAT_USER_DATA_UNAVAIL: u32 = 0x10;

impl<O: TcatOperation> TcatSectionSerdes<ExtendedSyncParameters> for O {
    const MIN_SIZE: usize = 16;

    const ERROR_TYPE: GeneralProtocolError = GeneralProtocolError::ExtendedSync;

    fn serialize(_: &ExtendedSyncParameters, _: &mut [u8]) -> Result<(), String> {
        // All of fields are read-only.
        Ok(())
    }

    fn deserialize(params: &mut ExtendedSyncParameters, raw: &[u8]) -> Result<(), String> {
        let mut val = 0u8;
        deserialize_u8(&mut val, &raw[..4]);
        deserialize_clock_source(&mut params.clk_src, &val)?;

        let mut val = 0u32;
        deserialize_u32(&mut val, &raw[4..8]);
        params.clk_src_locked = val > 0;

        let mut val = 0u8;
        deserialize_u8(&mut val, &raw[8..12]);
        deserialize_clock_rate(&mut params.clk_rate, &val)?;

        let mut val = 0u32;
        deserialize_u32(&mut val, &raw[12..16]);
        params.adat_user_data = if val & ADAT_USER_DATA_UNAVAIL > 0 {
            None
        } else {
            Some((val & ADAT_USER_DATA_MASK) as u8)
        };

        Ok(())
    }
}

impl<O: TcatOperation> TcatSectionOperation<ExtendedSyncParameters> for O {}

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
