// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use std::env;

fn print_help() {
    println!("
Usage:
  snd-firewire-tascam-ctl-service SUBSYSTEM ID

  where:
    SUBSYSTEM: The name of subsystem; 'snd' or 'fw'
    ID: The numerical ID of sound card or fw character device
    ");
}

fn main() {
    // Check arguments in command line.
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        print_help();
        std::process::exit(libc::EXIT_FAILURE);
    }

    let subsystem = &args[1];
    if subsystem != "snd" && subsystem != "fw" {
        println!("The first argument should be one of 'snd' and 'fw'.");
        print_help();
        std::process::exit(libc::EXIT_FAILURE);
    }

    let sysnum = &args[2];
    let sysnum = match sysnum.parse::<u32>() {
        Ok(n) => n,
        Err(_) => {
            println!("The second argument should be numerical number.");
            print_help();
            std::process::exit(libc::EXIT_FAILURE);
        }
    };

    let err = match tascam::unit::TascamUnit::new(subsystem, sysnum) {
        Err(err) => {
            println!(
                "The {}:{} is not for Tascam device: {}",
                subsystem, sysnum, err
            );
            Err(err)
        }
        Ok(mut unit) => {
            if let Err(err) = unit.listen() {
                println!("Fail to listen events: {}", err);
                Err(err)
            } else {
                unit.run();
                Ok(())
            }
        }
    };

    if err.is_err() {
        std::process::exit(libc::EXIT_FAILURE)
    }

    std::process::exit(libc::EXIT_SUCCESS)
}
