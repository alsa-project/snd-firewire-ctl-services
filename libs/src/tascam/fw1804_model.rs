// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndTscmExtManual;

use core::card_cntr;

use super::protocol::ClkSrc;
use super::common_ctl::CommonCtl;
use super::meter_ctl::MeterCtl;
use super::optical_ctl::OpticalCtl;
use super::rack_ctl::RackCtl;

pub struct Fw1804Model<'a> {
    req: hinawa::FwReq,
    common: CommonCtl<'a>,
    meter: MeterCtl<'a>,
    optical: OpticalCtl<'a>,
    rack: RackCtl,
}

impl<'a> Fw1804Model<'a> {
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

    const OPT_OUT_SRC_LABELS: &'a [&'a str] = &[
        "ADAT-1/2/3/4/5/6/7/8",
        "S/PDIF-1/2",
        "Analog-1/2",
    ];

    pub fn new() -> Self {
        Fw1804Model{
            req: hinawa::FwReq::new(),
            common: CommonCtl::new(Self::CLK_SRCS,
                                   Self::CLK_SRC_LABELS),
            meter: MeterCtl::new(Self::CLK_SRC_LABELS, 2, true, false),
            optical: OpticalCtl::new(Self::OPT_OUT_SRC_LABELS),
            rack: RackCtl::new(),
        }
    }
}

impl<'a> card_cntr::MeasureModel<hinawa::SndTscm> for Fw1804Model<'a> {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.meter.measure_elems);
    }

    fn measure_states(&mut self, unit: &hinawa::SndTscm) -> Result<(), Error> {
        let states = unit.get_state()?;
        self.meter.parse_states(states);
        Ok(())
    }

    fn measure_elem(&mut self, _: &hinawa::SndTscm, elem_id: &alsactl::ElemId,
                    elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.meter.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> card_cntr::CtlModel<hinawa::SndTscm> for Fw1804Model<'a> {
    fn load(
        &mut self,
        unit: &hinawa::SndTscm,
        card_cntr: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        self.common.load(unit, &self.req, card_cntr)?;
        self.meter.load(card_cntr)?;
        self.optical.load(unit, &self.req, card_cntr)?;
        self.rack.load(unit, &self.req, card_cntr)?;
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
        } else if self.optical.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else if self.rack.read(unit, &self.req, elem_id, elem_value)? {
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
        } else if self.optical.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if self.rack.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
