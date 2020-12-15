// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Application protocol specific to M-Audio ProFire series.
//!
//! The modules includes trait and its implementation for application protocol specific to M-Audio
//! ProFire series.

use glib::Error;

use hinawa::{FwReq, FwNode};

use super::tcat::extension::{*, appl_section::*};

/// The number of targets available to knob master.
pub const KNOB_COUNT: usize = 4;

/// The enumeration for mode of optical interface.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OptIfaceMode{
    Spdif,
    Adat,
}

/// The enumeration for mode of standalone converter.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StandaloneConerterMode{
    AdDa,
    AdOnly,
}

/// The trait for protocol defined by M-Audio specific to ProFire series.
pub trait MaudioPfireApplProtocol<T> : ApplSectionProtocol<T>
    where T: AsRef<FwNode>,
{
    const KNOB_ASSIGN_OFFSET: usize = 0x00;
    const STANDALONE_MODE_OFFSET: usize = 0x04;

    const KNOB_ASSIGN_MASK: u32 = 0x0f;
    const OPT_IFACE_B_IS_SPDIF_FLAG: u32 = 0x10;
    const STANDALONE_CONVERTER_IS_AD_ONLY_FLAG: u32 = 0x02;

    fn read_knob_assign(&self, node: &T, sections: &ExtensionSections, targets: &mut [bool;KNOB_COUNT],
                        timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut data = [0;4];
        self.read_appl_data(node, sections, Self::KNOB_ASSIGN_OFFSET, &mut data, timeout_ms)
            .map(|_| {
                let val = u32::from_be_bytes(data);
                targets.iter_mut()
                    .enumerate()
                    .for_each(|(i, v)| *v = val & (1 << i) > 0)
            })
    }

    fn write_knob_assign(&self, node: &T, sections: &ExtensionSections,
                         targets: &[bool;KNOB_COUNT], timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut data = [0;4];
        self.read_appl_data(node, sections, Self::KNOB_ASSIGN_OFFSET, &mut data, timeout_ms)?;
        let mut val = u32::from_be_bytes(data);

        targets.iter()
            .enumerate()
            .for_each(|(i, knob)| {
                val &= !(1 << i);
                if *knob {
                    val |= 1 << i;
                }
            });
        data.copy_from_slice(&val.to_be_bytes());

        self.write_appl_data(node, sections, Self::KNOB_ASSIGN_OFFSET, &mut data, timeout_ms)
    }

    fn read_opt_iface_b_mode(&self, node: &T, sections: &ExtensionSections, timeout_ms: u32)
        -> Result<OptIfaceMode, Error>
    {
        let mut data = [0;4];
        self.read_appl_data(node, sections, Self::KNOB_ASSIGN_OFFSET, &mut data, timeout_ms)
            .map(|_| {
                let val = u32::from_be_bytes(data);
                if val & Self::OPT_IFACE_B_IS_SPDIF_FLAG > 0 {
                    OptIfaceMode::Spdif
                } else {
                    OptIfaceMode::Adat
                }
            })
    }

    fn write_opt_iface_b_mode(&self, node: &T, sections: &ExtensionSections, mode: OptIfaceMode,
                              timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut data = [0;4];
        self.read_appl_data(node, sections, Self::KNOB_ASSIGN_OFFSET, &mut data, timeout_ms)?;
        let mut val = u32::from_be_bytes(data);

        val &= !Self::OPT_IFACE_B_IS_SPDIF_FLAG;
        if mode == OptIfaceMode::Spdif {
            val |= Self::OPT_IFACE_B_IS_SPDIF_FLAG;
        }
        data.copy_from_slice(&val.to_be_bytes());

        self.write_appl_data(node, sections, Self::KNOB_ASSIGN_OFFSET, &mut data, timeout_ms)
    }

    fn read_standalone_converter_mode(&self, node: &T, sections: &ExtensionSections, timeout_ms: u32)
        -> Result<StandaloneConerterMode, Error>
    {
        let mut data = [0;4];
        self.read_appl_data(node, sections, Self::STANDALONE_MODE_OFFSET, &mut data, timeout_ms)
            .map(|_| {
                let val = u32::from_be_bytes(data);
                if val & Self::STANDALONE_CONVERTER_IS_AD_ONLY_FLAG > 0 {
                    StandaloneConerterMode::AdOnly
                } else {
                    StandaloneConerterMode::AdDa
                }
            })
    }

    fn write_standalone_converter_mode(&self, node: &T, sections: &ExtensionSections,
                                       mode: StandaloneConerterMode, timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut data = [0;4];
        self.read_appl_data(node, sections, Self::STANDALONE_MODE_OFFSET, &mut data, timeout_ms)?;
        let mut val = u32::from_be_bytes(data);

        val &= !Self::STANDALONE_CONVERTER_IS_AD_ONLY_FLAG;
        if mode == StandaloneConerterMode::AdOnly {
            val |= Self::STANDALONE_CONVERTER_IS_AD_ONLY_FLAG;
        }
        data.copy_from_slice(&val.to_be_bytes());

        self.write_appl_data(node, sections, Self::STANDALONE_MODE_OFFSET, &mut data, timeout_ms)
    }
}

impl<O: AsRef<FwReq>, T: AsRef<FwNode>> MaudioPfireApplProtocol<T> for O {}
