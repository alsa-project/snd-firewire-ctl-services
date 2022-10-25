// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Rx Stream format section in general protocol for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for Rx stream
//! format section in general protocol defined by TCAT for ASICs of DICE.
use super::*;

/// Entry for stream format in stream received by the node.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct RxStreamFormatEntry {
    /// The channel number for isochronous packet stream.
    pub iso_channel: i8,
    /// The start position of data channels from the beginning of payload in quadlet count.
    pub start: u32,
    /// The number of PCM channels.
    pub pcm: u32,
    /// The number of MIDI ports.
    pub midi: u32,
    /// The list of names for data channel.
    pub labels: Vec<String>,
    /// The mode for each channel of IEC 60958.
    pub iec60958: [Iec60958Param; IEC60958_CHANNELS],
}

const MIN_SIZE: usize = 272;

fn serialize_rx_stream_entry(entry: &RxStreamFormatEntry, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= MIN_SIZE);

    (entry.iso_channel as i32).build_quadlet(&mut raw[..4]);

    entry.start.build_quadlet(&mut raw[4..8]);
    entry.pcm.build_quadlet(&mut raw[8..12]);
    entry.midi.build_quadlet(&mut raw[12..16]);

    raw[16..272].copy_from_slice(&mut build_labels(&entry.labels, STREAM_NAMES_SIZE));

    // NOTE: it's not supported by old version of firmware.
    if raw.len() >= 272 {
        serialize_iec60958_params(&entry.iec60958, &mut raw[272..280])?;
    }
    Ok(())
}

fn deserialize_rx_stream_entry(entry: &mut RxStreamFormatEntry, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= MIN_SIZE);

    let mut val = 0i32;
    val.parse_quadlet(&raw[..4]);
    entry.iso_channel = val as i8;

    entry.start.parse_quadlet(&raw[4..8]);
    entry.pcm.parse_quadlet(&raw[8..12]);
    entry.midi.parse_quadlet(&raw[12..16]);

    entry.labels =
        parse_labels(&raw[16..272]).map_err(|e| format!("Invalid data for string: {}", e))?;

    // NOTE: it's not supported by old version of firmware.
    if raw.len() >= MIN_SIZE {
        deserialize_iec60958_params(&mut entry.iec60958, &raw[272..280])?;
    }
    Ok(())
}

/// Parameters for format of receive streams.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct RxStreamFormatParameters(pub Vec<RxStreamFormatEntry>);

impl<O: TcatOperation> TcatSectionSerdes<RxStreamFormatParameters> for O {
    const MIN_SIZE: usize = 8;

    const ERROR_TYPE: GeneralProtocolError = GeneralProtocolError::RxStreamFormat;

    fn serialize(params: &RxStreamFormatParameters, raw: &mut [u8]) -> Result<(), String> {
        let mut val = 0u32;

        // The number of streams is read-only.
        val.parse_quadlet(&raw[..4]);
        let count = val as usize;

        if count != params.0.len() {
            let msg = format!(
                "The count of entries should be {}, actually {}",
                count,
                params.0.len()
            );
            Err(msg)?;
        }

        // The size of stream format entry is read-only as well.
        val.parse_quadlet(&raw[4..8]);
        let size = 4 * val as usize;

        let expected = 8 + size * count;
        if raw.len() < expected {
            let msg = format!(
                "The size of buffer should be greater than {}, actually {}",
                expected,
                raw.len()
            );
            Err(msg)?;
        }

        params.0.iter().enumerate().try_for_each(|(i, entry)| {
            let pos = 8 + size * i;
            serialize_rx_stream_entry(entry, &mut raw[pos..(pos + size)])
        })
    }

    fn deserialize(params: &mut RxStreamFormatParameters, raw: &[u8]) -> Result<(), String> {
        let mut val = 0u32;
        val.parse_quadlet(&raw[..4]);
        let count = val as usize;

        val.parse_quadlet(&raw[4..8]);
        let size = 4 * val as usize;

        let expected = 8 + size * count;
        if raw.len() < expected {
            let msg = format!(
                "The size of buffer should be greater than {}, actually {}",
                expected,
                raw.len()
            );
            Err(msg)?;
        }

        params.0.resize_with(count, Default::default);

        params.0.iter_mut().enumerate().try_for_each(|(i, entry)| {
            let pos = 8 + size * i;
            deserialize_rx_stream_entry(entry, &raw[pos..(pos + size)])
        })
    }
}

impl<O: TcatOperation> TcatSectionOperation<RxStreamFormatParameters> for O {}

impl<O: TcatSectionOperation<RxStreamFormatParameters>>
    TcatMutableSectionOperation<RxStreamFormatParameters> for O
{
}

impl<O: TcatSectionOperation<RxStreamFormatParameters>>
    TcatNotifiedSectionOperation<RxStreamFormatParameters> for O
{
    const NOTIFY_FLAG: u32 = NOTIFY_RX_CFG_CHG;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rx_stream_format_entry_serdes() {
        let params = RxStreamFormatEntry {
            iso_channel: 32,
            start: 4,
            pcm: 4,
            midi: 2,
            labels: vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ],
            iec60958: [Iec60958Param {
                cap: true,
                enable: false,
            }; IEC60958_CHANNELS],
        };
        let mut raw = [0u8; 2048];
        serialize_rx_stream_entry(&params, &mut raw).unwrap();

        let mut p = RxStreamFormatEntry::default();
        deserialize_rx_stream_entry(&mut p, &raw).unwrap();

        assert_eq!(params, p);
    }
}
