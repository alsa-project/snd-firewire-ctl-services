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
    meter_ctl: MeterCtl,
    common_ctl: CommonCtl,
    optical_ctl: OpticalCtl,
    console_ctl: ConsoleCtl,
    specific_ctl: SpecificCtl,
    seq_state: SequencerState<Fw1884SurfaceState>,
}

impl Default for Fw1884Model {
    fn default() -> Self {
        Self {
            req: Default::default(),
            image: vec![0u32; 64],
            meter_ctl: Default::default(),
            common_ctl: Default::default(),
            optical_ctl: Default::default(),
            console_ctl: Default::default(),
            specific_ctl: Default::default(),
            seq_state: Default::default(),
        }
    }
}

const TIMEOUT_MS: u32 = 50;

#[derive(Default)]
struct MeterCtl(IsochMeterState, Vec<ElemId>);

impl IsochMeterCtlOperation<Fw1884Protocol> for MeterCtl {
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

impl IsochCommonCtlOperation<Fw1884Protocol> for CommonCtl {}

#[derive(Default)]
struct OpticalCtl;

impl IsochOpticalCtlOperation<Fw1884Protocol> for OpticalCtl {
    const OPTICAL_OUTPUT_SOURCES: &'static [OpticalOutputSource] = &[
        OpticalOutputSource::StreamInputPairs,
        OpticalOutputSource::AnalogOutputPairs,
        OpticalOutputSource::CoaxialOutputPair0,
        OpticalOutputSource::AnalogInputPair0,
    ];
}

#[derive(Default)]
struct ConsoleCtl(IsochConsoleState, Vec<ElemId>);

impl IsochConsoleCtlOperation<Fw1884Protocol> for ConsoleCtl {
    fn state(&self) -> &IsochConsoleState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut IsochConsoleState {
        &mut self.0
    }
}

#[derive(Default)]
struct SpecificCtl;

impl SequencerCtlOperation<SndTascam, Fw1884Protocol, Fw1884SurfaceState> for Fw1884Model {
    fn state(&self) -> &SequencerState<Fw1884SurfaceState> {
        &self.seq_state
    }

    fn state_mut(&mut self) -> &mut SequencerState<Fw1884SurfaceState> {
        &mut self.seq_state
    }

    fn image(&self) -> &[u32] {
        &self.image
    }

    fn image_mut(&mut self) -> &mut Vec<u32> {
        &mut self.image
    }

    fn initialize_surface(
        &mut self,
        node: &mut FwNode,
        machine_values: &[(MachineItem, ItemValue)],
    ) -> Result<(), Error> {
        machine_values
            .iter()
            .filter(|(item, _)| {
                MachineItem::Bank.eq(item)
                    || Fw1884Protocol::TRANSPORT_ITEMS
                        .iter()
                        .find(|i| item.eq(i))
                        .is_some()
            })
            .try_for_each(|entry| self.feedback_to_surface(node, entry))
    }

    fn finalize_surface(&mut self, node: &mut FwNode) -> Result<(), Error> {
        Fw1884Protocol::finalize_surface(
            &mut self.seq_state.surface_state,
            &mut self.req,
            node,
            TIMEOUT_MS,
        )
    }

    fn feedback_to_surface(
        &mut self,
        node: &mut FwNode,
        event: &(MachineItem, ItemValue),
    ) -> Result<(), Error> {
        Fw1884Protocol::feedback_to_surface(
            &mut self.seq_state.surface_state,
            event,
            &mut self.req,
            node,
            TIMEOUT_MS,
        )
    }
}

impl MeasureModel<(SndTascam, FwNode)> for Fw1884Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
        elem_id_list.extend_from_slice(&self.console_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut (SndTascam, FwNode)) -> Result<(), Error> {
        unit.0.read_state(&mut self.image)?;
        self.meter_ctl.parse_state(&self.image)?;
        self.console_ctl.parse_states(&self.image)?;
        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &(SndTascam, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.console_ctl.read_states(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl CtlModel<(SndTascam, FwNode)> for Fw1884Model {
    fn load(
        &mut self,
        unit: &mut (SndTascam, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        unit.0.read_state(&mut self.image)?;
        self.meter_ctl
            .load_state(card_cntr, &self.image)
            .map(|mut elem_id_list| self.meter_ctl.1.append(&mut elem_id_list))?;

        self.common_ctl.load_params(card_cntr)?;
        self.optical_ctl.load_params(card_cntr)?;

        self.console_ctl
            .load_params(card_cntr, &self.image)
            .map(|mut elem_id_list| self.console_ctl.1.append(&mut elem_id_list))?;

        self.specific_ctl.load_params(card_cntr)?;

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
        } else if self.console_ctl.read_params(
            &mut unit.1,
            &mut self.req,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.specific_ctl.read_params(
            unit,
            &mut self.req,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
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
        } else if self.console_ctl.write_params(
            &mut unit.1,
            &mut self.req,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .specific_ctl
            .write_params(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
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

    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::MONITOR_ROTARY_ASSIGNS
            .iter()
            .map(|a| monitor_knob_target_to_str(a))
            .collect();
        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_ROTARY_ASSIGN_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, false)?;
        Ok(())
    }

    fn read_params(
        &mut self,
        unit: &mut (SndTascam, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MONITOR_ROTARY_ASSIGN_NAME => {
                let target = Fw1884Protocol::get_monitor_knob_target(req, &mut unit.1, timeout_ms)?;
                let pos = Self::MONITOR_ROTARY_ASSIGNS
                    .iter()
                    .position(|a| target.eq(a))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        unit: &mut (SndTascam, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MONITOR_ROTARY_ASSIGN_NAME => {
                let val = elem_value.enumerated()[0];
                let &target = Self::MONITOR_ROTARY_ASSIGNS
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index for monitor rotary targets: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })?;
                Fw1884Protocol::set_monitor_knob_target(req, &mut unit.1, target, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}