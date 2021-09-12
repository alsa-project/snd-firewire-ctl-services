// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Tx Stream format section in general protocol for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for Tx stream
//! format section in general protocol defined by TCAT for ASICs of DICE.
use super::{*, utils::*};

use std::convert::TryFrom;

/// The structure to represent an entry for stream format in stream transmitted by the node.
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct TxStreamFormatEntry{
    pub iso_channel: i8,
    pub pcm: u32,
    pub midi: u32,
    pub speed: u32,
    pub labels: Vec<String>,
    pub iec60958: [Iec60958Param;IEC60958_CHANNELS],
}

impl TryFrom<&[u8]> for TxStreamFormatEntry {
    type Error = Error;

    fn try_from(raw: &[u8]) -> Result<Self, Self::Error> {
        let mut quadlet = [0;4];
        quadlet.copy_from_slice(&raw[..4]);
        let iso_channel = i32::from_be_bytes(quadlet) as i8;

        quadlet.copy_from_slice(&raw[4..8]);
        let pcm = u32::from_be_bytes(quadlet);

        quadlet.copy_from_slice(&raw[8..12]);
        let midi = u32::from_be_bytes(quadlet);

        quadlet.copy_from_slice(&raw[12..16]);
        let speed = u32::from_be_bytes(quadlet);

        let labels = parse_labels(&raw[16..272])
            .map_err(|e| {
                let msg = format!("Invalid data for string: {}", e);
                Error::new(GeneralProtocolError::TxStreamFormat, &msg)
            })?;

        let iec60958 = if raw.len() > 272 {
            parse_iec60958_params(&raw[272..280])
        } else {
            // NOTE: it's not supported by old version of firmware.
            [Iec60958Param::default();IEC60958_CHANNELS]
        };

        let entry = TxStreamFormatEntry{
            iso_channel,
            pcm,
            midi,
            speed,
            labels,
            iec60958,
        };

        Ok(entry)
    }
}

impl From<&TxStreamFormatEntry> for Vec<u8>
{
    fn from(entry: &TxStreamFormatEntry) -> Self {
        let mut raw = Vec::new();

        let val = entry.iso_channel as i32;
        raw.extend_from_slice(&val.to_be_bytes());

        let mut val = entry.pcm;
        raw.extend_from_slice(&val.to_be_bytes());

        val = entry.midi;
        raw.extend_from_slice(&val.to_be_bytes());

        val = entry.speed;
        raw.extend_from_slice(&val.to_be_bytes());

        raw.append(&mut build_labels(&entry.labels, STREAM_NAMES_SIZE));

        raw.append(&mut build_iec60958_params(&entry.iec60958));

        raw
    }
}

pub trait TxStreamFormatSectionProtocol: GeneralProtocol {
    const SIZE_OFFSET: usize = 0x04;

    fn read_tx_stream_format_entries(
        &self,
        node: &mut FwNode,
        sections: &GeneralSections,
        timeout_ms: u32
    ) -> Result<Vec<TxStreamFormatEntry>, Error> {
        let mut data = [0;8];
        self.read(node, sections.tx_stream_format.offset, &mut data, timeout_ms)
            .map_err(|e| Error::new(GeneralProtocolError::TxStreamFormat, &e.to_string()))?;

        let mut quadlet = [0;4];
        quadlet.copy_from_slice(&data[..4]);
        let count = u32::from_be_bytes(quadlet) as usize;

        quadlet.copy_from_slice(&data[4..8]);
        let size = 4 * u32::from_be_bytes(quadlet) as usize;

        let mut entries = Vec::new();
        let mut data = vec![0;size];
        (0..count).try_for_each(|i| {
            self.read(node, sections.tx_stream_format.offset + 8 + (i * size), &mut data, timeout_ms)
                .map_err(|e| Error::new(GeneralProtocolError::TxStreamFormat, &e.to_string()))?;
            let entry = TxStreamFormatEntry::try_from(&data[..])
                .map_err(|e| Error::new(GeneralProtocolError::TxStreamFormat, &e.to_string()))?;
            entries.push(entry);
            Ok(())
        })
        .map(|_| entries)
    }

    fn write_tx_stream_format_entries(
        &self,
        node: &mut FwNode,
        sections: &GeneralSections,
        entries: &[TxStreamFormatEntry],
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut data = [0;8];
        self.read(node, sections.tx_stream_format.offset, &mut data, timeout_ms)
            .map_err(|e| Error::new(GeneralProtocolError::TxStreamFormat, &e.to_string()))?;

        let mut quadlet = [0;4];
        quadlet.copy_from_slice(&data[..4]);
        let count = std::cmp::min(u32::from_be_bytes(quadlet) as usize, entries.len());

        quadlet.copy_from_slice(&data[4..8]);
        let size = 4 * u32::from_be_bytes(quadlet) as usize;

        (0..count).try_for_each(|i| {
            let mut expected_fmt = entries[i].clone();

            let mut curr = vec![0;size];
            self.read(node, sections.tx_stream_format.offset + 8 + (i * size), &mut curr, timeout_ms)
                .map_err(|e| Error::new(GeneralProtocolError::TxStreamFormat, &e.to_string()))?;
            let curr_fmt = TxStreamFormatEntry::try_from(&curr[..])
                .map_err(|e| Error::new(GeneralProtocolError::TxStreamFormat, &e.to_string()))?;
            expected_fmt.iso_channel = curr_fmt.iso_channel;

            if expected_fmt != curr_fmt {
                let mut raw = Into::<Vec<u8>>::into(&expected_fmt);
                self.write(node, sections.tx_stream_format.offset + 8 + (i * size), &mut raw, timeout_ms)
                    .map_err(|e| Error::new(GeneralProtocolError::TxStreamFormat, &e.to_string()))?;
            }

            Ok(())
        })
    }
}

impl<O: AsRef<FwReq>> TxStreamFormatSectionProtocol for O {}
