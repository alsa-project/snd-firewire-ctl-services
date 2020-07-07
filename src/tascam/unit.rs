// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

pub enum TascamUnit {
    Isoch,
    Asynch,
}

impl TascamUnit {
    pub fn new(subsystem: &String, _: u32) -> Result<Self, Error> {
        let unit = match subsystem.as_str() {
            "snd" => Self::Isoch,
            "fw" => Self::Asynch,
            _ => {
                let label = "Invalid name of subsystem";
                return Err(Error::new(FileError::Nodev, &label));
            }
        };

        Ok(unit)
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        Ok(())
    }

    pub fn run(self: &mut Self) {
    }
}
