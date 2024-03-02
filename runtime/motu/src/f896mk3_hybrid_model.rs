// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::{command_dsp_ctls::*, command_dsp_runtime::*, common_ctls::*, v3_ctls::*};

const TIMEOUT_MS: u32 = 100;

#[derive(Default, Debug)]
pub struct F896mk3HybridModel {
    req: FwReq,
    resp: FwResp,
    clk_ctls: V3LcdClkCtl<F896mk3HybridProtocol>,
    port_assign_ctl: V3PortAssignCtl<F896mk3HybridProtocol>,
    opt_iface_ctl: V3OptIfaceCtl<F896mk3HybridProtocol>,
    phone_assign_ctl: PhoneAssignCtl<F896mk3HybridProtocol>,
    word_clk_ctl: WordClockCtl<F896mk3HybridProtocol>,
    sequence_number: u8,
    reverb_ctl: CommandDspReverbCtl<F896mk3HybridProtocol>,
    monitor_ctl: CommandDspMonitorCtl<F896mk3HybridProtocol>,
    mixer_ctl: CommandDspMixerCtl<F896mk3HybridProtocol>,
    input_ctl: CommandDspInputCtl<F896mk3HybridProtocol>,
    input_eq_ctl: CommandDspInputEqualizerCtl<F896mk3HybridProtocol>,
    input_dyn_ctl: CommandDspInputDynamicsCtl<F896mk3HybridProtocol>,
    output_ctl: CommandDspOutputCtl<F896mk3HybridProtocol>,
    output_eq_ctl: CommandDspOutputEqualizerCtl<F896mk3HybridProtocol>,
    output_dyn_ctl: CommandDspOutputDynamicsCtl<F896mk3HybridProtocol>,
    resource_ctl: CommandDspResourceCtl,
    level_meters_ctl: LevelMetersCtl<F896mk3HybridProtocol>,
    meter_ctl: CommandDspMeterCtl<F896mk3HybridProtocol>,
}

impl CtlModel<(SndMotu, FwNode)> for F896mk3HybridModel {
    fn cache(&mut self, (unit, node): &mut (SndMotu, FwNode)) -> Result<(), Error> {
        self.clk_ctls.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.port_assign_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.opt_iface_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.phone_assign_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.word_clk_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.level_meters_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;

        self.meter_ctl.read_dsp_meter(unit)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctls.load(card_cntr)?;
        self.port_assign_ctl.load(card_cntr)?;
        self.opt_iface_ctl.load(card_cntr)?;
        self.phone_assign_ctl.load(card_cntr)?;
        self.word_clk_ctl.load(card_cntr)?;
        self.reverb_ctl.load(card_cntr)?;
        self.monitor_ctl.load(card_cntr)?;
        self.mixer_ctl.load(card_cntr)?;
        self.input_ctl.load(card_cntr)?;
        self.input_eq_ctl.load(card_cntr)?;
        self.input_dyn_ctl.load(card_cntr)?;
        self.output_ctl.load(card_cntr)?;
        self.output_eq_ctl.load(card_cntr)?;
        self.output_dyn_ctl.load(card_cntr)?;
        self.resource_ctl.load(card_cntr)?;
        self.level_meters_ctl.load(card_cntr)?;
        self.meter_ctl.load(card_cntr)?;
        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.clk_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.port_assign_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.opt_iface_ctl.read(elem_id, elem_value)? {
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
        } else if self.input_eq_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_dyn_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_eq_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_dyn_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.resource_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.level_meters_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        (unit, node): &mut (SndMotu, FwNode),
        elem_id: &ElemId,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .clk_ctls
            .write(unit, &mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.port_assign_ctl.write(
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.opt_iface_ctl.write(
            unit,
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.phone_assign_ctl.write(
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .word_clk_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.reverb_ctl.write(
            &mut self.sequence_number,
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.monitor_ctl.write(
            &mut self.sequence_number,
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_ctl.write(
            &mut self.sequence_number,
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.input_ctl.write(
            &mut self.sequence_number,
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.input_eq_ctl.write(
            &mut self.sequence_number,
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.input_dyn_ctl.write(
            &mut self.sequence_number,
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.output_ctl.write(
            &mut self.sequence_number,
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.output_eq_ctl.write(
            &mut self.sequence_number,
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.output_dyn_ctl.write(
            &mut self.sequence_number,
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.level_meters_ctl.write(
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndMotu, FwNode), u32> for F896mk3HybridModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.level_meters_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.word_clk_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.phone_assign_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.port_assign_ctl.elem_id_list);
    }

    fn parse_notification(
        &mut self,
        (_, node): &mut (SndMotu, FwNode),
        msg: &u32,
    ) -> Result<(), Error> {
        if *msg & F896mk3HybridProtocol::NOTIFY_PORT_CHANGE_MASK > 0 {
            self.port_assign_ctl
                .cache(&mut self.req, node, TIMEOUT_MS)?;
            self.phone_assign_ctl
                .cache(&mut self.req, node, TIMEOUT_MS)?;
            self.word_clk_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
            self.level_meters_ctl
                .cache(&mut self.req, node, TIMEOUT_MS)?;
        }
        // TODO: what kind of event is preferable for NOTIFY_FOOTSWITCH_MASK?
        Ok(())
    }
}

impl NotifyModel<(SndMotu, FwNode), Vec<DspCmd>> for F896mk3HybridModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.reverb_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.monitor_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.mixer_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.input_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.input_eq_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.input_dyn_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.output_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.output_eq_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.output_dyn_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.resource_ctl.elem_id_list);
    }

    fn parse_notification(
        &mut self,
        _: &mut (SndMotu, FwNode),
        cmds: &Vec<DspCmd>,
    ) -> Result<(), Error> {
        for cmd in cmds {
            let _ = self.reverb_ctl.parse_command(cmd)
                || self.monitor_ctl.parse_command(cmd)
                || self.mixer_ctl.parse_command(cmd)
                || self.input_ctl.parse_command(cmd)
                || self.input_eq_ctl.parse_command(cmd)
                || self.input_dyn_ctl.parse_command(cmd)
                || self.output_ctl.parse_command(cmd)
                || self.output_eq_ctl.parse_command(cmd)
                || self.output_dyn_ctl.parse_command(cmd)
                || self.resource_ctl.parse_command(cmd);
        }
        Ok(())
    }
}

impl MeasureModel<(SndMotu, FwNode)> for F896mk3HybridModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.elem_id_list);
    }

    fn measure_states(&mut self, (unit, _): &mut (SndMotu, FwNode)) -> Result<(), Error> {
        self.meter_ctl.read_dsp_meter(unit)
    }
}

impl CommandDspModel for F896mk3HybridModel {
    fn prepare_message_handler<F>(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        handler: F,
    ) -> Result<(), Error>
    where
        F: Fn(&FwResp, FwTcode, u64, u32, u32, u32, u32, u32, &[u8]) -> FwRcode + 'static,
    {
        F896mk3HybridProtocol::register_message_destination_address(
            &mut self.resp,
            &mut self.req,
            &mut unit.1,
            TIMEOUT_MS,
        )?;
        self.resp.connect_requested(handler);
        Ok(())
    }

    fn begin_messaging(&mut self, unit: &mut (SndMotu, FwNode)) -> Result<(), Error> {
        F896mk3HybridProtocol::begin_messaging(
            &mut self.req,
            &mut unit.1,
            &mut self.sequence_number,
            TIMEOUT_MS,
        )
    }

    fn release_message_handler(&mut self, unit: &mut (SndMotu, FwNode)) -> Result<(), Error> {
        F896mk3HybridProtocol::cancel_messaging(
            &mut self.req,
            &mut unit.1,
            &mut self.sequence_number,
            TIMEOUT_MS,
        )?;
        F896mk3HybridProtocol::release_message_destination_address(
            &mut self.resp,
            &mut self.req,
            &mut unit.1,
            TIMEOUT_MS,
        )?;
        Ok(())
    }
}
