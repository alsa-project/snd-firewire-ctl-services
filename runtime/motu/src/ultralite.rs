// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::{register_dsp_ctls::*, register_dsp_runtime::*, v2_ctls::*};

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct UltraLite {
    req: FwReq,
    clk_ctls: V2ClkCtl<UltraliteProtocol>,
    main_assign_ctl: MainAssignCtl,
    phone_assign_ctl: RegisterDspPhoneAssignCtl<UltraliteProtocol>,
    mixer_return_ctl: RegisterDspMixerReturnCtl<UltraliteProtocol>,
    params: SndMotuRegisterDspParameter,
    mixer_output_ctl: RegisterDspMixerOutputCtl<UltraliteProtocol>,
    mixer_source_ctl: RegisterDspMixerMonauralSourceCtl<UltraliteProtocol>,
    output_ctl: RegisterDspOutputCtl<UltraliteProtocol>,
    input_ctl: InputCtl,
    meter: RegisterDspMeterImage,
    meter_ctl: MeterCtl,
}

#[derive(Default)]
struct MainAssignCtl(usize, Vec<ElemId>);

struct InputCtl(RegisterDspMonauralInputState, Vec<ElemId>);

impl Default for InputCtl {
    fn default() -> Self {
        Self(
            UltraliteProtocol::create_monaural_input_state(),
            Default::default(),
        )
    }
}

impl RegisterDspMonauralInputCtlOperation<UltraliteProtocol> for InputCtl {
    fn state(&self) -> &RegisterDspMonauralInputState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspMonauralInputState {
        &mut self.0
    }
}

struct MeterCtl(RegisterDspMeterState, Vec<ElemId>);

impl Default for MeterCtl {
    fn default() -> Self {
        Self(UltraliteProtocol::create_meter_state(), Default::default())
    }
}

impl RegisterDspMeterCtlOperation<UltraliteProtocol> for MeterCtl {
    fn state(&self) -> &RegisterDspMeterState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspMeterState {
        &mut self.0
    }
}

impl CtlModel<(SndMotu, FwNode)> for UltraLite {
    fn load(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        unit.0.read_parameter(&mut self.params)?;
        self.phone_assign_ctl.parse_dsp_parameter(&self.params);
        self.mixer_output_ctl.parse_dsp_parameter(&self.params);
        self.mixer_source_ctl.parse_dsp_parameter(&self.params);
        self.output_ctl.parse_dsp_parameter(&self.params);

        self.clk_ctls
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.phone_assign_ctl
            .0
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.mixer_return_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.mixer_source_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.output_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;

        self.clk_ctls.load(card_cntr)?;
        self.main_assign_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.phone_assign_ctl.0.load(card_cntr)?;
        self.mixer_return_ctl.load(card_cntr)?;
        self.mixer_output_ctl.load(card_cntr)?;
        self.mixer_source_ctl.load(card_cntr)?;
        self.output_ctl.load(card_cntr)?;
        self.input_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.input_ctl.1 = elem_id_list)?;
        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndMotu, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.main_assign_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.phone_assign_ctl.0.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_return_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_source_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctls.write(
            &mut unit.0,
            &mut self.req,
            &mut unit.1,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .main_assign_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.phone_assign_ctl.0.write(
            &mut self.req,
            &mut unit.1,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_return_ctl.write(
            &mut self.req,
            &mut unit.1,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_output_ctl.write(
            &mut self.req,
            &mut unit.1,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_source_ctl.write(
            &mut self.req,
            &mut unit.1,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .output_ctl
            .write(&mut self.req, &mut unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .input_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndMotu, FwNode), u32> for UltraLite {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.main_assign_ctl.1);
    }

    fn parse_notification(&mut self, unit: &mut (SndMotu, FwNode), msg: &u32) -> Result<(), Error> {
        if *msg & UltraliteProtocol::NOTIFY_PORT_CHANGE > 0 {
            // Just after changing, busy rcode returns so often.
            std::thread::sleep(std::time::Duration::from_millis(10));
            self.main_assign_ctl
                .cache(unit, &mut self.req, TIMEOUT_MS)?;
        }
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndMotu, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.main_assign_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndMotu, FwNode), bool> for UltraLite {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.phone_assign_ctl.0.elem_id_list);
        elem_id_list.extend_from_slice(&self.mixer_output_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.mixer_source_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.output_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.input_ctl.1);
    }

    fn parse_notification(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        is_locked: &bool,
    ) -> Result<(), Error> {
        if *is_locked {
            unit.0.read_parameter(&mut self.params).map(|_| {
                self.phone_assign_ctl.parse_dsp_parameter(&self.params);
                self.mixer_output_ctl.parse_dsp_parameter(&self.params);
                self.mixer_source_ctl.parse_dsp_parameter(&self.params);
                self.output_ctl.parse_dsp_parameter(&self.params);
                self.input_ctl.parse_dsp_parameter(&self.params);
            })
        } else {
            Ok(())
        }
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndMotu, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.phone_assign_ctl.0.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_source_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndMotu, FwNode), Vec<RegisterDspEvent>> for UltraLite {
    fn get_notified_elem_list(&mut self, _: &mut Vec<ElemId>) {
        // MEMO: handled by the above implementation.
    }

    fn parse_notification(
        &mut self,
        _: &mut (SndMotu, FwNode),
        events: &Vec<RegisterDspEvent>,
    ) -> Result<(), Error> {
        events.iter().for_each(|event| {
            let _ = self.mixer_output_ctl.parse_dsp_event(event)
                || self.mixer_source_ctl.parse_dsp_event(event)
                || self.output_ctl.parse_dsp_event(event)
                || self.input_ctl.parse_dsp_event(event);
        });
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndMotu, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.phone_assign_ctl.0.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_source_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndMotu, FwNode)> for UltraLite {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut (SndMotu, FwNode)) -> Result<(), Error> {
        self.meter_ctl.read_dsp_meter(&mut unit.0, &mut self.meter)
    }

    fn measure_elem(
        &mut self,
        _: &(SndMotu, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

const MAIN_ASSIGNMENT_NAME: &str = "main-assign";

impl MainAssignCtl {
    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.cache(unit, req, timeout_ms)?;

        let labels: Vec<String> = UltraliteProtocol::KNOB_TARGETS
            .iter()
            .map(|e| target_port_to_string(&e.0))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MAIN_ASSIGNMENT_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn cache(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        UltraliteProtocol::get_main_assign(req, &mut unit.1, timeout_ms).map(|idx| self.0 = idx)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MAIN_ASSIGNMENT_NAME => {
                elem_value.set_enum(&[self.0 as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MAIN_ASSIGNMENT_NAME => ElemValueAccessor::<u32>::get_val(new, |val| {
                UltraliteProtocol::set_main_assign(req, &mut unit.1, val as usize, timeout_ms)
                    .map(|_| self.0 = val as usize)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }
}
