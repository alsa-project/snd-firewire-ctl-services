// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::{command_dsp_ctls::*, command_dsp_runtime::*, common_ctls::*, v3_ctls::*};

const TIMEOUT_MS: u32 = 100;

#[derive(Default, Debug)]
pub struct Track16Model {
    req: FwReq,
    resp: FwResp,
    clk_ctls: V3ClkCtl<Track16Protocol>,
    port_assign_ctl: V3PortAssignCtl<Track16Protocol>,
    opt_iface_ctl: V3OptIfaceCtl<Track16Protocol>,
    phone_assign_ctl: PhoneAssignCtl<Track16Protocol>,
    sequence_number: u8,
    reverb_ctl: CommandDspReverbCtl<Track16Protocol>,
    monitor_ctl: CommandDspMonitorCtl<Track16Protocol>,
    mixer_ctl: CommandDspMixerCtl<Track16Protocol>,
    input_ctl: CommandDspInputCtl<Track16Protocol>,
    input_eq_ctl: CommandDspInputEqualizerCtl<Track16Protocol>,
    input_dyn_ctl: CommandDspInputDynamicsCtl<Track16Protocol>,
    output_ctl: CommandDspOutputCtl<Track16Protocol>,
    output_eq_ctl: CommandDspOutputEqualizerCtl<Track16Protocol>,
    output_dyn_ctl: CommandDspOutputDynamicsCtl<Track16Protocol>,
    resource_ctl: CommandDspResourceCtl,
    meter_ctl: CommandDspMeterCtl<Track16Protocol>,
}

impl CtlModel<(SndMotu, FwNode)> for Track16Model {
    fn cache(&mut self, (unit, node): &mut (SndMotu, FwNode)) -> Result<(), Error> {
        self.clk_ctls.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.port_assign_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.opt_iface_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.phone_assign_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;

        self.meter_ctl.read_dsp_meter(unit)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctls.load(card_cntr)?;
        self.port_assign_ctl.load(card_cntr)?;
        self.opt_iface_ctl.load(card_cntr)?;
        self.phone_assign_ctl.load(card_cntr)?;
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
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndMotu, FwNode), u32> for Track16Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.port_assign_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.phone_assign_ctl.elem_id_list);
    }

    fn parse_notification(
        &mut self,
        (_, node): &mut (SndMotu, FwNode),
        msg: &u32,
    ) -> Result<(), Error> {
        if *msg & Track16Protocol::NOTIFY_PORT_CHANGE > 0 {
            self.port_assign_ctl
                .cache(&mut self.req, node, TIMEOUT_MS)?;
            self.phone_assign_ctl
                .cache(&mut self.req, node, TIMEOUT_MS)?;
        }
        Ok(())
    }
}

impl NotifyModel<(SndMotu, FwNode), Vec<DspCmd>> for Track16Model {
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

impl MeasureModel<(SndMotu, FwNode)> for Track16Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.elem_id_list);
    }

    fn measure_states(&mut self, (unit, _): &mut (SndMotu, FwNode)) -> Result<(), Error> {
        self.meter_ctl.read_dsp_meter(unit)
    }
}

impl CommandDspModel for Track16Model {
    fn prepare_message_handler<F>(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        handler: F,
    ) -> Result<(), Error>
    where
        F: Fn(&FwResp, FwTcode, u64, u32, u32, u32, u32, u32, &[u8]) -> FwRcode + 'static,
    {
        Track16Protocol::register_message_destination_address(
            &mut self.resp,
            &mut self.req,
            &mut unit.1,
            TIMEOUT_MS,
        )?;
        self.resp.connect_requested(handler);
        Ok(())
    }

    fn begin_messaging(&mut self, unit: &mut (SndMotu, FwNode)) -> Result<(), Error> {
        Track16Protocol::begin_messaging(
            &mut self.req,
            &mut unit.1,
            &mut self.sequence_number,
            TIMEOUT_MS,
        )
    }

    fn release_message_handler(&mut self, unit: &mut (SndMotu, FwNode)) -> Result<(), Error> {
        Track16Protocol::cancel_messaging(
            &mut self.req,
            &mut unit.1,
            &mut self.sequence_number,
            TIMEOUT_MS,
        )?;
        Track16Protocol::release_message_destination_address(
            &mut self.resp,
            &mut self.req,
            &mut unit.1,
            TIMEOUT_MS,
        )?;
        Ok(())
    }
}
