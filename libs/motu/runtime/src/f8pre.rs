// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::register_dsp_runtime::*;

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct F8pre {
    req: FwReq,
    clk_ctls: ClkCtl,
    opt_iface_ctl: OptIfaceCtl,
    phone_assign_ctl: PhoneAssignCtl,
    mixer_output_ctl: MixerOutputCtl,
    mixer_return_ctl: MixerReturnCtl,
    mixer_source_ctl: MixerSourceCtl,
    output_ctl: OutputCtl,
    params: SndMotuRegisterDspParameter,
}

#[derive(Default)]
struct PhoneAssignCtl(usize, Vec<ElemId>);

impl PhoneAssignCtlOperation<F8preProtocol> for PhoneAssignCtl {
    fn state(&self) -> &usize {
        &self.0
    }

    fn state_mut(&mut self) -> &mut usize {
        &mut self.0
    }
}

impl RegisterDspPhoneAssignCtlOperation<F8preProtocol> for PhoneAssignCtl {}

#[derive(Default)]
struct ClkCtl;

impl V2ClkCtlOperation<F8preProtocol> for ClkCtl {}

#[derive(Default)]
struct OptIfaceCtl((usize, usize), Vec<ElemId>);

impl V2OptIfaceCtlOperation<F8preProtocol> for OptIfaceCtl {
    fn state(&self) -> &(usize, usize) {
        &self.0
    }

    fn state_mut(&mut self) -> &mut (usize, usize) {
        &mut self.0
    }
}

#[derive(Default)]
struct MixerOutputCtl(RegisterDspMixerOutputState, Vec<ElemId>);

impl RegisterDspMixerOutputCtlOperation<F8preProtocol> for MixerOutputCtl {
    fn state(&self) -> &RegisterDspMixerOutputState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspMixerOutputState {
        &mut self.0
    }
}

#[derive(Default)]
struct MixerReturnCtl(bool, Vec<ElemId>);

impl RegisterDspMixerReturnCtlOperation<F8preProtocol> for MixerReturnCtl {
    fn state(&self) -> &bool {
        &self.0
    }

    fn state_mut(&mut self) -> &mut bool {
        &mut self.0
    }
}

#[derive(Default)]
struct MixerSourceCtl(RegisterDspMixerMonauralSourceState, Vec<ElemId>);

impl RegisterDspMixerMonauralSourceCtlOperation<F8preProtocol> for MixerSourceCtl {
    fn state(&self) -> &RegisterDspMixerMonauralSourceState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspMixerMonauralSourceState {
        &mut self.0
    }
}

#[derive(Default)]
struct OutputCtl(RegisterDspOutputState, Vec<ElemId>);

impl RegisterDspOutputCtlOperation<F8preProtocol> for OutputCtl {
    fn state(&self) -> &RegisterDspOutputState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspOutputState {
        &mut self.0
    }
}

impl CtlModel<SndMotu> for F8pre {
    fn load(&mut self, unit: &mut SndMotu, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctls.load(card_cntr)?;
        self.opt_iface_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.opt_iface_ctl.1.append(&mut elem_id_list))?;
        self.phone_assign_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.phone_assign_ctl.1.append(&mut elem_id_list))?;
        self.mixer_output_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.mixer_output_ctl.1 = elem_id_list)?;
        let _ = self
            .mixer_return_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.mixer_source_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.mixer_source_ctl.1 = elem_id_list)?;
        self.output_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.output_ctl.1 = elem_id_list)?;
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
        } else if self.opt_iface_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.phone_assign_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_return_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_source_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut SndMotu,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .clk_ctls
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .opt_iface_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phone_assign_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_output_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_return_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_source_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .output_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndMotu, u32> for F8pre {
    fn get_notified_elem_list(&mut self, _: &mut Vec<ElemId>) {}

    fn parse_notification(&mut self, _: &mut SndMotu, _: &u32) -> Result<(), Error> {
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

impl NotifyModel<SndMotu, bool> for F8pre {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.mixer_output_ctl.1);
        elem_id_list.extend_from_slice(&self.mixer_source_ctl.1);
        elem_id_list.extend_from_slice(&self.output_ctl.1);
    }

    fn parse_notification(&mut self, unit: &mut SndMotu, is_locked: &bool) -> Result<(), Error> {
        if *is_locked {
            unit.read_register_dsp_parameter(&mut self.params).map(|_| {
                self.mixer_output_ctl.parse_dsp_parameter(&self.params);
                self.mixer_source_ctl.parse_dsp_parameter(&self.params);
                self.output_ctl.parse_dsp_parameter(&self.params);
            })
        } else {
            Ok(())
        }
    }

    fn read_notified_elem(
        &mut self,
        _: &SndMotu,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.phone_assign_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_source_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndMotu, Vec<RegisterDspEvent>> for F8pre {
    fn get_notified_elem_list(&mut self, _: &mut Vec<ElemId>) {
        // MEMO: handled by the above implementation.
    }

    fn parse_notification(
        &mut self,
        _: &mut SndMotu,
        events: &Vec<RegisterDspEvent>,
    ) -> Result<(), Error> {
        events.iter().for_each(|event| {
            let _ = self.mixer_output_ctl.parse_dsp_event(event)
                || self.mixer_source_ctl.parse_dsp_event(event)
                || self.output_ctl.parse_dsp_event(event);
        });
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndMotu,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.phone_assign_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_source_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<SndMotu> for F8pre {
    fn get_measure_elem_list(&mut self, _: &mut Vec<ElemId>) {}

    fn measure_states(&mut self, _: &mut SndMotu) -> Result<(), Error> {
        Ok(())
    }

    fn measure_elem(&mut self, _: &SndMotu, _: &ElemId, _: &mut ElemValue) -> Result<bool, Error> {
        Ok(false)
    }
}
