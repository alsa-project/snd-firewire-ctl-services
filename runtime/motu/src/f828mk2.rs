// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::register_dsp_runtime::*;

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct F828mk2 {
    req: FwReq,
    clk_ctls: ClkCtl,
    opt_iface_ctl: OptIfaceCtl,
    phone_assign_ctl: PhoneAssignCtl,
    word_clk_ctl: WordClkCtl,
    mixer_output_ctl: MixerOutputCtl,
    mixer_return_ctl: MixerReturnCtl,
    mixer_source_ctl: MixerSourceCtl,
    output_ctl: OutputCtl,
    line_input_ctl: LineInputCtl,
    params: SndMotuRegisterDspParameter,
    meter: RegisterDspMeterImage,
    meter_ctl: MeterCtl,
}

#[derive(Default)]
struct PhoneAssignCtl(usize, Vec<ElemId>);

impl PhoneAssignCtlOperation<F828mk2Protocol> for PhoneAssignCtl {
    fn state(&self) -> &usize {
        &self.0
    }

    fn state_mut(&mut self) -> &mut usize {
        &mut self.0
    }
}

impl RegisterDspPhoneAssignCtlOperation<F828mk2Protocol> for PhoneAssignCtl {}

#[derive(Default)]
struct WordClkCtl(WordClkSpeedMode, Vec<ElemId>);

impl WordClkCtlOperation<F828mk2Protocol> for WordClkCtl {
    fn state(&self) -> &WordClkSpeedMode {
        &self.0
    }

    fn state_mut(&mut self) -> &mut WordClkSpeedMode {
        &mut self.0
    }
}

#[derive(Default)]
struct ClkCtl;

impl V2ClkCtlOperation<F828mk2Protocol> for ClkCtl {}

#[derive(Default)]
struct OptIfaceCtl((usize, usize), Vec<ElemId>);

impl V2OptIfaceCtlOperation<F828mk2Protocol> for OptIfaceCtl {
    fn state(&self) -> &(usize, usize) {
        &self.0
    }

    fn state_mut(&mut self) -> &mut (usize, usize) {
        &mut self.0
    }
}

#[derive(Default)]
struct MixerOutputCtl(RegisterDspMixerOutputState, Vec<ElemId>);

impl RegisterDspMixerOutputCtlOperation<F828mk2Protocol> for MixerOutputCtl {
    fn state(&self) -> &RegisterDspMixerOutputState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspMixerOutputState {
        &mut self.0
    }
}

#[derive(Default)]
struct MixerReturnCtl(bool, Vec<ElemId>);

impl RegisterDspMixerReturnCtlOperation<F828mk2Protocol> for MixerReturnCtl {
    fn state(&self) -> &bool {
        &self.0
    }

    fn state_mut(&mut self) -> &mut bool {
        &mut self.0
    }
}

struct MixerSourceCtl(RegisterDspMixerMonauralSourceState, Vec<ElemId>);

impl Default for MixerSourceCtl {
    fn default() -> Self {
        Self(
            F828mk2Protocol::create_mixer_monaural_source_state(),
            Default::default(),
        )
    }
}

impl RegisterDspMixerMonauralSourceCtlOperation<F828mk2Protocol> for MixerSourceCtl {
    fn state(&self) -> &RegisterDspMixerMonauralSourceState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspMixerMonauralSourceState {
        &mut self.0
    }
}

#[derive(Default)]
struct OutputCtl(RegisterDspOutputState, Vec<ElemId>);

impl RegisterDspOutputCtlOperation<F828mk2Protocol> for OutputCtl {
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
            F828mk2Protocol::create_line_input_state(),
            Default::default(),
        )
    }
}

impl RegisterDspLineInputCtlOperation<F828mk2Protocol> for LineInputCtl {
    fn state(&self) -> &RegisterDspLineInputState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspLineInputState {
        &mut self.0
    }
}

struct MeterCtl(RegisterDspMeterState, Vec<ElemId>);

impl Default for MeterCtl {
    fn default() -> Self {
        Self(F828mk2Protocol::create_meter_state(), Default::default())
    }
}

impl RegisterDspMeterCtlOperation<F828mk2Protocol> for MeterCtl {
    fn state(&self) -> &RegisterDspMeterState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspMeterState {
        &mut self.0
    }
}

impl CtlModel<(SndMotu, FwNode)> for F828mk2 {
    fn load(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.clk_ctls.load(card_cntr)?;
        self.opt_iface_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.opt_iface_ctl.1.append(&mut elem_id_list))?;
        self.phone_assign_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.phone_assign_ctl.1.append(&mut elem_id_list))?;
        self.word_clk_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.word_clk_ctl.1.append(&mut elem_id_list))?;
        self.mixer_output_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.mixer_output_ctl.1 = elem_id_list)?;
        self.mixer_return_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.mixer_source_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.mixer_source_ctl.1 = elem_id_list)?;
        self.output_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.output_ctl.1 = elem_id_list)?;
        self.line_input_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.line_input_ctl.1 = elem_id_list)?;
        self.meter_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.meter_ctl.1 = elem_id_list)?;
        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut (SndMotu, FwNode),
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
        } else if self.word_clk_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_return_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_source_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.line_input_ctl.read(elem_id, elem_value)? {
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
            .word_clk_ctl
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
            .line_input_ctl
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

impl NotifyModel<(SndMotu, FwNode), u32> for F828mk2 {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.word_clk_ctl.1);
    }

    fn parse_notification(&mut self, unit: &mut (SndMotu, FwNode), msg: &u32) -> Result<(), Error> {
        if *msg & F828mk2Protocol::NOTIFY_PORT_CHANGE > 0 {
            self.word_clk_ctl.cache(unit, &mut self.req, TIMEOUT_MS)?;
        }
        // TODO: what kind of event is preferable for NOTIFY_FOOTSWITCH_MASK?
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndMotu, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.word_clk_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndMotu, FwNode), bool> for F828mk2 {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.phone_assign_ctl.1);
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
        if self.phone_assign_ctl.read(elem_id, elem_value)? {
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

impl NotifyModel<(SndMotu, FwNode), Vec<RegisterDspEvent>> for F828mk2 {
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
        if self.phone_assign_ctl.read(elem_id, elem_value)? {
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

impl MeasureModel<(SndMotu, FwNode)> for F828mk2 {
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
