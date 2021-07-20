// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for M-Audio FireWire 1814 and ProjectMix I/O.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by M-Audio FireWire 1814 and ProjectMix I/O.
//!
//! ## Diagram of internal signal flow for FireWire 1814 and ProjectMix I/O.
//!
//! ```text
//! analog-input-1/2 ---+-------------------------+--------------------------> stream-output-1/2
//! analog-input-3/4 ---|-+-----------------------|-+------------------------> stream-output-3/4
//! analog-input-5/6 ---|-|-+---------------------|-|-+----------------------> stream-output-5/6
//! analog-input-7/8 ---|-|-|-+-------------------|-|-|-+-----------------+
//! spdif-input-1/2 ----|-|-|-|-+-----------------|-|-|-|-+---------------+--> stream-output-7/8
//! adat-input-1/2 -----|-|-|-|-|-+---------------|-|-|-|-|-+----------------> stream-output-9/10
//! adat-input-3/4 -----|-|-|-|-|-|-+-------------|-|-|-|-|-|-+--------------> stream-output-11/12
//! adat-input-5/6 -----|-|-|-|-|-|-|-+-----------|-|-|-|-|-|-|-+------------> stream-output-13/14
//! adat-input-7/8 -----|-|-|-|-|-|-|-|-+---------|-|-|-|-|-|-|-|-+----------> stream-output-15/16
//!                     | | | | | | | | |         | | | | | | | | |
//!                     | | | | | | | | |         v v v v v v v v v
//!                     | | | | | | | | |       ++=================++
//!  stream-input-1/2 --|-|-|-|-|-|-|-|-|-+---> ||      22x2       ||
//!  stream-input-3/4 --|-|-|-|-|-|-|-|-|-|-+-> ||    aux mixer    || --+
//!                     | | | | | | | | | | |   ++=================++   |
//!                     | | | | | | | | | | |                           |
//!                     v v v v v v v v v v v                     aux-output-1/2
//!                   ++=====================++                       | | |
//!                   ||        22x4         || -- mixer-output-1/2 --+-|-|--> analog-output-1/2
//!                   ||        mixer        || -- mixer-output-3/4 --|-+-|--> analog-output-1/2
//!                   ++=====================++                       +-+-+--> headphone-1/2
//!
//!  stream-input-5/7 -------------------------------------------------------> digital-output-1/2
//!  stream-input-7/8 -------------------------------------------------------> digital-output-3/4
//!  stream-input-9/10 ------------------------------------------------------> digital-output-5/6
//!  stream-input-11/12 -----------------------------------------------------> digital-output-7/8
//! ```

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
