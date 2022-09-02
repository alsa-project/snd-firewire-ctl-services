// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
pub mod card_cntr;
pub mod dispatcher;
pub mod elem_value_accessor;

use glib::Error;

pub trait RuntimeOperation<T>: Sized {
    fn new(arg: T) -> Result<Self, Error>;
    fn listen(&mut self) -> Result<(), Error>;
    fn run(&mut self) -> Result<(), Error>;
}
