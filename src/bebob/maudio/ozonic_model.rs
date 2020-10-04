// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{FwFcpExt, SndUnitExt};

use crate::card_cntr;
use card_cntr::{CtlModel, MeasureModel};

use crate::ta1394::{AvcAddr, MUSIC_SUBUNIT_0, Ta1394Avc};
use crate::ta1394::general::UnitInfo;
use crate::ta1394::ccm::{SignalAddr, SignalSubunitAddr};

use crate::bebob::BebobAvc;
use crate::bebob::common_ctls::ClkCtl;

use super::common_proto::FCP_TIMEOUT_MS;

pub struct OzonicModel<'a>{
    avc: BebobAvc,
    clk_ctl: ClkCtl<'a>,
}

impl<'a> OzonicModel<'a> {
    const CLK_DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr{
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x05,
    });
    const CLK_SRCS: &'a [SignalAddr] = &[
        SignalAddr::Subunit(SignalSubunitAddr{
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x05,
        }),
    ];
    const CLK_LABELS: &'a [&'a str] = &["Internal", "S/PDIF"];
    pub fn new() -> Self {
        OzonicModel{
            avc: BebobAvc::new(),
            clk_ctl: ClkCtl::new(&Self::CLK_DST, Self::CLK_SRCS, Self::CLK_LABELS),
        }
    }
}

impl<'a> CtlModel<hinawa::SndUnit> for OzonicModel<'a> {
    fn load(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        self.avc.fcp.bind(&unit.get_node())?;

        let mut op = UnitInfo::new();
        self.avc.status(&AvcAddr::Unit, &mut op, FCP_TIMEOUT_MS)?;
        self.avc.company_id = op.company_id;

        self.clk_ctl.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn read(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &hinawa::SndUnit, elem_id: &alsactl::ElemId,
             old: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.write(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> MeasureModel<hinawa::SndUnit> for OzonicModel<'a> {
    fn get_measure_elem_list(&mut self, _: &mut Vec<alsactl::ElemId>) {
    }

    fn measure_states(&mut self, _: &hinawa::SndUnit) -> Result<(), Error> {
        Ok(())
    }

    fn measure_elem(&mut self, _: &hinawa::SndUnit, _: &alsactl::ElemId, _: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        Ok(false)
    }
}

impl<'a> card_cntr::NotifyModel<hinawa::SndUnit, bool> for OzonicModel<'a> {
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
        self.clk_ctl.read(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)
    }
}
