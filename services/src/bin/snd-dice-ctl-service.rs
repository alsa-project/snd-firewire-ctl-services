// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use snd_firewire_ctl_services::*;
use dice_runtime::DiceRuntime;

struct DiceServiceCmd;

impl<'a> ServiceCmd<'a, u32, DiceRuntime> for DiceServiceCmd {
    const CMD_NAME: &'a str = "snd-dice-ctl-service";
    const ARGS: &'a [(&'a str, &'a str)] = &[("CARD_ID", "The numeric ID of sound card")];

    fn parse_args(args: &[String]) -> Result<u32, String> {
        parse_arg_as_u32(&args[0])
    }
}

fn main() {
    DiceServiceCmd::run()
}
