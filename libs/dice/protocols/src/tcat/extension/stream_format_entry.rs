// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use super::{*, caps_section::*};

pub trait StreamFormatEntryProtocol<T> : ProtocolExtension<T>
    where T: AsRef<FwNode>,
{
    fn read_stream_format_entries(&self, node: &T, caps: &ExtensionCaps, offset: usize, timeout_ms: u32)
        -> Result<(Vec<FormatEntryData>, Vec<FormatEntryData>), Error>
    {
        let mut data = [0;8];
        ProtocolExtension::read(self, node, offset, &mut data, timeout_ms)?;

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
        (0..tx_count).try_for_each(|i| {
            let mut data = [0;FormatEntry::SIZE];
            ProtocolExtension::read(self, node, offset + 8 + FormatEntry::SIZE * i, &mut data,
                                    timeout_ms)
                .map(|_| tx_entries.push(data))
        })?;

        let mut rx_entries = Vec::new();
        (0..rx_count).try_for_each(|i| {
            let mut data = [0;FormatEntry::SIZE];
            ProtocolExtension::read(self, node, offset + 8 + FormatEntry::SIZE * (tx_count + i),
                                    &mut data, timeout_ms)
                .map(|_| rx_entries.push(data))
        })?;

        Ok((tx_entries, rx_entries))
    }

    fn write_stream_format_entries(&self, node: &T, caps: &ExtensionCaps, offset: usize,
                                   pair: &(Vec<FormatEntryData>, Vec<FormatEntryData>), timeout_ms: u32)
        -> Result<(), Error>
    {
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
        ProtocolExtension::write(self, node, offset, &mut data, timeout_ms)
    }
}

impl<O: AsRef<FwReq>, T: AsRef<FwNode>> StreamFormatEntryProtocol<T> for O {}
