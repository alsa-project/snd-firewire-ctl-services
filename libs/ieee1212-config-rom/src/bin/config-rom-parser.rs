// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use ieee1212_config_rom;
use std::fs::File;
use std::io::Read;
use std::convert::TryFrom;

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
            ieee1212_config_rom::ConfigRom::try_from(&data[..])
                .map(|config_rom| {
                    println!("{:?}", config_rom);
                })
                .map_err(|e| e.to_string())
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
