// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use super::{*, caps_section::*};

/// The alternative type of data for stream format.
pub type FormatEntryData = [u8;FormatEntry::SIZE];

impl FormatEntry {
    const SIZE: usize = 268;
    const NAMES_MAX_SIZE: usize = 256;
}

impl TryFrom<FormatEntryData> for FormatEntry {
    type Error = Error;

    fn try_from(raw: FormatEntryData) -> Result<Self, Self::Error> {
        let mut quadlet = [0;4];

        quadlet.copy_from_slice(&raw[..4]);
        let pcm_count = u32::from_be_bytes(quadlet) as u8;

        quadlet.copy_from_slice(&raw[4..8]);
        let midi_count = u32::from_be_bytes(quadlet) as u8;

        let labels = parse_labels(&raw[8..264])
            .map_err(|e| {
                let msg = format!("Invalid data for string: {}", e);
                Error::new(ProtocolExtensionError::StreamFormatEntry, &msg)
            })?;

        let mut enable_ac3 = [false;AC3_CHANNELS];
        quadlet.copy_from_slice(&raw[264..268]);
        let val = u32::from_be_bytes(quadlet);
        enable_ac3.iter_mut()
            .enumerate()
            .for_each(|(i, v)| *v = (1 << i) & val > 0);

        let entry = FormatEntry{
            pcm_count,
            midi_count,
            labels,
            enable_ac3,
        };

        Ok(entry)
    }
}

impl From<FormatEntry> for FormatEntryData {
    fn from(entry: FormatEntry) -> Self {
        let mut raw = [0;FormatEntry::SIZE];
        raw[..4].copy_from_slice(&(entry.pcm_count as u32).to_be_bytes());
        raw[4..8].copy_from_slice(&(entry.midi_count as u32).to_be_bytes());

        raw[8..264].copy_from_slice(&build_labels(&entry.labels, FormatEntry::NAMES_MAX_SIZE));

        let val = entry.enable_ac3.iter()
            .enumerate()
            .filter(|(_, &enabled)| enabled)
            .fold(0 as u32, |val, (i, _)| val | (1 << i));
        raw[264..268].copy_from_slice(&val.to_be_bytes());

        raw
    }
}

pub fn read_stream_format_entries(
    req: &mut FwReq,
    node: &mut FwNode,
    caps: &ExtensionCaps,
    offset: usize,
    timeout_ms: u32
) -> Result<(Vec<FormatEntry>, Vec<FormatEntry>), Error> {
    let mut data = [0;8];
    ProtocolExtension::read(req, node, offset, &mut data, timeout_ms)?;

    let mut quadlet = [0;4];
    quadlet.copy_from_slice(&data[..4]);
    let tx_count = u32::from_be_bytes(quadlet) as usize;
    if tx_count > caps.general.max_tx_streams as usize {
        let msg = format!("Unexpected count of tx streams: {} but {} expected",
                          tx_count, caps.general.max_tx_streams);
        Err(Error::new(ProtocolExtensionError::StreamFormatEntry, &msg))?
    }

    quadlet.copy_from_slice(&data[4..]);
    let rx_count = u32::from_be_bytes(quadlet) as usize;
    if rx_count > caps.general.max_rx_streams as usize {
        let msg = format!("Unexpected count of rx streams: {} but {} expected",
                          rx_count, caps.general.max_rx_streams);
        Err(Error::new(ProtocolExtensionError::StreamFormatEntry, &msg))?
    }

    let mut tx_entries = Vec::new();
    (0..tx_count)
        .try_for_each(|i| {
            let mut raw = [0;FormatEntry::SIZE];
            ProtocolExtension::read(
                req,
                node,
                offset + 8 + FormatEntry::SIZE * i,
                &mut raw,
                timeout_ms
            )?;
            FormatEntry::try_from(raw)
                .map_err(|e| {
                    let msg = format!("Fail to parse TX stream entry {}: {}", i, e);
                    Error::new(ProtocolExtensionError::StreamFormatEntry, &msg)
                })
                .map(|entry| tx_entries.push(entry))
        })?;

    let mut rx_entries = Vec::new();
    (0..rx_count)
        .try_for_each(|i| {
            let mut raw = [0;FormatEntry::SIZE];
            ProtocolExtension::read(
                req,
                node,
                offset + 8 + FormatEntry::SIZE * (tx_count + i),
                &mut raw,
                timeout_ms
            )?;
            FormatEntry::try_from(raw)
                .map_err(|e| {
                    let msg = format!("Fail to parse RX stream entry {}: {}", i, e);
                    Error::new(ProtocolExtensionError::StreamFormatEntry, &msg)
                })
                .map(|entry| rx_entries.push(entry))
        })?;

    Ok((tx_entries, rx_entries))
}

pub fn write_stream_format_entries(
    req: &mut FwReq,
    node: &mut FwNode,
    caps: &ExtensionCaps,
    offset: usize,
    pair: &(Vec<FormatEntryData>, Vec<FormatEntryData>),
    timeout_ms: u32
) -> Result<(), Error> {
    let (tx, rx) = pair;

    if tx.len() != caps.general.max_tx_streams as usize {
        let msg = format!("Unexpected count of tx streams: {} but {} expected",
                          tx.len(), caps.general.max_tx_streams);
        Err(Error::new(ProtocolExtensionError::StreamFormatEntry, &msg))?
    }

    if rx.len() != caps.general.max_rx_streams as usize {
        let msg = format!("Unexpected count of rx streams: {} but {} expected",
                          rx.len(), caps.general.max_rx_streams);
        Err(Error::new(ProtocolExtensionError::StreamFormatEntry, &msg))?
    }

    let mut data = Vec::new();
    data.extend_from_slice(&(tx.len() as u32).to_be_bytes());
    data.extend_from_slice(&(rx.len() as u32).to_be_bytes());
    tx.iter().for_each(|entry| {
        data.extend_from_slice(entry);
    });
    rx.iter().for_each(|entry| {
        data.extend_from_slice(entry);
    });
    ProtocolExtension::write(req, node, offset, &mut data, timeout_ms)
}

#[cfg(test)]
mod test {
    use super::{FormatEntry, FormatEntryData, AC3_CHANNELS};

    use std::convert::TryFrom;

    #[test]
    fn stream_format_entry_from() {
        let entry = FormatEntry{
            pcm_count: 0xfe,
            midi_count: 0x3c,
            labels: vec![
                "To say".to_string(),
                "Good bye".to_string(),
                "is to die".to_string(),
                "a little.".to_string(),
            ],
            enable_ac3: [true;AC3_CHANNELS],
        };
        let data = Into::<FormatEntryData>::into(entry.clone());
        assert_eq!(entry, FormatEntry::try_from(data).unwrap());
    }
}
