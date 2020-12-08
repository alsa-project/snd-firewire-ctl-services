// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::FwReq;
use hinawa::{SndDice, SndUnitExt};

use core::card_cntr::*;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::tcat::extension::{*, caps_section::*, cmd_section::*};

use super::tcd22xx_spec::{Tcd22xxState, Tcd22xxSpec, Tcd22xxStateOperation};

#[derive(Default, Debug)]
pub struct Tcd22xxCtl<S>
    where for<'a> S: Tcd22xxSpec<'a> + AsRef<Tcd22xxState> + AsMut<Tcd22xxState>,
{
    state: S,
    caps: ExtensionCaps,
}

impl<S> Tcd22xxCtl<S>
    where for<'a> S: Tcd22xxSpec<'a> + AsRef<Tcd22xxState> + AsMut<Tcd22xxState>,
{
    pub fn load(&mut self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections,
                _: &ClockCaps, _: &ClockSourceLabels, timeout_ms: u32, _: &mut CardCntr)
        -> Result<(), Error>
    {
        let node = unit.get_node();

        self.caps = proto.read_caps(&node, sections, timeout_ms)?;

        Ok(())
    }

    pub fn cache(&mut self, unit: &SndDice, proto: &FwReq, sections: &GeneralSections,
                 extension_sections: &ExtensionSections, timeout_ms: u32)
        -> Result<(), Error>
    {
        let node = unit.get_node();
        let config = proto.read_clock_config(&node, &sections, timeout_ms)?;
        let rate_mode = RateMode::from(config.rate);

        self.state.cache(&node, proto, extension_sections, &self.caps, rate_mode, timeout_ms)
    }
}
