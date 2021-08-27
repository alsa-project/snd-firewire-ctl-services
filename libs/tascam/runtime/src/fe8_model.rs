// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use hinawa::FwReq;

use tascam_protocols::asynch::fe8::*;

use crate::*;

#[derive(Default)]
pub struct Fe8Model{
    req: FwReq,
    seq_state: SequencerState<Fe8SurfaceState>,
}

const TIMEOUT_MS: u32 = 50;

impl AsRef<SequencerState<Fe8SurfaceState>> for Fe8Model {
    fn as_ref(&self) -> &SequencerState<Fe8SurfaceState> {
        &self.seq_state
    }
}

impl AsMut<SequencerState<Fe8SurfaceState>> for Fe8Model {
    fn as_mut(&mut self) -> &mut SequencerState<Fe8SurfaceState> {
        &mut self.seq_state
    }
}

impl SequencerCtlOperation<FwNode, Fe8Protocol, Fe8SurfaceState> for Fe8Model {
    fn initialize_surface(
        &mut self,
        node: &mut FwNode,
        _: &[(MachineItem, ItemValue)],
    ) -> Result<(), Error> {
        Fe8Protocol::operate_firewire_led(&mut self.req, node, true, TIMEOUT_MS)
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
