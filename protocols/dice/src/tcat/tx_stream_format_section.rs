// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Tx Stream format section in general protocol for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for Tx stream
//! format section in general protocol defined by TCAT for ASICs of DICE.
use super::*;

/// Entry for stream format in stream transmitted by the node.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct TxStreamFormatEntry {
    /// The channel number for isochronous packet stream.
    pub iso_channel: i8,
    /// The number of PCM channels.
    pub pcm: u32,
    /// The number of MIDI ports.
    pub midi: u32,
    /// The code to express transferring speed defined in IEEE 1394 specification.
    pub speed: u32,
    /// The list of names for data channel.
    pub labels: Vec<String>,
    /// The mode for each channel of IEC 60958.
    pub iec60958: [Iec60958Param; IEC60958_CHANNELS],
}

const MIN_SIZE: usize = 272;

fn serialize_tx_stream_entry(entry: &TxStreamFormatEntry, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= MIN_SIZE);

    (entry.iso_channel as i32).build_quadlet(&mut raw[..4]);

    entry.pcm.build_quadlet(&mut raw[4..8]);
    entry.midi.build_quadlet(&mut raw[8..12]);
    entry.speed.build_quadlet(&mut raw[12..16]);

    raw[16..272].copy_from_slice(&mut build_labels(&entry.labels, STREAM_NAMES_SIZE));

    // NOTE: it's not supported by old version of firmware.
    if raw.len() >= 272 {
        serialize_iec60958_params(&entry.iec60958, &mut raw[272..280])?;
    }

    Ok(())
}

fn deserialize_tx_stream_entry(entry: &mut TxStreamFormatEntry, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= MIN_SIZE);

    let mut val = 0i32;
    val.parse_quadlet(&raw[..4]);
    entry.iso_channel = val as i8;

    entry.pcm.parse_quadlet(&raw[4..8]);
    entry.midi.parse_quadlet(&raw[8..12]);
    entry.speed.parse_quadlet(&raw[12..16]);

    entry.labels =
        parse_labels(&raw[16..272]).map_err(|e| format!("Invalid data for string: {}", e))?;

    // NOTE: it's not supported by old version of firmware.
    if raw.len() >= MIN_SIZE {
        deserialize_iec60958_params(&mut entry.iec60958, &raw[272..280])?;
    }

    Ok(())
}

/// Parameters for format of transmit streams.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct TxStreamFormatParameters(pub Vec<TxStreamFormatEntry>);

impl<O: TcatOperation> TcatSectionSerdes<TxStreamFormatParameters> for O {
    const MIN_SIZE: usize = 8;

    const ERROR_TYPE: GeneralProtocolError = GeneralProtocolError::TxStreamFormat;

    fn serialize(params: &TxStreamFormatParameters, raw: &mut [u8]) -> Result<(), String> {
        let mut val = 0u32;

        // The number of streams is read-only.
        val.parse_quadlet(&raw[..4]);
        let count = val as usize;

        if count != params.0.len() {
            Err(format!(
                "The count of entries should be {}, actually {}",
                count,
                params.0.len(),
            ))?;
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
            serialize_tx_stream_entry(entry, &mut raw[pos..(pos + size)])
        })
    }

    fn deserialize(params: &mut TxStreamFormatParameters, raw: &[u8]) -> Result<(), String> {
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
            deserialize_tx_stream_entry(entry, &raw[pos..(pos + size)])
        })
    }
}

impl<O: TcatOperation> TcatSectionOperation<TxStreamFormatParameters> for O {}

impl<O: TcatSectionOperation<TxStreamFormatParameters>>
    TcatMutableSectionOperation<TxStreamFormatParameters> for O
{
}

impl<O: TcatSectionOperation<TxStreamFormatParameters>>
    TcatNotifiedSectionOperation<TxStreamFormatParameters> for O
{
    const NOTIFY_FLAG: u32 = NOTIFY_TX_CFG_CHG;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tx_stream_format_params_serdes() {
        let params = TxStreamFormatEntry {
            iso_channel: 32,
            pcm: 4,
            midi: 2,
            speed: 4,
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
        serialize_tx_stream_entry(&params, &mut raw).unwrap();

        let mut p = TxStreamFormatEntry::default();
        deserialize_tx_stream_entry(&mut p, &raw).unwrap();

        assert_eq!(params, p);
    }
}
