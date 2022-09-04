// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

use {clap::Parser, core::cmdline::*, fireface_runtime::FfRuntime};

struct FfServiceCmd;

#[derive(Parser, Default)]
#[clap(name = "snd-fireface-ctl-service")]
struct Arguments {
    /// The numeric identifier of sound card in Linux sound subsystem.
    card_id: u32,
}

impl ServiceCmd<Arguments, u32, FfRuntime> for FfServiceCmd {
    fn params(args: &Arguments) -> u32 {
        args.card_id
    }
}

fn main() {
    FfServiceCmd::run()
}
