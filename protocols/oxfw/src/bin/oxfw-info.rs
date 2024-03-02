// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    firewire_oxfw_protocols as protocols,
    glib::{FileError, MainContext, MainLoop},
    hinawa::{prelude::FwNodeExt, FwNode, FwNodeError, FwReq},
    protocols::oxford::*,
    std::{sync::Arc, thread},
};

const TIMEOUT_MS: u32 = 20;

struct Protocol;

impl OxfordOperation for Protocol {}

fn main() {
    let code = std::env::args().nth(1)
        .ok_or("At least one argument is required for path to special file of FireWire character device".to_string())
        .and_then(|path| {
            let node = FwNode::new();
            node.open(&path, 0)
                .map_err(|e| {
                    let cause = if let Some(error) = e.kind::<FileError>() {
                        match error {
                            FileError::Isdir => "is directory",
                            FileError::Acces => "access permission",
                            FileError::Noent => "not exists",
                            _ => "unknown",
                        }.to_string()
                    } else if let Some(error) = e.kind::<FwNodeError>() {
                        match error {
                            FwNodeError::Disconnected => "disconnected",
                            FwNodeError::Failed => "ioctl error",
                            _ => "unknown",
                        }.to_string()
                    } else {
                        e.to_string()
                    };
                    format!("Fail to open firewire character device {}: {} {}", path, cause, e)
                })
                .and_then(|_| {
                    node.create_source()
                        .map_err(|e| e.to_string())
                        .map(|src| (node, src))
                })
        })
        .and_then(|(mut node, src)| {
            let ctx = MainContext::new();
            let _ = src.attach(Some(&ctx));
            let dispatcher = Arc::new(MainLoop::new(Some(&ctx), false));
            let d = dispatcher.clone();
            let th = thread::spawn(move || d.run());

            let result = {
                let mut firmware_id = 0;
                let mut hardware_id = 0;

                let mut req = FwReq::new();
                Protocol::read_firmware_id(&mut req, &mut node, &mut firmware_id, TIMEOUT_MS)
                    .map_err(|e| e.to_string())?;
                Protocol::read_hardware_id(&mut req, &mut node, &mut hardware_id, TIMEOUT_MS)
                    .map_err(|e| e.to_string())?;

                println!("firmware: 0x{:08x}", firmware_id);
                println!("hardware: 0x{:08x}", hardware_id);

                Ok(())
            };

            dispatcher.quit();
            th.join().unwrap();
            result
        })
        .map(|_| 0)
        .unwrap_or_else(|msg| {
            eprintln!("{}", msg);
            print_help();
            1
        });

    std::process::exit(code)
}

fn print_help() {
    print!(
        r###"
Usage:
  oxfw-info CDEV

  where:
    CDEV:       The path to special file of firewire character device, typically '/dev/fw1'.
"###
    );
}
