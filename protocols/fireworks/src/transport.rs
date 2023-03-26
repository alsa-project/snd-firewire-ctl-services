// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol about hardware information.
//!
//! The module includes protocol about hardware information defined by Echo Audio Digital Corporation
//! for Fireworks board module.

use super::*;

const CATEGORY_TRANSPORT: u32 = 2;

const CMD_SET_TRANSMIT_MODE: u32 = 0;

/// The format of packet in transmission.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TxPacketFormat {
    /// Unique.
    Unique,
    /// Compliant to IEC 61883-1/6, actually with some quirks.
    Iec61883,
}

impl Default for TxPacketFormat {
    fn default() -> Self {
        Self::Iec61883
    }
}

fn serialize_tx_packet_format(fmt: &TxPacketFormat) -> u32 {
    match fmt {
        TxPacketFormat::Unique => 0,
        TxPacketFormat::Iec61883 => 1,
    }
}

#[cfg(test)]
fn deserialize_tx_packet_format(fmt: &mut TxPacketFormat, val: u32) {
    *fmt = match val {
        1 => TxPacketFormat::Iec61883,
        _ => TxPacketFormat::Unique,
    };
}

impl<O, P> EfwWhollyUpdatableParamsOperation<P, TxPacketFormat> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn update_wholly(proto: &mut P, states: &TxPacketFormat, timeout_ms: u32) -> Result<(), Error> {
        assert!(Self::CAPABILITIES
            .iter()
            .find(|cap| HwCap::ChangeableRespAddr.eq(cap))
            .is_some());

        let args = [serialize_tx_packet_format(states)];
        let mut params = Vec::new();
        proto.transaction(
            CATEGORY_TRANSPORT,
            CMD_SET_TRANSMIT_MODE,
            &args,
            &mut params,
            timeout_ms,
        )
    }
}

/// Protocol about transmission for Fireworks board module.
pub trait TransportProtocol: EfwProtocolExtManual {
    fn get_packet_format(&mut self, fmt: TxPacketFormat, timeout_ms: u32) -> Result<(), Error> {
        let args = [serialize_tx_packet_format(&fmt)];
        self.transaction(
            CATEGORY_TRANSPORT,
            CMD_SET_TRANSMIT_MODE,
            &args,
            &mut Vec::new(),
            timeout_ms,
        )
    }
}

impl<O: EfwProtocolExtManual> TransportProtocol for O {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tx_packet_format_serdes() {
        [TxPacketFormat::Unique, TxPacketFormat::Iec61883]
            .iter()
            .for_each(|fmt| {
                let val = serialize_tx_packet_format(fmt);
                let mut f = TxPacketFormat::default();
                deserialize_tx_packet_format(&mut f, val);
                assert_eq!(*fmt, f);
            });
    }
}
