// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by Loud Technologies for Tapco Link.FireWire 4x6.
//!
//! The module includes protocol implementation defined by Loud Technologies for
//! Tapco Link.FireWire 4x6.

use super::*;

/// Source of capture.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LinkFwInputSource {
    /// Analog inputs.
    Analog,
    /// Digital inputs.
    Digital,
}

impl Default for LinkFwInputSource {
    fn default() -> Self {
        Self::Analog
    }
}

/// The protocol implementation for Tapco Link.FireWire 4x6.
#[derive(Default, Debug)]
pub struct LinkFwProtocol;

impl OxfordOperation for LinkFwProtocol {}

impl OxfwStreamFormatOperation<OxfwAvc> for LinkFwProtocol {}

impl LinkFwProtocol {
    const SIG_DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: AvcAddrSubunit {
            subunit_type: AvcSubunitType::Audio,
            subunit_id: 0,
        },
        plug_id: 1,
    });

    const SIG_SRCS: [SignalAddr; 2] = [
        // Analog inputs.
        SignalAddr::Unit(SignalUnitAddr::Ext(0x00)),
        // Digital inputs.
        SignalAddr::Unit(SignalUnitAddr::Ext(0x01)),
    ];
}

impl OxfwFcpParamsOperation<OxfwAvc, LinkFwInputSource> for LinkFwProtocol {
    fn cache(
        avc: &mut OxfwAvc,
        params: &mut LinkFwInputSource,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = SignalSource::new(&Self::SIG_DST);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
        let pos = Self::SIG_SRCS
            .iter()
            .position(|src| src.eq(&op.src))
            .unwrap();
        *params = if pos > 0 {
            LinkFwInputSource::Digital
        } else {
            LinkFwInputSource::Analog
        };
        Ok(())
    }
}

impl OxfwFcpMutableParamsOperation<OxfwAvc, LinkFwInputSource> for LinkFwProtocol {
    fn update(
        avc: &mut OxfwAvc,
        params: &LinkFwInputSource,
        prev: &mut LinkFwInputSource,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if *params != *prev {
            let pos = if *params == LinkFwInputSource::Digital {
                1
            } else {
                0
            };
            let mut op = SignalSource {
                src: Self::SIG_SRCS[pos],
                dst: Self::SIG_DST,
            };
            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
        }
        *prev = *params;
        Ok(())
    }
}
