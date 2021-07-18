// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::Error;

use hinawa::FwFcpExt;
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemValue};

use core::card_cntr::*;

use ta1394::MUSIC_SUBUNIT_0;
use ta1394::ccm::{SignalAddr, SignalUnitAddr, SignalSubunitAddr};

use bebob_protocols::*;

use crate::common_ctls::ClkCtl;

const FCP_TIMEOUT_MS: u32 = 100;

pub struct Fca610Model<'a>{
    avc: BebobAvc,
    clk_ctl: ClkCtl<'a>,
}

impl<'a> Fca610Model<'a> {
    const CLK_DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr{
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x05,
    });
    const CLK_SRCS: &'a [SignalAddr] = &[
        SignalAddr::Unit(SignalUnitAddr::Ext(0x04)),
        SignalAddr::Unit(SignalUnitAddr::Ext(0x03)),
        // NOTE: This is the same source as Internal in former BeBoB models.
        SignalAddr::Subunit(SignalSubunitAddr{
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x07,
        }),
    ];
    const CLK_LABELS: &'a [&'a str] = &["Device Internal Clock", "S/PDIF", "Firewire Bus"];
}

impl<'a> Default for Fca610Model<'a> {
    fn default() -> Self {
        Self{
            avc: Default::default(),
            clk_ctl: ClkCtl::new(&Self::CLK_DST, Self::CLK_SRCS, &Self::CLK_LABELS),
        }
    }
}

impl<'a> CtlModel<SndUnit> for Fca610Model<'a> {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.get_node())?;

        self.clk_ctl.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn read(&mut self, _: &mut SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndUnit, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.write(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> NotifyModel<SndUnit, bool> for Fca610Model<'a> {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, _: &mut SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.clk_ctl.read(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)
    }
}
