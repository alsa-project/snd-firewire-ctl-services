// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::register_dsp_runtime::*;

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct UltraLite {
    req: FwReq,
    clk_ctls: ClkCtl,
    main_assign_ctl: MainAssignCtl,
    phone_assign_ctl: PhoneAssignCtl,
    mixer_output_ctl: MixerOutputCtl,
    mixer_return_ctl: MixerReturnCtl,
    mixer_source_ctl: MixerSourceCtl,
    output_ctl: OutputCtl,
    input_ctl: InputCtl,
}

#[derive(Default)]
struct PhoneAssignCtl(usize, Vec<ElemId>);

impl PhoneAssignCtlOperation<UltraliteProtocol> for PhoneAssignCtl {
    fn state(&self) -> &usize {
        &self.0
    }

    fn state_mut(&mut self) -> &mut usize {
        &mut self.0
    }
}

#[derive(Default)]
struct ClkCtl;

impl V2ClkCtlOperation<UltraliteProtocol> for ClkCtl {}

#[derive(Default)]
struct MainAssignCtl(usize, Vec<ElemId>);

#[derive(Default)]
struct MixerOutputCtl(RegisterDspMixerOutputState, Vec<ElemId>);

impl RegisterDspMixerOutputCtlOperation<UltraliteProtocol> for MixerOutputCtl {
    fn state(&self) -> &RegisterDspMixerOutputState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspMixerOutputState {
        &mut self.0
    }
}

#[derive(Default)]
struct MixerReturnCtl(bool, Vec<ElemId>);

impl RegisterDspMixerReturnCtlOperation<UltraliteProtocol> for MixerReturnCtl {
    fn state(&self) -> &bool {
        &self.0
    }

    fn state_mut(&mut self) -> &mut bool {
        &mut self.0
    }
}

#[derive(Default)]
struct MixerSourceCtl(RegisterDspMixerMonauralSourceState, Vec<ElemId>);

impl RegisterDspMixerMonauralSourceCtlOperation<UltraliteProtocol> for MixerSourceCtl {
    fn state(&self) -> &RegisterDspMixerMonauralSourceState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspMixerMonauralSourceState {
        &mut self.0
    }
}

#[derive(Default)]
struct OutputCtl(RegisterDspOutputState, Vec<ElemId>);

impl RegisterDspOutputCtlOperation<UltraliteProtocol> for OutputCtl {
    fn state(&self) -> &RegisterDspOutputState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspOutputState {
        &mut self.0
    }
}

#[derive(Default)]
struct InputCtl(UltraliteInputState, Vec<ElemId>);

impl CtlModel<SndMotu> for UltraLite {
    fn load(&mut self, unit: &mut SndMotu, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctls.load(card_cntr)?;
        self.main_assign_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
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
        self.input_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.input_ctl.1 = elem_id_list)?;
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
        } else if self.main_assign_ctl.read(elem_id, elem_value)? {
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
        } else if self.input_ctl.read(elem_id, elem_value)? {
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
            .main_assign_ctl
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

impl NotifyModel<SndMotu, u32> for UltraLite {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.main_assign_ctl.1);
    }

    fn parse_notification(&mut self, unit: &mut SndMotu, msg: &u32) -> Result<(), Error> {
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
        _: &SndMotu,
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

const MAIN_ASSIGNMENT_NAME: &str = "main-assign";

impl MainAssignCtl {
    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndMotu,
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

    fn cache(&mut self, unit: &mut SndMotu, req: &mut FwReq, timeout_ms: u32) -> Result<(), Error> {
        UltraliteProtocol::get_main_assign(req, &mut unit.get_node(), timeout_ms)
            .map(|idx| self.0 = idx)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MAIN_ASSIGNMENT_NAME => {
                elem_value.set_enum(&[self.0 as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_id: &ElemId,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MAIN_ASSIGNMENT_NAME => ElemValueAccessor::<u32>::get_val(new, |val| {
                UltraliteProtocol::set_main_assign(
                    req,
                    &mut unit.get_node(),
                    val as usize,
                    timeout_ms,
                )
                .map(|_| self.0 = val as usize)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }
}

const INPUT_GAIN_NAME: &str = "input-gain";
const INPUT_INVERT_NAME: &str = "input-revert";

impl InputCtl {
    const GAIN_TLV: DbInterval = DbInterval {
        min: -6400,
        max: 0,
        linear: true,
        mute_avail: false,
    };

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndMotu,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        UltraliteProtocol::read_input_state(req, &mut unit.get_node(), &mut self.0, timeout_ms)?;

        let mut notified_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                UltraliteProtocol::INPUT_GAIN_MIN as i32,
                UltraliteProtocol::INPUT_GAIN_MAX as i32,
                UltraliteProtocol::INPUT_GAIN_STEP as i32,
                UltraliteProtocol::INPUT_COUNT,
                Some(&Vec::<u32>::from(&Self::GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_INVERT_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, UltraliteProtocol::INPUT_COUNT, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            INPUT_GAIN_NAME => {
                let vals: Vec<i32> = self.0.gain.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            INPUT_INVERT_NAME => {
                elem_value.set_bool(&self.0.invert);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            INPUT_GAIN_NAME => {
                let mut vals = [0; UltraliteProtocol::INPUT_COUNT];
                elem_value.get_int(&mut vals);
                let gain: Vec<u8> = vals.iter().map(|&val| val as u8).collect();
                UltraliteProtocol::write_input_gain(
                    req,
                    &mut unit.get_node(),
                    &gain,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            INPUT_INVERT_NAME => {
                let mut invert = [false; UltraliteProtocol::INPUT_COUNT];
                elem_value.get_bool(&mut invert);
                UltraliteProtocol::write_input_invert(
                    req,
                    &mut unit.get_node(),
                    &invert,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
