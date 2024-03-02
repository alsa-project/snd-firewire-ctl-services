// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    firewire_tascam_protocols as protocols,
    glib::{FileError, MainContext, MainLoop},
    hinawa::{
        prelude::{FwNodeExt, FwNodeExtManual},
        FwNode, FwNodeError,
    },
    ieee1212_config_rom::*,
    protocols::config_rom::*,
    std::{convert::TryFrom, sync::Arc, thread},
};

fn main() {
    let result: Result<(), String> = (|| {
        let path = std::env::args().nth(1).ok_or_else(|| {
            let msg =
                "One argument is required for path to special file of FireWire character device";
            msg.to_string()
        })?;

        let node = FwNode::default();
        node.open(&path, 0).map_err(|e| {
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

        let src = node.create_source().map_err(|e| e.to_string())?;

        let ctx = MainContext::new();
        let _ = src.attach(Some(&ctx));
        let dispatcher = Arc::new(MainLoop::new(Some(&ctx), false));
        let d = dispatcher.clone();
        let th = thread::spawn(move || d.run());

        let result = {
            let data = node.config_rom().map_err(|e| e.to_string())?;
            let config_rom = ConfigRom::try_from(data).map_err(|e| e.to_string())?;

            config_rom
                .get_unit_data()
                .map(|unit_data| {
                    println!("Unit data:");
                    println!("  specifier_id: 0x{:06x}", unit_data.specifier_id);
                    println!("  version:      0x{:06x}", unit_data.version);
                    println!("  vendor_name:  {}", unit_data.vendor_name);
                    println!("  model_name:   {}", unit_data.model_name);
                })
                .map_err(|e| format!("Fail to parse content of configuration ROM: {}", e))?;

            Ok(())
        };

        dispatcher.quit();
        th.join().unwrap();
        result
    })();

    let code = match result {
        Ok(_) => 0,
        Err(msg) => {
            println!("{}", msg);
            print_help();
            1
        }
    };

    std::process::exit(code)
}

fn print_help() {
    print!(
        r###"
Usage:
  tascam-config-rom-parser CDEV

  where:
    CDEV:       The path to special file of firewire character device, typically '/dev/fw1'.
"###
    );
}
