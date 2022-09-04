// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

use {clap::Parser, core::cmdline::*, oxfw_runtime::OxfwRuntime};

struct OxfwServiceCmd;

#[derive(Parser, Default)]
#[clap(name = "snd-oxfw-ctl-service")]
struct Arguments {
    /// The numeric identifier of sound card in Linux sound subsystem.
    card_id: u32,
}

impl ServiceCmd<Arguments, u32, OxfwRuntime> for OxfwServiceCmd {
    fn params(args: &Arguments) -> u32 {
        args.card_id
    }
}

fn main() {
    OxfwServiceCmd::run()
}
