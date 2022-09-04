// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

use {clap::Parser, core::cmdline::*, fireface_runtime::FfRuntime};

struct FfServiceCmd;

#[derive(Parser, Default)]
struct Arguments {
    /// The numeric identifier of sound card in Linux sound subsystem.
    card_id: u32,
}

impl ServiceCmd<u32, FfRuntime> for FfServiceCmd {
    const CMD_NAME: &'static str = "snd-fireface-ctl-service";
    const ARGS: &'static [(&'static str, &'static str)] =
        &[("CARD_ID", "The numeric ID of sound card")];

    fn params(_: &[String]) -> Result<u32, String> {
        Arguments::try_parse()
            .map(|args| args.card_id)
            .map_err(|err| err.to_string())
    }
}

fn main() {
    FfServiceCmd::run()
}
