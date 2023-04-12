// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{isoch_ctls::*, *},
    alsactl::{prelude::*, *},
    protocols::isoch::{fw1884::*, *},
};

pub struct Fw1884Model {
    req: FwReq,
    image: Vec<u32>,
    clock_ctl: ClockCtl<Fw1884Protocol>,
    input_threshold_ctl: InputDetectionThreshold<Fw1884Protocol>,
    coax_output_ctl: CoaxOutputCtl<Fw1884Protocol>,
    opt_iface_ctl: OpticalIfaceCtl<Fw1884Protocol>,
    meter_ctl: MeterCtl<Fw1884Protocol>,
    console_ctl: ConsoleCtl<Fw1884Protocol>,
    specific_ctl: SpecificCtl,
    common_state: TascamSurfaceCommonState,
    isoch_state: TascamSurfaceIsochState,
    machine_state: MachineState,
}

impl Default for Fw1884Model {
    fn default() -> Self {
        Self {
            req: Default::default(),
            image: Fw1884Protocol::create_hardware_image(),
            clock_ctl: Default::default(),
            input_threshold_ctl: Default::default(),
            coax_output_ctl: Default::default(),
            opt_iface_ctl: Default::default(),
            console_ctl: Default::default(),
            meter_ctl: Default::default(),
            specific_ctl: Default::default(),
            common_state: Default::default(),
            isoch_state: Default::default(),
            machine_state: Fw1884Protocol::create_machine_state(),
        }
    }
}

const TIMEOUT_MS: u32 = 50;

impl SurfaceCtlOperation<SndTascam> for Fw1884Model {
    fn init(&mut self, _: &mut FwNode) -> Result<(), Error> {
        Fw1884Protocol::init(&mut self.common_state);
        Fw1884Protocol::init(&mut self.isoch_state);
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
            Fw1884Protocol::peek(&self.common_state, &self.image, index, before, after);
        machine_values.append(&mut Fw1884Protocol::peek(
            &self.isoch_state,
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
        let res = Fw1884Protocol::operate_leds(
            &mut self.common_state,
            machine_value,
            &mut self.req,
            node,
            TIMEOUT_MS,
        )
        .map(|_| Fw1884Protocol::ack(&mut self.common_state, machine_value));
        debug!(params = ?self.common_state, ?res);
        res?;

        let res = Fw1884Protocol::operate_leds(
            &mut self.isoch_state,
            machine_value,
            &mut self.req,
            node,
            TIMEOUT_MS,
        )
        .map(|_| Fw1884Protocol::ack(&mut self.isoch_state, machine_value));
        debug!(params = ?self.isoch_state, ?res);
        res?;

        Ok(())
    }

    fn fin(&mut self, node: &mut FwNode) -> Result<(), Error> {
        Fw1884Protocol::clear_leds(&mut self.common_state, &mut self.req, node, TIMEOUT_MS)?;
        Fw1884Protocol::clear_leds(&mut self.isoch_state, &mut self.req, node, TIMEOUT_MS)?;
        Ok(())
    }
}

impl SequencerCtlOperation<SndTascam, Fw1884Protocol> for Fw1884Model {
    fn state(&self) -> &MachineState {
        &self.machine_state
    }

    fn state_mut(&mut self) -> &mut MachineState {
        &mut self.machine_state
    }
}

impl MeasureModel<(SndTascam, FwNode)> for Fw1884Model {
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

impl CtlModel<(SndTascam, FwNode)> for Fw1884Model {
    fn cache(&mut self, (unit, node): &mut (SndTascam, FwNode)) -> Result<(), Error> {
        unit.read_state(&mut self.image)?;
        self.meter_ctl.parse(&self.image)?;
        self.console_ctl.parse(&self.image)?;

        self.clock_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.input_threshold_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.coax_output_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.opt_iface_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.specific_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.console_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clock_ctl.load(card_cntr)?;
        self.input_threshold_ctl.load(card_cntr)?;
        self.coax_output_ctl.load(card_cntr)?;
        self.opt_iface_ctl.load(card_cntr)?;
        self.specific_ctl.load(card_cntr)?;
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
        } else if self.opt_iface_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.specific_ctl.read(elem_id, elem_value)? {
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
            .opt_iface_ctl
            .write(&mut self.req, &mut unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .specific_ctl
            .write(&mut self.req, &mut unit.1, elem_id, new, TIMEOUT_MS)?
        {
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

#[derive(Default, Debug)]
struct SpecificCtl {
    elem_id_list: Vec<ElemId>,
    params: Fw1884MonitorKnobTarget,
}

const MONITOR_ROTARY_ASSIGN_NAME: &str = "monitor-rotary-assign";

fn monitor_knob_target_to_str(target: &Fw1884MonitorKnobTarget) -> &'static str {
    match target {
        Fw1884MonitorKnobTarget::AnalogOutputPair0 => "analog-output-1/2",
        Fw1884MonitorKnobTarget::AnalogOutput3Pairs => "analog-output-1/2/3/4/5/6",
        Fw1884MonitorKnobTarget::AnalogOutput4Pairs => "analog-output-1/2/3/4/5/6/7/8",
    }
}

impl SpecificCtl {
    const MONITOR_ROTARY_ASSIGNS: [Fw1884MonitorKnobTarget; 3] = [
        Fw1884MonitorKnobTarget::AnalogOutputPair0,
        Fw1884MonitorKnobTarget::AnalogOutput3Pairs,
        Fw1884MonitorKnobTarget::AnalogOutput4Pairs,
    ];

    fn cache(&mut self, req: &mut FwReq, node: &mut FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = Fw1884Protocol::cache_wholly(req, node, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::MONITOR_ROTARY_ASSIGNS
            .iter()
            .map(|a| monitor_knob_target_to_str(a))
            .collect();
        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_ROTARY_ASSIGN_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MONITOR_ROTARY_ASSIGN_NAME => {
                let pos = Self::MONITOR_ROTARY_ASSIGNS
                    .iter()
                    .position(|a| self.params.eq(a))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MONITOR_ROTARY_ASSIGN_NAME => {
                let val = elem_value.enumerated()[0];
                let target = Self::MONITOR_ROTARY_ASSIGNS
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index for monitor rotary targets: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })?;
                let res = Fw1884Protocol::update_wholly(req, node, target, timeout_ms)
                    .map(|_| self.params = *target);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
