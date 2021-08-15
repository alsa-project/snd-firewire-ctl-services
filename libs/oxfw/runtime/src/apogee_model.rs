// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{FwReq, FwFcp, FwFcpExt};
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt};

use core::card_cntr::*;

use ta1394::{Ta1394Avc, AvcAddr, AvcSubunitType};
use ta1394::general::UnitInfo;

use oxfw_protocols::apogee::*;

use super::common_ctl::CommonCtl;
use super::apogee_ctls::{OutputCtl, MixerCtl, InputCtl, DisplayCtl, HwState};

#[derive(Default, Debug)]
pub struct ApogeeModel{
    req: FwReq,
    avc: FwFcp,
    company_id: [u8; 3],
    common_ctl: CommonCtl,
    meter_ctl: MeterCtl,
    output_ctl: OutputCtl,
    mixer_ctl: MixerCtl,
    input_ctl: InputCtl,
    display_ctl: DisplayCtl,
    hwstate: HwState,
}

const TIMEOUT_MS: u32 = 50;

impl ApogeeModel {
    const FCP_TIMEOUT_MS: u32 = 100;
}

impl CtlModel<hinawa::SndUnit> for ApogeeModel {
    fn load(&mut self, unit: &mut hinawa::SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.bind(&unit.get_node())?;

        let mut op = UnitInfo{
            unit_type: AvcSubunitType::Reserved(0xff),
            unit_id: 0xff,
            company_id: [0xff;3],
        };
        self.avc.status(&AvcAddr::Unit, &mut op, 100)?;
        self.company_id.copy_from_slice(&op.company_id);

        self.common_ctl.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;
        self.meter_ctl.load_state(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
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
        } else if self.meter_ctl.read_state(elem_id, elem_value)? {
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

impl MeasureModel<hinawa::SndUnit> for ApogeeModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.2);
        elem_id_list.extend_from_slice(&self.hwstate.measure_elems);
    }

    fn measure_states(&mut self, unit: &mut hinawa::SndUnit) -> Result<(), Error> {
        self.meter_ctl.measure_state(unit, &mut self.req, TIMEOUT_MS)?;
        self.hwstate.measure_states(&unit.get_node(), &self.avc, &self.company_id)
    }

    fn measure_elem(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId,
                    elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.hwstate.measure_elems(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<hinawa::SndUnit, bool> for ApogeeModel {
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

#[derive(Default, Debug)]
struct MeterCtl(DuetFwInputMeterState, DuetFwMixerMeterState, Vec<ElemId>);

const ANALOG_INPUT_METER_NAME: &str = "analog-input-meters";
const STREAM_INPUT_METER_NAME: &str = "stream-input-meters";
const MIXER_OUTPUT_METER_NAME: &str = "mixer-output-meters";

impl MeterCtl {
    const ANALOG_INPUT_LABELS: [&'static str; 2] = ["analog-input-1", "analog-input-2"];
    const STREAM_INPUT_LABELS: [&'static str; 2] = ["stream-input-1", "stream-input-2"];
    const MIXER_OUTPUT_LABELS: [&'static str; 2] = ["mixer-output-1", "mixer-output-2"];

    fn load_state(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndUnit,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ANALOG_INPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                DuetFwInputMeterProtocol::LEVEL_MIN,
                DuetFwInputMeterProtocol::LEVEL_MAX,
                DuetFwInputMeterProtocol::LEVEL_STEP,
                Self::ANALOG_INPUT_LABELS.len(),
                None,
                false,
            )
            .map(|mut elem_id_list| self.2.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, STREAM_INPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                DuetFwMixerMeterProtocol::LEVEL_MIN,
                DuetFwMixerMeterProtocol::LEVEL_MAX,
                DuetFwMixerMeterProtocol::LEVEL_STEP,
                Self::STREAM_INPUT_LABELS.len(),
                None,
                false,
            )
            .map(|mut elem_id_list| self.2.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUTPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                DuetFwMixerMeterProtocol::LEVEL_MIN,
                DuetFwMixerMeterProtocol::LEVEL_MAX,
                DuetFwMixerMeterProtocol::LEVEL_STEP,
                Self::MIXER_OUTPUT_LABELS.len(),
                None,
                false,
            )
            .map(|mut elem_id_list| self.2.append(&mut elem_id_list))?;

        self.measure_state(unit, req, timeout_ms)
    }

    fn measure_state(
        &mut self,
        unit: &mut SndUnit,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let node = &mut unit.get_node();
        DuetFwInputMeterProtocol::read_state(req, node, &mut self.0, timeout_ms)?;
        DuetFwMixerMeterProtocol::read_state(req, node, &mut self.1, timeout_ms)?;
        Ok(())
    }

    fn read_state(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            ANALOG_INPUT_METER_NAME => {
                elem_value.set_int(&self.0.0);
                Ok(true)
            }
            STREAM_INPUT_METER_NAME => {
                elem_value.set_int(&self.1.stream_inputs);
                Ok(true)
            }
            MIXER_OUTPUT_METER_NAME => {
                elem_value.set_int(&self.1.mixer_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
