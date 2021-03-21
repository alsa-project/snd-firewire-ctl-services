// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::Error;

use hinawa::{SndUnit, SndUnitExt, FwFcpExt};
use alsactl::{ElemId, ElemValue};

use core::card_cntr::*;

use ta1394::{*, ccm::*};

use super::{*, common_ctls::ClkCtl};

pub struct QuatafireModel<'a>{
    avc: BebobAvc,
    clk_ctl: ClkCtl<'a>,
}

impl<'a> QuatafireModel<'a> {
    const FCP_TIMEOUT_MS: u32 = 100;

    const CLK_DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr{
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x01,
    });
    const CLK_SRCS: [SignalAddr;1] = [
        SignalAddr::Subunit(SignalSubunitAddr{
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x01,
        }),
    ];

    const CLK_LABELS: [&'a str;1] = [
        "Internal",
    ];
}

impl<'a> Default for QuatafireModel<'a> {
    fn default() -> Self {
        Self{
            avc: Default::default(),
            clk_ctl: ClkCtl::new(&Self::CLK_DST, &Self::CLK_SRCS, &Self::CLK_LABELS),
        }
    }
}

impl<'a> CtlModel<SndUnit> for QuatafireModel<'a> {
    fn load(&mut self, unit: &SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.fcp.bind(&unit.get_node())?;
        self.clk_ctl.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;
        Ok(())
    }

    fn read(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.clk_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)
    }

    fn write(&mut self, unit: &SndUnit, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        self.clk_ctl.write(unit, &self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)
    }
}

impl<'a> NotifyModel<SndUnit, bool> for QuatafireModel<'a> {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, _: &SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.clk_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)
    }
}
