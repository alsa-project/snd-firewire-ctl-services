// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {oxfw_runtime::OxfwRuntime, snd_firewire_ctl_services::*};

struct OxfwServiceCmd;

impl ServiceCmd<u32, OxfwRuntime> for OxfwServiceCmd {
    const CMD_NAME: &'static str = "snd-oxfw-ctl-service";
    const ARGS: &'static [(&'static str, &'static str)] =
        &[("CARD_ID", "The numeric ID of sound card")];

    fn parse_args(args: &[String]) -> Result<u32, String> {
        parse_arg_as_u32(&args[0])
    }
}

fn main() {
    OxfwServiceCmd::run()
}
