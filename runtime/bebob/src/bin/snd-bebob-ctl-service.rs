// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

use {bebob_runtime::BebobRuntime, core::cmdline::*};

struct BebobServiceCmd;

impl ServiceCmd<u32, BebobRuntime> for BebobServiceCmd {
    const CMD_NAME: &'static str = "snd-bebob-ctl-service";
    const ARGS: &'static [(&'static str, &'static str)] =
        &[("CARD_ID", "The numeric ID of sound card")];

    fn parse_args(args: &[String]) -> Result<u32, String> {
        parse_arg_as_u32(&args[0])
    }
}

fn main() {
    BebobServiceCmd::run()
}
