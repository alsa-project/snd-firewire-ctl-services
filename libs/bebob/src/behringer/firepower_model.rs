// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{FwFcpExt, SndUnitExt};

use core::card_cntr;
use card_cntr::CtlModel;

use ta1394::MUSIC_SUBUNIT_0;
use ta1394::ccm::{SignalAddr, SignalUnitAddr, SignalSubunitAddr};

use super::super::BebobAvc;
use super::super::common_ctls::ClkCtl;

pub struct FirepowerModel<'a>{
    avc: BebobAvc,
    clk_ctl: ClkCtl<'a>,
}

impl<'a> FirepowerModel<'a> {
    const FCP_TIMEOUT_MS: u32 = 100;

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

impl<'a> Default for FirepowerModel<'a> {
    fn default() -> Self {
        Self{
            avc: BebobAvc::new(),
            clk_ctl: ClkCtl::new(&Self::CLK_DST, Self::CLK_SRCS, &Self::CLK_LABELS),
        }
    }
}

impl<'a> CtlModel<hinawa::SndUnit> for FirepowerModel<'a> {
    fn load(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        self.avc.fcp.bind(&unit.get_node())?;

        self.clk_ctl.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn read(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &hinawa::SndUnit, elem_id: &alsactl::ElemId, old: &alsactl::ElemValue,
             new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.write(unit, &self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> card_cntr::NotifyModel<hinawa::SndUnit, bool> for FirepowerModel<'a> {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, _: &hinawa::SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId,
                          elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        self.clk_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)
    }
}
