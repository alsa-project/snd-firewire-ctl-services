// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use core::RuntimeOperation;

pub struct FfRuntime;

impl RuntimeOperation<u32> for FfRuntime {
    fn new(_: u32) -> Result<Self, Error> {
        Ok(FfRuntime{})
    }

    fn listen(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn run(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
