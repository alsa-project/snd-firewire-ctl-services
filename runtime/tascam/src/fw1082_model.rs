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
    meter_ctl: MeterCtl,
    common_ctl: CommonCtl,
    console_ctl: ConsoleCtl,
    seq_state: SequencerState<Fw1082SurfaceState>,
}

impl Default for Fw1082Model {
    fn default() -> Self {
        Self {
            req: Default::default(),
            image: vec![0u32; 64],
            clock_ctl: Default::default(),
            meter_ctl: Default::default(),
            common_ctl: Default::default(),
            console_ctl: Default::default(),
            seq_state: Default::default(),
        }
    }
}

const TIMEOUT_MS: u32 = 50;

#[derive(Default)]
struct MeterCtl(IsochMeterState, Vec<ElemId>);

impl IsochMeterCtlOperation<Fw1082Protocol> for MeterCtl {
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
        "spdif-input-1",
        "spdif-input-2",
    ];
    const OUTPUT_LABELS: &'static [&'static str] = &[
        "analog-output-1",
        "analog-output-2",
        "spdif-output-1",
        "spdif-output-2",
    ];
}

#[derive(Default)]
struct CommonCtl;

impl IsochCommonCtlOperation<Fw1082Protocol> for CommonCtl {}

#[derive(Default)]
struct ConsoleCtl(IsochConsoleState, Vec<ElemId>);

impl IsochConsoleCtlOperation<Fw1082Protocol> for ConsoleCtl {
    fn state(&self) -> &IsochConsoleState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut IsochConsoleState {
        &mut self.0
    }
}

impl SequencerCtlOperation<SndTascam, Fw1082Protocol, Fw1082SurfaceState> for Fw1082Model {
    fn state(&self) -> &SequencerState<Fw1082SurfaceState> {
        &self.seq_state
    }

    fn state_mut(&mut self) -> &mut SequencerState<Fw1082SurfaceState> {
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
                    || MachineItem::EncoderMode.eq(item)
                    || Fw1082Protocol::TRANSPORT_ITEMS
                        .iter()
                        .find(|i| item.eq(i))
                        .is_some()
            })
            .try_for_each(|entry| self.feedback_to_surface(node, entry))
    }

    fn finalize_surface(&mut self, node: &mut FwNode) -> Result<(), Error> {
        Fw1082Protocol::finalize_surface(
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
        Fw1082Protocol::feedback_to_surface(
            &mut self.seq_state.surface_state,
            event,
            &mut self.req,
            node,
            TIMEOUT_MS,
        )
    }
}

impl MeasureModel<(SndTascam, FwNode)> for Fw1082Model {
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

impl CtlModel<(SndTascam, FwNode)> for Fw1082Model {
    fn load(
        &mut self,
        unit: &mut (SndTascam, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        unit.0.read_state(&mut self.image)?;

        self.clock_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;

        self.clock_ctl.load(card_cntr)?;

        self.meter_ctl
            .load_state(card_cntr, &self.image)
            .map(|mut elem_id_list| self.meter_ctl.1.append(&mut elem_id_list))?;

        self.common_ctl.load_params(card_cntr)?;

        self.console_ctl
            .load_params(card_cntr, &self.image)
            .map(|mut elem_id_list| self.console_ctl.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut (SndTascam, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.clock_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.common_ctl.read_params(
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
        } else if self
            .common_ctl
            .write_params(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.console_ctl.write_params(
            &mut unit.1,
            &mut self.req,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
