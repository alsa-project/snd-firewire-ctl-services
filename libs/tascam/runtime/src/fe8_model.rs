// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    tascam_protocols::asynch::{fe8::*, *},
};

#[derive(Default)]
pub struct Fe8Model {
    req: FwReq,
    seq_state: SequencerState<Fe8SurfaceState>,
}

const TIMEOUT_MS: u32 = 50;

impl SequencerCtlOperation<FwNode, Fe8Protocol, Fe8SurfaceState> for Fe8Model {
    fn state(&self) -> &SequencerState<Fe8SurfaceState> {
        &self.seq_state
    }

    fn state_mut(&mut self) -> &mut SequencerState<Fe8SurfaceState> {
        &mut self.seq_state
    }

    fn initialize_surface(
        &mut self,
        node: &mut FwNode,
        _: &[(MachineItem, ItemValue)],
    ) -> Result<(), Error> {
        Fe8Protocol::operate_firewire_led(&mut self.req, node, true, TIMEOUT_MS)?;
        Ok(())
    }

    fn finalize_surface(&mut self, node: &mut FwNode) -> Result<(), Error> {
        Fe8Protocol::finalize_surface(
            &mut self.seq_state.surface_state,
            &mut self.req,
            node,
            TIMEOUT_MS,
        )?;
        Fe8Protocol::operate_firewire_led(&mut self.req, node, false, TIMEOUT_MS)?;
        Ok(())
    }

    fn feedback_to_surface(
        &mut self,
        node: &mut FwNode,
        event: &(MachineItem, ItemValue),
    ) -> Result<(), Error> {
        Fe8Protocol::feedback_to_surface(
            &mut self.seq_state.surface_state,
            event,
            &mut self.req,
            node,
            TIMEOUT_MS,
        )
    }
}

impl AsynchCtlOperation for Fe8Model {
    fn register_notification_address(&mut self, node: &mut FwNode, addr: u64) -> Result<(), Error> {
        Fe8Protocol::register_notification_address(&mut self.req, node, addr, TIMEOUT_MS)
    }

    fn enable_notification(&mut self, node: &mut FwNode, enable: bool) -> Result<(), Error> {
        Fe8Protocol::enable_notification(&mut self.req, node, enable, TIMEOUT_MS)
    }
}
