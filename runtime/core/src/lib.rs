// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
pub mod card_cntr;
pub mod cmdline;
pub mod dispatcher;

use {clap::ValueEnum, glib::Error};

/// The level to debug runtime.
#[derive(ValueEnum, Debug, Copy, Clone, Eq, PartialEq)]
pub enum LogLevel {
    Debug,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Debug
    }
}

pub trait RuntimeOperation<T>: Sized {
    fn new(arg: T, log_level: Option<LogLevel>) -> Result<Self, Error>;
    fn listen(&mut self) -> Result<(), Error>;
    fn run(&mut self) -> Result<(), Error>;
}
