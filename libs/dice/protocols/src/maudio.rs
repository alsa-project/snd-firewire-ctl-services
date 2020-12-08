// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Application protocol specific to M-Audio ProFire series.
//!
//! The modules includes trait and its implementation for application protocol specific to M-Audio
//! ProFire series.

use glib::Error;

use hinawa::{FwReq, FwNode};

use super::tcat::extension::{*, appl_section::*};

pub const KNOB_COUNT: usize = 4;

pub trait MaudioPfireApplProtocol<T> : ApplSectionProtocol<T>
    where T: AsRef<FwNode>,
{
    const KNOB_ASSIGN_OFFSET: usize = 0x00;

    fn write_knob_assign(&self, node: &T, sections: &ExtensionSections,
                         targets: &[bool;KNOB_COUNT], timeout_ms: u32)
        -> Result<(), Error>
    {
        let val: u32 = targets.iter()
            .enumerate()
            .filter(|&(_, &knob)| knob)
            .fold(0, |val, (i, _)| val | (1 << i));
        let mut data = val.to_be_bytes().clone();
        self.write_appl_data(node, sections, Self::KNOB_ASSIGN_OFFSET, &mut data, timeout_ms)
    }
}

impl<O: AsRef<FwReq>, T: AsRef<FwNode>> MaudioPfireApplProtocol<T> for O {}
