// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use crate::card_cntr;

use super::protocol::ClkSrc;
use super::common_ctl::CommonCtl;

pub struct Fw1082Model<'a> {
    req: hinawa::FwReq,
    common: CommonCtl<'a>,
}

impl<'a> Fw1082Model<'a> {
    const CLK_SRCS: &'a [ClkSrc] = &[
        ClkSrc::Internal,
        ClkSrc::Spdif,
    ];

    const CLK_SRC_LABELS: &'a [&'a str] = &[
        "Internal",
        "S/PDIF",
    ];

    pub fn new() -> Self {
        Fw1082Model{
            req: hinawa::FwReq::new(),
            common: CommonCtl::new(Self::CLK_SRCS,
                                   Self::CLK_SRC_LABELS),
        }
    }
}

impl<'a> card_cntr::MonitorModel<hinawa::SndTscm> for Fw1082Model<'a> {
    fn get_monitored_elems(&mut self, _: &mut Vec<alsactl::ElemId>) {
    }

    fn monitor_unit(&mut self, _: &hinawa::SndTscm) -> Result<(), Error> {
        Ok(())
    }

    fn monitor_elems(
        &mut self,
        _: &hinawa::SndTscm,
        _: &alsactl::ElemId,
        _: &alsactl::ElemValue,
        _: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        Ok(false)
    }
}

impl<'a> card_cntr::CtlModel<hinawa::SndTscm> for Fw1082Model<'a> {
    fn load(
        &mut self,
        unit: &hinawa::SndTscm,
        card_cntr: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        self.common.load(unit, &self.req, card_cntr)?;
        Ok(())
    }

    fn read(
        &mut self,
        unit: &hinawa::SndTscm,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        if self.common.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &hinawa::SndTscm,
        elem_id: &alsactl::ElemId,
        old: &alsactl::ElemValue,
        new: &alsactl::ElemValue,
    ) -> Result<bool, Error> {
        if self.common.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
