// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Application protocol specific to Avid Mbox 3 Pro
//!
//! The module includes structure, enumeration, and trait and its implementation for application
//! protocol specific to Avid Mbox 3 Pro.

use glib::Error;

use hinawa::{FwReq, FwNode};

use super::tcat::extension::{*, appl_section::*};

/// The enumeration to represent usecase of standalone mode.
pub enum StandaloneUseCase {
    Preamp,
    AdDa,
    Mixer,
}

impl StandaloneUseCase {
    const MIXER: u32 = 0;
    const AD_DA: u32 = 1;
    const PREAMP: u32 = 2;
}

impl From<u32> for StandaloneUseCase {
    fn from(val: u32) -> Self {
        match val {
            StandaloneUseCase::MIXER => StandaloneUseCase::Mixer,
            StandaloneUseCase::AD_DA => StandaloneUseCase::AdDa,
            _ => StandaloneUseCase::Preamp,
        }
    }
}

impl From<StandaloneUseCase> for u32 {
    fn from(use_case: StandaloneUseCase) -> u32 {
        match use_case {
            StandaloneUseCase::Mixer => StandaloneUseCase::MIXER,
            StandaloneUseCase::AdDa => StandaloneUseCase::AD_DA,
            StandaloneUseCase::Preamp => StandaloneUseCase::PREAMP,
        }
    }
}

/// The trait and its implementation to represent standalone protocol for Avid Mbox 3 pro.
pub trait AvidMbox3StandaloneProtocol<T> : ApplSectionProtocol<T>
    where T: AsRef<FwNode>,
{
    const USE_CASE_OFFSET: usize = 0x00;

    fn read_standalone_use_case(&self, node: &T, sections: &ExtensionSections, timeout_ms: u32)
        -> Result<StandaloneUseCase, Error>
    {
        let mut data = [0;4];
        self.read_appl_data(node, sections, Self::USE_CASE_OFFSET, &mut data, timeout_ms)
            .map(|_| u32::from_be_bytes(data).into())
    }

    fn write_standalone_use_case(&self, node: &T, sections: &ExtensionSections,
                                 use_case: StandaloneUseCase, timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut data = u32::from(use_case).to_be_bytes().clone();
        self.write_appl_data(node, sections, Self::USE_CASE_OFFSET, &mut data, timeout_ms)
    }
}

impl<O: AsRef<FwReq>, T: AsRef<FwNode>> AvidMbox3StandaloneProtocol<T> for O {}
