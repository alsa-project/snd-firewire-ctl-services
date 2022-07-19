// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use alsa_ctl_tlv_codec::*;
use std::convert::TryFrom;

use std::io::{Read, Write};
use std::str::FromStr;

fn generate_indent(level: usize) -> String {
    (0..(level * INDENTS_PER_LEVEL)).fold(String::new(), |mut l, _| {
        l.push(' ');
        l
    })
}

trait PrintAsMacro {
    fn print_as_macro(&self, level: usize);
}

impl PrintAsMacro for DbScale {
    fn print_as_macro(&self, _: usize) {
        print!(
            "SNDRV_CTL_TLVD_ITEM ( SNDRV_CTL_TLVT_DB_SCALE, 0x{:x}, 0x{:x}{} )",
            self.min as u32,
            self.step,
            if self.mute_avail {
                " | SNDRV_CTL_TLVD_DB_SCALE_MUTE"
            } else {
                ""
            }
        );
    }
}

impl PrintAsMacro for DbInterval {
    fn print_as_macro(&self, _: usize) {
        let label = if self.linear {
            "SNDRV_CTL_TLVT_DB_LINEAR"
        } else {
            if self.mute_avail {
                "SNDRV_CTL_TLVT_DB_MINMAX_MUTE"
            } else {
                "SNDRV_CTL_TLVT_DB_MINMAX"
            }
        };
        print!(
            "SNDRV_CTL_TLVD_ITEM ( {}, 0x{:x}, 0x{:x} )",
            label, self.min as u32, self.max as u32
        );
    }
}

impl PrintAsMacro for ChmapEntry {
    fn print_as_macro(&self, _: usize) {
        match self.pos {
            ChmapPos::Generic(p) => {
                let label = match p {
                    ChmapGenericPos::Unknown => "SNDRV_CHMAP_UNKNOWN".to_string(),
                    ChmapGenericPos::NotAvailable => "SNDRV_CHMAP_NA".to_string(),
                    ChmapGenericPos::Monaural => "SNDRV_CHMAP_MONO".to_string(),
                    ChmapGenericPos::FrontLeft => "SNDRV_CHMAP_FL".to_string(),
                    ChmapGenericPos::FrontRight => "SNDRV_CHMAP_FR".to_string(),
                    ChmapGenericPos::RearLeft => "SNDRV_CHMAP_RL".to_string(),
                    ChmapGenericPos::RearRight => "SNDRV_CHMAP_RR".to_string(),
                    ChmapGenericPos::FrontCenter => "SNDRV_CHMAP_FC".to_string(),
                    ChmapGenericPos::LowFrequencyEffect => "SNDRV_CHMAP_LFE".to_string(),
                    ChmapGenericPos::SideLeft => "SNDRV_CHMAP_SL".to_string(),
                    ChmapGenericPos::SideRight => "SNDRV_CHMAP_SR".to_string(),
                    ChmapGenericPos::RearCenter => "SNDRV_CHMAP_RC".to_string(),
                    ChmapGenericPos::FrontLeftCenter => "SNDRV_CHMAP_FLC".to_string(),
                    ChmapGenericPos::FrontRightCenter => "SNDRV_CHMAP_FRC".to_string(),
                    ChmapGenericPos::RearLeftCenter => "SNDRV_CHMAP_RLC".to_string(),
                    ChmapGenericPos::RearRightCenter => "SNDRV_CHMAP_RRC".to_string(),
                    ChmapGenericPos::FrontLeftWide => "SNDRV_CHMAP_FLW".to_string(),
                    ChmapGenericPos::FrontRightWide => "SNDRV_CHMAP_FRW".to_string(),
                    ChmapGenericPos::FrontLeftHigh => "SNDRV_CHMAP_FLH".to_string(),
                    ChmapGenericPos::FrontCenterHigh => "SNDRV_CHMAP_FCH".to_string(),
                    ChmapGenericPos::FrontRightHigh => "SNDRV_CHMAP_FRH".to_string(),
                    ChmapGenericPos::TopCenter => "SNDRV_CHMAP_TC".to_string(),
                    ChmapGenericPos::TopFrontLeft => "SNDRV_CHMAP_TFL".to_string(),
                    ChmapGenericPos::TopFrontRight => "SNDRV_CHMAP_TFR".to_string(),
                    ChmapGenericPos::TopFrontCenter => "SNDRV_CHMAP_TFC".to_string(),
                    ChmapGenericPos::TopRearLeft => "SNDRV_CHMAP_TRL".to_string(),
                    ChmapGenericPos::TopRearRight => "SNDRV_CHMAP_TRR".to_string(),
                    ChmapGenericPos::TopRearCenter => "SNDRV_CHMAP_TRC".to_string(),
                    ChmapGenericPos::TopFrontLeftCenter => "SNDRV_CHMAP_TFLC".to_string(),
                    ChmapGenericPos::TopFrontRightCenter => "SNDRV_CHMAP_TFRC".to_string(),
                    ChmapGenericPos::TopSideLeft => "SNDRV_CHMAP_TSL".to_string(),
                    ChmapGenericPos::TopSideRight => "SNDRV_CHMAP_TSR".to_string(),
                    ChmapGenericPos::LeftLowFrequencyEffect => "SNDRV_CHMAP_LLFE".to_string(),
                    ChmapGenericPos::RightLowFrequencyEffect => "SNDRV_CHMAP_RLFE".to_string(),
                    ChmapGenericPos::BottomCenter => "SNDRV_CHMAP_BC".to_string(),
                    ChmapGenericPos::BottomLeftCenter => "SNDRV_CHMAP_BLC".to_string(),
                    ChmapGenericPos::BottomRightCenter => "SNDRV_CHMAP_BRC".to_string(),
                    ChmapGenericPos::Reserved(val) => format!("RESERVED({})", val),
                };
                print!("{}", label);
            }
            ChmapPos::Specific(p) => {
                print!("{} | SNDRV_CHMAP_DRIVER_SPEC", p)
            }
        };
        if self.phase_inverse {
            print!(" | SNDRV_CHMAP_PHASE_INVERSE");
        }
    }
}

impl PrintAsMacro for Chmap {
    fn print_as_macro(&self, mut level: usize) {
        level += 1;
        let indent = generate_indent(level);

        let label = match self.mode {
            ChmapMode::Fixed => "SNDRV_CTL_TLVT_CHMAP_FIXED",
            ChmapMode::ArbitraryExchangeable => "SNDRV_CTL_TLVT_CHMAP_VAR",
            ChmapMode::PairedExchangeable => "SNDRV_CTL_TLVT_CHMAP_PAIRED",
        };

        println!("SNDRV_CTL_TLVD_ITEM ( {},", label);
        self.entries.iter().for_each(|entry| {
            print!("{}", indent);
            entry.print_as_macro(level);
            println!(",");
        });
    }
}

impl PrintAsMacro for DbRangeEntry {
    fn print_as_macro(&self, mut level: usize) {
        level += 1;

        print!("{}, {}, ", self.min_val, self.max_val);

        match &self.data {
            DbRangeEntryData::DbScale(d) => d.print_as_macro(level),
            DbRangeEntryData::DbInterval(d) => d.print_as_macro(level),
            DbRangeEntryData::DbRange(d) => d.print_as_macro(level),
        };
    }
}

impl PrintAsMacro for DbRange {
    fn print_as_macro(&self, mut level: usize) {
        level += 1;
        let indent = generate_indent(level);

        println!("SNDRV_CTL_TLVD_ITEM ( SNDRV_CTL_TLVT_DB_RANGE,");
        self.entries.iter().for_each(|entry| {
            print!("{}", indent);
            entry.print_as_macro(level);
            println!(",");
        });
    }
}

const INDENTS_PER_LEVEL: usize = 4;

impl PrintAsMacro for Container {
    fn print_as_macro(&self, mut level: usize) {
        level += 1;
        let indent = generate_indent(level);

        println!("SNDRV_CTL_TLVD_ITEM ( SNDRV_CTL_TLVT_CONTAINER, ");
        self.entries.iter().for_each(|entry| {
            print!("{}", indent);
            entry.print_as_macro(level);
            println!("{}),", indent)
        });
        print!(")");
    }
}

impl PrintAsMacro for Vec<u32> {
    fn print_as_macro(&self, _: usize) {
        print!("SNDRV_CTL_TLVD_ITEM ( {}", self[0]);
        self[2..].iter().for_each(|val| print!(", 0x{:x}", val));
        print!(" )");
    }
}

impl PrintAsMacro for TlvItem {
    fn print_as_macro(&self, level: usize) {
        match self {
            TlvItem::Container(d) => d.print_as_macro(level),
            TlvItem::DbRange(d) => d.print_as_macro(level),
            TlvItem::DbScale(d) => d.print_as_macro(level),
            TlvItem::DbInterval(d) => d.print_as_macro(level),
            TlvItem::Chmap(d) => d.print_as_macro(level),
            TlvItem::Unknown(d) => d.print_as_macro(level),
        }
    }
}

#[derive(PartialEq, Eq)]
enum Mode {
    Structure,
    Literal,
    Raw,
    Macro,
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let code = (if args.len() < 2 {
        Err("2 arguments are required in command line at least.".to_string())
    } else {
        Ok(args)
    })
    .and_then(|args| {
        let mode = match args[0].as_str() {
            "structure" => Ok(Mode::Structure),
            "literal" => Ok(Mode::Literal),
            "raw" => Ok(Mode::Raw),
            "macro" => Ok(Mode::Macro),
            _ => {
                let label = format!("Invalid argument for mode: {}", args[0]);
                Err(label)
            }
        }?;

        let raw = match args[1].as_str() {
            "-" => interpret_tlv_data_from_stdin(),
            _ => interpret_tlv_data_from_command_line(&args[1..]),
        }?;

        let item = TlvItem::try_from(&raw[..]).map_err(|e| e.to_string())?;

        Ok((mode, item))
    })
    .and_then(|(mode, item)| match mode {
        Mode::Structure => {
            println!("{:?}", item);
            Ok(())
        }
        Mode::Macro => {
            item.print_as_macro(0);
            println!("");
            Ok(())
        }
        _ => {
            let raw: Vec<u32> = match item {
                TlvItem::Container(d) => d.into(),
                TlvItem::DbRange(d) => d.into(),
                TlvItem::DbScale(d) => d.into(),
                TlvItem::DbInterval(d) => d.into(),
                TlvItem::Chmap(d) => d.into(),
                TlvItem::Unknown(d) => d,
            };

            if mode == Mode::Literal {
                raw.iter().for_each(|val| print!("{} ", val));
                println!("");
                Ok(())
            } else {
                let mut bytes = Vec::new();
                raw.iter()
                    .for_each(|val| bytes.extend_from_slice(&val.to_ne_bytes()));
                std::io::stdout()
                    .lock()
                    .write_all(&bytes)
                    .map_err(|e| e.to_string())
            }
        }
    })
    .map(|_| 0)
    .unwrap_or_else(|msg| {
        eprintln!("{}", msg);
        print_help();
        1
    });

    std::process::exit(code);
}

fn print_help() {
    print!(
        r###"
Usage:
  tlv-decode MODE DATA | "-"

  where:
    MODE:           The mode to process after parsing DATA:
                        "structure":    prints data structures.
                        "macro":        prints C macro representation
                        "literal":      prints space-separated decimal array.
                        "raw":          prints binary with host endian.
    DATA:           space-separated DECIMAL and HEXADECIMAL array for the data of TLV.
    "-":            use binary from STDIN to interpret DATA according to host endian.
    DECIMAL:        decimal number. It can be signed if needed.
    HEXADECIMAL:    hexadecimal number. It should have '0x' as prefix.
"###
    );
}

fn interpret_tlv_data_from_stdin() -> Result<Vec<u32>, String> {
    let mut buf = Vec::new();

    std::io::stdin()
        .lock()
        .read_to_end(&mut buf)
        .map_err(|e| e.to_string())
        .and_then(|len| {
            if len == 0 {
                Err("Nothing available via standard input.".to_string())
            } else if len % 4 > 0 {
                Err("The length of data via standard input is not multiples of 4.".to_string())
            } else {
                Ok(())
            }
        })
        .map(|_| {
            let mut raw = Vec::new();

            let mut quadlet = [0; 4];
            (0..(buf.len() / 4)).for_each(|i| {
                let pos = i * 4;
                quadlet.copy_from_slice(&buf[pos..(pos + 4)]);
                raw.push(u32::from_ne_bytes(quadlet));
            });
            raw
        })
}

fn interpret_tlv_data_from_command_line(args: &[String]) -> Result<Vec<u32>, String> {
    let mut raw = Vec::new();

    args.iter().try_for_each(|arg| {
        (if arg.starts_with("0x") {
            u32::from_str_radix(arg.trim_start_matches("0x"), 16)
        } else if arg
            .find(&['A', 'B', 'C', 'D', 'E', 'F', 'a', 'b', 'c', 'd', 'e', 'f'][..])
            .is_some()
        {
            u32::from_str_radix(arg, 16)
        } else {
            u32::from_str(arg)
        })
        .map_err(|e| format!("Invalid argument for data of TLV: {}, {}", arg, e))
        .map(|val| {
            raw.push(val);
            ()
        })
    })?;

    Ok(raw)
}
