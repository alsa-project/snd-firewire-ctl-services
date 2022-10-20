// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {super::*, protocols::focusrite::spro24dsp::*};

#[derive(Default)]
pub struct SPro24DspModel {
    req: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    common_ctl: CommonCtl,
    tcd22xx_ctl: SPro24DspTcd22xxCtl,
    out_grp_ctl: OutGroupCtl<SPro24DspProtocol>,
    input_ctl: SaffireproInputCtl<SPro24DspProtocol>,
    effect_ctl: EffectGeneralCtl,
    comp_ctl: CompressorCtl,
    eq_ctl: EqualizerCtl,
    reverb_ctl: ReverbCtl,
}

const TIMEOUT_MS: u32 = 20;

impl SPro24DspModel {
    pub fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        SPro24DspProtocol::read_general_sections(
            &self.req,
            &unit.1,
            &mut self.sections,
            TIMEOUT_MS,
        )?;

        self.common_ctl
            .whole_cache(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        Ok(())
    }
}

impl CtlModel<(SndDice, FwNode)> for SPro24DspModel {
    fn load(
        &mut self,
        unit: &mut (SndDice, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.common_ctl.load(card_cntr, &self.sections).map(
            |(measured_elem_id_list, notified_elem_id_list)| {
                self.common_ctl.0 = measured_elem_id_list;
                self.common_ctl.1 = notified_elem_id_list;
            },
        )?;

        self.extension_sections =
            ProtocolExtension::read_extension_sections(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.tcd22xx_ctl.load(
            unit,
            &mut self.req,
            &self.extension_sections,
            &self.sections.global.params.avail_rates,
            &self.sections.global.params.avail_sources,
            &self.sections.global.params.clock_source_labels,
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

        self.out_grp_ctl.cache(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;

        self.input_ctl.cache(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;

        self.effect_ctl.cache(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;

        self.comp_ctl.cache(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;

        self.eq_ctl.cache(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;

        self.reverb_ctl.cache(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;

        self.out_grp_ctl.load(card_cntr)?;
        self.input_ctl.load(card_cntr)?;
        self.effect_ctl.load(card_cntr)?;
        self.comp_ctl.load(card_cntr)?;
        self.eq_ctl.load(card_cntr)?;
        self.reverb_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(&self.sections, elem_id, elem_value)? {
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
        } else if self.input_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.effect_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.comp_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.eq_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read(elem_id, elem_value)? {
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
        if self.common_ctl.write(
            &unit.0,
            &self.req,
            &unit.1,
            &mut self.sections,
            elem_id,
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
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.input_ctl.write(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.effect_ctl.write(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.comp_ctl.write(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.eq_ctl.write(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.reverb_ctl.write(
            &mut self.req,
            &mut unit.1,
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
        elem_id_list.extend_from_slice(&self.common_ctl.1);
        self.tcd22xx_ctl.get_notified_elem_list(elem_id_list);
        elem_id_list.extend_from_slice(&self.out_grp_ctl.1);
    }

    fn parse_notification(&mut self, unit: &mut (SndDice, FwNode), msg: &u32) -> Result<(), Error> {
        self.common_ctl.parse_notification(
            &self.req,
            &unit.1,
            &mut self.sections,
            *msg,
            TIMEOUT_MS,
        )?;
        self.tcd22xx_ctl.parse_notification(
            unit,
            &mut self.req,
            &self.sections,
            &self.extension_sections,
            TIMEOUT_MS,
            *msg,
        )?;
        self.out_grp_ctl.parse_notification(
            &mut self.req,
            &mut unit.1,
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
        if self.common_ctl.read(&self.sections, elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_grp_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndDice, FwNode)> for SPro24DspModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.0);
        self.tcd22xx_ctl.get_measured_elem_list(elem_id_list);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .measure(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;
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
        if self.common_ctl.read(&self.sections, elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Debug)]
struct CommonCtl(Vec<ElemId>, Vec<ElemId>);

impl CommonCtlOperation<SPro24DspProtocol> for CommonCtl {}

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

const F32_CONVERT_SCALE: f32 = 1000000.0;

fn convert_from_f32_array(elem_value: &mut ElemValue, raw: &[f32]) {
    let vals: Vec<i32> = raw
        .iter()
        .map(|&r| (r * F32_CONVERT_SCALE) as i32)
        .collect();
    elem_value.set_int(&vals);
}

fn convert_to_f32_array(elem_value: &ElemValue, raw: &mut [f32]) {
    let vals = &elem_value.int()[..raw.len()];
    raw.iter_mut()
        .zip(vals)
        .for_each(|(r, val)| *r = (*val as f32) / F32_CONVERT_SCALE);
}

#[derive(Default, Debug)]
struct CompressorCtl(Spro24DspCompressorState);

const COMPRESSOR_OUTPUT_NAME: &str = "compressor-output-volume";
const COMPRESSOR_THRESHOLD_NAME: &str = "compressor-threshold";
const COMPRESSOR_RATIO_NAME: &str = "compressor-ratio";
const COMPRESSOR_ATTACK_NAME: &str = "compressor-attack";
const COMPRESSOR_RELEASE_NAME: &str = "compressor-release";

impl CompressorCtl {
    fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = SPro24DspProtocol::cache_whole_comp_params(
            req,
            node,
            sections,
            &mut self.0,
            timeout_ms,
        );
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, COMPRESSOR_OUTPUT_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            (SPro24DspProtocol::COMPRESSOR_OUTPUT_MIN * F32_CONVERT_SCALE) as i32,
            (SPro24DspProtocol::COMPRESSOR_OUTPUT_MAX * F32_CONVERT_SCALE) as i32,
            1,
            self.0.output.len(),
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, COMPRESSOR_THRESHOLD_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            (SPro24DspProtocol::COMPRESSOR_THRESHOLD_MIN * F32_CONVERT_SCALE) as i32,
            (SPro24DspProtocol::COMPRESSOR_THRESHOLD_MAX * F32_CONVERT_SCALE) as i32,
            1,
            self.0.threshold.len(),
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, COMPRESSOR_RATIO_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            (SPro24DspProtocol::COMPRESSOR_RATIO_MIN * F32_CONVERT_SCALE) as i32,
            (SPro24DspProtocol::COMPRESSOR_RATIO_MAX * F32_CONVERT_SCALE) as i32,
            1,
            self.0.ratio.len(),
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, COMPRESSOR_ATTACK_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            (SPro24DspProtocol::COMPRESSOR_ATTACK_MIN * F32_CONVERT_SCALE) as i32,
            (SPro24DspProtocol::COMPRESSOR_ATTACK_MAX * F32_CONVERT_SCALE) as i32,
            1,
            self.0.attack.len(),
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, COMPRESSOR_RELEASE_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            (SPro24DspProtocol::COMPRESSOR_RELEASE_MIN * F32_CONVERT_SCALE) as i32,
            (SPro24DspProtocol::COMPRESSOR_RELEASE_MAX * F32_CONVERT_SCALE) as i32,
            1,
            self.0.release.len(),
            None,
            true,
        )?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            COMPRESSOR_OUTPUT_NAME => {
                convert_from_f32_array(elem_value, &self.0.output);
                Ok(true)
            }
            COMPRESSOR_THRESHOLD_NAME => {
                convert_from_f32_array(elem_value, &self.0.threshold);
                Ok(true)
            }
            COMPRESSOR_RATIO_NAME => {
                convert_from_f32_array(elem_value, &self.0.ratio);
                Ok(true)
            }
            COMPRESSOR_ATTACK_NAME => {
                convert_from_f32_array(elem_value, &self.0.attack);
                Ok(true)
            }
            COMPRESSOR_RELEASE_NAME => {
                convert_from_f32_array(elem_value, &self.0.release);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            COMPRESSOR_OUTPUT_NAME => {
                let mut params = self.0.clone();
                convert_to_f32_array(elem_value, &mut params.output);
                let res = SPro24DspProtocol::update_partial_comp_params(
                    req,
                    node,
                    sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            COMPRESSOR_THRESHOLD_NAME => {
                let mut params = self.0.clone();
                convert_to_f32_array(elem_value, &mut params.threshold);
                let res = SPro24DspProtocol::update_partial_comp_params(
                    req,
                    node,
                    sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            COMPRESSOR_RATIO_NAME => {
                let mut params = self.0.clone();
                convert_to_f32_array(elem_value, &mut params.ratio);
                let res = SPro24DspProtocol::update_partial_comp_params(
                    req,
                    node,
                    sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            COMPRESSOR_ATTACK_NAME => {
                let mut params = self.0.clone();
                convert_to_f32_array(elem_value, &mut params.attack);
                let res = SPro24DspProtocol::update_partial_comp_params(
                    req,
                    node,
                    sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            COMPRESSOR_RELEASE_NAME => {
                let mut params = self.0.clone();
                convert_to_f32_array(elem_value, &mut params.release);
                let res = SPro24DspProtocol::update_partial_comp_params(
                    req,
                    node,
                    sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct EqualizerCtl(Spro24DspEqualizerState);

const EQUALIZER_OUTPUT_NAME: &str = "equalizer-output-volume";

impl EqualizerCtl {
    fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res =
            SPro24DspProtocol::cache_whole_eq_params(req, node, sections, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, EQUALIZER_OUTPUT_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            (SPro24DspProtocol::EQUALIZER_OUTPUT_MIN * F32_CONVERT_SCALE) as i32,
            (SPro24DspProtocol::EQUALIZER_OUTPUT_MAX * F32_CONVERT_SCALE) as i32,
            1,
            self.0.output.len(),
            None,
            true,
        )?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            EQUALIZER_OUTPUT_NAME => {
                let params = &self.0;
                convert_from_f32_array(elem_value, &params.output);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            EQUALIZER_OUTPUT_NAME => {
                let mut params = self.0.clone();
                convert_to_f32_array(elem_value, &mut params.output);
                let res = SPro24DspProtocol::update_partial_eq_params(
                    req,
                    node,
                    sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct ReverbCtl(Spro24DspReverbState);

const REVERB_SIZE_NAME: &str = "reverb-size";
const REVERB_AIR_NAME: &str = "reverb-air";
const REVERB_ENABLE_NAME: &str = "reverb-enable";
const REVERB_PRE_FILTER_NAME: &str = "reverb-pre-filter";

impl ReverbCtl {
    fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = SPro24DspProtocol::cache_whole_reverb_params(
            req,
            node,
            sections,
            &mut self.0,
            timeout_ms,
        );
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_SIZE_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            (SPro24DspProtocol::REVERB_SIZE_MIN * F32_CONVERT_SCALE) as i32,
            (SPro24DspProtocol::REVERB_SIZE_MAX * F32_CONVERT_SCALE) as i32,
            1,
            1,
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_AIR_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            (SPro24DspProtocol::REVERB_AIR_MIN * F32_CONVERT_SCALE) as i32,
            (SPro24DspProtocol::REVERB_AIR_MAX * F32_CONVERT_SCALE) as i32,
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
            (SPro24DspProtocol::REVERB_PRE_FILTER_MIN * F32_CONVERT_SCALE) as i32,
            (SPro24DspProtocol::REVERB_PRE_FILTER_MAX * F32_CONVERT_SCALE) as i32,
            1,
            1,
            None,
            true,
        )?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            REVERB_SIZE_NAME => {
                let params = &self.0;
                convert_from_f32_array(elem_value, &[params.size]);
                Ok(true)
            }
            REVERB_AIR_NAME => {
                let params = &self.0;
                convert_from_f32_array(elem_value, &[params.air]);
                Ok(true)
            }
            REVERB_ENABLE_NAME => {
                let params = &self.0;
                elem_value.set_bool(&[params.enabled]);
                Ok(true)
            }
            REVERB_PRE_FILTER_NAME => {
                let params = &self.0;
                convert_from_f32_array(elem_value, &[params.pre_filter]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            REVERB_SIZE_NAME => {
                let mut vals = [0.0];
                convert_to_f32_array(elem_value, &mut vals);
                let mut params = self.0.clone();
                params.size = vals[0];
                let res = SPro24DspProtocol::update_partial_reverb_params(
                    req,
                    node,
                    sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            REVERB_AIR_NAME => {
                let mut vals = [0.0];
                convert_to_f32_array(elem_value, &mut vals);
                let mut params = self.0.clone();
                params.air = vals[0];
                let res = SPro24DspProtocol::update_partial_reverb_params(
                    req,
                    node,
                    sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            REVERB_ENABLE_NAME => {
                let val = elem_value.boolean()[0];
                let mut params = self.0.clone();
                params.enabled = val;
                let res = SPro24DspProtocol::update_partial_reverb_params(
                    req,
                    node,
                    sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            REVERB_PRE_FILTER_NAME => {
                let mut vals = [0.0];
                convert_to_f32_array(elem_value, &mut vals);
                let mut params = self.0.clone();
                params.pre_filter = vals[0];
                let res = SPro24DspProtocol::update_partial_reverb_params(
                    req,
                    node,
                    sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct EffectGeneralCtl(Spro24DspEffectGeneralParams);

const CH_STRIP_ORDER_NAME: &str = "ch-strip-order";
const COMPRESSOR_ENABLE_NAME: &str = "compressor-enable";
const EQUALIZER_ENABLE_NAME: &str = "equalizer-enable";

impl EffectGeneralCtl {
    const CH_STRIP_ORDERS: [&'static str; 2] =
        ["compressor-after-equalizer", "equalizer-after-compressor"];

    fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res =
            SPro24DspProtocol::cache_effect_general(req, node, sections, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
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

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CH_STRIP_ORDER_NAME => {
                let params = &self.0;
                let vals: Vec<u32> = params
                    .eq_after_comp
                    .iter()
                    .map(|enable| *enable as u32)
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            COMPRESSOR_ENABLE_NAME => {
                let params = &self.0;
                elem_value.set_bool(&params.comp_enable);
                Ok(true)
            }
            EQUALIZER_ENABLE_NAME => {
                let params = &self.0;
                elem_value.set_bool(&params.eq_enable);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CH_STRIP_ORDER_NAME => {
                let mut params = self.0.clone();
                params
                    .eq_after_comp
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .for_each(|(order, &val)| *order = val > 0);
                let res = SPro24DspProtocol::update_effect_general(
                    req,
                    node,
                    sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            COMPRESSOR_ENABLE_NAME => {
                let mut params = self.0.clone();
                params
                    .comp_enable
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(enable, val)| *enable = val);
                let res = SPro24DspProtocol::update_effect_general(
                    req,
                    node,
                    sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            EQUALIZER_ENABLE_NAME => {
                let mut params = self.0.clone();
                params
                    .eq_enable
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(enable, val)| *enable = val);
                let res = SPro24DspProtocol::update_effect_general(
                    req,
                    node,
                    sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
