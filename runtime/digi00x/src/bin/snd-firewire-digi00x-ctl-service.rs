// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

use {
    clap::Parser,
    core::{cmdline::*, LogLevel},
    digi00x_runtime::Dg00xRuntime,
};

struct Dg00xServiceCmd;

#[derive(Parser, Default)]
#[clap(name = "snd-firewire-digi00x-ctl-service")]
struct Arguments {
    /// The numeric identifier of sound card in Linux sound subsystem.
    card_id: u32,

    /// The level to debug runtime, disabled as a default.
    #[clap(long, short, arg_enum)]
    log_level: Option<LogLevel>,
}

impl ServiceCmd<Arguments, u32, Dg00xRuntime> for Dg00xServiceCmd {
    fn params(args: &Arguments) -> (u32, Option<LogLevel>) {
        (args.card_id, args.log_level)
    }
}

fn main() {
    Dg00xServiceCmd::run()
}
