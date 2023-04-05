// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::{common_ctls::*, register_dsp_ctls::*, register_dsp_runtime::*, v2_ctls::*};

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct Traveler {
    req: FwReq,
    clk_ctls: V2ClkCtl<TravelerProtocol>,
    opt_iface_ctl: V2OptIfaceCtl<TravelerProtocol>,
    phone_assign_ctl: RegisterDspPhoneAssignCtl<TravelerProtocol>,
    word_clk_ctl: WordClockCtl<TravelerProtocol>,
    mixer_return_ctl: RegisterDspMixerReturnCtl<TravelerProtocol>,
    mixer_output_ctl: MixerOutputCtl,
    mixer_source_ctl: MixerSourceCtl,
    output_ctl: OutputCtl,
    line_input_ctl: LineInputCtl,
    mic_input_ctl: MicInputCtl,
    params: SndMotuRegisterDspParameter,
    meter: RegisterDspMeterImage,
    meter_ctl: MeterCtl,
}

#[derive(Default)]
struct MixerOutputCtl(RegisterDspMixerOutputState, Vec<ElemId>);

impl RegisterDspMixerOutputCtlOperation<TravelerProtocol> for MixerOutputCtl {
    fn state(&self) -> &RegisterDspMixerOutputState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspMixerOutputState {
        &mut self.0
    }
}

struct MixerSourceCtl(RegisterDspMixerMonauralSourceState, Vec<ElemId>);

impl Default for MixerSourceCtl {
    fn default() -> Self {
        Self(
            TravelerProtocol::create_mixer_monaural_source_state(),
            Default::default(),
        )
    }
}

impl RegisterDspMixerMonauralSourceCtlOperation<TravelerProtocol> for MixerSourceCtl {
    fn state(&self) -> &RegisterDspMixerMonauralSourceState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspMixerMonauralSourceState {
        &mut self.0
    }
}

#[derive(Default)]
struct OutputCtl(RegisterDspOutputState, Vec<ElemId>);

impl RegisterDspOutputCtlOperation<TravelerProtocol> for OutputCtl {
    fn state(&self) -> &RegisterDspOutputState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspOutputState {
        &mut self.0
    }
}

struct LineInputCtl(RegisterDspLineInputState, Vec<ElemId>);

impl Default for LineInputCtl {
    fn default() -> Self {
        Self(
            TravelerProtocol::create_line_input_state(),
            Default::default(),
        )
    }
}

impl RegisterDspLineInputCtlOperation<TravelerProtocol> for LineInputCtl {
    fn state(&self) -> &RegisterDspLineInputState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspLineInputState {
        &mut self.0
    }
}

#[derive(Default)]
struct MicInputCtl(TravelerMicInputState, Vec<ElemId>);

struct MeterCtl(RegisterDspMeterState, Vec<ElemId>);

impl Default for MeterCtl {
    fn default() -> Self {
        Self(TravelerProtocol::create_meter_state(), Default::default())
    }
}

impl RegisterDspMeterCtlOperation<TravelerProtocol> for MeterCtl {
    fn state(&self) -> &RegisterDspMeterState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspMeterState {
        &mut self.0
    }
}

impl CtlModel<(SndMotu, FwNode)> for Traveler {
    fn load(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        unit.0.read_parameter(&mut self.params)?;
        self.phone_assign_ctl.parse_dsp_parameter(&self.params);

        self.clk_ctls
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.word_clk_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.opt_iface_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.phone_assign_ctl
            .0
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.mixer_return_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;

        self.clk_ctls.load(card_cntr)?;
        self.opt_iface_ctl.load(card_cntr)?;
        self.phone_assign_ctl.0.load(card_cntr)?;
        self.word_clk_ctl.load(card_cntr)?;
        self.mixer_return_ctl.load(card_cntr)?;
        self.mixer_output_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.mixer_output_ctl.1 = elem_id_list)?;
        self.mixer_source_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.mixer_source_ctl.1 = elem_id_list)?;
        self.output_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.output_ctl.1 = elem_id_list)?;
        self.line_input_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.line_input_ctl.1 = elem_id_list)?;
        self.mic_input_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.mic_input_ctl.1 = elem_id_list)?;
        self.meter_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.meter_ctl.1 = elem_id_list)?;
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
        } else if self.opt_iface_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.phone_assign_ctl.0.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.word_clk_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_return_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_source_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.line_input_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mic_input_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read(elem_id, elem_value)? {
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
        } else if self.opt_iface_ctl.write(
            &mut unit.0,
            &mut self.req,
            &mut unit.1,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.phone_assign_ctl.0.write(
            &mut self.req,
            &mut unit.1,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .word_clk_ctl
            .write(&mut self.req, &mut unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.mixer_return_ctl.write(
            &mut self.req,
            &mut unit.1,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .mixer_output_ctl
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
            .line_input_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mic_input_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .meter_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndMotu, FwNode), u32> for Traveler {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.mic_input_ctl.1);
        elem_id_list.extend_from_slice(&self.phone_assign_ctl.0.elem_id_list);
        elem_id_list.extend_from_slice(&self.word_clk_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.opt_iface_ctl.elem_id_list);
    }

    fn parse_notification(&mut self, unit: &mut (SndMotu, FwNode), msg: &u32) -> Result<(), Error> {
        if *msg & TravelerProtocol::NOTIFY_MIC_PARAM_MASK > 0 {
            self.mic_input_ctl.cache(unit, &mut self.req, TIMEOUT_MS)?;
        }
        if *msg & TravelerProtocol::NOTIFY_PORT_CHANGE > 0 {
            self.phone_assign_ctl
                .0
                .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
            self.word_clk_ctl
                .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        }
        if *msg & TravelerProtocol::NOTIFY_FORMAT_CHANGE > 0 {
            self.opt_iface_ctl
                .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        }
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndMotu, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.mic_input_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.phone_assign_ctl.0.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.word_clk_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.opt_iface_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndMotu, FwNode), bool> for Traveler {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.phone_assign_ctl.0.elem_id_list);
        elem_id_list.extend_from_slice(&self.mixer_output_ctl.1);
        elem_id_list.extend_from_slice(&self.mixer_source_ctl.1);
        elem_id_list.extend_from_slice(&self.output_ctl.1);
        elem_id_list.extend_from_slice(&self.line_input_ctl.1);
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
                self.line_input_ctl.parse_dsp_parameter(&self.params);
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
        } else if self.line_input_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndMotu, FwNode), Vec<RegisterDspEvent>> for Traveler {
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
                || self.line_input_ctl.parse_dsp_event(event);
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
        } else if self.line_input_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndMotu, FwNode)> for Traveler {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut (SndMotu, FwNode)) -> Result<(), Error> {
        self.meter_ctl.read_dsp_meter(&unit.0, &mut self.meter)
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

const MIC_GAIN_NAME: &str = "mic-gain-name";
const MIC_PAD_NAME: &str = "mic-pad-name";

impl MicInputCtl {
    const GAIN_TLV: DbInterval = DbInterval {
        min: -6400,
        max: 0,
        linear: true,
        mute_avail: false,
    };

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        self.cache(unit, req, timeout_ms)?;

        let mut notified_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                TravelerProtocol::MIC_GAIN_MIN as i32,
                TravelerProtocol::MIC_GAIN_MAX as i32,
                TravelerProtocol::MIC_GAIN_STEP as i32,
                TravelerProtocol::MIC_INPUT_COUNT,
                Some(&Vec::<u32>::from(&Self::GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_PAD_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, TravelerProtocol::MIC_INPUT_COUNT, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn cache(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        TravelerProtocol::read_mic_input_state(req, &mut unit.1, &mut self.0, timeout_ms)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIC_GAIN_NAME => {
                let vals: Vec<i32> = self.0.gain.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MIC_PAD_NAME => {
                elem_value.set_bool(&self.0.pad);
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
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIC_GAIN_NAME => {
                let vals = &elem_value.int()[..TravelerProtocol::MIC_INPUT_COUNT];
                let gain: Vec<u8> = vals.iter().map(|&val| val as u8).collect();
                TravelerProtocol::write_mic_gain(req, &mut unit.1, &gain, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MIC_PAD_NAME => {
                let pad = &elem_value.boolean()[..TravelerProtocol::MIC_INPUT_COUNT];
                TravelerProtocol::write_mic_pad(req, &mut unit.1, &pad, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
