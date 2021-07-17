// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for M-Audio FireWire 1814 and ProjectMix I/O.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by M-Audio FireWire 1814 and ProjectMix I/O.

use crate::*;

/// The protocol implementation for media clock of FireWire 1814.
#[derive(Default)]
pub struct Fw1814ClkProtocol;

impl MediaClockFrequencyOperation for Fw1814ClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000, 176400, 192000];

    fn read_clk_freq(avc: &BebobAvc, timeout_ms: u32) -> Result<usize, Error> {
        read_clk_freq(avc, Self::FREQ_LIST, timeout_ms)
    }
}

/// The protocol implementation for media clock of ProjectMix I/O.
#[derive(Default)]
pub struct ProjectMixClkProtocol;

impl MediaClockFrequencyOperation for ProjectMixClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];

    fn read_clk_freq(avc: &BebobAvc, timeout_ms: u32) -> Result<usize, Error> {
        read_clk_freq(avc, Self::FREQ_LIST, timeout_ms)
    }
}

// NOTE: Special models doesn't support any bridgeco extension.
fn read_clk_freq(avc: &BebobAvc, freq_list: &[u32], timeout_ms: u32) -> Result<usize, Error> {
    let mut op = OutputPlugSignalFormat::new(0);
    avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
    let fdf = AmdtpFdf::from(&op.fdf[..]);
    freq_list
        .iter()
        .position(|&freq| freq == fdf.freq)
        .ok_or_else(|| {
            let msg = format!("Unexpected value of FDF: {:?}", fdf);
            Error::new(FileError::Io, &msg)
        })
}
