// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocols defined for Oxford Semiconductor FW970/971 chipset.
//!
//! The crate includes various kind of protocols defined by Oxford Semiconductor as well as
//! hardware vendors for FW970/971 ASICs.

pub mod apogee;
pub mod griffin;
pub mod lacie;
pub mod loud;
pub mod oxford;
pub mod tascam;

use {
    glib::{Error, FileError},
    hinawa::{FwFcp, FwNode, FwReq, FwReqExtManual, FwTcode},
    ta1394::{audio::*, ccm::*, general::*, *},
};
