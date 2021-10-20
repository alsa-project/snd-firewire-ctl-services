// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{FwReq, FwRcode, FwResp, FwRespExtManual, FwTcode};
use hinawa::{SndMotu, SndUnitExt};

use alsactl::{ElemId, ElemValue};

use core::card_cntr::{CardCntr, CtlModel, NotifyModel};

use motu_protocols::{command_dsp::*, version_3::*, *};

use super::{command_dsp_ctls::*, common_ctls::*, v3_ctls::*};
use super::command_dsp_runtime::*;

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct F828mk3Hybrid {
    req: FwReq,
    resp: FwResp,
    clk_ctls: ClkCtl,
    port_assign_ctl: PortAssignCtl,
    opt_iface_ctl: OptIfaceCtl,
    phone_assign_ctl: PhoneAssignCtl,
    word_clk_ctl: WordClkCtl,
    sequence_number: u8,
    reverb_ctl: ReverbCtl,
    monitor_ctl: MonitorCtl,
    mixer_ctl: MixerCtl,
    input_ctl: InputCtl,
    output_ctl: OutputCtl,
    resource_ctl: ResourceCtl,
}

#[derive(Default)]
struct PhoneAssignCtl(usize, Vec<ElemId>);

impl PhoneAssignCtlOperation<F828mk3HybridProtocol> for PhoneAssignCtl {
    fn state(&self) -> &usize {
        &self.0
    }

    fn state_mut(&mut self) -> &mut usize {
        &mut self.0
    }
}

#[derive(Default)]
struct WordClkCtl(WordClkSpeedMode, Vec<ElemId>);

impl WordClkCtlOperation<F828mk3HybridProtocol> for WordClkCtl {
    fn state(&self) -> &WordClkSpeedMode {
        &self.0
    }

    fn state_mut(&mut self) -> &mut WordClkSpeedMode {
        &mut self.0
    }
}

#[derive(Default)]
struct ClkCtl;

impl V3ClkCtlOperation<F828mk3HybridProtocol> for ClkCtl {}

#[derive(Default)]
struct PortAssignCtl(V3PortAssignState, Vec<ElemId>);

impl V3PortAssignCtlOperation<F828mk3HybridProtocol> for PortAssignCtl {
    fn state(&self) -> &V3PortAssignState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut V3PortAssignState {
        &mut self.0
    }
}

#[derive(Default)]
struct OptIfaceCtl;

impl V3OptIfaceCtlOperation<F828mk3HybridProtocol> for OptIfaceCtl {}

#[derive(Default)]
struct ReverbCtl(CommandDspReverbState, Vec<ElemId>);

impl CommandDspReverbCtlOperation<F828mk3HybridProtocol> for ReverbCtl {
    fn state(&self) -> &CommandDspReverbState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut CommandDspReverbState {
        &mut self.0
    }
}

#[derive(Default)]
struct MonitorCtl(CommandDspMonitorState, Vec<ElemId>);

impl CommandDspMonitorCtlOperation<F828mk3HybridProtocol> for MonitorCtl {
    fn state(&self) -> &CommandDspMonitorState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut CommandDspMonitorState {
        &mut self.0
    }
}

#[derive(Default)]
struct MixerCtl(CommandDspMixerState, Vec<ElemId>);

impl CommandDspMixerCtlOperation<F828mk3HybridProtocol> for MixerCtl {
    fn state(&self) -> &CommandDspMixerState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut CommandDspMixerState {
        &mut self.0
    }
}

#[derive(Default)]
struct InputCtl(CommandDspInputState, Vec<ElemId>);

impl CommandDspInputCtlOperation<F828mk3HybridProtocol> for InputCtl {
    fn state(&self) -> &CommandDspInputState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut CommandDspInputState {
        &mut self.0
    }
}

#[derive(Default)]
struct OutputCtl(CommandDspOutputState, Vec<ElemId>);

impl CommandDspOutputCtlOperation<F828mk3HybridProtocol> for OutputCtl {
    fn state(&self) -> &CommandDspOutputState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut CommandDspOutputState {
        &mut self.0
    }
}

#[derive(Default)]
struct ResourceCtl(u32, Vec<ElemId>);

impl CommandDspResourcebCtlOperation for ResourceCtl {
    fn state(&self) -> &u32 {
        &self.0
    }

    fn state_mut(&mut self) -> &mut u32 {
        &mut self.0
    }
}

impl CtlModel<SndMotu> for F828mk3Hybrid {
    fn load(&mut self, unit: &mut SndMotu, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctls.load(card_cntr)?;
        self.port_assign_ctl.load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.port_assign_ctl.1.append(&mut elem_id_list))?;
        self.opt_iface_ctl.load(card_cntr)?;
        self.phone_assign_ctl.load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.phone_assign_ctl.1.append(&mut elem_id_list))?;
        self.word_clk_ctl.load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.word_clk_ctl.1.append(&mut elem_id_list))?;
        self.reverb_ctl.load(card_cntr)
            .map(|mut elem_id_list| self.reverb_ctl.1.append(&mut elem_id_list))?;
        self.monitor_ctl.load(card_cntr)
            .map(|mut elem_id_list| self.monitor_ctl.1.append(&mut elem_id_list))?;
        self.mixer_ctl.load(card_cntr)
            .map(|mut elem_id_list| self.mixer_ctl.1.append(&mut elem_id_list))?;
        self.input_ctl.load(card_cntr)
            .map(|mut elem_id_list| self.input_ctl.1.append(&mut elem_id_list))?;
        self.input_ctl.load_equalizer(card_cntr)
            .map(|mut elem_id_list| self.input_ctl.1.append(&mut elem_id_list))?;
        self.input_ctl.load_dynamics(card_cntr)
            .map(|mut elem_id_list| self.input_ctl.1.append(&mut elem_id_list))?;
        self.output_ctl.load(card_cntr)
            .map(|mut elem_id_list| self.output_ctl.1.append(&mut elem_id_list))?;
        self.output_ctl.load_equalizer(card_cntr)
            .map(|mut elem_id_list| self.output_ctl.1.append(&mut elem_id_list))?;
        self.output_ctl.load_dynamics(card_cntr)
            .map(|mut elem_id_list| self.output_ctl.1.append(&mut elem_id_list))?;
        self.resource_ctl.load(card_cntr)
            .map(|mut elem_id_list| self.resource_ctl.1.append(&mut elem_id_list))?;
        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut SndMotu,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        if self.clk_ctls.read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.port_assign_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.opt_iface_ctl.read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.phone_assign_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.word_clk_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.monitor_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read_equalizer(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read_dynamics(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read_equalizer(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read_dynamics(elem_id, elem_value)? {
            Ok(true)
        } else if self.resource_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut SndMotu,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue
    ) -> Result<bool, Error> {
        if self.clk_ctls.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.port_assign_ctl.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.opt_iface_ctl.write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.phone_assign_ctl.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.word_clk_ctl.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.reverb_ctl.write(
            &mut self.sequence_number,
            unit,
            &mut self.req,
            elem_id,
            new,
            TIMEOUT_MS
        )? {
            Ok(true)
        } else if self.monitor_ctl.write(
            &mut self.sequence_number,
            unit,
            &mut self.req,
            elem_id,
            new,
            TIMEOUT_MS
        )? {
            Ok(true)
        } else if self.mixer_ctl.write(
            &mut self.sequence_number,
            unit,
            &mut self.req,
            elem_id,
            new,
            TIMEOUT_MS
        )? {
            Ok(true)
        } else if self.input_ctl.write(
            &mut self.sequence_number,
            unit,
            &mut self.req,
            elem_id,
            new,
            TIMEOUT_MS
        )? {
            Ok(true)
        } else if self.input_ctl.write_equalizer(
            &mut self.sequence_number,
            unit,
            &mut self.req,
            elem_id,
            new,
            TIMEOUT_MS
        )? {
            Ok(true)
        } else if self.input_ctl.write_dynamics(
            &mut self.sequence_number,
            unit,
            &mut self.req,
            elem_id,
            new,
            TIMEOUT_MS
        )? {
            Ok(true)
        } else if self.output_ctl.write(
            &mut self.sequence_number,
            unit,
            &mut self.req,
            elem_id,
            new,
            TIMEOUT_MS
        )? {
            Ok(true)
        } else if self.output_ctl.write_equalizer(
            &mut self.sequence_number,
            unit,
            &mut self.req,
            elem_id,
            new,
            TIMEOUT_MS
        )? {
            Ok(true)
        } else if self.output_ctl.write_dynamics(
            &mut self.sequence_number,
            unit,
            &mut self.req,
            elem_id,
            new,
            TIMEOUT_MS
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndMotu, u32> for F828mk3Hybrid {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.port_assign_ctl.1);
        elem_id_list.extend_from_slice(&self.phone_assign_ctl.1);
        elem_id_list.extend_from_slice(&self.word_clk_ctl.1);
    }

    fn parse_notification(&mut self, unit: &mut SndMotu, msg: &u32) -> Result<(), Error> {
        if *msg & F828mk3HybridProtocol::NOTIFY_PORT_CHANGE > 0 {
            self.port_assign_ctl.cache(unit, &mut self.req, TIMEOUT_MS)?;
            self.phone_assign_ctl.cache(unit, &mut self.req, TIMEOUT_MS)?;
            self.word_clk_ctl.cache(unit, &mut self.req, TIMEOUT_MS)?;
        }
        // TODO: what kind of event is preferable for NOTIFY_FOOTSWITCH_MASK?
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndMotu,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        if self.port_assign_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.phone_assign_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.word_clk_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndMotu, &[DspCmd]> for F828mk3Hybrid {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.reverb_ctl.1);
        elem_id_list.extend_from_slice(&self.monitor_ctl.1);
        elem_id_list.extend_from_slice(&self.mixer_ctl.1);
        elem_id_list.extend_from_slice(&self.input_ctl.1);
        elem_id_list.extend_from_slice(&self.output_ctl.1);
        elem_id_list.extend_from_slice(&self.resource_ctl.1);
    }

    fn parse_notification(&mut self, _: &mut SndMotu, cmds: &&[DspCmd]) -> Result<(), Error> {
        self.reverb_ctl.parse_commands(*cmds);
        self.monitor_ctl.parse_commands(*cmds);
        self.mixer_ctl.parse_commands(*cmds);
        self.input_ctl.parse_commands(*cmds);
        self.output_ctl.parse_commands(*cmds);
        self.resource_ctl.parse_commands(*cmds);
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndMotu,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        if self.reverb_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.monitor_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read_equalizer(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read_dynamics(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read_equalizer(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read_dynamics(elem_id, elem_value)? {
            Ok(true)
        } else if self.resource_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> CommandDspModel<'a> for F828mk3Hybrid {
    fn prepare_message_handler<F>(&mut self, unit: &mut SndMotu, handler: F) -> Result<(), Error>
        where F: Fn(&FwResp, FwTcode, u64, u32, u32, u32, u32, &[u8]) -> FwRcode + 'static
    {
        F828mk3HybridProtocol::register_message_destination_address(
            &mut self.resp,
            &mut self.req,
            &mut unit.get_node(),
            TIMEOUT_MS,
        )?;
        self.resp.connect_requested2(handler);
        Ok(())
    }

    fn begin_messaging(&mut self, unit: &mut SndMotu) -> Result<(), Error> {
        UltraliteMk3Protocol::begin_messaging(
            &mut self.req,
            &mut unit.get_node(),
            &mut self.sequence_number,
            TIMEOUT_MS
        )
    }

    fn release_message_handler(&mut self, unit: &mut SndMotu) -> Result<(), Error> {
        UltraliteMk3Protocol::cancel_messaging(
            &mut self.req,
            &mut unit.get_node(),
            &mut self.sequence_number,
            TIMEOUT_MS
        )?;
        F828mk3HybridProtocol::release_message_destination_address(
            &mut self.resp,
            &mut self.req,
            &mut unit.get_node(),
            TIMEOUT_MS
        )?;
        Ok(())
    }
}
