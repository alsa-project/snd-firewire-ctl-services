// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{FwFcpExt, SndUnitExt};

use core::card_cntr;

use super::super::BebobAvc;

use super::special_ctls::{ClkCtl, MeterCtl, StateCache, MixerCtl, InputCtl, OutputCtl, AuxCtl, HpCtl};

pub struct SpecialModel {
    avc: BebobAvc,
    req: hinawa::FwReq,
    clk_ctl: ClkCtl,
    meter_ctl: MeterCtl,
    cache: StateCache,
}

impl SpecialModel {
    pub fn new(is_fw1814: bool) -> Self {
        SpecialModel {
            avc: BebobAvc::new(),
            req: hinawa::FwReq::new(),
            clk_ctl: ClkCtl::new(is_fw1814),
            meter_ctl: MeterCtl::new(),
            cache: StateCache::new(),
        }
    }
}

impl card_cntr::CtlModel<hinawa::SndUnit> for SpecialModel {
    fn load(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        self.avc.fcp.bind(&unit.get_node())?;

        self.clk_ctl.load(card_cntr)?;
        self.meter_ctl.load(unit, &self.req, &self.avc, card_cntr)?;

        MixerCtl::load(&mut self.cache, card_cntr)?;
        InputCtl::load(&mut self.cache, card_cntr)?;
        OutputCtl::load(&mut self.cache, card_cntr)?;
        AuxCtl::load(&mut self.cache, card_cntr)?;
        HpCtl::load(&mut self.cache, card_cntr)?;

        self.cache.upload(unit, &self.req)?;

        Ok(())
    }

    fn read(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read(&self.avc, elem_id, elem_value)? {
            Ok(true)
        } else if MixerCtl::read(&mut self.cache, elem_id, elem_value)? {
            Ok(true)
        } else if InputCtl::read(&mut self.cache, elem_id, elem_value)? {
            Ok(true)
        } else if OutputCtl::read(&mut self.cache, elem_id, elem_value)? {
            Ok(true)
        } else if AuxCtl::read(&mut self.cache, elem_id, elem_value)? {
            Ok(true)
        } else if HpCtl::read(&mut self.cache, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &hinawa::SndUnit, elem_id: &alsactl::ElemId,
             old: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.write(unit, &self.avc, elem_id, old, new)? {
            Ok(true)
        } else if MixerCtl::write(&mut self.cache, unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if InputCtl::write(&mut self.cache, unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if OutputCtl::write(&mut self.cache, unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if AuxCtl::write(&mut self.cache, unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if HpCtl::write(&mut self.cache, unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl card_cntr::MeasureModel<hinawa::SndUnit> for SpecialModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.measure_elems);
    }

    fn measure_states(&mut self, unit: &hinawa::SndUnit) -> Result<(), Error> {
        self.meter_ctl.measure_states(unit, &self.req, &self.avc)
    }

    fn measure_elem(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        self.meter_ctl.measure_elem(elem_id, elem_value)
    }
}

impl card_cntr::NotifyModel<hinawa::SndUnit, bool> for SpecialModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, _: &hinawa::SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        self.clk_ctl.read(&self.avc, elem_id, elem_value)
    }
}
