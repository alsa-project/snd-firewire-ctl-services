// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Loaded program protocol defined by TC Electronic for Konnekt series.
//! The module includes structure for data of loaded program defined by TC Electronic For Konnekt
//! series.

use super::*;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct TcKonnektLoadedProgram(pub u32);

impl TcKonnektLoadedProgram {
    pub fn build(&self, raw: &mut [u8]) {
        self.0.build_quadlet(&mut raw[..4]);
    }

    pub fn parse(&mut self, raw: &[u8]) {
        self.0.parse_quadlet(&raw[..4]);
    }
}
