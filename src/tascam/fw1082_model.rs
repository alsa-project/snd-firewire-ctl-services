// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndTscmExtManual;

use crate::card_cntr;

use super::protocol::ClkSrc;
use super::common_ctl::CommonCtl;
use super::meter_ctl::MeterCtl;
use super::console_ctl::ConsoleCtl;

pub struct Fw1082Model<'a> {
    req: hinawa::FwReq,
    common: CommonCtl<'a>,
    meter: MeterCtl<'a>,
    console: ConsoleCtl,
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
            meter: MeterCtl::new(Self::CLK_SRC_LABELS, 2, false, false),
            console: ConsoleCtl::new(),
        }
    }
}

impl<'a> card_cntr::MonitorModel<hinawa::SndTscm> for Fw1082Model<'a> {
    fn get_monitored_elems(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.meter.get_monitored_elems());
        elem_id_list.extend_from_slice(&self.console.get_monitored_elems());
    }

    fn monitor_unit(&mut self, unit: &hinawa::SndTscm) -> Result<(), Error> {
        let states = unit.get_state()?;
        self.meter.parse_states(states);
        self.console.parse_states(states);
        Ok(())
    }

    fn monitor_elems(
        &mut self,
        unit: &hinawa::SndTscm,
        elem_id: &alsactl::ElemId,
        _: &alsactl::ElemValue,
        new: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        if self.meter.read(elem_id, new)? {
            Ok(true)
        } else if self.console.read(unit, &self.req, elem_id, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> card_cntr::CtlModel<hinawa::SndTscm> for Fw1082Model<'a> {
    fn load(
        &mut self,
        unit: &hinawa::SndTscm,
        card_cntr: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        self.common.load(unit, &self.req, card_cntr)?;
        self.meter.load(card_cntr)?;
        self.console.load(unit, &self.req, card_cntr)?;

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
        } else if self.console.read(unit, &self.req, elem_id, elem_value)? {
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
        } else if self.console.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
