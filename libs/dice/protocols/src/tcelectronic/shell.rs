// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol defined by TC Electronic for Konnekt 24d, Konnekt 8, Konnekt Live, and Impact Twin.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Konnekt 24d, Konnekt 8, Konnekt Live, and Impact Twin.

pub mod k8;
pub mod k24d;
pub mod klive;
pub mod itwin;

const SHELL_CH_STRIP_NOTIFY_FLAG: u32 = 0x00100000;

const SHELL_CH_STRIP_COUNT: usize = 2;
