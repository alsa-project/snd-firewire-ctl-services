// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Tx Stream format section in general protocol for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for Tx stream
//! format section in general protocol defined by TCAT for ASICs of DICE.
use super::{utils::*, *};

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

impl TryFrom<&[u8]> for TxStreamFormatEntry {
    type Error = Error;

    fn try_from(raw: &[u8]) -> Result<Self, Self::Error> {
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&raw[..4]);
        let iso_channel = i32::from_be_bytes(quadlet) as i8;

        quadlet.copy_from_slice(&raw[4..8]);
        let pcm = u32::from_be_bytes(quadlet);

        quadlet.copy_from_slice(&raw[8..12]);
        let midi = u32::from_be_bytes(quadlet);

        quadlet.copy_from_slice(&raw[12..16]);
        let speed = u32::from_be_bytes(quadlet);

        let labels = parse_labels(&raw[16..272]).map_err(|e| {
            let msg = format!("Invalid data for string: {}", e);
            Error::new(GeneralProtocolError::TxStreamFormat, &msg)
        })?;

        let iec60958 = if raw.len() > 272 {
            parse_iec60958_params(&raw[272..280])
        } else {
            // NOTE: it's not supported by old version of firmware.
            [Iec60958Param::default(); IEC60958_CHANNELS]
        };

        let entry = TxStreamFormatEntry {
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

impl From<&TxStreamFormatEntry> for Vec<u8> {
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

/// Parameters for format of transmit streams.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct TxStreamFormatParameters(pub Vec<TxStreamFormatEntry>);

impl<O: TcatOperation> TcatSectionSerdes<TxStreamFormatParameters> for O {
    const MIN_SIZE: usize = 8;

    const ERROR_TYPE: GeneralProtocolError = GeneralProtocolError::TxStreamFormat;

    fn serialize(params: &TxStreamFormatParameters, raw: &mut [u8]) -> Result<(), String> {
        // The number of streams is read-only.
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&raw[..4]);
        let count = u32::from_be_bytes(quadlet) as usize;

        if count != params.0.len() {
            Err(format!(
                "The count of entries should be {}, actually {}",
                count,
                params.0.len(),
            ))?;
        }

        // The size of stream format entry is read-only as well.
        quadlet.copy_from_slice(&raw[4..8]);
        let size = 4 * u32::from_be_bytes(quadlet) as usize;

        let mut entries = Vec::new();
        params.0.iter().try_for_each(|entry| {
            let mut data = Vec::from(entry);
            if data.len() > size {
                Err(format!(
                    "The size of interpreted entry should be less than {}, actually {}",
                    size,
                    data.len()
                ))
            } else {
                if data.len() < size {
                    data.resize(size, 0);
                }
                entries.push(data);
                Ok(())
            }
        })?;

        entries.iter_mut().enumerate().try_for_each(|(i, entry)| {
            let pos = 8 + i * size;
            raw[pos..(pos + size)].copy_from_slice(entry);
            Ok(())
        })
    }

    fn deserialize(params: &mut TxStreamFormatParameters, raw: &[u8]) -> Result<(), String> {
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&raw[..4]);
        let count = u32::from_be_bytes(quadlet) as usize;

        quadlet.copy_from_slice(&raw[4..8]);
        let size = 4 * u32::from_be_bytes(quadlet) as usize;

        let mut entries = Vec::new();
        (0..count)
            .try_for_each(|i| {
                let pos = 8 + i * size;
                if raw[pos..].len() < size {
                    Err(format!("Expected {}, actually {}", size, raw[pos..].len()))
                } else {
                    TxStreamFormatEntry::try_from(&raw[pos..(pos + size)])
                        .map(|entry| entries.push(entry))
                        .map_err(|e| e.message().to_string())
                }
            })
            .map(|_| params.0 = entries)
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

/// Protocol implementation of tx stream format section.
#[derive(Default)]
pub struct TxStreamFormatSectionProtocol;

impl TxStreamFormatSectionProtocol {
    pub fn read_entries(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &GeneralSections,
        timeout_ms: u32,
    ) -> Result<Vec<TxStreamFormatEntry>, Error> {
        let mut data = [0; 8];
        GeneralProtocol::read(
            req,
            node,
            sections.tx_stream_format.offset,
            &mut data,
            timeout_ms,
        )
        .map_err(|e| Error::new(GeneralProtocolError::TxStreamFormat, &e.to_string()))?;

        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&data[..4]);
        let count = u32::from_be_bytes(quadlet) as usize;

        quadlet.copy_from_slice(&data[4..8]);
        let size = 4 * u32::from_be_bytes(quadlet) as usize;

        let mut entries = Vec::new();
        let mut data = vec![0; size];
        (0..count)
            .try_for_each(|i| {
                GeneralProtocol::read(
                    req,
                    node,
                    sections.tx_stream_format.offset + 8 + (i * size),
                    &mut data,
                    timeout_ms,
                )
                .map_err(|e| Error::new(GeneralProtocolError::TxStreamFormat, &e.to_string()))?;
                let entry = TxStreamFormatEntry::try_from(&data[..]).map_err(|e| {
                    Error::new(GeneralProtocolError::TxStreamFormat, &e.to_string())
                })?;
                entries.push(entry);
                Ok(())
            })
            .map(|_| entries)
    }

    pub fn write_entries(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &GeneralSections,
        entries: &[TxStreamFormatEntry],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut data = [0; 8];
        GeneralProtocol::read(
            req,
            node,
            sections.tx_stream_format.offset,
            &mut data,
            timeout_ms,
        )
        .map_err(|e| Error::new(GeneralProtocolError::TxStreamFormat, &e.to_string()))?;

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
                sections.tx_stream_format.offset + 8 + (i * size),
                &mut curr,
                timeout_ms,
            )
            .map_err(|e| Error::new(GeneralProtocolError::TxStreamFormat, &e.to_string()))?;
            let curr_fmt = TxStreamFormatEntry::try_from(&curr[..])
                .map_err(|e| Error::new(GeneralProtocolError::TxStreamFormat, &e.to_string()))?;
            expected_fmt.iso_channel = curr_fmt.iso_channel;

            if expected_fmt != curr_fmt {
                let mut raw = Into::<Vec<u8>>::into(&expected_fmt);
                GeneralProtocol::write(
                    req,
                    node,
                    sections.tx_stream_format.offset + 8 + (i * size),
                    &mut raw,
                    timeout_ms,
                )
                .map_err(|e| Error::new(GeneralProtocolError::TxStreamFormat, &e.to_string()))?;
            }

            Ok(())
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct Protocol;

    impl TcatOperation for Protocol {}

    #[test]
    fn tx_stream_format_params_serdes() {
        let params = TxStreamFormatParameters(vec![
            TxStreamFormatEntry {
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
            4
        ]);
        let mut raw = [0u8; 2048];
        raw[..4].copy_from_slice(&4u32.to_be_bytes());
        raw[4..8].copy_from_slice(&80u32.to_be_bytes());
        Protocol::serialize(&params, &mut raw).unwrap();
        let mut p = TxStreamFormatParameters(vec![Default::default(); 4]);
        Protocol::deserialize(&mut p, &raw).unwrap();

        assert_eq!(params, p);
    }
}
