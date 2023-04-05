// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use super::{common_ctls::*, register_dsp_ctls::*, register_dsp_runtime::*, v2_ctls::*};

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct F896hd {
    req: FwReq,
    clk_ctls: V2ClkCtl<F896hdProtocol>,
    opt_iface_ctl: V2OptIfaceCtl<F896hdProtocol>,
    word_clk_ctl: WordClockCtl<F896hdProtocol>,
    aesebu_rate_convert_ctl: AesebuRateConvertCtl<F896hdProtocol>,
    level_meters_ctl: LevelMetersCtl<F896hdProtocol>,
    mixer_return_ctl: RegisterDspMixerReturnCtl<F896hdProtocol>,
    params: SndMotuRegisterDspParameter,
    mixer_output_ctl: RegisterDspMixerOutputCtl<F896hdProtocol>,
    mixer_source_ctl: MixerSourceCtl,
    output_ctl: OutputCtl,
    meter: RegisterDspMeterImage,
    meter_ctl: MeterCtl,
}

struct MixerSourceCtl(RegisterDspMixerMonauralSourceState, Vec<ElemId>);

impl Default for MixerSourceCtl {
    fn default() -> Self {
        Self(
            F896hdProtocol::create_mixer_monaural_source_state(),
            Default::default(),
        )
    }
}

impl RegisterDspMixerMonauralSourceCtlOperation<F896hdProtocol> for MixerSourceCtl {
    fn state(&self) -> &RegisterDspMixerMonauralSourceState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspMixerMonauralSourceState {
        &mut self.0
    }
}

#[derive(Default)]
struct OutputCtl(RegisterDspOutputState, Vec<ElemId>);

impl RegisterDspOutputCtlOperation<F896hdProtocol> for OutputCtl {
    fn state(&self) -> &RegisterDspOutputState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspOutputState {
        &mut self.0
    }
}

struct MeterCtl(RegisterDspMeterState, Vec<ElemId>);

impl Default for MeterCtl {
    fn default() -> Self {
        Self(F896hdProtocol::create_meter_state(), Default::default())
    }
}

impl RegisterDspMeterCtlOperation<F896hdProtocol> for MeterCtl {
    fn state(&self) -> &RegisterDspMeterState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut RegisterDspMeterState {
        &mut self.0
    }
}

impl CtlModel<(SndMotu, FwNode)> for F896hd {
    fn load(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        unit.0.read_parameter(&mut self.params)?;
        self.mixer_output_ctl.parse_dsp_parameter(&self.params);

        self.clk_ctls
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.word_clk_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.aesebu_rate_convert_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.level_meters_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.opt_iface_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.mixer_return_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.mixer_output_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;

        self.clk_ctls.load(card_cntr)?;
        self.opt_iface_ctl.load(card_cntr)?;
        self.word_clk_ctl.load(card_cntr)?;
        self.aesebu_rate_convert_ctl.load(card_cntr)?;
        self.level_meters_ctl.load(card_cntr)?;
        self.mixer_return_ctl.load(card_cntr)?;
        self.mixer_output_ctl.load(card_cntr)?;
        self.mixer_source_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.mixer_source_ctl.1 = elem_id_list)?;
        self.output_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|elem_id_list| self.output_ctl.1 = elem_id_list)?;
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
        } else if self.word_clk_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.aesebu_rate_convert_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.level_meters_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_return_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_source_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
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
        } else if self
            .word_clk_ctl
            .write(&mut self.req, &mut unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.aesebu_rate_convert_ctl.write(
            &mut self.req,
            &mut unit.1,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.level_meters_ctl.write(
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
            .meter_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndMotu, FwNode), u32> for F896hd {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.level_meters_ctl.elem_id_list);
    }

    fn parse_notification(
        &mut self,
        (_, node): &mut (SndMotu, FwNode),
        msg: &u32,
    ) -> Result<(), Error> {
        if *msg & F896hdProtocol::NOTIFY_PROGRAMMABLE_METER_MASK > 0 {
            self.level_meters_ctl
                .cache(&mut self.req, node, TIMEOUT_MS)?;
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
        if self.level_meters_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndMotu, FwNode), bool> for F896hd {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.mixer_output_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.mixer_source_ctl.1);
        elem_id_list.extend_from_slice(&self.output_ctl.1);
    }

    fn parse_notification(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        is_locked: &bool,
    ) -> Result<(), Error> {
        if *is_locked {
            unit.0.read_parameter(&mut self.params).map(|_| {
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
        _: &(SndMotu, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.mixer_output_ctl.read(elem_id, elem_value)? {
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

impl NotifyModel<(SndMotu, FwNode), Vec<RegisterDspEvent>> for F896hd {
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
                || self.output_ctl.parse_dsp_event(event);
        });
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndMotu, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.mixer_output_ctl.read(elem_id, elem_value)? {
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

impl MeasureModel<(SndMotu, FwNode)> for F896hd {
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
