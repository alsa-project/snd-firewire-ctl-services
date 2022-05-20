// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::command_dsp_runtime::*;

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct Track16 {
    req: FwReq,
    resp: FwResp,
    clk_ctls: ClkCtl,
    port_assign_ctl: PortAssignCtl,
    opt_iface_ctl: OptIfaceCtl,
    phone_assign_ctl: PhoneAssignCtl,
    sequence_number: u8,
}

#[derive(Default)]
struct PhoneAssignCtl(usize, Vec<ElemId>);

impl PhoneAssignCtlOperation<Track16Protocol> for PhoneAssignCtl {
    fn state(&self) -> &usize {
        &self.0
    }

    fn state_mut(&mut self) -> &mut usize {
        &mut self.0
    }
}

#[derive(Default)]
struct ClkCtl;

impl V3ClkCtlOperation<Track16Protocol> for ClkCtl {}

#[derive(Default)]
struct PortAssignCtl(V3PortAssignState, Vec<ElemId>);

impl V3PortAssignCtlOperation<Track16Protocol> for PortAssignCtl {
    fn state(&self) -> &V3PortAssignState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut V3PortAssignState {
        &mut self.0
    }
}

#[derive(Default)]
struct OptIfaceCtl;

impl V3OptIfaceCtlOperation<Track16Protocol> for OptIfaceCtl {}

impl CtlModel<SndMotu> for Track16 {
    fn load(&mut self, unit: &mut SndMotu, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctls.load(card_cntr)?;
        self.port_assign_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.port_assign_ctl.1.append(&mut elem_id_list))?;
        self.opt_iface_ctl.load(card_cntr)?;
        self.phone_assign_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.phone_assign_ctl.1.append(&mut elem_id_list))?;
        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut SndMotu,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self
            .clk_ctls
            .read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.port_assign_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self
            .opt_iface_ctl
            .read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.phone_assign_ctl.read(elem_id, elem_value)? {
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
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .clk_ctls
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .port_assign_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .opt_iface_ctl
            .write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phone_assign_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndMotu, u32> for Track16 {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.port_assign_ctl.1);
        elem_id_list.extend_from_slice(&self.phone_assign_ctl.1);
    }

    fn parse_notification(&mut self, unit: &mut SndMotu, msg: &u32) -> Result<(), Error> {
        if *msg & Track16Protocol::NOTIFY_PORT_CHANGE > 0 {
            self.port_assign_ctl
                .cache(unit, &mut self.req, TIMEOUT_MS)?;
            self.phone_assign_ctl
                .cache(unit, &mut self.req, TIMEOUT_MS)?;
        }
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndMotu,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.port_assign_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.phone_assign_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndMotu, Vec<DspCmd>> for Track16 {
    fn get_notified_elem_list(&mut self, _: &mut Vec<ElemId>) {}

    fn parse_notification(&mut self, _: &mut SndMotu, _: &Vec<DspCmd>) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndMotu,
        _: &ElemId,
        _: &mut ElemValue,
    ) -> Result<bool, Error> {
        Ok(false)
    }
}

impl MeasureModel<SndMotu> for Track16 {
    fn get_measure_elem_list(&mut self, _: &mut Vec<ElemId>) {}

    fn measure_states(&mut self, _: &mut SndMotu) -> Result<(), Error> {
        Ok(())
    }

    fn measure_elem(&mut self, _: &SndMotu, _: &ElemId, _: &mut ElemValue) -> Result<bool, Error> {
        Ok(false)
    }
}

impl CommandDspModel for Track16 {
    fn prepare_message_handler<F>(&mut self, unit: &mut SndMotu, handler: F) -> Result<(), Error>
    where
        F: Fn(&FwResp, FwTcode, u64, u32, u32, u32, u32, &[u8]) -> FwRcode + 'static,
    {
        Track16Protocol::register_message_destination_address(
            &mut self.resp,
            &mut self.req,
            &mut unit.get_node(),
            TIMEOUT_MS,
        )?;
        self.resp.connect_requested2(handler);
        Ok(())
    }

    fn begin_messaging(&mut self, unit: &mut SndMotu) -> Result<(), Error> {
        Track16Protocol::begin_messaging(
            &mut self.req,
            &mut unit.get_node(),
            &mut self.sequence_number,
            TIMEOUT_MS,
        )
    }

    fn release_message_handler(&mut self, unit: &mut SndMotu) -> Result<(), Error> {
        Track16Protocol::cancel_messaging(
            &mut self.req,
            &mut unit.get_node(),
            &mut self.sequence_number,
            TIMEOUT_MS,
        )?;
        Track16Protocol::release_message_destination_address(
            &mut self.resp,
            &mut self.req,
            &mut unit.get_node(),
            TIMEOUT_MS,
        )?;
        Ok(())
    }
}
