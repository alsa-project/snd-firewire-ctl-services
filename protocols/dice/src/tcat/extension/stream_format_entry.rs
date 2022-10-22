// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

impl FormatEntry {
    const SIZE: usize = 268;
    const NAMES_MAX_SIZE: usize = 256;
}

fn serialize_stream_format_entry(entry: &FormatEntry, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= FormatEntry::SIZE);

    entry.pcm_count.build_quadlet(&mut raw[..4]);
    entry.midi_count.build_quadlet(&mut raw[4..8]);

    raw[8..264].copy_from_slice(&build_labels(&entry.labels, FormatEntry::NAMES_MAX_SIZE));

    let val = entry
        .enable_ac3
        .iter()
        .enumerate()
        .filter(|(_, &enabled)| enabled)
        .fold(0 as u32, |val, (i, _)| val | (1 << i));
    raw[264..268].copy_from_slice(&val.to_be_bytes());

    Ok(())
}

fn deserialize_stream_format_entry(entry: &mut FormatEntry, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= FormatEntry::SIZE);

    entry.pcm_count.parse_quadlet(&raw[..4]);
    entry.midi_count.parse_quadlet(&raw[4..8]);
    entry.labels = parse_labels(&raw[8..264])
        .map_err(|e| format!("Fail to parse label of stream channel {}", e))?;

    let mut val = 0u32;
    val.parse_quadlet(&raw[264..268]);

    entry
        .enable_ac3
        .iter_mut()
        .enumerate()
        .for_each(|(i, v)| *v = (1 << i) & val > 0);

    Ok(())
}

pub(crate) fn calculate_stream_format_entries_size(
    tx_entry_count: usize,
    rx_entry_count: usize,
) -> usize {
    8 + (tx_entry_count + rx_entry_count) * FormatEntry::SIZE
}

pub(crate) fn serialize_stream_format_entries(
    (tx_entries, rx_entries): (&[FormatEntry], &[FormatEntry]),
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= calculate_stream_format_entries_size(tx_entries.len(), rx_entries.len()));

    let tx_entry_count = tx_entries.len() as u32;
    tx_entry_count.build_quadlet(&mut raw[..4]);

    let rx_entry_count = rx_entries.len() as u32;
    rx_entry_count.build_quadlet(&mut raw[4..8]);

    tx_entries
        .iter()
        .chain(rx_entries)
        .enumerate()
        .try_for_each(|(i, entry)| {
            let pos = 8 + i * FormatEntry::SIZE;
            serialize_stream_format_entry(entry, &mut raw[pos..(pos + FormatEntry::SIZE)])
        })
}

pub(crate) fn deserialize_stream_format_entries(
    (tx_entries, rx_entries): (&mut Vec<FormatEntry>, &mut Vec<FormatEntry>),
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= calculate_stream_format_entries_size(tx_entries.len(), rx_entries.len()));

    let mut tx_entry_count = 0u32;
    tx_entry_count.parse_quadlet(&raw[..4]);

    let mut rx_entry_count = 0u32;
    rx_entry_count.parse_quadlet(&raw[4..8]);

    tx_entries.resize_with(tx_entry_count as usize, Default::default);
    rx_entries.resize_with(rx_entry_count as usize, Default::default);

    tx_entries
        .iter_mut()
        .chain(rx_entries)
        .enumerate()
        .try_for_each(|(i, entry)| {
            let pos = 8 + i * FormatEntry::SIZE;
            deserialize_stream_format_entry(entry, &raw[pos..(pos + FormatEntry::SIZE)])
        })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stream_format_entry_from() {
        let entry = FormatEntry {
            pcm_count: 0xfe,
            midi_count: 0x3c,
            labels: vec![
                "To say".to_string(),
                "Good bye".to_string(),
                "is to die".to_string(),
                "a little.".to_string(),
            ],
            enable_ac3: [true; AC3_CHANNELS],
        };

        let mut raw = vec![0u8; FormatEntry::SIZE];
        serialize_stream_format_entry(&entry, &mut raw).unwrap();

        let mut e = FormatEntry::default();
        deserialize_stream_format_entry(&mut e, &raw).unwrap();

        assert_eq!(entry, e);
    }
}
