// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {ff_runtime::FfRuntime, snd_firewire_ctl_services::*};

struct FfServiceCmd;

impl ServiceCmd<u32, FfRuntime> for FfServiceCmd {
    const CMD_NAME: &'static str = "snd-fireface-ctl-service";
    const ARGS: &'static [(&'static str, &'static str)] =
        &[("CARD_ID", "The numeric ID of sound card")];

    fn parse_args(args: &[String]) -> Result<u32, String> {
        parse_arg_as_u32(&args[0])
    }
}

fn main() {
    FfServiceCmd::run()
}
