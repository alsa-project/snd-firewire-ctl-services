// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

use {
    clap::Parser,
    core::{cmdline::*, LogLevel},
    tascam_runtime::TascamRuntime,
};

struct TascamServiceCmd;

#[derive(Parser, Default)]
#[clap(name = "snd-firewire-tascam-ctl-service")]
struct Arguments {
    /// The name of subsystem; 'snd' or 'fw'.
    subsystem: String,
    /// The numeric identifier of sound card or firewire character device.
    sysnum: u32,

    /// The level to debug runtime, disabled as a default.
    #[clap(long, short, arg_enum)]
    log_level: Option<LogLevel>,
}

impl ServiceCmd<Arguments, (String, u32), TascamRuntime> for TascamServiceCmd {
    fn params(args: &Arguments) -> ((String, u32), Option<LogLevel>) {
        ((args.subsystem.clone(), args.sysnum), args.log_level)
    }
}

fn main() {
    TascamServiceCmd::run()
}
