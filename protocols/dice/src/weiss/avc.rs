// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2024 Takashi Sakamoto

//! Protocol specific to Weiss Engineering AV/C models.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Weiss Engineering.
//!
//! MAN301 includes two units in the root directory of its configuration ROM. The first unit
//! expresses AV/C protocol, and the second unit expresses TCAT protocol.

use super::*;

/// Protocol implementation specific to MAN301.
#[derive(Default, Debug)]
pub struct WeissMan301Protocol;

impl TcatOperation for WeissMan301Protocol {}

// clock caps: 44100 48000 88200 96000 176400 192000
// clock source names: AES/EBU (XLR)\S/PDIF (RCA)\S/PDIF (TOS)\Unused\Unused\Unused\Unused\Word Clock (BNC)\Unused\Unused\Unused\Unused\Internal\\
impl TcatGlobalSectionSpecification for WeissMan301Protocol {}
