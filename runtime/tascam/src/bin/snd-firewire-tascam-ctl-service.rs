// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

use {clap::Parser, core::cmdline::*, tascam_runtime::TascamRuntime};

struct TascamServiceCmd;

#[derive(Parser, Default)]
struct Arguments {
    /// The name of subsystem; 'snd' or 'fw'.
    subsystem: String,
    /// The numeric identifier of sound card or firewire character device.
    sysnum: u32,
}

impl ServiceCmd<(String, u32), TascamRuntime> for TascamServiceCmd {
    const CMD_NAME: &'static str = "snd-firewire-tascam-ctl-service";
    const ARGS: &'static [(&'static str, &'static str)] = &[
        ("SUBSYSTEM", "The name of subsystem; 'snd' or 'fw'"),
        (
            "SYSNUM",
            "The numeric ID of sound card or fw character device",
        ),
    ];

    fn params(_: &[String]) -> Result<(String, u32), String> {
        Arguments::try_parse()
            .map(|args| (args.subsystem, args.sysnum))
            .map_err(|err| err.to_string())
    }
}

fn main() {
    TascamServiceCmd::run()
}
