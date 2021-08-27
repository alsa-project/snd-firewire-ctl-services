// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndTscmExtManual;

use alsactl::ElemId;

use core::card_cntr;

use tascam_protocols::isoch::{fw1804::*, *};

use super::isoch_ctls::*;

use super::optical_ctl::OpticalCtl;
use super::rack_ctl::RackCtl;

pub struct Fw1804Model<'a> {
    req: hinawa::FwReq,
    meter_ctl: MeterCtl,
    common_ctl: CommonCtl,
    optical: OpticalCtl<'a>,
    rack: RackCtl,
}

const TIMEOUT_MS: u32 = 50;

#[derive(Default)]
struct MeterCtl(IsochMeterState, Vec<ElemId>);

impl AsRef<IsochMeterState> for MeterCtl {
    fn as_ref(&self) -> &IsochMeterState {
        &self.0
    }
}

impl AsMut<IsochMeterState> for MeterCtl {
    fn as_mut(&mut self) -> &mut IsochMeterState {
        &mut self.0
    }
}

impl IsochMeterCtl<Fw1804Protocol> for MeterCtl {
    const INPUT_LABELS: &'static [&'static str] = &[
        "analog-input-1", "analog-input-2", "analog-input-3", "analog-input-4",
        "analog-input-5", "analog-input-6", "analog-input-7", "analog-input-8",
        "adat-input-1", "adat-input-2", "adat-input-3", "adat-input-4",
        "adat-input-5", "adat-input-6", "adat-input-7", "adat-input-8",
        "spdif-input-1", "spdif-input-2",
    ];
    const OUTPUT_LABELS: &'static [&'static str] = &[
        "analog-output-1", "analog-output-2", "analog-output-3", "analog-output-4",
        "analog-output-5", "analog-output-6", "analog-output-7", "analog-output-8",
        "adat-output-1", "adat-output-2", "adat-output-3", "adat-output-4",
        "adat-output-5", "adat-output-6", "adat-output-7", "adat-output-8",
        "spdif-input-1", "spdif-input-2",
    ];
}

#[derive(Default)]
struct CommonCtl;

impl IsochCommonCtl<Fw1804Protocol> for CommonCtl {}

impl<'a> Fw1804Model<'a> {
    const OPT_OUT_SRC_LABELS: &'a [&'a str] = &[
        "ADAT-1/2/3/4/5/6/7/8",
        "S/PDIF-1/2",
        "Analog-1/2",
    ];

    pub fn new() -> Self {
        Fw1804Model{
            req: hinawa::FwReq::new(),
            meter_ctl: Default::default(),
            common_ctl: Default::default(),
            optical: OpticalCtl::new(Self::OPT_OUT_SRC_LABELS),
            rack: RackCtl::new(),
        }
    }
}

impl<'a> card_cntr::MeasureModel<hinawa::SndTscm> for Fw1804Model<'a> {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut hinawa::SndTscm) -> Result<(), Error> {
        let image = unit.get_state()?;
        self.meter_ctl.parse_state(image)
    }

    fn measure_elem(&mut self, _: &hinawa::SndTscm, elem_id: &alsactl::ElemId,
                    elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> card_cntr::CtlModel<hinawa::SndTscm> for Fw1804Model<'a> {
    fn load(
        &mut self,
        unit: &mut hinawa::SndTscm,
        card_cntr: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        let image = unit.get_state()?;
        self.meter_ctl.load_state(card_cntr, image)?;
        self.common_ctl.load_params(card_cntr)?;
        self.optical.load(unit, &self.req, card_cntr)?;
        self.rack.load(unit, &self.req, card_cntr)?;
        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut hinawa::SndTscm,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.common_ctl.read_params(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)? {
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
        unit: &mut hinawa::SndTscm,
        elem_id: &alsactl::ElemId,
        old: &alsactl::ElemValue,
        new: &alsactl::ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.write_params(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
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
