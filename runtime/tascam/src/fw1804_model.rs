// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{isoch_ctls::*, *},
    alsactl::*,
    protocols::isoch::{fw1804::*, *},
};

pub struct Fw1804Model {
    req: FwReq,
    image: Vec<u32>,
    meter_ctl: MeterCtl,
    common_ctl: CommonCtl,
    optical_ctl: OpticalCtl,
    rack_ctl: RackCtl,
}

impl Default for Fw1804Model {
    fn default() -> Self {
        Self {
            req: Default::default(),
            image: vec![0u32; 64],
            meter_ctl: Default::default(),
            common_ctl: Default::default(),
            optical_ctl: Default::default(),
            rack_ctl: Default::default(),
        }
    }
}

const TIMEOUT_MS: u32 = 50;

#[derive(Default)]
struct MeterCtl(IsochMeterState, Vec<ElemId>);

impl IsochMeterCtlOperation<Fw1804Protocol> for MeterCtl {
    fn meter(&self) -> &IsochMeterState {
        &self.0
    }

    fn meter_mut(&mut self) -> &mut IsochMeterState {
        &mut self.0
    }

    const INPUT_LABELS: &'static [&'static str] = &[
        "analog-input-1",
        "analog-input-2",
        "analog-input-3",
        "analog-input-4",
        "analog-input-5",
        "analog-input-6",
        "analog-input-7",
        "analog-input-8",
        "adat-input-1",
        "adat-input-2",
        "adat-input-3",
        "adat-input-4",
        "adat-input-5",
        "adat-input-6",
        "adat-input-7",
        "adat-input-8",
        "spdif-input-1",
        "spdif-input-2",
    ];
    const OUTPUT_LABELS: &'static [&'static str] = &[
        "analog-output-1",
        "analog-output-2",
        "analog-output-3",
        "analog-output-4",
        "analog-output-5",
        "analog-output-6",
        "analog-output-7",
        "analog-output-8",
        "adat-output-1",
        "adat-output-2",
        "adat-output-3",
        "adat-output-4",
        "adat-output-5",
        "adat-output-6",
        "adat-output-7",
        "adat-output-8",
        "spdif-input-1",
        "spdif-input-2",
    ];
}

#[derive(Default)]
struct CommonCtl;

impl IsochCommonCtlOperation<Fw1804Protocol> for CommonCtl {}

#[derive(Default)]
struct OpticalCtl;

impl IsochOpticalCtlOperation<Fw1804Protocol> for OpticalCtl {
    const OPTICAL_OUTPUT_SOURCES: &'static [OpticalOutputSource] = &[
        OpticalOutputSource::StreamInputPairs,
        OpticalOutputSource::CoaxialOutputPair0,
        OpticalOutputSource::AnalogInputPair0,
    ];
}

#[derive(Default)]
struct RackCtl(IsochRackState);

impl IsochRackCtlOperation<Fw1804Protocol> for RackCtl {
    fn state(&self) -> &IsochRackState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut IsochRackState {
        &mut self.0
    }
}

impl MeasureModel<(SndTascam, FwNode)> for Fw1804Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut (SndTascam, FwNode)) -> Result<(), Error> {
        unit.0.read_state(&mut self.image)?;
        self.meter_ctl.parse_state(&self.image)
    }

    fn measure_elem(
        &mut self,
        _: &(SndTascam, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl CtlModel<(SndTascam, FwNode)> for Fw1804Model {
    fn load(
        &mut self,
        unit: &mut (SndTascam, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        unit.0.read_state(&mut self.image)?;
        self.meter_ctl.load_state(card_cntr, &self.image)?;
        self.common_ctl.load_params(card_cntr)?;
        self.optical_ctl.load_params(card_cntr)?;
        self.rack_ctl
            .load_params(card_cntr, &mut unit.1, &mut self.req, TIMEOUT_MS)?;
        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut (SndTascam, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.common_ctl.read_params(
            &mut unit.1,
            &mut self.req,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.optical_ctl.read_params(
            &mut unit.1,
            &mut self.req,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.rack_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndTascam, FwNode),
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .common_ctl
            .write_params(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.optical_ctl.write_params(
            &mut unit.1,
            &mut self.req,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.rack_ctl.write_params(
            &mut unit.1,
            &mut self.req,
            elem_id,
            old,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
