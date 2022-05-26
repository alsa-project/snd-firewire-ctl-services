// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    tascam_protocols::asynch::{fe8::*, *},
};

pub struct Fe8Model {
    req: FwReq,
    image: Vec<u32>,
    seq_state: SequencerState<Fe8SurfaceState>,
}

impl Default for Fe8Model {
    fn default() -> Self {
        Self {
            req: Default::default(),
            image: vec![0; TascamExpander::QUADLET_COUNT],
            seq_state: Default::default(),
        }
    }
}

const TIMEOUT_MS: u32 = 50;

impl SequencerCtlOperation<TascamExpander, Fe8Protocol, Fe8SurfaceState> for Fe8Model {
    fn state(&self) -> &SequencerState<Fe8SurfaceState> {
        &self.seq_state
    }

    fn state_mut(&mut self) -> &mut SequencerState<Fe8SurfaceState> {
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
