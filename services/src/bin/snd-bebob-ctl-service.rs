// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use bebob_runtime::BebobRuntime;
use snd_firewire_ctl_services::*;

struct BebobServiceCmd;

impl<'a> ServiceCmd<'a, u32, BebobRuntime> for BebobServiceCmd {
    const CMD_NAME: &'a str = "snd-bebob-ctl-service";
    const ARGS: &'a [(&'a str, &'a str)] = &[("CARD_ID", "The numeric ID of sound card")];

    fn parse_args(args: &[String]) -> Result<u32, String> {
        parse_arg_as_u32(&args[0])
    }
}

fn main() {
    BebobServiceCmd::run()
}
