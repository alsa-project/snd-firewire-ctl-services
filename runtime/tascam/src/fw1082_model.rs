// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{isoch_ctls::*, *},
    alsactl::*,
    protocols::isoch::{fw1082::*, *},
};

pub struct Fw1082Model {
    req: FwReq,
    image: Vec<u32>,
    clock_ctl: ClockCtl<Fw1082Protocol>,
    input_threshold_ctl: InputDetectionThreshold<Fw1082Protocol>,
    coax_output_ctl: CoaxOutputCtl<Fw1082Protocol>,
    meter_ctl: MeterCtl<Fw1082Protocol>,
    console_ctl: ConsoleCtl<Fw1082Protocol>,
    common_state: TascamSurfaceCommonState,
    isoch_state: TascamSurfaceIsochState,
    specific_state: TascamSurfaceFw1082State,
    machine_state: MachineState,
}

impl Default for Fw1082Model {
    fn default() -> Self {
        Self {
            req: Default::default(),
            image: Fw1082Protocol::create_hardware_image(),
            clock_ctl: Default::default(),
            input_threshold_ctl: Default::default(),
            coax_output_ctl: Default::default(),
            meter_ctl: Default::default(),
            console_ctl: Default::default(),
            common_state: Default::default(),
            isoch_state: Default::default(),
            specific_state: Default::default(),
            machine_state: Fw1082Protocol::create_machine_state(),
        }
    }
}

const TIMEOUT_MS: u32 = 50;

impl SurfaceCtlOperation<SndTascam> for Fw1082Model {
    fn init(&mut self, _: &mut FwNode) -> Result<(), Error> {
        Fw1082Protocol::init(&mut self.common_state);
        Fw1082Protocol::init(&mut self.isoch_state);
        Fw1082Protocol::init(&mut self.specific_state);
        Ok(())
    }

    fn peek(
        &mut self,
        unit: &mut SndTascam,
        index: u32,
        before: u32,
        after: u32,
    ) -> Result<Vec<(MachineItem, ItemValue)>, Error> {
        unit.read_state(&mut self.image)?;
        let mut machine_values =
            Fw1082Protocol::peek(&self.common_state, &self.image, index, before, after);
        machine_values.append(&mut Fw1082Protocol::peek(
            &self.isoch_state,
            &self.image,
            index,
            before,
            after,
        ));
        machine_values.append(&mut Fw1082Protocol::peek(
            &self.specific_state,
            &self.image,
            index,
            before,
            after,
        ));
        Ok(machine_values)
    }

    fn ack(
        &mut self,
        machine_value: &(MachineItem, ItemValue),
        node: &mut FwNode,
    ) -> Result<(), Error> {
        let res = Fw1082Protocol::operate_leds(
            &mut self.common_state,
            machine_value,
            &mut self.req,
            node,
            TIMEOUT_MS,
        )
        .map(|_| Fw1082Protocol::ack(&mut self.common_state, machine_value));
        debug!(params = ?self.common_state);
        res?;

        let res = Fw1082Protocol::operate_leds(
            &mut self.isoch_state,
            machine_value,
            &mut self.req,
            node,
            TIMEOUT_MS,
        )
        .map(|_| Fw1082Protocol::ack(&mut self.isoch_state, machine_value));
        debug!(params = ?self.isoch_state);
        res?;

        let res = Fw1082Protocol::operate_leds(
            &mut self.specific_state,
            machine_value,
            &mut self.req,
            node,
            TIMEOUT_MS,
        )
        .map(|_| Fw1082Protocol::ack(&mut self.specific_state, machine_value));
        debug!(params = ?self.specific_state);
        res?;

        Ok(())
    }

    fn fin(&mut self, node: &mut FwNode) -> Result<(), Error> {
        Fw1082Protocol::clear_leds(&mut self.common_state, &mut self.req, node, TIMEOUT_MS)?;
        Fw1082Protocol::clear_leds(&mut self.isoch_state, &mut self.req, node, TIMEOUT_MS)?;
        Fw1082Protocol::clear_leds(&mut self.specific_state, &mut self.req, node, TIMEOUT_MS)?;
        Ok(())
    }
}

impl SequencerCtlOperation<SndTascam, Fw1082Protocol> for Fw1082Model {
    fn state(&self) -> &MachineState {
        &self.machine_state
    }

    fn state_mut(&mut self) -> &mut MachineState {
        &mut self.machine_state
    }
}

impl MeasureModel<(SndTascam, FwNode)> for Fw1082Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.console_ctl.elem_id_list);
    }

    fn measure_states(&mut self, unit: &mut (SndTascam, FwNode)) -> Result<(), Error> {
        unit.0.read_state(&mut self.image)?;
        self.meter_ctl.parse(&self.image)?;
        self.console_ctl.parse(&self.image)?;
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
        } else if self.console_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl CtlModel<(SndTascam, FwNode)> for Fw1082Model {
    fn cache(&mut self, (unit, node): &mut (SndTascam, FwNode)) -> Result<(), Error> {
        unit.read_state(&mut self.image)?;
        self.meter_ctl.parse(&self.image)?;
        self.console_ctl.parse(&self.image)?;

        self.clock_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.input_threshold_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.coax_output_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.console_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clock_ctl.load(card_cntr)?;
        self.input_threshold_ctl.load(card_cntr)?;
        self.coax_output_ctl.load(card_cntr)?;
        self.meter_ctl.load(card_cntr)?;
        self.console_ctl.load(card_cntr)?;
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
        } else if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.console_ctl.read(elem_id, elem_value)? {
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
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.clock_ctl.write(
            &mut unit.0,
            &mut self.req,
            &mut unit.1,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.input_threshold_ctl.write(
            &mut self.req,
            &mut unit.1,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.coax_output_ctl.write(
            &mut self.req,
            &mut unit.1,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .console_ctl
            .write(&mut self.req, &mut unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
