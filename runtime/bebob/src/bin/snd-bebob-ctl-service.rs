// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

use {
    bebob_runtime::BebobRuntime,
    clap::Parser,
    core::{cmdline::*, LogLevel},
};

struct BebobServiceCmd;

#[derive(Parser, Default)]
#[clap(name = "snd-bebob-ctl-service")]
struct Arguments {
    /// The numeric identifier of sound card in Linux sound subsystem.
    card_id: u32,

    /// The level to debug runtime, disabled as a default.
    #[clap(long, short, arg_enum)]
    log_level: Option<LogLevel>,
}

impl ServiceCmd<Arguments, u32, BebobRuntime> for BebobServiceCmd {
    fn params(args: &Arguments) -> (u32, Option<LogLevel>) {
        (args.card_id, args.log_level)
    }
}

fn main() {
    BebobServiceCmd::run()
}
