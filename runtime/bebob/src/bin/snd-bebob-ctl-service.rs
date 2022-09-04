// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

use {bebob_runtime::BebobRuntime, clap::Parser, core::cmdline::*};

struct BebobServiceCmd;

#[derive(Parser, Default)]
struct Arguments {
    /// The numeric identifier of sound card in Linux sound subsystem.
    card_id: u32,
}

impl ServiceCmd<u32, BebobRuntime> for BebobServiceCmd {
    const CMD_NAME: &'static str = "snd-bebob-ctl-service";
    const ARGS: &'static [(&'static str, &'static str)] =
        &[("CARD_ID", "The numeric ID of sound card")];

    fn params(_: &[String]) -> Result<u32, String> {
        Arguments::try_parse()
            .map(|args| args.card_id)
            .map_err(|err| err.to_string())
    }
}

fn main() {
    BebobServiceCmd::run()
}
