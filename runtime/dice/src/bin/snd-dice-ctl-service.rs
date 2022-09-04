// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

use {clap::Parser, core::cmdline::*, dice_runtime::DiceRuntime};

struct DiceServiceCmd;

#[derive(Parser, Default)]
#[clap(name = "snd-dice-ctl-service")]
struct Arguments {
    /// The numeric identifier of sound card in Linux sound subsystem.
    card_id: u32,
}

impl ServiceCmd<Arguments, u32, DiceRuntime> for DiceServiceCmd {
    fn params(args: &Arguments) -> u32 {
        args.card_id
    }
}

fn main() {
    DiceServiceCmd::run()
}
