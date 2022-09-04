// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

use {clap::Parser, core::cmdline::*, motu_runtime::MotuRuntime};

struct MotuServiceCmd;

#[derive(Parser, Default)]
#[clap(name = "snd-firewire-motu-ctl-service")]
struct Arguments {
    /// The numeric identifier of sound card in Linux sound subsystem.
    card_id: u32,
}

impl ServiceCmd<Arguments, u32, MotuRuntime> for MotuServiceCmd {
    fn params(args: &Arguments) -> u32 {
        args.card_id
    }
}

fn main() {
    MotuServiceCmd::run()
}
