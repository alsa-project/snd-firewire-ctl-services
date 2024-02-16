// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    firewire_dice_protocols as protocols,
    glib::FileError,
    hinawa::{
        prelude::{FwNodeExt, FwNodeExtManual},
        FwNode, FwNodeError,
    },
    ieee1212_config_rom::*,
    protocols::tcat::config_rom::*,
    std::{convert::TryFrom, io::Read},
};

fn main() {
    let code = std::env::args()
        .nth(1)
        .ok_or("The first argument is required for target to parse.".to_string())
        .and_then(|path| {
            if path == "-" {
                let raw = read_data_from_stdin()?;
                if raw.len() % 4 != 0 {
                    let msg = format!(
                        "The length of data is not aligned to quadlet: {}",
                        raw.len()
                    );
                    Err(msg)?
                }

                let mut data = Vec::new();
                let mut quadlet = [0; 4];
                (0..(raw.len() / 4)).for_each(|i| {
                    let pos = i * 4;
                    quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                    data.extend_from_slice(&u32::from_be_bytes(quadlet).to_ne_bytes());
                });
                Ok(data)
            } else {
                let node = FwNode::new();
                node.open(&path).map_err(|e| {
                    let cause = if let Some(error) = e.kind::<FileError>() {
                        match error {
                            FileError::Isdir => "is directory",
                            FileError::Acces => "access permission",
                            FileError::Noent => "not exists",
                            _ => "unknown",
                        }
                        .to_string()
                    } else if let Some(error) = e.kind::<FwNodeError>() {
                        match error {
                            FwNodeError::Disconnected => "disconnected",
                            FwNodeError::Failed => "ioctl error",
                            _ => "unknown",
                        }
                        .to_string()
                    } else {
                        e.to_string()
                    };
                    format!(
                        "Fail to open firewire character device {}: {} {}",
                        path, cause, e
                    )
                })?;

                node.config_rom()
                    .map_err(|e| format!("Fail to get content of configuration ROM: {}", e))
                    .map(|raw| raw.to_vec())
            }
        })
        .and_then(|raw| {
            let config_rom = ConfigRom::try_from(&raw[..])
                .map_err(|e| format!("Malformed configuration ROM detected: {}", e))?;

            config_rom.get_identifier().map(|identifier| {
                println!("Identifier:");
                println!("  vendor_id:      0x{:06x}", identifier.vendor_id);
                println!("  category:       {}", identifier.category);
                println!("  product_id:     0x{:06x}", identifier.product_id);
                println!("  serial:         {}", identifier.serial);
            });

            config_rom.get_root_data().map(|root| {
                println!("Root:");
                println!("  vendor_id:      0x{:06x}", root.vendor_id);
                println!("  vendor_name:    '{}'", root.vendor_name);
                println!("  product_id:     0x{:06x}", root.product_id);
                println!("  product_name:   '{}'", root.product_name);
            });

            let units = config_rom.get_unit_data();
            if units.len() > 0 {
                println!("Units:");

                units.iter().enumerate().for_each(|(i, unit)| {
                    println!("  {}:", i);
                    println!("    model_id:     0x{:06x}", unit.model_id);
                    println!("    model_name:   '{}'", unit.model_name);
                    println!("    specifier_id: 0x{:06x}", unit.specifier_id);
                    println!("    version:      0x{:06x}", unit.version);
                });
            }

            Ok(())
        })
        .map(|_| 0)
        .unwrap_or_else(|msg| {
            eprintln!("{}", msg);
            print_help();
            1
        });

    std::process::exit(code)
}

fn read_data_from_stdin() -> Result<Vec<u8>, String> {
    let mut raw = Vec::new();

    let len = std::io::stdin()
        .lock()
        .read_to_end(&mut raw)
        .map_err(|e| e.to_string())?;

    if len == 0 {
        Err("Nothing available via standard input.".to_string())?;
    }

    Ok(raw)
}

fn print_help() {
    print!(
        r###"
Usage:
  tcat-config-rom-parser CDEV | "-"

  where:
    CDEV:       the path to special file of firewire character device, typically '/dev/fw1'.
    "-"         use STDIN for the content of configuration ROM to parse. It should be aligned to big endian.
"###
    );
}
