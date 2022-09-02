// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {core::cmdline::*, motu_runtime::MotuRuntime};

struct MotuServiceCmd;

impl ServiceCmd<u32, MotuRuntime> for MotuServiceCmd {
    const CMD_NAME: &'static str = "snd-firewire-motu-ctl-service";
    const ARGS: &'static [(&'static str, &'static str)] =
        &[("CARD_ID", "The numeric ID of sound card")];

    fn parse_args(args: &[String]) -> Result<u32, String> {
        parse_arg_as_u32(&args[0])
    }
}

fn main() {
    MotuServiceCmd::run()
}
