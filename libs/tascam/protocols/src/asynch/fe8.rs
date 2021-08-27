// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for FE-8
//!
//! The module includes protocol implementation defined by Tascam for FE-8.

use crate::asynch::*;

#[derive(Default)]
pub struct Fe8Protocol;

impl ExpanderOperation for Fe8Protocol {}
