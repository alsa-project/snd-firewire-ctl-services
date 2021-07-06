// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use snd_firewire_ctl_services::*;
use efw_runtime::EfwRuntime;

struct EfwServiceCmd;

impl<'a> ServiceCmd<'a, u32, EfwRuntime> for EfwServiceCmd {
    const CMD_NAME: &'a str = "snd-fireworks-ctl-service";
    const ARGS: &'a [(&'a str, &'a str)] = &[("CARD_ID", "The numeric ID of sound card")];

    fn parse_args(args: &[String]) -> Result<u32, String> {
        parse_arg_as_u32(&args[0])
    }
}

fn main() {
    EfwServiceCmd::run()
}
