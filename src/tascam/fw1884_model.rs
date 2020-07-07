// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndTscmExtManual;

use crate::card_cntr;

use super::protocol::ClkSrc;
use super::common_ctl::CommonCtl;
use super::meter_ctl::MeterCtl;

pub struct Fw1884Model<'a> {
    req: hinawa::FwReq,
    common: CommonCtl<'a>,
    meter: MeterCtl<'a>,
}

impl<'a> Fw1884Model<'a> {
    const CLK_SRCS: &'a [ClkSrc] = &[
        ClkSrc::Internal,
        ClkSrc::Wordclock,
        ClkSrc::Spdif,
        ClkSrc::Adat,
    ];

    const CLK_SRC_LABELS: &'a [&'a str] = &[
        "Internal",
        "Word-clock",
        "S/PDIF",
        "ADAT",
    ];

    pub fn new() -> Self {
        Fw1884Model{
            req: hinawa::FwReq::new(),
            common: CommonCtl::new(Self::CLK_SRCS,
                                   Self::CLK_SRC_LABELS),
            meter: MeterCtl::new(Self::CLK_SRC_LABELS, 8, true, true),
        }
    }
}

impl<'a> card_cntr::MonitorModel<hinawa::SndTscm> for Fw1884Model<'a> {
    fn get_monitored_elems(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.meter.get_monitored_elems());
    }

    fn monitor_unit(&mut self, unit: &hinawa::SndTscm) -> Result<(), Error> {
        let states = unit.get_state()?;
        self.meter.parse_states(states);
        Ok(())
    }

    fn monitor_elems(
        &mut self,
        _: &hinawa::SndTscm,
        elem_id: &alsactl::ElemId,
        _: &alsactl::ElemValue,
        new: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        if self.meter.read(elem_id, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> card_cntr::CtlModel<hinawa::SndTscm> for Fw1884Model<'a> {
    fn load(
        &mut self,
        unit: &hinawa::SndTscm,
        card_cntr: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        self.common.load(unit, &self.req, card_cntr)?;
        self.meter.load(card_cntr)?;
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
