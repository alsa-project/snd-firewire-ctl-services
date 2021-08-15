// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{SndUnitExt, FwFcpExt};

use core::card_cntr;

use ta1394::{Ta1394Avc, AvcAddr, AvcSubunitType};
use ta1394::general::UnitInfo;

use super::common_ctl::CommonCtl;
use super::apogee_ctls::{OutputCtl, MixerCtl, InputCtl, DisplayCtl, HwState};

#[derive(Default, Debug)]
pub struct ApogeeModel{
    avc: hinawa::FwFcp,
    company_id: [u8; 3],
    common_ctl: CommonCtl,
    output_ctl: OutputCtl,
    mixer_ctl: MixerCtl,
    input_ctl: InputCtl,
    display_ctl: DisplayCtl,
    hwstate: HwState,
}

impl ApogeeModel {
    const FCP_TIMEOUT_MS: u32 = 100;
}

impl card_cntr::CtlModel<hinawa::SndUnit> for ApogeeModel {
    fn load(&mut self, unit: &mut hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        self.avc.bind(&unit.get_node())?;

        let mut op = UnitInfo{
            unit_type: AvcSubunitType::Reserved(0xff),
            unit_id: 0xff,
            company_id: [0xff;3],
        };
        self.avc.status(&AvcAddr::Unit, &mut op, 100)?;
        self.company_id.copy_from_slice(&op.company_id);

        self.common_ctl.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;
        self.output_ctl.load(&self.avc, card_cntr)?;
        self.mixer_ctl.load(&self.avc, card_cntr)?;
        self.input_ctl.load(&self.avc, card_cntr)?;
        self.display_ctl.load(&self.avc, card_cntr)?;
        self.hwstate.load(&self.avc, card_cntr)?;

        Ok(())
    }

    fn read(&mut self, _: &mut hinawa::SndUnit, elem_id: &alsactl::ElemId,
            elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.common_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.output_ctl.read(&self.avc, &self.company_id, elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(&self.avc, &self.company_id, elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read(&self.avc, &self.company_id, elem_id, elem_value)? {
            Ok(true)
        } else if self.display_ctl.read(&self.avc, &self.company_id, elem_id, elem_value)? {
            Ok(true)
        } else if self.hwstate.read(&self.avc, &self.company_id, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut hinawa::SndUnit, elem_id: &alsactl::ElemId, old: &alsactl::ElemValue,
             new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.common_ctl.write(unit, &self.avc, elem_id, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.output_ctl.write(&self.avc, &self.company_id, elem_id, old, new)? {
            Ok(true)
        } else if self.mixer_ctl.write(&self.avc, &self.company_id, elem_id, old, new)? {
            Ok(true)
        } else if self.input_ctl.write(&self.avc, &self.company_id, elem_id, old, new)? {
            Ok(true)
        } else if self.display_ctl.write(&self.avc, &self.company_id, elem_id, old, new)? {
            Ok(true)
        } else if self.hwstate.write(&self.avc, &self.company_id, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl card_cntr::MeasureModel<hinawa::SndUnit> for ApogeeModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.hwstate.measure_elems);
    }

    fn measure_states(&mut self, unit: &mut hinawa::SndUnit) -> Result<(), Error> {
        self.hwstate.measure_states(&unit.get_node(), &self.avc, &self.company_id)
    }

    fn measure_elem(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId,
                    elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        self.hwstate.measure_elems(elem_id, elem_value)
    }
}

impl card_cntr::NotifyModel<hinawa::SndUnit, bool> for ApogeeModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, _: &mut hinawa::SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        self.common_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)
    }
}
