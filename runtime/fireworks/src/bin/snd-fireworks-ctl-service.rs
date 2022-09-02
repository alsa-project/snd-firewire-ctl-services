// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

use {core::cmdline::*, fireworks_runtime::EfwRuntime};

struct EfwServiceCmd;

impl ServiceCmd<u32, EfwRuntime> for EfwServiceCmd {
    const CMD_NAME: &'static str = "snd-fireworks-ctl-service";
    const ARGS: &'static [(&'static str, &'static str)] =
        &[("CARD_ID", "The numeric ID of sound card")];

    fn parse_args(args: &[String]) -> Result<u32, String> {
        parse_arg_as_u32(&args[0])
    }
}

fn main() {
    EfwServiceCmd::run()
}
