// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {super::*, protocols::focusrite::spro24dsp::*};

#[derive(Default)]
pub struct SPro24DspModel {
    req: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    ctl: CommonCtl,
    tcd22xx_ctl: SPro24DspTcd22xxCtl,
    out_grp_ctl: OutGroupCtl,
    input_ctl: InputCtl,
    effect_ctl: EffectCtl,
}

const TIMEOUT_MS: u32 = 20;

impl SPro24DspModel {
    pub fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.sections =
            GeneralProtocol::read_general_sections(&mut self.req, &mut unit.1, TIMEOUT_MS)?;

        Ok(())
    }
}

impl CtlModel<(SndDice, FwNode)> for SPro24DspModel {
    fn load(
        &mut self,
        unit: &mut (SndDice, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        let caps = GlobalSectionProtocol::read_clock_caps(
            &mut self.req,
            &mut unit.1,
            &self.sections,
            TIMEOUT_MS,
        )?;
        let src_labels = GlobalSectionProtocol::read_clock_source_labels(
            &mut self.req,
            &mut unit.1,
            &self.sections,
            TIMEOUT_MS,
        )?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.extension_sections =
            ProtocolExtension::read_extension_sections(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.tcd22xx_ctl.load(
            unit,
            &mut self.req,
            &self.extension_sections,
            &caps,
            &src_labels,
            TIMEOUT_MS,
            card_cntr,
        )?;

        self.tcd22xx_ctl.cache(
            unit,
            &mut self.req,
            &self.sections,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;

        self.out_grp_ctl
            .load(
                card_cntr,
                unit,
                &mut self.req,
                &self.extension_sections,
                TIMEOUT_MS,
            )
            .map(|mut elem_id_list| self.out_grp_ctl.1.append(&mut elem_id_list))?;

        self.input_ctl.load(card_cntr)?;

        self.effect_ctl.load(
            card_cntr,
            unit,
            &mut self.req,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;

        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.ctl.read(
            unit,
            &mut self.req,
            &self.sections,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.tcd22xx_ctl.read(
            unit,
            &mut self.req,
            &self.extension_sections,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.out_grp_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read(
            unit,
            &mut self.req,
            &self.extension_sections,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.effect_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.ctl.write(
            unit,
            &mut self.req,
            &self.sections,
            elem_id,
            old,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.tcd22xx_ctl.write(
            unit,
            &mut self.req,
            &self.extension_sections,
            elem_id,
            old,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.out_grp_ctl.write(
            unit,
            &mut self.req,
            &self.extension_sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.input_ctl.write(
            unit,
            &mut self.req,
            &self.extension_sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.effect_ctl.write(
            unit,
            &mut self.req,
            &self.extension_sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for SPro24DspModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        self.tcd22xx_ctl.get_notified_elem_list(elem_id_list);
        elem_id_list.extend_from_slice(&self.out_grp_ctl.1);
    }

    fn parse_notification(&mut self, unit: &mut (SndDice, FwNode), msg: &u32) -> Result<(), Error> {
        self.ctl
            .parse_notification(unit, &mut self.req, &self.sections, *msg, TIMEOUT_MS)?;
        self.tcd22xx_ctl.parse_notification(
            unit,
            &mut self.req,
            &self.sections,
            &self.extension_sections,
            TIMEOUT_MS,
            *msg,
        )?;
        self.out_grp_ctl.parse_notification(
            unit,
            &mut self.req,
            &self.extension_sections,
            *msg,
            TIMEOUT_MS,
        )?;
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_grp_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndDice, FwNode)> for SPro24DspModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        self.tcd22xx_ctl.get_measured_elem_list(elem_id_list);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.ctl
            .measure_states(unit, &mut self.req, &self.sections, TIMEOUT_MS)?;
        self.tcd22xx_ctl.measure_states(
            unit,
            &mut self.req,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;
        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default)]
struct SPro24DspTcd22xxCtl(Tcd22xxCtl);

impl Tcd22xxCtlOperation<SPro24DspProtocol> for SPro24DspTcd22xxCtl {
    fn tcd22xx_ctl(&self) -> &Tcd22xxCtl {
        &self.0
    }

    fn tcd22xx_ctl_mut(&mut self) -> &mut Tcd22xxCtl {
        &mut self.0
    }
}

#[derive(Default)]
struct OutGroupCtl(OutGroupState, Vec<ElemId>);

impl OutGroupCtlOperation<SPro24DspProtocol> for OutGroupCtl {
    fn state(&self) -> &OutGroupState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut OutGroupState {
        &mut self.0
    }
}

#[derive(Default)]
struct InputCtl;

impl SaffireproInputCtlOperation<SPro24DspProtocol> for InputCtl {}

#[derive(Default)]
struct EffectCtl(Spro24DspEffectState, Vec<ElemId>);

const CH_STRIP_ORDER_NAME: &str = "ch-strip-order";
const COMPRESSOR_ENABLE_NAME: &str = "compressor-enable";
const EQUALIZER_ENABLE_NAME: &str = "equalizer-enable";

const COMPRESSOR_OUTPUT_NAME: &str = "compressor-output-volume";
const COMPRESSOR_THRESHOLD_NAME: &str = "compressor-threshold";
const COMPRESSOR_RATIO_NAME: &str = "compressor-ratio";
const COMPRESSOR_ATTACK_NAME: &str = "compressor-attack";
const COMPRESSOR_RELEASE_NAME: &str = "compressor-release";

const EQUALIZER_OUTPUT_NAME: &str = "equalizer-output-volume";

const REVERB_SIZE_NAME: &str = "reverb-size";
const REVERB_AIR_NAME: &str = "reverb-air";
const REVERB_ENABLE_NAME: &str = "reverb-enable";
const REVERB_PRE_FILTER_NAME: &str = "reverb-pre-filter";

impl EffectCtl {
    const CH_STRIP_ORDERS: [&'static str; 2] =
        ["compressor-after-equalizer", "equalizer-after-compressor"];
    const F32_CONVERT_SCALE: f32 = 1000000.0;

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        SPro24DspProtocol::read_effect_state(req, &mut unit.1, sections, &mut self.0, timeout_ms)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, CH_STRIP_ORDER_NAME, 0);
        let _ = card_cntr.add_enum_elems(
            &elem_id,
            1,
            self.0.eq_after_comp.len(),
            &Self::CH_STRIP_ORDERS,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, COMPRESSOR_ENABLE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, self.0.comp_enable.len(), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, EQUALIZER_ENABLE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, self.0.eq_enable.len(), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, COMPRESSOR_OUTPUT_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            (SPro24DspProtocol::COMPRESSOR_OUTPUT_MIN * Self::F32_CONVERT_SCALE) as i32,
            (SPro24DspProtocol::COMPRESSOR_OUTPUT_MAX * Self::F32_CONVERT_SCALE) as i32,
            1,
            self.0.comp.output.len(),
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, COMPRESSOR_THRESHOLD_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            (SPro24DspProtocol::COMPRESSOR_THRESHOLD_MIN * Self::F32_CONVERT_SCALE) as i32,
            (SPro24DspProtocol::COMPRESSOR_THRESHOLD_MAX * Self::F32_CONVERT_SCALE) as i32,
            1,
            self.0.comp.threshold.len(),
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, COMPRESSOR_RATIO_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            (SPro24DspProtocol::COMPRESSOR_RATIO_MIN * Self::F32_CONVERT_SCALE) as i32,
            (SPro24DspProtocol::COMPRESSOR_RATIO_MAX * Self::F32_CONVERT_SCALE) as i32,
            1,
            self.0.comp.ratio.len(),
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, COMPRESSOR_ATTACK_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            (SPro24DspProtocol::COMPRESSOR_ATTACK_MIN * Self::F32_CONVERT_SCALE) as i32,
            (SPro24DspProtocol::COMPRESSOR_ATTACK_MAX * Self::F32_CONVERT_SCALE) as i32,
            1,
            self.0.comp.attack.len(),
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, COMPRESSOR_RELEASE_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            (SPro24DspProtocol::COMPRESSOR_RELEASE_MIN * Self::F32_CONVERT_SCALE) as i32,
            (SPro24DspProtocol::COMPRESSOR_RELEASE_MAX * Self::F32_CONVERT_SCALE) as i32,
            1,
            self.0.comp.release.len(),
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, EQUALIZER_OUTPUT_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            (SPro24DspProtocol::EQUALIZER_OUTPUT_MIN * Self::F32_CONVERT_SCALE) as i32,
            (SPro24DspProtocol::EQUALIZER_OUTPUT_MAX * Self::F32_CONVERT_SCALE) as i32,
            1,
            self.0.eq.output.len(),
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_SIZE_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            (SPro24DspProtocol::REVERB_SIZE_MIN * Self::F32_CONVERT_SCALE) as i32,
            (SPro24DspProtocol::REVERB_SIZE_MAX * Self::F32_CONVERT_SCALE) as i32,
            1,
            1,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_AIR_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            (SPro24DspProtocol::REVERB_AIR_MIN * Self::F32_CONVERT_SCALE) as i32,
            (SPro24DspProtocol::REVERB_AIR_MAX * Self::F32_CONVERT_SCALE) as i32,
            1,
            1,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_ENABLE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_PRE_FILTER_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            (SPro24DspProtocol::REVERB_PRE_FILTER_MIN * Self::F32_CONVERT_SCALE) as i32,
            (SPro24DspProtocol::REVERB_PRE_FILTER_MAX * Self::F32_CONVERT_SCALE) as i32,
            1,
            1,
            None,
            true,
        )?;

        Ok(())
    }

    fn convert_from_f32_array(elem_value: &mut ElemValue, raw: &[f32]) {
        let vals: Vec<i32> = raw
            .iter()
            .map(|&r| (r * Self::F32_CONVERT_SCALE) as i32)
            .collect();
        elem_value.set_int(&vals);
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CH_STRIP_ORDER_NAME => {
                let vals: Vec<u32> = self
                    .0
                    .eq_after_comp
                    .iter()
                    .map(|enable| *enable as u32)
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            COMPRESSOR_ENABLE_NAME => {
                elem_value.set_bool(&self.0.comp_enable);
                Ok(true)
            }
            EQUALIZER_ENABLE_NAME => {
                elem_value.set_bool(&self.0.eq_enable);
                Ok(true)
            }
            COMPRESSOR_OUTPUT_NAME => {
                Self::convert_from_f32_array(elem_value, &self.0.comp.output);
                Ok(true)
            }
            COMPRESSOR_THRESHOLD_NAME => {
                Self::convert_from_f32_array(elem_value, &self.0.comp.threshold);
                Ok(true)
            }
            COMPRESSOR_RATIO_NAME => {
                Self::convert_from_f32_array(elem_value, &self.0.comp.ratio);
                Ok(true)
            }
            COMPRESSOR_ATTACK_NAME => {
                Self::convert_from_f32_array(elem_value, &self.0.comp.attack);
                Ok(true)
            }
            COMPRESSOR_RELEASE_NAME => {
                Self::convert_from_f32_array(elem_value, &self.0.comp.release);
                Ok(true)
            }
            EQUALIZER_OUTPUT_NAME => {
                Self::convert_from_f32_array(elem_value, &self.0.eq.output);
                Ok(true)
            }
            REVERB_SIZE_NAME => {
                Self::convert_from_f32_array(elem_value, &[self.0.reverb.size]);
                Ok(true)
            }
            REVERB_AIR_NAME => {
                Self::convert_from_f32_array(elem_value, &[self.0.reverb.air]);
                Ok(true)
            }
            REVERB_ENABLE_NAME => {
                elem_value.set_bool(&[self.0.reverb.enabled]);
                Ok(true)
            }
            REVERB_PRE_FILTER_NAME => {
                Self::convert_from_f32_array(elem_value, &[self.0.reverb.pre_filter]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn convert_to_f32_array(elem_value: &ElemValue, raw: &mut [f32]) {
        let vals = &elem_value.int()[..raw.len()];
        raw.iter_mut()
            .zip(vals)
            .for_each(|(r, val)| *r = (*val as f32) / Self::F32_CONVERT_SCALE);
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CH_STRIP_ORDER_NAME => {
                let vals = &elem_value.enumerated()[..self.0.eq_after_comp.len()];
                let eq_after_comp: Vec<bool> = vals.iter().map(|&val| val > 0).collect();
                SPro24DspProtocol::write_eq_after_comp(
                    req,
                    &mut unit.1,
                    sections,
                    &eq_after_comp,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            COMPRESSOR_ENABLE_NAME => {
                let vals = &elem_value.boolean()[..self.0.comp_enable.len()];
                SPro24DspProtocol::write_comp_enable(
                    req,
                    &mut unit.1,
                    sections,
                    &vals,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            EQUALIZER_ENABLE_NAME => {
                let vals = &elem_value.boolean()[..self.0.eq_enable.len()];
                SPro24DspProtocol::write_eq_enable(
                    req,
                    &mut unit.1,
                    sections,
                    &vals,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            COMPRESSOR_OUTPUT_NAME => {
                let mut comp = self.0.comp.clone();
                Self::convert_to_f32_array(elem_value, &mut comp.output);
                SPro24DspProtocol::write_comp_effect(
                    req,
                    &mut unit.1,
                    sections,
                    &comp,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            COMPRESSOR_THRESHOLD_NAME => {
                let mut comp = self.0.comp.clone();
                Self::convert_to_f32_array(elem_value, &mut comp.threshold);
                SPro24DspProtocol::write_comp_effect(
                    req,
                    &mut unit.1,
                    sections,
                    &comp,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            COMPRESSOR_RATIO_NAME => {
                let mut comp = self.0.comp.clone();
                Self::convert_to_f32_array(elem_value, &mut comp.ratio);
                SPro24DspProtocol::write_comp_effect(
                    req,
                    &mut unit.1,
                    sections,
                    &comp,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            COMPRESSOR_ATTACK_NAME => {
                let mut comp = self.0.comp.clone();
                Self::convert_to_f32_array(elem_value, &mut comp.attack);
                SPro24DspProtocol::write_comp_effect(
                    req,
                    &mut unit.1,
                    sections,
                    &comp,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            COMPRESSOR_RELEASE_NAME => {
                let mut comp = self.0.comp.clone();
                Self::convert_to_f32_array(elem_value, &mut comp.release);
                SPro24DspProtocol::write_comp_effect(
                    req,
                    &mut unit.1,
                    sections,
                    &comp,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            EQUALIZER_OUTPUT_NAME => {
                let mut eq = self.0.eq.clone();
                Self::convert_to_f32_array(elem_value, &mut eq.output);
                SPro24DspProtocol::write_eq_effect(
                    req,
                    &mut unit.1,
                    sections,
                    &eq,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            REVERB_SIZE_NAME => {
                let mut vals = [0.0];
                Self::convert_to_f32_array(elem_value, &mut vals);
                let mut reverb = self.0.reverb.clone();
                reverb.size = vals[0];
                SPro24DspProtocol::write_reverb_effect(
                    req,
                    &mut unit.1,
                    sections,
                    &reverb,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            REVERB_AIR_NAME => {
                let mut vals = [0.0];
                Self::convert_to_f32_array(elem_value, &mut vals);
                let mut reverb = self.0.reverb.clone();
                reverb.air = vals[0];
                SPro24DspProtocol::write_reverb_effect(
                    req,
                    &mut unit.1,
                    sections,
                    &reverb,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            REVERB_ENABLE_NAME => {
                let val = elem_value.boolean()[0];
                let mut reverb = self.0.reverb.clone();
                reverb.enabled = val;
                SPro24DspProtocol::write_reverb_effect(
                    req,
                    &mut unit.1,
                    sections,
                    &reverb,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            REVERB_PRE_FILTER_NAME => {
                let mut vals = [0.0];
                Self::convert_to_f32_array(elem_value, &mut vals);
                let mut reverb = self.0.reverb.clone();
                reverb.pre_filter = vals[0];
                SPro24DspProtocol::write_reverb_effect(
                    req,
                    &mut unit.1,
                    sections,
                    &reverb,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
