// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use core::RuntimeOperation;

pub struct DiceRuntime;

impl RuntimeOperation<u32> for DiceRuntime {
    fn new(_: u32) -> Result<Self, Error> {
        Ok(DiceRuntime{})
    }

    fn listen(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn run(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
