// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use core::card_cntr::*;

use hinawa::FwReq;
use hinawa::SndDg00x;

use alsactl::{ElemId, ElemValue};

use super::common_ctl::CommonCtl;
use super::monitor_ctl::MonitorCtl;

pub struct Digi002Model {
    req: FwReq,
    common: CommonCtl,
    monitor: MonitorCtl,
}

impl Default for Digi002Model {
    fn default() -> Self {
        Self {
            req: FwReq::new(),
            common: CommonCtl::new(false),
            monitor: MonitorCtl::new(),
        }
    }
}

impl NotifyModel<SndDg00x, bool> for Digi002Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.monitor.notified_elems);
    }

    fn parse_notification(&mut self, _: &mut SndDg00x, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        unit: &SndDg00x,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.monitor.read_notified_elems(unit, &self.req, elem_id, elem_value)
    }
}

impl CtlModel<SndDg00x> for Digi002Model {
    fn load(
        &mut self,
        unit: &mut SndDg00x,
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.common.load(&unit, &self.req, card_cntr)?;
        self.monitor.load(&unit, &self.req, card_cntr)?;
        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut SndDg00x,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else if self.monitor.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDg00x,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.common.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if self.monitor.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

pub struct Digi003Model {
    req: FwReq,
    common: CommonCtl,
    monitor: MonitorCtl,
}

impl Default for Digi003Model {
    fn default() -> Self {
        Self {
            req: FwReq::new(),
            common: CommonCtl::new(false),
            monitor: MonitorCtl::new(),
        }
    }
}

impl NotifyModel<SndDg00x, bool> for Digi003Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.monitor.notified_elems);
    }

    fn parse_notification(&mut self, _: &mut SndDg00x, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        unit: &SndDg00x,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.monitor.read_notified_elems(unit, &self.req, elem_id, elem_value)
    }
}

impl CtlModel<SndDg00x> for Digi003Model {
    fn load(
        &mut self,
        unit: &mut SndDg00x,
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.common.load(&unit, &self.req, card_cntr)?;
        self.monitor.load(&unit, &self.req, card_cntr)?;
        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut SndDg00x,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else if self.monitor.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDg00x,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.common.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if self.monitor.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
