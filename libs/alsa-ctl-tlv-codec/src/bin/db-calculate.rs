// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use alsa_ctl_tlv_codec::{*, items::*, containers::*, range_utils::*};

use std::num::ParseIntError;
use std::str::FromStr;
use std::io::Read;
use std::convert::TryFrom;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ErrorTarget {
    Container,
    DbRange,
    DbScale,
    DbInterval,
    Chmap,
}

impl From<&TlvItem> for ErrorTarget {
    fn from(item: &TlvItem) -> Self {
        match item {
            TlvItem::Container(_) => ErrorTarget::Container,
            TlvItem::DbRange(_) => ErrorTarget::DbRange,
            TlvItem::DbScale(_) => ErrorTarget::DbScale,
            TlvItem::DbInterval(_) => ErrorTarget::DbInterval,
            TlvItem::Chmap(_) => ErrorTarget::Chmap,
        }
    }
}

impl std::fmt::Display for ErrorTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            ErrorTarget::Container => "Container",
            ErrorTarget::DbRange => "DbRange",
            ErrorTarget::DbScale => "DbScale",
            ErrorTarget::DbInterval => "DbInterval",
            ErrorTarget::Chmap => "Chmap",
        };
        write!(f, "{}", label)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ErrorCause {
    NoEntryAvail,
    CalculationFailed,
    ToDbInterval,
    OutOfRange,
}

impl std::fmt::Display for ErrorCause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            ErrorCause::NoEntryAvail => "No entry available",
            ErrorCause::CalculationFailed => "Calculation failed",
            ErrorCause::ToDbInterval => "dB information not found",
            ErrorCause::OutOfRange => "Out of range",
        };
        write!(f, "{}", label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalError{
    target: ErrorTarget,
    ctx: ErrorCause,
    msg: String,
}

impl LocalError {
    fn new(target: ErrorTarget, ctx: ErrorCause, msg: String) -> Self {
        LocalError{target, ctx, msg}
    }
}

impl std::fmt::Display for LocalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "target: {}, ctx: {}, msg: {}", self.target, self.ctx, self.msg)
    }
}

// Unit conversion between raw dB and dB in data of TLV in ALSA control interface.
trait DbUnitConvert {
    const UNIT: f64 = DB_VALUE_MULTIPLIER as f64;
    fn min_f(&self) -> f64;
    fn max_f(&self) -> f64;
}

// Decibel calculation for SNDRV_CTL_TLVT_DB_LINEAR.
trait LinearDbCalc {
    const REFERENCE: f64 = 20.0;
    fn val_to_linear_for_db(&self, val: i32, range: &ValueRange) -> Result<f64, LocalError>;
    fn val_from_linear_for_db(&self, db: f64, range: &ValueRange) -> Result<i32, LocalError>;
}

// Common calculation methods between raw value and raw dB.
trait DbCalc {
    fn val_to_db(&self, val: i32, range: &ValueRange) -> Result<f64, LocalError>;
    fn val_from_db(&self, db: f64, range: &ValueRange) -> Result<i32, LocalError>;
}

// Extensions of DbScale implementation.
impl DbCalc for DbScale {
    fn val_to_db(&self, val: i32, range: &ValueRange) -> Result<f64, LocalError> {
        let interval = self.to_dbinterval(&range).unwrap();
        interval.val_to_db(val, range)
    }

    fn val_from_db(&self, db: f64, range: &ValueRange) -> Result<i32, LocalError> {
        let interval = self.to_dbinterval(&range).unwrap();
        interval.val_from_db(db, range)
    }
}

// Extensions of DbInterval implementation.
impl DbUnitConvert for DbInterval {
    fn min_f(&self) -> f64 {
        (self.min as f64) / Self::UNIT
    }

    fn max_f(&self) -> f64 {
        (self.max as f64) / Self::UNIT
    }
}

impl LinearDbCalc for DbInterval {
    fn val_to_linear_for_db(&self, val: i32, range: &ValueRange) -> Result<f64, LocalError> {
        if val == CTL_VALUE_MUTE && self.mute_avail {
            Ok(f64::NEG_INFINITY)
        } else if !range.contains(val) {
            let msg = format!("{} is not between {} and {}", val, range.min, range.max);
            Err(LocalError::new(ErrorTarget::DbInterval, ErrorCause::OutOfRange, msg))
        } else if val == range.min {
            Ok(self.min_f())
        } else if val == range.max {
            Ok(self.max_f())
        } else {
            let linear_min = 10f64.powf(self.min_f() / Self::REFERENCE);
            let linear_max = 10f64.powf(self.max_f() / Self::REFERENCE);
            let linear_length = (linear_min - linear_max).abs();
            let linear_val = linear_min + linear_length * ((val - range.min) as f64) / (range.length() as f64);
            Ok(Self::REFERENCE * f64::log10(linear_val))
        }
    }

    fn val_from_linear_for_db(&self, db: f64, range: &ValueRange) -> Result<i32, LocalError> {
        if db == f64::NEG_INFINITY {
            if self.mute_avail {
                Ok(CTL_VALUE_MUTE)
            } else {
                let msg = format!("{} is not supported for mute", db);
                Err(LocalError::new(ErrorTarget::DbInterval, ErrorCause::OutOfRange, msg))
            }
        } else {
            let min = self.min_f();
            let max = self.max_f();
            if db < min || db > max {
                let msg = format!("{} is not between {} and {}", db, min, max);
                Err(LocalError::new(ErrorTarget::DbInterval, ErrorCause::OutOfRange, msg))
            } else if db == min {
                Ok(range.min)
            } else if db >= max {
                Ok(range.max)
            } else {
                let linear_val = 10f64.powf(db / Self::REFERENCE);
                let linear_min = 10f64.powf(self.min_f() / Self::REFERENCE);
                let linear_max = 10f64.powf(self.max_f() / Self::REFERENCE);
                let linear_length = (linear_max - linear_min).abs();
                Ok(((range.min as f64) + (range.length() as f64) * ((linear_val - linear_min) / linear_length)) as i32)
            }
        }
    }
}

impl DbCalc for DbInterval {
    fn val_to_db(&self, val: i32, range: &ValueRange) -> Result<f64, LocalError> {
        if self.linear {
            self.val_to_linear_for_db(val, range)
        } else {
            if val == CTL_VALUE_MUTE && self.mute_avail {
                Ok(f64::NEG_INFINITY)
            } else if !range.contains(val) {
                let msg = format!("{} is not between {} and {}", val, range.min, range.max);
                Err(LocalError::new(ErrorTarget::DbInterval, ErrorCause::OutOfRange, msg))
            } else if val == range.min {
                Ok(self.min_f())
            } else if val == range.max {
                Ok(self.max_f())
            } else {
                let db_min = self.min_f();
                let db_max = self.max_f();
                let db_length = (db_max - db_min).abs();
                Ok(db_min + db_length * ((val - range.min) as f64) / (range.length() as f64))
            }
        }
    }

    fn val_from_db(&self, db: f64, range: &ValueRange) -> Result<i32, LocalError> {
        if self.linear {
            self.val_from_linear_for_db(db, range)
        } else {
            if db == f64::NEG_INFINITY {
                if self.mute_avail {
                    Ok(CTL_VALUE_MUTE)
                } else {
                    let msg = format!("{} is not supported for mute", db);
                    Err(LocalError::new(ErrorTarget::DbInterval, ErrorCause::OutOfRange, msg))
                }
            } else {
                let min = self.min_f();
                let max = self.max_f();
                if db < min || db > max {
                    let msg = format!("{} is not between {} and {}", db, min, max);
                    Err(LocalError::new(ErrorTarget::DbInterval, ErrorCause::OutOfRange, msg))
                } else if db == min {
                    Ok(range.min)
                } else if db == max {
                    Ok(range.max)
                } else {
                    let db_min = self.min_f();
                    let db_max = self.max_f();
                    let db_length = (db_max - db_min).abs();
                    let v = (range.min as f64) + (range.length() as f64) * (db - db_min) / db_length;
                    Ok(v as i32)
                }
            }
        }
    }
}

// Extensions for DbRange implementation.
impl DbCalc for DbRangeEntry {
    fn val_to_db(&self, val: i32, range: &ValueRange) -> Result<f64, LocalError> {
        let r = self.to_valuerange(range).unwrap();
        match &self.data {
            DbRangeEntryData::DbScale(d) => d.val_to_db(val, &r),
            DbRangeEntryData::DbInterval(d) => d.val_to_db(val, &r),
            DbRangeEntryData::DbRange(d) => d.val_to_db(val, &r),
        }
    }

    fn val_from_db(&self, db: f64, range: &ValueRange) -> Result<i32, LocalError> {
        let r = self.to_valuerange(range).unwrap();
        match &self.data {
            DbRangeEntryData::DbScale(d) => d.val_from_db(db, &r),
            DbRangeEntryData::DbInterval(d) => d.val_from_db(db, &r),
            DbRangeEntryData::DbRange(d) => d.val_from_db(db, &r),
        }
    }
}

// Extensions for DbRange implementation.
impl DbCalc for DbRange {
    fn val_to_db(&self, val: i32, range: &ValueRange) -> Result<f64, LocalError> {
        (if val == CTL_VALUE_MUTE {
            self.entries.iter()
                .filter_map(|entry| {
                    let r = entry.to_valuerange(&range).unwrap();
                    entry.to_dbinterval(&r)
                        .ok()
                        .and_then(|i| Some((i.min, r, entry)))
                })
                .min_by(|r, l| r.0.cmp(&l.0))
                .map(|(_, r, entry)| (r, entry))
        } else {
            self.entries.iter()
                .find_map(|entry| {
                    let r = entry.to_valuerange(&range).unwrap();
                    if r.contains(val) { Some((r, entry)) } else { None }
                })
        })
        .ok_or_else(|| {
            let msg = format!("{:?}", self);
            LocalError::new(ErrorTarget::DbRange, ErrorCause::NoEntryAvail, msg)
        })
        .and_then(|(r, entry)| {
            entry.val_to_db(val, &r).or_else(|e| {
                let msg = format!("{}: {:?}", e.msg, entry);
                Err(LocalError::new(ErrorTarget::DbRange, ErrorCause::CalculationFailed, msg))
            })
        })
    }

    fn val_from_db(&self, db: f64, range: &ValueRange) -> Result<i32, LocalError> {
        (if db == f64::NEG_INFINITY {
            self.entries.iter()
                .filter_map(|entry| {
                    let r = entry.to_valuerange(&range).unwrap();
                    entry.to_dbinterval(&range)
                        .ok()
                        .and_then(|i| Some((i.min, r, entry)))
                })
                .min_by(|r, l| r.0.cmp(&l.0))
                .map(|(_, r, entry)| (r, entry))
        } else {
            let db_devaluated = (db * (DB_VALUE_MULTIPLIER as f64)) as i32;
            self.entries.iter()
                .find_map(|entry| {
                    let r = entry.to_valuerange(&range).unwrap();
                    entry.to_dbinterval(&r)
                        .ok()
                        .and_then(|i| if i.contains(db_devaluated) { Some((r, entry)) } else { None })
                })
        })
        .ok_or_else(|| {
            let msg = format!("{:?}", self);
            LocalError::new(ErrorTarget::DbRange, ErrorCause::NoEntryAvail, msg)
        })
        .and_then(|(r, entry)| {
            entry.val_from_db(db, &r).or_else(|e| {
                let msg = format!("{}: {:?}", e.msg, entry);
                Err(LocalError::new(ErrorTarget::DbRange, ErrorCause::CalculationFailed, msg))
            })
        })
    }
}

// Extensions of Container implementation.
impl DbCalc for Container {
    fn val_to_db(&self, val: i32, range: &ValueRange) -> Result<f64, LocalError> {
        (if val == CTL_VALUE_MUTE {
            self.entries.iter()
                .filter_map(|entry| {
                    entry.to_valuerange(&range)
                        .and_then(|r| {
                            entry.to_dbinterval(&r)
                                .ok()
                                .and_then(|i| Some((i.min, r, entry)))
                        })
                })
                .min_by(|r, l| r.0.cmp(&l.0))
                .and_then(|(_, r, entry)| Some((r, entry)))
        } else {
            self.entries.iter()
                .find_map(|entry| {
                    entry.to_valuerange(&range)
                        .and_then(|r| if r.contains(val) { Some((r, entry)) } else { None })
                })
        })
        .ok_or_else(|| {
            let msg = format!("{:?}", self);
            LocalError::new(ErrorTarget::DbRange, ErrorCause::NoEntryAvail, msg)
        })
        .and_then(|(r, entry)| {
            entry.val_to_db(val, &r).or_else(|e| {
                let msg = format!("{}: {:?}", e.msg, entry);
                Err(LocalError::new(ErrorTarget::Container, ErrorCause::CalculationFailed, msg))
            })
        })
    }

    fn val_from_db(&self, db: f64, range: &ValueRange) -> Result<i32, LocalError> {
        (if db == f64::NEG_INFINITY {
            self.entries.iter()
                .filter_map(|entry| {
                    entry.to_valuerange(&range)
                        .and_then(|r| {
                            entry.to_dbinterval(&range)
                                .ok()
                                .and_then(|i| Some((i.min, r, entry)))
                        })
                })
                .min_by(|r, l| r.0.cmp(&l.0))
                .map(|(_, r, entry)| (r, entry))
        } else {
            let db_devaluated = (db * (DB_VALUE_MULTIPLIER as f64)) as i32;
            self.entries.iter()
                .find_map(|entry| {
                    entry.to_valuerange(&range)
                        .and_then(|r| {
                            entry.to_dbinterval(&r)
                                .ok()
                                .and_then(|i| if i.contains(db_devaluated) { Some((r, entry)) } else { None })
                        })
                })
        })
        .ok_or_else(|| {
            let msg = format!("{:?}", self);
            LocalError::new(ErrorTarget::DbRange, ErrorCause::NoEntryAvail, msg)
        })
        .and_then(|(r, entry)| {
            entry.val_from_db(db, &r).or_else(|e| {
                let msg = format!("{}: {:?}", e.msg, entry);
                Err(LocalError::new(ErrorTarget::DbRange, ErrorCause::CalculationFailed, msg))
            })
        })
    }
}

// Extensions of TlvItem implementation.
impl DbCalc for TlvItem {
    fn val_to_db(&self, val: i32, range: &ValueRange) -> Result<f64, LocalError> {
        self.to_dbinterval(&range)
            .or_else(|e| {
                let msg = format!("{}: {:?}", e.msg, self);
                Err(LocalError::new(ErrorTarget::from(self), ErrorCause::ToDbInterval, msg))
            })
            .and_then(|i| i.val_to_db(val, range))
    }

    fn val_from_db(&self, db: f64, range: &ValueRange) -> Result<i32, LocalError> {
        self.to_dbinterval(&range)
            .or_else(|e| {
                let msg = format!("{}: {:?}", e.msg, self);
                Err(LocalError::new(ErrorTarget::from(self), ErrorCause::ToDbInterval, msg))
            })
            .and_then(|i| i.val_from_db(db, range))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Mode {
    Db(f64),
    Value(i32),
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 6 {
        print_help();
        std::process::exit(0);
    }

    let mode = if args[0] == "db" {
        let db = f64::from_str(&args[1]).unwrap_or_else(|_| {
            eprintln!("Invalid argument for decimal floating point value: {}", args[1]);
            std::process::exit(1);
        });
        Mode::Db(db)
    } else if args[0] == "value" {
        let val = interpret_i32(&args[1]).unwrap_or_else(|_| {
            eprintln!("Invalid argument for decimal value: {}", args[1]);
            std::process::exit(1);
        });
        Mode::Value(val)
    } else {
        eprintln!("Invalid argument for operation mode: {}", args[0]);
        print_help();
        std::process::exit(1);
    };

    let min = interpret_i32(&args[2]).unwrap_or_else(|_| {
        eprintln!("Invalid argument for minimum value: {}", args[1]);
        std::process::exit(1);
    });
    let max = interpret_i32(&args[3]).unwrap_or_else(|_| {
        eprintln!("Invalid argument for minimum value: {}", args[1]);
        std::process::exit(1);
    });
    let step = interpret_i32(&args[4]).unwrap_or_else(|_| {
        eprintln!("Invalid argument for minimum value: {}", args[1]);
        std::process::exit(1);
    });
    let range = ValueRange{min, max, step};

    let raw = if args[5] == "-" {
        interpret_tlv_data_from_stdin().unwrap_or_else(|msg| {
            eprintln!("{}", msg);
            std::process::exit(1);
        })
    } else {
        interpret_tlv_data_from_command_line(&args[5..]).unwrap_or_else(|msg| {
            eprintln!("{}", msg);
            std::process::exit(1);
        })
    };

    let data = TlvItem::try_from(&raw[..]).unwrap_or_else(|error| {
        eprintln!("{}", error);
        std::process::exit(1)
    });

    if let Err(e) = match mode {
        Mode::Db(db) => {
            data.val_from_db(db, &range)
                .and_then(|val| {
                    println!("{}", val);
                    Ok(())
                })
        }
        Mode::Value(val) => {
            data.val_to_db(val, &range)
                .and_then(|db| {
                    println!("{}", db);
                    Ok(())
                })
        }
    } {
        eprintln!("{}", e);
        std::process::exit(1);
    } else {
        std::process::exit(0);
    }
}

fn print_help() {
    print!(
r###"
Usage:
  db-calculate "db" DECIMAL-FLOATING-POINT VALUE-RANGE DATA | "-"
  db-calculate "value" DECIMAL | HEXADECIMAL VALUE-RANGE DATA | "-"

  where:
    "db":                   Use this program for db calculation.
    "value":                Use this program for value calculation.
    DECIMAL-FLOATING-POINT: decimal floating point number. It can be signed if needed.
    DECIMAL:                decimal number. It can be signed if needed.
    HEXADECIMAL:            hexadecimal number. It should have '0x' as prefix.
    VALUE-RANGE:            space-separated triplet of MIN, MAX, and STEP comes from information of
                            control element. All of them are DECIMAL or HEXADECIMAL.
    DATA:                   space-separated DECIMAL and HEXADECIMAL array for the data of TLV.
    "-":                    use STDIN to interpret DATA according to host endian.

   When data of TLV has information to support mute, "-9999999" for value and "-inf" for db are
   available.
"###);
}

fn interpret_i32(arg: &str) -> Result<i32, ParseIntError> {
    if arg.starts_with("0x") {
        i32::from_str_radix(arg.trim_start_matches("0x"), 16)
    } else if arg.find(&['A', 'B', 'C', 'D', 'E', 'F', 'a', 'b', 'c', 'd', 'e', 'f'][..]).is_some() {
        i32::from_str_radix(arg, 16)
    } else {
        i32::from_str(arg)
    }
}

fn interpret_tlv_data_from_stdin() -> Result<Vec<u32>, String> {
    let mut raw = Vec::new();

    let input = std::io::stdin();
    let mut handle = input.lock();

    let mut buf = Vec::new();
    match handle.read_to_end(&mut buf) {
        Ok(len) => {
            if len == 0 {
                return Err("Nothing available via standard input.".to_string());
            } else if len % 4 > 0 {
                return Err("The length of data via standard input is not multiples of 4.".to_string());
            } else {
                let mut quadlet = [0;4];
                (0..(buf.len() / 4)).for_each(|i| {
                    let pos = i * 4;
                    quadlet.copy_from_slice(&buf[pos..(pos + 4)]);
                    raw.push(u32::from_ne_bytes(quadlet));
                });
            }
        }
        Err(e) => return Err(e.to_string()),
    };

    Ok(raw)
}

fn interpret_tlv_data_from_command_line(args: &[String]) -> Result<Vec<u32>, String> {
    let mut raw = Vec::new();

    if let Err(e) = args.iter().try_for_each(|arg| {
        let val = if arg.starts_with("0x") {
            u32::from_str_radix(arg.trim_start_matches("0x"), 16)?
        } else if arg.find(&['A', 'B', 'C', 'D', 'E', 'F', 'a', 'b', 'c', 'd', 'e', 'f'][..]).is_some() {
            u32::from_str_radix(arg, 16)?
        } else {
            u32::from_str(arg)?
        };
        raw.push(val);
        Ok::<(), ParseIntError>(())
    }) {
        return Err(e.to_string());
    }

    Ok(raw)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn dbcalc_dbscale() {
        let range = ValueRange{min: -10, max: 0, step: 1};
        let scale = DbScale{min: 1, step: 100, mute_avail: false};

        assert_eq!(scale.val_to_db(-10, &range).unwrap(), 0.01f64);
        assert_eq!(scale.val_to_db(0, &range).unwrap(), 10.01f64);
        assert_eq!(scale.val_to_db(-5, &range).unwrap(), 5.01f64);

        assert_eq!(scale.val_from_db(0.01f64, &range).unwrap(), -10);
        assert_eq!(scale.val_from_db(10.01f64, &range).unwrap(), 0);
        assert_eq!(scale.val_from_db(5.01f64, &range).unwrap(), -5);
    }

    #[test]
    fn dbconvert_dbinterval() {
        let interval = DbInterval{min: 100, max: 1000, linear: false, mute_avail: true};
        assert_eq!(interval.min_f(), 1f64);
        assert_eq!(interval.max_f(), 10f64);
    }

    #[test]
    fn lineardbcalc_dbinterval() {
        let interval = DbInterval{min: 2000, max: 6000, linear: false, mute_avail: true};
        let range = ValueRange{min: 33, max: 133, step: 1};

        let value_midpoint = range.min + (range.max - range.min) / 2;
        let linear_min = 10f64.powf((interval.min as f64) / 20f64 / 100f64);
        let linear_max = 10f64.powf((interval.max as f64) / 20f64 / 100f64);
        let linear_midpoint = linear_min + (linear_max - linear_min) / 2f64;
        let db_midpoint = 20f64 * linear_midpoint.log10();

        assert_eq!(interval.val_to_linear_for_db(CTL_VALUE_MUTE, &range).unwrap(), f64::NEG_INFINITY);
        assert_eq!(interval.val_to_linear_for_db(33, &range).unwrap(), 20f64);
        assert_eq!(interval.val_to_linear_for_db(133, &range).unwrap(), 60f64);
        assert_eq!(interval.val_to_linear_for_db(value_midpoint, &range).unwrap(), db_midpoint);

        assert_eq!(interval.val_from_linear_for_db(f64::NEG_INFINITY, &range).unwrap(), CTL_VALUE_MUTE);
        assert_eq!(interval.val_from_linear_for_db(20f64, &range).unwrap(), 33);
        assert_eq!(interval.val_from_linear_for_db(60f64, &range).unwrap(), 133);
        assert_eq!(interval.val_from_linear_for_db(db_midpoint, &range).unwrap(), value_midpoint);
    }

    #[test]
    fn dbcalc_dbinterval() {
        let interval = DbInterval{min: 1, max: 1001, linear: false, mute_avail: false};
        let range = ValueRange{min: -10, max: 0, step: 1};

        assert_eq!(interval.val_to_db(-10, &range).unwrap(), 0.01f64);
        assert_eq!(interval.val_to_db(0, &range).unwrap(), 10.01f64);
        assert_eq!(interval.val_to_db(-5, &range).unwrap(), 5.01f64);

        assert_eq!(interval.val_from_db(0.01f64, &range).unwrap(), -10);
        assert_eq!(interval.val_from_db(10.01f64, &range).unwrap(), 0);
        assert_eq!(interval.val_from_db(5.01f64, &range).unwrap(), -5);
    }

    #[test]
    fn dbcalc_dbrange() {
        let first_data = DbInterval{min: 1, max: 501, linear: false, mute_avail: true};
        let second_data = DbInterval{min: 501, max: 1001, linear: false, mute_avail: false};

        let db_range = DbRange{
            entries: vec![
                DbRangeEntry{
                    min_val: -10,
                    max_val: -5,
                    data: DbRangeEntryData::DbInterval(first_data),
                },
                DbRangeEntry{
                    min_val: -5,
                    max_val: 0,
                    data: DbRangeEntryData::DbInterval(second_data),
                },
            ],
        };
        let val_range = ValueRange{min: -10, max: 0, step: 1};

        assert_eq!(db_range.val_to_db(CTL_VALUE_MUTE, &val_range).unwrap(), f64::NEG_INFINITY);
        assert_eq!(db_range.val_to_db(-10, &val_range), Ok(0.01f64));
        assert_eq!(db_range.val_to_db(-5, &val_range), Ok(5.01f64));
        assert_eq!(db_range.val_to_db(0, &val_range), Ok(10.01f64));

        assert_eq!(db_range.val_from_db(f64::NEG_INFINITY, &val_range).unwrap(), CTL_VALUE_MUTE);
        assert_eq!(db_range.val_from_db(0.01f64, &val_range), Ok(-10));
        assert_eq!(db_range.val_from_db(5.01f64, &val_range), Ok(-5));
        assert_eq!(db_range.val_from_db(10.01f64, &val_range), Ok(0));
    }
}
