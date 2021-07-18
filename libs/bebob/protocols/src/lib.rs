// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocols defined for BridgeCo. Enhanced Break Out Box (BeBoB) solution.
//!
//! The crate includes various kind of protocols defined by BridgeCo. AG and application vendors
//! for DM1000, DM1100, and DM1500 ASICs with its BridgeCo. Enhanced Break Out Box (BeBoB) solution.

pub mod bridgeco;

pub mod apogee;
pub mod behringer;
pub mod esi;
pub mod maudio;
pub mod stanton;

use glib::{Error, FileError};

use hinawa::FwFcp;

use ta1394::{amdtp::*, ccm::*, general::*, *};

use bridgeco::*;

/// The structure for AV/C transaction helper with quirks specific to BeBoB solution.
#[derive(Default, Debug)]
pub struct BebobAvc(FwFcp);

impl AsRef<FwFcp> for BebobAvc {
    fn as_ref(&self) -> &FwFcp {
        &self.0
    }
}

impl Ta1394Avc for BebobAvc {
    fn control<O: AvcOp + AvcControl>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut operands = Vec::new();
        AvcControl::build_operands(op, addr, &mut operands)?;
        let (rcode, operands) = self.trx(
            AvcCmdType::Control,
            addr,
            O::OPCODE,
            &mut operands,
            timeout_ms,
        )?;
        let expected = match O::OPCODE {
            InputPlugSignalFormat::OPCODE
            | OutputPlugSignalFormat::OPCODE
            | SignalSource::OPCODE => {
                // NOTE: quirk.
                rcode == AvcRespCode::Accepted || rcode == AvcRespCode::Reserved(0x00)
            }
            _ => rcode == AvcRespCode::Accepted,
        };
        if !expected {
            let label = format!(
                "Unexpected response code for control opcode {}: {:?}",
                O::OPCODE,
                rcode
            );
            Err(Error::new(Ta1394AvcError::UnexpectedRespCode, &label))
        } else {
            AvcControl::parse_operands(op, addr, &operands)
        }
    }
}

/// The trait of frequency operation for media clock.
pub trait MediaClockFrequencyOperation {
    const FREQ_LIST: &'static [u32];

    fn read_clk_freq(avc: &BebobAvc, timeout_ms: u32) -> Result<usize, Error> {
        let plug_addr =
            BcoPlugAddr::new_for_unit(BcoPlugDirection::Output, BcoPlugAddrUnitType::Isoc, 0);
        let mut op = ExtendedStreamFormatSingle::new(&plug_addr);

        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;

        op.stream_format
            .as_bco_compound_am824_stream()
            .and_then(|format| {
                Self::FREQ_LIST
                    .iter()
                    .position(|&r| r == format.freq)
                    .ok_or_else(|| {
                        let msg = format!("Unexpected entry for source of clock: {}", format.freq);
                        Error::new(FileError::Io, &msg)
                    })
            })
    }

    /// Change frequency of media clock. This operation can involve INTERIM AV/C response to expand
    /// response time of AV/C transaction.
    fn write_clk_freq(avc: &BebobAvc, idx: usize, timeout_ms: u32) -> Result<(), Error> {
        let fdf = Self::FREQ_LIST
            .iter()
            .nth(idx)
            .ok_or_else(|| {
                let msg = format!("Invalid argument for index of frequency: {}", idx);
                Error::new(FileError::Inval, &msg)
            })
            .map(|&freq| AmdtpFdf::new(AmdtpEventType::Am824, false, freq))?;

        let mut op = InputPlugSignalFormat {
            plug_id: 0,
            fmt: FMT_IS_AMDTP,
            fdf: fdf.into(),
        };
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = OutputPlugSignalFormat {
            plug_id: 0,
            fmt: FMT_IS_AMDTP,
            fdf: fdf.into(),
        };
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }
}
