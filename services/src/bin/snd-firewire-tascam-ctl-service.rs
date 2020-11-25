// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use snd_firewire_ctl_services::*;
use tascam::runtime::TascamRuntime;

struct TascamServiceCmd;

impl<'a> ServiceCmd<'a, (String, u32), TascamRuntime<'a>> for TascamServiceCmd {
    const CMD_NAME: &'a str = "snd-firewire-tascam-ctl-service";
    const ARGS: &'a [(&'a str, &'a str)] = &[
        ("SUBSYSTEM", "The name of subsystem; 'snd' or 'fw'"),
        ("SYSNUM", "The numeric ID of sound card or fw character device"),
    ];

    fn parse_args(args: &[String]) -> Result<(String, u32), String> {
        match args[0].as_str() {
            "snd" | "fw" => Ok(args[0].clone()),
            _ => {
                let msg = format!("The first argument should be one of 'snd' and 'fw': {}", args[0]);
                Err(msg)
            }
        }
        .and_then(|subsystem| {
            parse_arg_as_u32(&args[1])
                .map(|sysnum| (subsystem, sysnum))
        })
    }
}

fn main() {
    TascamServiceCmd::run()
}
