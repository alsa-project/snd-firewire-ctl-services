// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

use {
    clap::Parser,
    core::{cmdline::*, LogLevel},
    fireworks_runtime::EfwRuntime,
};

struct EfwServiceCmd;

#[derive(Parser, Default)]
#[clap(name = "snd-fireworks-ctl-service")]
struct Arguments {
    /// The numeric identifier of sound card in Linux sound subsystem.
    card_id: u32,
}

impl ServiceCmd<Arguments, u32, EfwRuntime> for EfwServiceCmd {
    fn params(args: &Arguments) -> (u32, Option<LogLevel>) {
        (args.card_id, None) 
    }
}

fn main() {
    EfwServiceCmd::run()
}
