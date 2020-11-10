// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use alsa_ctl_tlv_codec::{*, items::*, containers::*};
use std::convert::TryFrom;

use std::io::{Read, Write};
use std::str::FromStr;
use std::num::ParseIntError;


fn generate_indent(level: usize) -> String {
    (0..(level * INDENTS_PER_LEVEL)).fold(String::new(), |mut l, _| {l.push(' '); l})
}

trait PrintAsMacro {
    fn print_as_macro(&self, level: usize);
}

impl PrintAsMacro for DbScale {
    fn print_as_macro(&self, _: usize) {
        print!("SNDRV_CTL_TLVD_ITEM ( SNDRV_CTL_TLVT_DB_SCALE, 0x{:x}, 0x{:x}{} )",
               self.min as u32, self.step,
               if self.mute_avail { " | SNDRV_CTL_TLVD_DB_SCALE_MUTE" } else { "" });
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
        print!("SNDRV_CTL_TLVD_ITEM ( {}, 0x{:x}, 0x{:x} )",
               label, self.min as u32, self.max as u32);
    }
}

impl PrintAsMacro for ChmapEntry {
    fn print_as_macro(&self, _: usize) {
        match self.pos {
            ChmapPos::Generic(p) => {
                let label = match p {
                    ChmapGenericPos::Unknown => "SNDRV_CHMAP_UNKNOWN",
                    ChmapGenericPos::NotAvailable => "SNDRV_CHMAP_NA",
                    ChmapGenericPos::Monaural => "SNDRV_CHMAP_MONO",
                    ChmapGenericPos::FrontLeft => "SNDRV_CHMAP_FL",
                    ChmapGenericPos::FrontRight => "SNDRV_CHMAP_FR",
                    ChmapGenericPos::RearLeft => "SNDRV_CHMAP_RL",
                    ChmapGenericPos::RearRight => "SNDRV_CHMAP_RR",
                    ChmapGenericPos::FrontCenter => "SNDRV_CHMAP_FC",
                    ChmapGenericPos::LowFrequencyEffect => "SNDRV_CHMAP_LFE",
                    ChmapGenericPos::SideLeft => "SNDRV_CHMAP_SL",
                    ChmapGenericPos::SideRight => "SNDRV_CHMAP_SR",
                    ChmapGenericPos::RearCenter => "SNDRV_CHMAP_RC",
                    ChmapGenericPos::FrontLeftCenter => "SNDRV_CHMAP_FLC",
                    ChmapGenericPos::FrontRightCenter => "SNDRV_CHMAP_FRC",
                    ChmapGenericPos::RearLeftCenter => "SNDRV_CHMAP_RLC",
                    ChmapGenericPos::RearRightCenter => "SNDRV_CHMAP_RRC",
                    ChmapGenericPos::FrontLeftWide => "SNDRV_CHMAP_FLW",
                    ChmapGenericPos::FrontRightWide => "SNDRV_CHMAP_FRW",
                    ChmapGenericPos::FrontLeftHigh => "SNDRV_CHMAP_FLH",
                    ChmapGenericPos::FrontCenterHigh => "SNDRV_CHMAP_FCH",
                    ChmapGenericPos::FrontRightHigh => "SNDRV_CHMAP_FRH",
                    ChmapGenericPos::TopCenter => "SNDRV_CHMAP_TC",
                    ChmapGenericPos::TopFrontLeft => "SNDRV_CHMAP_TFL",
                    ChmapGenericPos::TopFrontRight => "SNDRV_CHMAP_TFR",
                    ChmapGenericPos::TopFrontCenter => "SNDRV_CHMAP_TFC",
                    ChmapGenericPos::TopRearLeft => "SNDRV_CHMAP_TRL",
                    ChmapGenericPos::TopRearRight => "SNDRV_CHMAP_TRR",
                    ChmapGenericPos::TopRearCenter => "SNDRV_CHMAP_TRC",
                    ChmapGenericPos::TopFrontLeftCenter => "SNDRV_CHMAP_TFLC",
                    ChmapGenericPos::TopFrontRightCenter => "SNDRV_CHMAP_TFRC",
                    ChmapGenericPos::TopSideLeft => "SNDRV_CHMAP_TSL",
                    ChmapGenericPos::TopSideRight => "SNDRV_CHMAP_TSR",
                    ChmapGenericPos::LeftLowFrequencyEffect => "SNDRV_CHMAP_LLFE",
                    ChmapGenericPos::RightLowFrequencyEffect => "SNDRV_CHMAP_RLFE",
                    ChmapGenericPos::BottomCenter => "SNDRV_CHMAP_BC",
                    ChmapGenericPos::BottomLeftCenter => "SNDRV_CHMAP_BLC",
                    ChmapGenericPos::BottomRightCenter => "SNDRV_CHMAP_BRC",
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
        let indent =generate_indent(level);

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

impl PrintAsMacro for TlvItem {
    fn print_as_macro(&self, level: usize) {
        match self {
            TlvItem::Container(d) => d.print_as_macro(level),
            TlvItem::DbRange(d) => d.print_as_macro(level),
            TlvItem::DbScale(d) => d.print_as_macro(level),
            TlvItem::DbInterval(d) => d.print_as_macro(level),
            TlvItem::Chmap(d) => d.print_as_macro(level),
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
    if args.len() < 2 {
        print_help();
        std::process::exit(0);
    }

    let mode = match args[0].as_str() {
        "structure" => Mode::Structure,
        "literal" => Mode::Literal,
        "raw" => Mode::Raw,
        "macro" => Mode::Macro,
        _ => {
            eprintln!("Invalid argument for mode: {}", args[0]);
            std::process::exit(1);
        }
    };

    let raw = if args[1] == "-" {
        interpret_tlv_data_from_stdin().unwrap_or_else(|msg| {
            eprintln!("{}", msg);
            std::process::exit(1);
        })
    } else {
        interpret_tlv_data_from_command_line(&args[1..]).unwrap_or_else(|msg| {
            eprintln!("{}", msg);
            std::process::exit(1);
        })
    };

    let item = TlvItem::try_from(&raw[..]).unwrap_or_else(|msg| {
        eprintln!("{}", msg);
        std::process::exit(1);
    });

    if mode == Mode::Structure {
        println!("{:?}", item);
    } else if mode == Mode::Macro {
        item.print_as_macro(0);
        println!("");
    } else {
        let raw: Vec<u32> = match item {
          TlvItem::Container(d) => d.into(),
          TlvItem::DbRange(d) => d.into(),
          TlvItem::DbScale(d) => d.into(),
          TlvItem::DbInterval(d) => d.into(),
          TlvItem::Chmap(d) => d.into(),
        };

        if mode == Mode::Literal {
            raw.iter().for_each(|val| print!("{} ", val));
            println!("")
        } else {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            raw.iter().for_each(|val| out.write_all(&val.to_ne_bytes()).unwrap_or(()))
        }
    }
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
"###);
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
