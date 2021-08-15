// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol defined by Loud Technologies for Tapco Link.FireWire 4x6.
//!
//! The module includes protocol implementation defined by Loud Technologies for
//! Tapco Link.FireWire 4x6.

use glib::Error;

use hinawa::FwFcp;

use ta1394::{ccm::*, *};

/// The enumeration for source of capture.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LinkFwInputSource {
    Analog,
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

impl LinkFwProtocol {
    const SIG_DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: AvcAddrSubunit {
            subunit_type: AvcSubunitType::Audio,
            subunit_id: 0,
        },
        plug_id: 1,
    });

    const SIG_SRCS: [SignalAddr; 2] = [
        SignalAddr::Unit(SignalUnitAddr::Ext(0x00)),
        SignalAddr::Unit(SignalUnitAddr::Ext(0x01)),
    ];

    pub fn read_input_source(
        avc: &mut FwFcp,
        src: &mut LinkFwInputSource,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = SignalSource::new(&Self::SIG_DST);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms).map(|_| {
            let pos = Self::SIG_SRCS
                .iter()
                .position(|src| src.eq(&op.src))
                .unwrap();
            *src = if pos > 0 {
                LinkFwInputSource::Digital
            } else {
                LinkFwInputSource::Analog
            };
        })
    }

    pub fn write_input_source(
        avc: &mut FwFcp,
        src: LinkFwInputSource,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let pos = if src == LinkFwInputSource::Digital {
            1
        } else {
            0
        };
        let mut op = SignalSource {
            src: Self::SIG_SRCS[pos],
            dst: Self::SIG_DST,
        };
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }
}
