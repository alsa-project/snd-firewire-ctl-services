// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

use {clap::Parser, core::cmdline::*, tascam_runtime::TascamRuntime};

struct TascamServiceCmd;

#[derive(Parser, Default)]
#[clap(name = "snd-firewire-tascam-ctl-service")]
struct Arguments {
    /// The name of subsystem; 'snd' or 'fw'.
    subsystem: String,
    /// The numeric identifier of sound card or firewire character device.
    sysnum: u32,
}

impl ServiceCmd<Arguments, (String, u32), TascamRuntime> for TascamServiceCmd {
    fn params(args: &Arguments) -> (String, u32) {
        (args.subsystem.clone(), args.sysnum)
    }
}

fn main() {
    TascamServiceCmd::run()
}
