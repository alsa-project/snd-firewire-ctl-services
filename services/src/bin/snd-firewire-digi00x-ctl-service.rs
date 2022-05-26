// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use dg00x_runtime::Dg00xRuntime;
use snd_firewire_ctl_services::*;

struct Dg00xServiceCmd;

impl<'a> ServiceCmd<'a, u32, Dg00xRuntime> for Dg00xServiceCmd {
    const CMD_NAME: &'a str = "snd-firewire-digi00x-ctl-service";
    const ARGS: &'a [(&'a str, &'a str)] = &[("CARD_ID", "The numeric ID of sound card")];

    fn parse_args(args: &[String]) -> Result<u32, String> {
        parse_arg_as_u32(&args[0])
    }
}

fn main() {
    Dg00xServiceCmd::run()
}
