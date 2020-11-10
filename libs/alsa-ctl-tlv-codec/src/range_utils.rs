// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
//! A set of trait and implementation to retrieve range of dB value and raw value from data of TLV.
use super::{*, items::*, containers::*};

/// The structure represents the range of available value in the state of control element with
/// step to change it.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ValueRange {
    /// The minimum value in the state of control element.
    pub min: i32,
    /// The maximum value in the state of control element.
    pub max: i32,
    /// The step of value in the state of control element.
    pub step: i32,
}

/// The trait for utilities about range information.
pub trait RangeUtil<T> {
    /// The length from the minimum to maximum.
    fn length(&self) -> T;

    /// Whether the val is between the minimum and maximum.
    fn contains(&self, val: T) -> bool;
}

impl RangeUtil<i32> for ValueRange {
    fn length(&self) -> i32 {
        (self.max - self.min).abs()
    }

    fn contains(&self, val: i32) -> bool {
        val >= self.min && val <= self.max
    }
}

// NOTE: in 0.1 dB unit.
impl RangeUtil<i32> for DbInterval {
    fn length(&self) -> i32 {
        (self.max - self.min).abs()
    }

    fn contains(&self, db: i32) -> bool {
        db >= self.min && db <= self.max
    }
}

/// The trait for conversion into range of raw value on control element.
pub trait ToValueRange {
    fn to_valuerange(&self, range: &ValueRange) -> Option<ValueRange>;
}

impl ToValueRange for DbScale {
    fn to_valuerange(&self, range: &ValueRange) -> Option<ValueRange> {
        Some(*range)
    }
}

impl ToValueRange for DbInterval {
    fn to_valuerange(&self, range: &ValueRange) -> Option<ValueRange> {
        Some(*range)
    }
}

impl ToValueRange for DbRangeEntry {
    fn to_valuerange(&self, range: &ValueRange) -> Option<ValueRange> {
        Some(ValueRange{min: self.min_val, max: self.max_val, step: range.step})
    }
}

impl ToValueRange for DbRange {
    fn to_valuerange(&self, range: &ValueRange) -> Option<ValueRange> {
        let mut r = ValueRange{min: i32::MAX, max: i32::MIN, step: range.step};
        self.entries.iter().for_each(|entry| {
            if !r.contains(entry.min_val) {
                r.min = entry.min_val;
            }
            if !r.contains(entry.max_val) {
                r.max = entry.max_val;
            }
        });
        if r.min != i32::MAX && r.max != i32::MIN {
            Some(r)
        } else {
            None
        }
    }
}

impl ToValueRange for Container {
    fn to_valuerange(&self, range: &ValueRange) -> Option<ValueRange> {
        let mut r = ValueRange{min: i32::MAX, max: i32::MIN, step: range.step};
        self.entries.iter().for_each(|entry| {
            if let Some(range) = entry.to_valuerange(&range) {
                if !r.contains(range.min) {
                    r.min = range.min;
                }
                if !r.contains(range.max) {
                    r.max = range.max;
                }
            }
        });
        if r.min != i32::MAX && r.max != i32::MIN {
            Some(r)
        } else {
            None
        }
    }
}

impl ToValueRange for TlvItem {
    fn to_valuerange(&self, range: &ValueRange) -> Option<ValueRange> {
        match self {
            TlvItem::DbRange(d) => d.to_valuerange(&range),
            TlvItem::Container(d) => d.to_valuerange(&range),
            TlvItem::DbScale(_) |
            TlvItem::DbInterval(_) => Some(*range),
            _ => None,
        }
    }
}

/// The structure to represent conversin error into interval of dB
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToDbIntervalError{
    /// Arbitrary message for error cause.
    pub msg: String,
}

impl ToDbIntervalError {
    pub fn new(msg: String) -> Self {
        ToDbIntervalError{msg}
    }
}

impl std::fmt::Display for ToDbIntervalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

/// The trait for conversion into interval of dB.
pub trait ToDbInterval {
    fn to_dbinterval(&self, range: &ValueRange) -> Result<DbInterval, ToDbIntervalError>;
}

impl ToDbInterval for DbScale {
    fn to_dbinterval(&self, range: &ValueRange) -> Result<DbInterval, ToDbIntervalError> {
        Ok(DbInterval{
            min: self.min,
            max: self.min + range.length() * self.step as i32,
            linear: false,
            mute_avail: self.mute_avail,
        })
    }
}

impl ToDbInterval for DbInterval {
    fn to_dbinterval(&self, _: &ValueRange) -> Result<DbInterval, ToDbIntervalError> {
        Ok(*self)
    }
}

impl ToDbInterval for DbRangeEntry {
    fn to_dbinterval(&self, range: &ValueRange) -> Result<DbInterval, ToDbIntervalError> {
        let r = ValueRange{min: self.min_val, max: self.max_val, step: range.step};
        match &self.data {
            DbRangeEntryData::DbScale(d) => d.to_dbinterval(&r),
            DbRangeEntryData::DbInterval(d) => Ok(*d),
            DbRangeEntryData::DbRange(d) => d.to_dbinterval(&r),
        }
    }
}

impl ToDbInterval for DbRange {
    fn to_dbinterval(&self, range: &ValueRange) -> Result<DbInterval, ToDbIntervalError> {
        let entries = self.entries.iter().map(|entry| {
            if range.contains(entry.min_val) && range.contains(entry.max_val) {
                let r = ValueRange{min: entry.min_val, max: entry.max_val, step: range.step};
                entry.to_dbinterval(&r).and_then(|i| Ok((r, i)))
            } else {
                let msg = format!("DbRange includes entry in which value range is out of expectation:{}:{} but {}:{}",
                                  entry.min_val, entry.max_val, range.min, range.max);
                Err(ToDbIntervalError{msg})
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

        if entries.len() > 0 {
            let mut interval = entries[0].1;
            entries[1..].iter().try_for_each(|entry| {
                let i = entry.1;
                if i.linear != interval.linear {
                    let msg = "DbRange includes entries for both of non-linear and linear value".to_string();
                    Err(ToDbIntervalError{msg})
                } else {
                    if !interval.contains(i.min) {
                        interval.min = i.min;
                        interval.mute_avail = i.mute_avail;
                    }
                    if !interval.contains(i.max) {
                        interval.max = i.max;
                    }
                    Ok(())
                }
            })?;
            Ok(interval)
        } else {
            let msg = "DbRange includes no entry for dB information".to_string();
            Err(ToDbIntervalError{msg})
        }
    }
}

impl ToDbInterval for Container {
    fn to_dbinterval(&self, range: &ValueRange) -> Result<DbInterval, ToDbIntervalError> {
        let intervals = self.entries.iter()
            .map(|entry| entry.to_dbinterval(&range))
            .collect::<Result<Vec<_>, _>>()?;

        if intervals.len() > 0 {
            let mut interval = intervals[0];
            intervals[1..].iter().try_for_each(|i| {
                if i.linear != interval.linear {
                    let msg = "Container includes entries for both of non-linear and linear value".to_string();
                    Err(ToDbIntervalError{msg})
                } else {
                    if !interval.contains(i.min) {
                        interval.min = i.min;
                        interval.mute_avail = i.mute_avail;
                    }
                    if !interval.contains(i.max) {
                        interval.max = i.max;
                    }
                    Ok(())
                }
            })?;
            Ok(interval)
        } else {
            let msg = "Container includes no entry for dB information".to_string();
            Err(ToDbIntervalError{msg})
        }
    }
}

impl ToDbInterval for TlvItem {
    fn to_dbinterval(&self, range: &ValueRange) -> Result<DbInterval, ToDbIntervalError> {
        match self {
            TlvItem::Container(d) => d.to_dbinterval(&range),
            TlvItem::DbRange(d) => d.to_dbinterval(&range),
            TlvItem::DbScale(d) => d.to_dbinterval(&range),
            TlvItem::DbInterval(d) => Ok(*d),
            _ => {
                let msg = "Container includes entry without dB information".to_string();
                Err(ToDbIntervalError{msg})
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_dbinterval_dbscale() {
        let scale = &DbScale{min: 100, step: 10, mute_avail: true};
        let range = ValueRange{min: 33, max: 333, step: 1};
        let interval = scale.to_dbinterval(&range).unwrap();
        assert_eq!(interval, DbInterval{min: 100, max: 3100, linear: false, mute_avail: true});
    }

    #[test]
    fn to_valuerange_dbrange() {
        let first_data = DbInterval{min: 1, max: 5, linear: false, mute_avail: true};
        let second_data = DbInterval{min: 5, max: 10, linear: false, mute_avail: false};
        let third_data = DbInterval{min: 10, max: 20, linear: false, mute_avail: false};

        let dbrange = DbRange{
            entries: vec![
                DbRangeEntry{
                    min_val: 0,
                    max_val: 10,
                    data: DbRangeEntryData::DbInterval(first_data),
                },
                DbRangeEntry{
                    min_val: 10,
                    max_val: 20,
                    data: DbRangeEntryData::DbInterval(second_data),
                },
                DbRangeEntry{
                    min_val: 20,
                    max_val: 40,
                    data: DbRangeEntryData::DbInterval(third_data),
                },
            ],
        };
        let r = ValueRange{min: 0, max: 40, step: 1};
        let range = dbrange.to_valuerange(&r).unwrap();
        assert_eq!(range.min, 0);
        assert_eq!(range.max, 40);
        assert_eq!(range.step, 1);
    }

    #[test]
    fn to_dbinterval_dbrange() {
        let first_data = DbInterval{min: 1, max: 5, linear: false, mute_avail: true};
        let second_data = DbInterval{min: 5, max: 10, linear: false, mute_avail: false};
        let third_data = DbInterval{min: 10, max: 20, linear: false, mute_avail: false};
        let range = DbRange{
            entries: vec![
                DbRangeEntry{
                    min_val: 0,
                    max_val: 10,
                    data: DbRangeEntryData::DbInterval(first_data),
                },
                DbRangeEntry{
                    min_val: 10,
                    max_val: 20,
                    data: DbRangeEntryData::DbInterval(second_data),
                },
                DbRangeEntry{
                    min_val: 20,
                    max_val: 40,
                    data: DbRangeEntryData::DbInterval(third_data),
                },
            ],
        };
        let r = ValueRange{min: 0, max: 40, step: 1};
        let interval = range.to_dbinterval(&r).unwrap();
        assert_eq!(interval, DbInterval{min: 1, max: 20, linear: false, mute_avail: true});
    }
}
