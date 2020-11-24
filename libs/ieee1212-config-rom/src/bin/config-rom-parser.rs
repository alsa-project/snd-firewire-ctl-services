// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use ieee1212_config_rom::{*, entry::*, desc::*};
use std::fs::File;
use std::io::Read;
use std::convert::TryFrom;

fn print_raw(raw: &[u8], level: usize) {
    let mut indent = String::new();
    (0..(level * INDENT_PER_LEVEL)).for_each(|_| indent.push(' '));

    let mut iter = raw.iter();

    iter.by_ref()
        .nth(0)
        .map(|b| print!("{}{:02x}", indent, b));

    iter.for_each(|b| print!(" {:02x}", b));

    println!("");
}

fn print_leaf(raw: &[u8], level: usize) -> Result<(), String> {
    let mut indent = String::new();
    (0..(level * INDENT_PER_LEVEL)).for_each(|_| indent.push(' '));

    let desc = Descriptor::try_from(raw)
        .map_err(|e| e.to_string())?;
    match &desc.data {
        DescriptorData::Textual(d) => {
            println!("{}Texual descriptor:", indent);
            println!("{}  specifier_id: {}", indent, desc.spec_id);
            println!("{}  width: {}", indent, d.width);
            println!("{}  character_set: {}", indent, d.character_set);
            println!("{}  language: {}", indent, d.language);
            println!("{}  text: {}", indent, d.text);
        }
        DescriptorData::Reserved(d) => {
            println!("{}Unspecified descriptor:", indent);
            print_raw(d, level);
        }
    }

    Ok(())
}

const INDENT_PER_LEVEL: usize = 2;

fn print_directory_entries(entries: &[Entry], level: usize) -> Result<(), String> {
    let mut indent = String::new();
    (0..(level * INDENT_PER_LEVEL)).for_each(|_| indent.push(' '));

    entries.iter().try_for_each(|entry| {
        match &entry.data {
            EntryData::Immediate(value) => println!("{}{:?} (immediate): 0x{:08x}", indent, entry.key, value),
            EntryData::CsrOffset(offset) => println!("{}{:?} (offset): 0x{:024x}", indent, entry.key, offset),
            EntryData::Leaf(leaf) => {
                println!("{}{:?} (leaf):", indent, entry.key);
                print_leaf(leaf, level + 1)?;
            }
            EntryData::Directory(raw) => {
                println!("{}{:?} (directory):", indent, entry.key);
                print_directory_entries(&raw, level + 1)?;
            }
        }
        Ok(())
    })
}

fn main() {
    let code = std::env::args()
        .nth(1)
        .ok_or_else(|| {
            "The first argument is required for target to parse.".to_string()
        })
        .and_then(|arg| {
            if arg == "-" {
                read_data_from_stdin()
            } else {
                read_data_from_file(&arg)
            }
        })
        .and_then(|raw| {
            if raw.len() % 4 > 0 {
                let label = format!("The length of data is not aligned to quadlet: {}", raw.len());
                Err(label)
            } else {
                let mut data = Vec::new();
                let mut quadlet = [0;4];
                (0..(raw.len() / 4))
                    .for_each(|i| {
                        let pos = i * 4;
                        quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                        data.extend_from_slice(&u32::from_be_bytes(quadlet).to_ne_bytes());
                    });
                Ok(data)
            }
        })
        .and_then(|data| {
            ConfigRom::try_from(&data[..])
                .map_err(|e| e.to_string())
                .map(|config_rom| {
                    println!("Bus information block:");
                    print_raw(config_rom.bus_info, 1);

                    println!("Root directory block:");
                    print_directory_entries(&config_rom.root[..], 1)
                })
        })
        .map(|_| 0)
        .unwrap_or_else(|msg| {
            eprintln!("{}", msg);
            print_help();
            1
        });

    std::process::exit(code);
}

fn read_data_from_stdin() -> Result<Vec<u8>, String> {
    let mut raw = Vec::new();

    std::io::stdin().lock().read_to_end(&mut raw)
        .map_err(|e| e.to_string())
        .and_then(|len| {
            if len == 0 {
                Err("Nothing available via standard input.".to_string())
            } else {
                Ok(raw)
            }
        })
}

fn read_data_from_file(filename: &str) -> Result<Vec<u8>, String> {
    File::open(filename)
        .map_err(|e| e.to_string())
        .and_then(|mut handle| {
            let mut raw = Vec::new();
            handle.read_to_end(&mut raw)
                .map_err(|e| e.to_string())
                .and_then(|len| {
                    if len == 0 {
                        let label = format!("Nothing available in {}", filename);
                        Err(label)
                    } else {
                        Ok(raw)
                    }
                })
        })
}

fn print_help() {
    print!(
r###"
Usage:
  config-rom-parser FILENAME | "-"

  where:
    FILENAME:       the name of file for the image of configuration ROM to parse
    "-":            the content of configuration ROM to parse. It should be aligned to big endian.
"###);
}
