// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{isoch_ctls::*, *},
    alsactl::*,
    protocols::isoch::fw1804::*,
};

pub struct Fw1804Model {
    req: FwReq,
    image: Vec<u32>,
    clock_ctl: ClockCtl<Fw1804Protocol>,
    input_threshold_ctl: InputDetectionThreshold<Fw1804Protocol>,
    coax_output_ctl: CoaxOutputCtl<Fw1804Protocol>,
    opt_iface_ctl: OpticalIfaceCtl<Fw1804Protocol>,
    rack_ctl: RackCtl<Fw1804Protocol>,
    meter_ctl: MeterCtl<Fw1804Protocol>,
}

impl Default for Fw1804Model {
    fn default() -> Self {
        Self {
            req: Default::default(),
            image: Fw1804Protocol::create_hardware_image(),
            clock_ctl: Default::default(),
            coax_output_ctl: Default::default(),
            input_threshold_ctl: Default::default(),
            opt_iface_ctl: Default::default(),
            rack_ctl: Default::default(),
            meter_ctl: Default::default(),
        }
    }
}

const TIMEOUT_MS: u32 = 50;

impl MeasureModel<(SndTascam, FwNode)> for Fw1804Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.elem_id_list);
    }

    fn measure_states(&mut self, unit: &mut (SndTascam, FwNode)) -> Result<(), Error> {
        unit.0.read_state(&mut self.image)?;
        self.meter_ctl.parse(&self.image)?;
        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &(SndTascam, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl CtlModel<(SndTascam, FwNode)> for Fw1804Model {
    fn cache(&mut self, (unit, node): &mut (SndTascam, FwNode)) -> Result<(), Error> {
        unit.read_state(&mut self.image)?;
        self.meter_ctl.parse(&self.image)?;

        self.clock_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.input_threshold_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.coax_output_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.opt_iface_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.rack_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clock_ctl.load(card_cntr)?;
        self.input_threshold_ctl.load(card_cntr)?;
        self.coax_output_ctl.load(card_cntr)?;
        self.opt_iface_ctl.load(card_cntr)?;
        self.rack_ctl.load(card_cntr)?;
        self.meter_ctl.load(card_cntr)?;
        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndTascam, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.clock_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_threshold_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.coax_output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.opt_iface_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.rack_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndTascam, FwNode),
        elem_id: &ElemId,
        _: &ElemValue,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        if self.clock_ctl.write(
            &mut unit.0,
            &mut self.req,
            &mut unit.1,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.input_threshold_ctl.write(
            &mut self.req,
            &mut unit.1,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.coax_output_ctl.write(
            &mut self.req,
            &mut unit.1,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.opt_iface_ctl.write(
            &mut self.req,
            &mut unit.1,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.rack_ctl.write(
            &mut self.req,
            &mut unit.1,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
