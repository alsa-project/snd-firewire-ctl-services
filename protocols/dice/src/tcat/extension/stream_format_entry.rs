// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

impl FormatEntry {
    const SIZE: usize = 268;
}

fn serialize_stream_format_entry(entry: &FormatEntry, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= FormatEntry::SIZE);

    serialize_u8(&entry.pcm_count, &mut raw[..4]);
    serialize_u8(&entry.midi_count, &mut raw[4..8]);

    serialize_labels(&entry.labels, &mut raw[8..264])?;

    let val = entry
        .enable_ac3
        .iter()
        .enumerate()
        .filter(|(_, &enabled)| enabled)
        .fold(0 as u32, |val, (i, _)| val | (1 << i));
    serialize_u32(&val, &mut raw[264..268]);

    Ok(())
}

fn deserialize_stream_format_entry(entry: &mut FormatEntry, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= FormatEntry::SIZE);

    deserialize_u8(&mut entry.pcm_count, &raw[..4]);
    deserialize_u8(&mut entry.midi_count, &raw[4..8]);
    deserialize_labels(&mut entry.labels, &raw[8..264])?;

    let mut val = 0u32;
    deserialize_u32(&mut val, &raw[264..268]);

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

    serialize_usize(&tx_entries.len(), &mut raw[..4]);
    serialize_usize(&rx_entries.len(), &mut raw[4..8]);

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

    let mut tx_entry_count = 0usize;
    deserialize_usize(&mut tx_entry_count, &raw[..4]);

    let mut rx_entry_count = 0usize;
    deserialize_usize(&mut rx_entry_count, &raw[4..8]);

    tx_entries.resize_with(tx_entry_count, Default::default);
    rx_entries.resize_with(rx_entry_count, Default::default);

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
