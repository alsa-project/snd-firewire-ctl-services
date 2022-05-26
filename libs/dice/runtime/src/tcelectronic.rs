// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
pub mod desktopk6_model;
pub mod itwin_model;
pub mod k24d_model;
pub mod k8_model;
pub mod klive_model;
pub mod studiok48_model;

pub mod ch_strip_ctl;
pub mod fw_led_ctl;
pub mod midi_send_ctl;
pub mod prog_ctl;
pub mod reverb_ctl;
pub mod shell_ctl;
pub mod standalone_ctl;

use {
    self::ch_strip_ctl::*,
    self::fw_led_ctl::*,
    self::midi_send_ctl::*,
    self::prog_ctl::*,
    self::reverb_ctl::*,
    self::standalone_ctl::*,
    super::{common_ctl::*, *},
    dice_protocols::{
        tcat::{global_section::*, *},
        tcelectronic::{
            ch_strip::*, fw_led::*, midi_send::*, prog::*, reverb::*, standalone::*, *,
        },
    },
};
