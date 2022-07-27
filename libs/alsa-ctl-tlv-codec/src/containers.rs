// SPDX-License-Identifier: MIT
// Copyright (c) 2020 Takashi Sakamoto

//! A set of containers to aggregate items in TLV (Type-Length-Value) of ALSA control interface.

use super::*;

trait DataEntry<'a>: std::convert::TryFrom<&'a [u32]> {
    fn raw_length(&self) -> usize;
}

/// The enumeration to dispatch each type of data for entry of dB range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DbRangeEntryData {
    DbScale(DbScale),
    DbInterval(DbInterval),
    DbRange(DbRange),
}

/// The entry to express information of each entry of dB range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbRangeEntry {
    pub min_val: i32,
    pub max_val: i32,
    /// The data of dB expression for the minimum/maximum range in the state of control element.
    pub data: DbRangeEntryData,
}

impl<'a> DataEntry<'a> for DbRangeEntry {
    fn raw_length(&self) -> usize {
        let data_value_length = match &self.data {
            DbRangeEntryData::DbScale(d) => d.value_length(),
            DbRangeEntryData::DbInterval(d) => d.value_length(),
            DbRangeEntryData::DbRange(d) => d.value_length(),
        };
        let data_length = 2 + data_value_length;
        2 + data_length
    }
}

const TYPES_FOR_DB_RANGE_ENTRY: &'static [u32] = &[
    SNDRV_CTL_TLVT_DB_SCALE,
    SNDRV_CTL_TLVT_DB_RANGE,
    SNDRV_CTL_TLVT_DB_LINEAR,
    SNDRV_CTL_TLVT_DB_MINMAX,
    SNDRV_CTL_TLVT_DB_MINMAX_MUTE,
];

impl std::convert::TryFrom<&[u32]> for DbRangeEntry {
    type Error = TlvDecodeError;

    fn try_from(raw: &[u32]) -> Result<Self, Self::Error> {
        if raw.len() < 4 {
            Err(Self::Error::new(TlvDecodeErrorCtx::Length(raw.len(), 4), 0))
        } else {
            let min_val = raw[0] as i32;
            let max_val = raw[1] as i32;

            let entry_data = &raw[2..];
            match entry_data[0] {
                SNDRV_CTL_TLVT_DB_SCALE => DbScale::try_from(entry_data).map(|d| Self {
                    min_val,
                    max_val,
                    data: DbRangeEntryData::DbScale(d),
                }),
                SNDRV_CTL_TLVT_DB_RANGE => DbRange::try_from(entry_data).map(|d| Self {
                    min_val,
                    max_val,
                    data: DbRangeEntryData::DbRange(d),
                }),
                SNDRV_CTL_TLVT_DB_LINEAR
                | SNDRV_CTL_TLVT_DB_MINMAX
                | SNDRV_CTL_TLVT_DB_MINMAX_MUTE => DbInterval::try_from(entry_data).map(|d| Self {
                    min_val,
                    max_val,
                    data: DbRangeEntryData::DbInterval(d),
                }),
                _ => Err(Self::Error::new(
                    TlvDecodeErrorCtx::ValueType(entry_data[0], TYPES_FOR_DB_RANGE_ENTRY),
                    0,
                )),
            }
            .map_err(|e| Self::Error::new(TlvDecodeErrorCtx::ValueContent(Box::new(e)), 2))
        }
    }
}

impl From<&DbRangeEntry> for Vec<u32> {
    fn from(entry: &DbRangeEntry) -> Self {
        let mut raw = Vec::new();
        raw.push(entry.min_val as u32);
        raw.push(entry.max_val as u32);
        let mut data_raw = match &entry.data {
            DbRangeEntryData::DbScale(d) => Vec::<u32>::from(d),
            DbRangeEntryData::DbRange(d) => Vec::<u32>::from(d),
            DbRangeEntryData::DbInterval(d) => Vec::<u32>::from(d),
        };
        raw.append(&mut data_raw);
        raw
    }
}

impl From<DbRangeEntry> for Vec<u32> {
    fn from(entry: DbRangeEntry) -> Self {
        (&entry).into()
    }
}

/// The data to express multiple ranges in the state of control element for dB expression.
/// It has `SNDRV_CTL_TLVT_DB_RANGE` (=3) in its type field and has variable number of elements in
/// value field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbRange {
    /// The entries of ranges for dB expression.
    pub entries: Vec<DbRangeEntry>,
}

impl<'a> TlvData<'a> for DbRange {
    fn value_type(&self) -> u32 {
        SNDRV_CTL_TLVT_DB_RANGE
    }

    fn value_length(&self) -> usize {
        self.entries
            .iter()
            .fold(0, |length, entry| length + entry.raw_length())
    }

    fn value(&self) -> Vec<u32> {
        let mut raw = Vec::new();
        self.entries.iter().for_each(|entry| {
            let mut entry_raw = Vec::<u32>::from(entry);
            raw.append(&mut entry_raw);
        });
        raw
    }
}

const TYPES_FOR_DB_RANGE: &'static [u32] = &[SNDRV_CTL_TLVT_DB_RANGE];

impl std::convert::TryFrom<&[u32]> for DbRange {
    type Error = TlvDecodeError;

    fn try_from(raw: &[u32]) -> Result<Self, Self::Error> {
        // At least, type and length field should be included.
        if raw.len() < 2 {
            Err(Self::Error::new(TlvDecodeErrorCtx::Length(raw.len(), 2), 0))
        // Check type field.
        } else if raw[0] != SNDRV_CTL_TLVT_DB_RANGE {
            Err(Self::Error::new(
                TlvDecodeErrorCtx::ValueType(raw[0], TYPES_FOR_DB_RANGE),
                0,
            ))
        } else {
            // Check length field against length of value field.
            let value_length = (raw[1] / 4) as usize;
            let value = &raw[2..];
            if value.len() < value_length {
                Err(Self::Error::new(
                    TlvDecodeErrorCtx::ValueLength(value_length, value.len()),
                    1,
                ))
            } else {
                // Decode value field.
                let mut pos = 0;

                let mut entries = Vec::new();
                while pos < value_length {
                    DbRangeEntry::try_from(&value[pos..])
                        .map(|entry| {
                            entries.push(entry);
                            pos += 4 + (value[pos + 3] / 4) as usize;
                        })
                        .map_err(|e| {
                            Self::Error::new(TlvDecodeErrorCtx::ValueContent(Box::new(e)), pos + 2)
                        })?;
                }

                Ok(Self { entries })
            }
        }
    }
}

impl From<&DbRange> for Vec<u32> {
    fn from(data: &DbRange) -> Self {
        let mut raw = Vec::new();
        raw.push(data.value_type());
        raw.push(4 * data.value_length() as u32);
        raw.append(&mut data.value());
        raw
    }
}

impl From<DbRange> for Vec<u32> {
    fn from(data: DbRange) -> Self {
        (&data).into()
    }
}

impl<'a> DataEntry<'a> for TlvItem {
    fn raw_length(&self) -> usize {
        let entry_value_length = match self {
            TlvItem::Container(d) => d.value_length(),
            TlvItem::DbRange(d) => d.value_length(),
            TlvItem::DbScale(d) => d.value_length(),
            TlvItem::DbInterval(d) => d.value_length(),
            TlvItem::Chmap(d) => d.value_length(),
            TlvItem::Unknown(d) => d.len(),
        };
        2 + entry_value_length
    }
}

/// The data to express container to aggregate multiple data for TLV (Type-Length-Value) of ALSA
/// control interface.
///
/// It has `SNDRV_CTL_TLVT_CONTAINER` (=0) in its type field and has variable number of elements in
/// value field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    /// The entries of data for TLV.
    pub entries: Vec<TlvItem>,
}

impl<'a> TlvData<'a> for Container {
    fn value_type(&self) -> u32 {
        SNDRV_CTL_TLVT_CONTAINER
    }

    fn value_length(&self) -> usize {
        self.entries
            .iter()
            .fold(0, |length, entry| length + entry.raw_length())
    }

    fn value(&self) -> Vec<u32> {
        let mut raw = Vec::new();
        self.entries.iter().for_each(|entry| {
            raw.append(&mut entry.into());
        });
        raw
    }
}

const TYPES_FOR_CONTAINER: &'static [u32] = &[SNDRV_CTL_TLVT_CONTAINER];

impl std::convert::TryFrom<&[u32]> for Container {
    type Error = TlvDecodeError;

    fn try_from(raw: &[u32]) -> Result<Self, Self::Error> {
        // At least, type and length field should be included.
        if raw.len() < 2 {
            Err(Self::Error::new(TlvDecodeErrorCtx::Length(raw.len(), 2), 0))
        // Check type field.
        } else if raw[0] != SNDRV_CTL_TLVT_CONTAINER {
            Err(Self::Error::new(
                TlvDecodeErrorCtx::ValueType(raw[0], TYPES_FOR_CONTAINER),
                0,
            ))
        } else {
            // Check length field against length of value field.
            let value_length = (raw[1] / 4) as usize;
            let value = &raw[2..];
            if value.len() < value_length {
                Err(Self::Error::new(
                    TlvDecodeErrorCtx::ValueLength(value_length, value.len()),
                    1,
                ))
            } else {
                // Decode value field.
                let mut pos = 0;

                let mut entries = Vec::new();
                while pos < value_length {
                    TlvItem::try_from(&value[pos..])
                        .map(|entry| {
                            entries.push(entry);
                            pos += 2 + (value[pos + 1] / 4) as usize;
                        })
                        .map_err(|e| {
                            Self::Error::new(TlvDecodeErrorCtx::ValueContent(Box::new(e)), pos + 2)
                        })?;
                }

                Ok(Self { entries })
            }
        }
    }
}

impl From<&Container> for Vec<u32> {
    fn from(data: &Container) -> Self {
        let mut raw = Vec::new();
        raw.push(data.value_type());
        raw.push(4 * data.value_length() as u32);
        raw.append(&mut data.value());
        raw
    }
}

impl From<Container> for Vec<u32> {
    fn from(data: Container) -> Self {
        (&data).into()
    }
}

#[cfg(test)]
mod test {
    use super::{Container, TlvItem};
    use super::{DbInterval, DbScale};
    use super::{DbRange, DbRangeEntry, DbRangeEntryData};
    use std::convert::TryFrom;

    #[test]
    fn test_dbrangeentry_dbscale() {
        let raw = [-9i32 as u32, 100, 1, 8, 0, 10];
        let entry = DbRangeEntry::try_from(&raw[..]).unwrap();
        assert_eq!(entry.min_val, -9);
        assert_eq!(entry.max_val, 100);
        assert_eq!(
            entry.data,
            DbRangeEntryData::DbScale(DbScale {
                min: 0,
                step: 10,
                mute_avail: false
            })
        );
        assert_eq!(&Vec::<u32>::from(entry)[..], &raw[..]);
    }

    #[test]
    fn test_dbrangeentry_dbinterval_linear() {
        let raw = [-9i32 as u32, 100, 2, 8, 0, 10];
        let entry = DbRangeEntry::try_from(&raw[..]).unwrap();
        assert_eq!(entry.min_val, -9);
        assert_eq!(entry.max_val, 100);
        assert_eq!(
            entry.data,
            DbRangeEntryData::DbInterval(DbInterval {
                min: 0,
                max: 10,
                linear: true,
                mute_avail: true
            })
        );
        assert_eq!(&Vec::<u32>::from(entry)[..], &raw[..]);
    }

    #[test]
    fn test_dbrange() {
        let raw = [
            3u32,
            72,
            0,
            10,
            2,
            8,
            -110i32 as u32,
            10,
            10,
            20,
            4,
            8,
            -10i32 as u32,
            0,
            20,
            30,
            5,
            8,
            0,
            20,
        ];
        let range = DbRange::try_from(&raw[..]).unwrap();
        assert_eq!(
            range.entries[0],
            DbRangeEntry {
                min_val: 0,
                max_val: 10,
                data: DbRangeEntryData::DbInterval(DbInterval {
                    min: -110,
                    max: 10,
                    linear: true,
                    mute_avail: true
                }),
            }
        );
        assert_eq!(
            range.entries[1],
            DbRangeEntry {
                min_val: 10,
                max_val: 20,
                data: DbRangeEntryData::DbInterval(DbInterval {
                    min: -10,
                    max: 0,
                    linear: false,
                    mute_avail: false
                }),
            }
        );
        assert_eq!(
            range.entries[2],
            DbRangeEntry {
                min_val: 20,
                max_val: 30,
                data: DbRangeEntryData::DbInterval(DbInterval {
                    min: 0,
                    max: 20,
                    linear: false,
                    mute_avail: true
                }),
            }
        );
        assert_eq!(&Vec::<u32>::from(range)[..], &raw[..]);
    }

    #[test]
    fn test_containerentry_dbscale() {
        let raw = [0u32, 32, 1, 8, 0, 5, 1, 8, 5, 5];
        let cntr = Container::try_from(&raw[..]).unwrap();
        assert_eq!(
            cntr.entries[0],
            TlvItem::DbScale(DbScale {
                min: 0,
                step: 5,
                mute_avail: false
            })
        );
        assert_eq!(
            cntr.entries[1],
            TlvItem::DbScale(DbScale {
                min: 5,
                step: 5,
                mute_avail: false
            })
        );
        assert_eq!(&Vec::<u32>::from(cntr)[..], &raw);
    }

    #[test]
    fn test_containerentry_dbrange() {
        let raw = [
            0u32, 136, 3, 48, 0, 10, 4, 8, 0, 5, 10, 20, 4, 8, 0, 10, 3, 72, 0, 10, 4, 8, 0, 5, 10,
            20, 4, 8, 5, 10, 20, 40, 4, 8, 10, 20,
        ];
        let cntr = Container::try_from(&raw[..]).unwrap();
        assert_eq!(
            cntr.entries[0],
            TlvItem::DbRange(DbRange {
                entries: vec![
                    DbRangeEntry {
                        min_val: 0,
                        max_val: 10,
                        data: DbRangeEntryData::DbInterval(DbInterval {
                            min: 0,
                            max: 5,
                            linear: false,
                            mute_avail: false
                        }),
                    },
                    DbRangeEntry {
                        min_val: 10,
                        max_val: 20,
                        data: DbRangeEntryData::DbInterval(DbInterval {
                            min: 0,
                            max: 10,
                            linear: false,
                            mute_avail: false
                        }),
                    },
                ],
            })
        );
        assert_eq!(
            cntr.entries[1],
            TlvItem::DbRange(DbRange {
                entries: vec![
                    DbRangeEntry {
                        min_val: 0,
                        max_val: 10,
                        data: DbRangeEntryData::DbInterval(DbInterval {
                            min: 0,
                            max: 5,
                            linear: false,
                            mute_avail: false
                        }),
                    },
                    DbRangeEntry {
                        min_val: 10,
                        max_val: 20,
                        data: DbRangeEntryData::DbInterval(DbInterval {
                            min: 5,
                            max: 10,
                            linear: false,
                            mute_avail: false
                        }),
                    },
                    DbRangeEntry {
                        min_val: 20,
                        max_val: 40,
                        data: DbRangeEntryData::DbInterval(DbInterval {
                            min: 10,
                            max: 20,
                            linear: false,
                            mute_avail: false
                        }),
                    },
                ],
            })
        );
        assert_eq!(&Vec::<u32>::from(cntr)[..], &raw[..]);
    }
}
