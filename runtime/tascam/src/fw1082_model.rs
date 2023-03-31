// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{isoch_ctls::*, *},
    alsactl::*,
    protocols::isoch::fw1082::*,
};

pub struct Fw1082Model {
    req: FwReq,
    image: Vec<u32>,
    clock_ctl: ClockCtl<Fw1082Protocol>,
    input_threshold_ctl: InputDetectionThreshold<Fw1082Protocol>,
    coax_output_ctl: CoaxOutputCtl<Fw1082Protocol>,
    meter_ctl: MeterCtl<Fw1082Protocol>,
    console_ctl: ConsoleCtl<Fw1082Protocol>,
    seq_state: SequencerState<Fw1082SurfaceState>,
}

impl Default for Fw1082Model {
    fn default() -> Self {
        Self {
            req: Default::default(),
            image: vec![0u32; 64],
            clock_ctl: Default::default(),
            input_threshold_ctl: Default::default(),
            coax_output_ctl: Default::default(),
            meter_ctl: Default::default(),
            console_ctl: Default::default(),
            seq_state: Default::default(),
        }
    }
}

const TIMEOUT_MS: u32 = 50;

impl IsochConsoleCtlModel<Fw1082Protocol, Fw1082SurfaceState> for Fw1082Model {
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
    fn load(&mut self, _: &mut (SndTascam, FwNode), card_cntr: &mut CardCntr) -> Result<(), Error> {
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
