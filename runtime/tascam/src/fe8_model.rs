// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    protocols::asynch::{fe8::*, *},
};

pub struct Fe8Model {
    req: FwReq,
    image: Vec<u32>,
    common_state: TascamSurfaceCommonState,
    machine_state: MachineState,
}

impl Default for Fe8Model {
    fn default() -> Self {
        Self {
            req: Default::default(),
            image: Fe8Protocol::create_hardware_image(),
            common_state: Default::default(),
            machine_state: Fe8Protocol::create_machine_state(),
        }
    }
}

const TIMEOUT_MS: u32 = 50;

impl SurfaceCtlOperation<TascamExpander> for Fe8Model {
    fn init(&mut self, node: &mut FwNode) -> Result<(), Error> {
        Fe8Protocol::operate_firewire_led(&mut self.req, node, true, TIMEOUT_MS)?;
        Fe8Protocol::init(&mut self.common_state);
        Ok(())
    }

    fn peek(
        &mut self,
        unit: &mut TascamExpander,
        index: u32,
        before: u32,
        after: u32,
    ) -> Result<Vec<(MachineItem, ItemValue)>, Error> {
        unit.read_state(&mut self.image)?;
        let machine_values =
            Fe8Protocol::peek(&self.common_state, &self.image, index, before, after);
        Ok(machine_values)
    }

    fn ack(
        &mut self,
        machine_value: &(MachineItem, ItemValue),
        node: &mut FwNode,
    ) -> Result<(), Error> {
        Fe8Protocol::operate_leds(
            &mut self.common_state,
            machine_value,
            &mut self.req,
            node,
            TIMEOUT_MS,
        )
        .map(|_| Fe8Protocol::ack(&mut self.common_state, machine_value))
    }

    fn fin(&mut self, node: &mut FwNode) -> Result<(), Error> {
        Fe8Protocol::clear_leds(&mut self.common_state, &mut self.req, node, TIMEOUT_MS)?;
        Fe8Protocol::operate_firewire_led(&mut self.req, node, false, TIMEOUT_MS)?;
        Ok(())
    }
}

impl SequencerCtlOperation<TascamExpander, Fe8Protocol> for Fe8Model {
    fn state(&self) -> &MachineState {
        &self.machine_state
    }

    fn state_mut(&mut self) -> &mut MachineState {
        &mut self.machine_state
    }
}
