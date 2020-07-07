// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use super::async_unit::ConsoleData;

pub struct Fe8Model{}

impl<'a> ConsoleData<'a> for Fe8Model {
    const FW_LED: &'a [u16] = &[0x16, 0x8e];
}
