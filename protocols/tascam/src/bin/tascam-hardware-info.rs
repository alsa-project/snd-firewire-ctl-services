// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    firewire_tascam_protocols as protocols,
    glib::{FileError, MainContext, MainLoop},
    hinawa::{prelude::FwNodeExt, FwNode, FwNodeError, FwReq},
    protocols::*,
    std::{sync::Arc, thread},
};

const TIMEOUT_MS: u32 = 50;

fn main() {
    let result: Result<(), String> = (|| {
        let path = std::env::args().nth(1).ok_or_else(|| {
            let msg =
                "One argument is required for path to special file of FireWire character device";
            msg.to_string()
        })?;

        let mut node = FwNode::default();
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
            let mut req = FwReq::default();
            let mut info = HardwareInformation::default();

            HardwareInformationProtocol::read_hardware_information(
                &mut req, &mut node, &mut info, TIMEOUT_MS,
            )
            .map_err(|e| format!("Read transaction failed: {}", e))?;

            println!("Hardware information:");
            println!("  Register: 0x{:08x}", info.register);
            println!("  FPGA:     0x{:08x}", info.fpga);
            println!("  ARM:      0x{:08x}", info.arm);
            println!("  Hardware: 0x{:08x}", info.hardware);

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
  tascam-hardware-info CDEV

  where:
    CDEV:       The path to special file of firewire character device, typically '/dev/fw1'.
"###
    );
}
