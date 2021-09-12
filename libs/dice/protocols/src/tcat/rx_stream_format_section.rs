// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Rx Stream format section in general protocol for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for Rx stream
//! format section in general protocol defined by TCAT for ASICs of DICE.
use super::{*, utils::*};

use std::convert::TryFrom;

/// The structure to represent an entry for stream format in stream received by the node.
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct RxStreamEntry {
    pub iso_channel: i8,
    pub start: u32,
    pub pcm: u32,
    pub midi: u32,
    pub labels: Vec<String>,
    pub iec60958: [Iec60958Param;IEC60958_CHANNELS],
}

impl TryFrom<&[u8]> for RxStreamEntry {
    type Error = Error;

    fn try_from(raw: &[u8]) -> Result<Self, Self::Error> {
        let mut quadlet = [0;4];
        quadlet.copy_from_slice(&raw[..4]);
        let iso_channel = i32::from_be_bytes(quadlet) as i8;

        quadlet.copy_from_slice(&raw[4..8]);
        let start = u32::from_be_bytes(quadlet);

        quadlet.copy_from_slice(&raw[8..12]);
        let pcm = u32::from_be_bytes(quadlet);

        quadlet.copy_from_slice(&raw[12..16]);
        let midi = u32::from_be_bytes(quadlet);

        let labels = parse_labels(&raw[16..272])
            .map_err(|e| {
                let msg = format!("Invalid data for string: {}", e);
                Error::new(GeneralProtocolError::RxStreamFormat, &msg)
            })?;

        let iec60958 = if raw.len() > 272 {
            parse_iec60958_params(&raw[272..280])
        } else {
            // NOTE: it's not supported by old version of firmware.
            [Iec60958Param::default();IEC60958_CHANNELS]
        };

        let entry = RxStreamEntry{
            iso_channel,
            start,
            pcm,
            midi,
            labels,
            iec60958,
        };

        Ok(entry)
    }
}

impl From<&RxStreamEntry> for Vec<u8> {
    fn from(entry: &RxStreamEntry) -> Self {
        let mut raw = Vec::new();

        let val = entry.iso_channel as i32;
        raw.extend_from_slice(&val.to_be_bytes());

        let mut val = entry.start;
        raw.extend_from_slice(&val.to_be_bytes());

        val = entry.pcm;
        raw.extend_from_slice(&val.to_be_bytes());

        val = entry.midi;
        raw.extend_from_slice(&val.to_be_bytes());

        raw.append(&mut build_labels(&entry.labels, STREAM_NAMES_SIZE));

        raw.append(&mut build_iec60958_params(&entry.iec60958));

        raw
    }
}

/// The structure for protocol implementation of rx stream format section.
#[derive(Default)]
pub struct RxStreamFormatSectionProtocol;

impl RxStreamFormatSectionProtocol {
    pub fn read_entries(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &GeneralSections,
        timeout_ms: u32
    ) -> Result<Vec<RxStreamEntry>, Error> {
        let mut data = [0;8];
        GeneralProtocol::read(req, node, sections.rx_stream_format.offset, &mut data, timeout_ms)
            .map_err(|e| Error::new(GeneralProtocolError::RxStreamFormat, &e.to_string()))?;

        let mut quadlet = [0;4];
        quadlet.copy_from_slice(&data[0..4]);
        let count = u32::from_be_bytes(quadlet) as usize;

        quadlet.copy_from_slice(&data[4..8]);
        let size = 4 * u32::from_be_bytes(quadlet) as usize;

        let mut entries = Vec::new();
        let mut data = vec![0;size];
        (0..count).try_for_each(|i| {
            GeneralProtocol::read(
                req,
                node,
                sections.rx_stream_format.offset + 8 + (i * size),
                &mut data,
                timeout_ms
            )
                .map_err(|e| Error::new(GeneralProtocolError::RxStreamFormat, &e.to_string()))?;
            let entry = RxStreamEntry::try_from(&data[..])
                .map_err(|e| Error::new(GeneralProtocolError::RxStreamFormat, &e.to_string()))?;
            entries.push(entry);
            Ok(())
        })?;
        Ok(entries)
    }

    pub fn write_entries(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &GeneralSections,
        entries: &[RxStreamEntry],
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut data = [0; 8];
        GeneralProtocol::read(req, node, sections.rx_stream_format.offset, &mut data, timeout_ms)
            .map_err(|e| Error::new(GeneralProtocolError::RxStreamFormat, &e.to_string()))?;

        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&data[..4]);
        let count = std::cmp::min(u32::from_be_bytes(quadlet) as usize, entries.len());

        quadlet.copy_from_slice(&data[4..8]);
        let size = 4 * u32::from_be_bytes(quadlet) as usize;

        (0..count).try_for_each(|i| {
            let mut expected_fmt = entries[i].clone();

            let mut curr = vec![0; size];
            GeneralProtocol::read(
                req,
                node,
                sections.rx_stream_format.offset + 8 + (i * size),
                &mut curr,
                timeout_ms
            )
                .map_err(|e| Error::new(GeneralProtocolError::RxStreamFormat, &e.to_string()))?;
            let curr_fmt = RxStreamEntry::try_from(&curr[..])
                .map_err(|e| Error::new(GeneralProtocolError::RxStreamFormat, &e.to_string()))?;
            expected_fmt.iso_channel = curr_fmt.iso_channel;

            if expected_fmt != curr_fmt {
                let mut raw = Into::<Vec<u8>>::into(&expected_fmt);
                GeneralProtocol::write(
                    req,
                    node,
                    sections.rx_stream_format.offset + 8 + (i * size),
                    &mut raw,
                    timeout_ms
                )
                    .map_err(|e| Error::new(GeneralProtocolError::RxStreamFormat, &e.to_string()))?;
            }

            Ok(())
        })?;

        Ok(())
    }
}
