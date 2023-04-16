// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    firewire_bebob_protocols as protocols,
    glib::{FileError, MainContext, MainLoop},
    hinawa::{prelude::FwNodeExt, FwNode, FwNodeError, FwReq},
    protocols::bridgeco::*,
    std::{sync::Arc, thread},
};

const TIMEOUT_MS: u32 = 20;

struct Protocol;

impl BcoBootloaderOperation for Protocol {}

fn main() {
    let code = std::env::args().nth(1)
        .ok_or("At least one argument is required for path to special file of FireWire character device".to_string())
        .and_then(|path| {
            let node = FwNode::new();
            node.open(&path)
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
        .and_then(|(node, src)| {
            let ctx = MainContext::new();
            let _ = src.attach(Some(&ctx));
            let dispatcher = Arc::new(MainLoop::new(Some(&ctx), false));
            let d = dispatcher.clone();
            let th = thread::spawn(move || d.run());

            let req = FwReq::new();
            let mut info = BcoBootloaderInformation::default();
            let result = Protocol::read_info(&req, &node, &mut info, TIMEOUT_MS)
                .map(|_| print_info(&info))
                .map_err(|err| err.to_string());

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
  bco-bootloader-info CDEV

  where:
    CDEV:       The path to special file of firewire character device, typically '/dev/fw1'.
"###
    );
}

fn parse_version_triplet(val: u32) -> String {
    format!(
        "{}.{}.{}",
        val >> 24,
        (val & 0x00ff0000) >> 16,
        val & 0xffff
    )
}

fn print_info(info: &BcoBootloaderInformation) {
    println!("protocol:");
    println!("  version: {}", info.protocol_version);

    println!("bootloader:");
    let _ = info
        .bootloader_timestamp
        .format("%Y-%m-%dT%I:%M:%S%z")
        .map(|literal| println!("  timestamp: {:}", literal));
    println!(
        "  version: {}",
        parse_version_triplet(info.bootloader_version)
    );

    println!("hardware:");
    println!("  GUID: 0x{:016x}", info.guid);
    println!("  model ID: 0x{:06x}", info.hardware_model_id);
    println!(
        "  revision: {}",
        parse_version_triplet(info.hardware_revision)
    );

    println!("software:");
    let _ = info
        .software
        .timestamp
        .format("%Y-%m-%dT%I:%M:%S%z")
        .map(|literal| println!("  timestamp: {:}", literal));
    println!("  ID: 0x{:08x}", info.software.id);
    println!(
        "  revision: {}",
        parse_version_triplet(info.software.version)
    );

    println!("image:");
    println!("  base address: 0x{:x}", info.image_base_address);
    println!("  maximum size: 0x{:x}", info.image_maximum_size);

    if let Some(debugger) = &info.debugger {
        println!("debugger:");
        let _ = debugger
            .timestamp
            .format("%Y-%m-%dT%I:%M:%S%z")
            .map(|literal| println!("  timestamp: {:}", literal));
        println!("  ID: 0x{:x}", debugger.id);
        println!("  revision: {}", parse_version_triplet(debugger.version));
    }
}
