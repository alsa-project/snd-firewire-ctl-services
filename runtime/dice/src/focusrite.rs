// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub mod liquids56_model;
pub mod spro14_model;
pub mod spro24_model;
pub mod spro24dsp_model;
pub mod spro26_model;
pub mod spro40_model;
pub mod spro40d3_model;

use {
    super::{tcd22xx_ctl::*, *},
    protocols::{focusrite::*, tcat::extension::*},
    std::marker::PhantomData,
};

const VOL_NAME: &str = "output-group-volume";
const VOL_TARGET_NAME: &str = "output-group-volume-target";
const VOL_MUTE_NAME: &str = "output-group-volume-mute";
const MUTE_NAME: &str = "output-group-mute";
const DIM_NAME: &str = "output-group-dim";
const DIM_TARGET_NAME: &str = "output-group-dim-target";
const MUTE_TARGET_NAME: &str = "output-group-mute-target";

#[derive(Debug)]
pub struct OutGroupCtl<T>(OutGroupState, Vec<ElemId>, PhantomData<T>)
where
    T: SaffireproOutGroupSpecification
        + TcatExtensionSectionParamsOperation<OutGroupState>
        + TcatExtensionSectionPartialMutableParamsOperation<OutGroupState>
        + TcatExtensionSectionNotifiedParamsOperation<OutGroupState>;

impl<T> Default for OutGroupCtl<T>
where
    T: SaffireproOutGroupSpecification
        + TcatExtensionSectionParamsOperation<OutGroupState>
        + TcatExtensionSectionPartialMutableParamsOperation<OutGroupState>
        + TcatExtensionSectionNotifiedParamsOperation<OutGroupState>,
{
    fn default() -> Self {
        Self(
            T::create_out_group_state(),
            Default::default(),
            Default::default(),
        )
    }
}

impl<T> OutGroupCtl<T>
where
    T: SaffireproOutGroupSpecification
        + TcatExtensionSectionParamsOperation<OutGroupState>
        + TcatExtensionSectionPartialMutableParamsOperation<OutGroupState>
        + TcatExtensionSectionNotifiedParamsOperation<OutGroupState>,
{
    const LEVEL_MIN: i32 = T::VOL_MIN as i32;
    const LEVEL_MAX: i32 = T::VOL_MAX as i32;
    const LEVEL_STEP: i32 = 0x01;

    fn cache(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res =
            T::cache_extension_whole_params(req, node, sections, caps, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, DIM_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, VOL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::LEVEL_MIN,
                Self::LEVEL_MAX,
                Self::LEVEL_STEP,
                T::ENTRY_COUNT,
                None,
                true,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, VOL_MUTE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, T::ENTRY_COUNT, true)?;

        if T::HAS_VOL_HWCTL {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, VOL_TARGET_NAME, 0);
            card_cntr.add_bool_elems(&elem_id, 1, T::ENTRY_COUNT, true)?;
        }

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, DIM_TARGET_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, T::ENTRY_COUNT, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MUTE_TARGET_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, T::ENTRY_COUNT, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            VOL_NAME => {
                let params = &self.0;
                let vals: Vec<i32> = params.vols.iter().map(|&vol| vol as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            VOL_MUTE_NAME => {
                let params = &self.0;
                elem_value.set_bool(&params.vol_mutes);
                Ok(true)
            }
            VOL_TARGET_NAME => {
                let params = &self.0;
                elem_value.set_bool(&params.vol_hwctls);
                Ok(true)
            }
            MUTE_NAME => {
                let params = &self.0;
                elem_value.set_bool(&[params.mute_enabled]);
                Ok(true)
            }
            MUTE_TARGET_NAME => {
                let params = &self.0;
                elem_value.set_bool(&params.mute_hwctls);
                Ok(true)
            }
            DIM_NAME => {
                let params = &self.0;
                elem_value.set_bool(&[params.dim_enabled]);
                Ok(true)
            }
            DIM_TARGET_NAME => {
                let params = &self.0;
                elem_value.set_bool(&params.dim_hwctls);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            VOL_NAME => {
                let mut params = self.0.clone();
                params
                    .vols
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(vol, &val)| *vol = val as i8);
                let res = T::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            VOL_MUTE_NAME => {
                let mut params = self.0.clone();
                params
                    .vol_mutes
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(vol_mute, val)| *vol_mute = val);
                let res = T::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            VOL_TARGET_NAME => {
                let mut params = self.0.clone();
                params
                    .vol_hwctls
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(vol_hwctl, val)| *vol_hwctl = val);
                let res = T::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            MUTE_NAME => {
                let mut params = self.0.clone();
                params.mute_enabled = elem_value.boolean()[0];
                let res = T::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            MUTE_TARGET_NAME => {
                let mut params = self.0.clone();
                params
                    .mute_hwctls
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(mute_hwctl, val)| *mute_hwctl = val);
                let res = T::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            DIM_NAME => {
                let mut params = self.0.clone();
                params.dim_enabled = elem_value.boolean()[0];
                let res = T::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            DIM_TARGET_NAME => {
                let mut params = self.0.clone();
                params
                    .dim_hwctls
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(dim_hwctl, val)| *dim_hwctl = val);
                let res = T::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
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

    fn parse_notification(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::cache_extension_notified_params(
            req,
            node,
            sections,
            caps,
            &mut self.0,
            msg,
            timeout_ms,
        );
        debug!(params = ?self.0, ?res);
        res
    }
}

const MIC_INPUT_LEVEL_NAME: &str = "mic-input-level";
const LINE_INPUT_LEVEL_NAME: &str = "line-input-level";

fn mic_input_level_to_str(level: &SaffireproMicInputLevel) -> &'static str {
    match level {
        SaffireproMicInputLevel::Line => "line",
        SaffireproMicInputLevel::Instrument => "instrument",
    }
}

fn line_input_level_to_str(level: &SaffireproLineInputLevel) -> &'static str {
    match level {
        SaffireproLineInputLevel::Low => "low",
        SaffireproLineInputLevel::High => "high",
    }
}

#[derive(Default, Debug)]
pub struct SaffireproInputCtl<T>(SaffireproInputParams, PhantomData<T>)
where
    T: SaffireproInputSpecification
        + TcatExtensionSectionParamsOperation<SaffireproInputParams>
        + TcatExtensionSectionPartialMutableParamsOperation<SaffireproInputParams>;

impl<T> SaffireproInputCtl<T>
where
    T: SaffireproInputSpecification
        + TcatExtensionSectionParamsOperation<SaffireproInputParams>
        + TcatExtensionSectionPartialMutableParamsOperation<SaffireproInputParams>,
{
    const MIC_LEVELS: [SaffireproMicInputLevel; 2] = [
        SaffireproMicInputLevel::Line,
        SaffireproMicInputLevel::Instrument,
    ];

    const LINE_LEVELS: [SaffireproLineInputLevel; 2] = [
        SaffireproLineInputLevel::Low,
        SaffireproLineInputLevel::High,
    ];

    fn cache(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res =
            T::cache_extension_whole_params(req, node, sections, caps, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::MIC_LEVELS
            .iter()
            .map(|l| mic_input_level_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIC_INPUT_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, T::MIC_INPUT_COUNT, &labels, None, true)?;

        let labels: Vec<&str> = Self::LINE_LEVELS
            .iter()
            .map(|l| line_input_level_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, LINE_INPUT_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, T::LINE_INPUT_COUNT, &labels, None, true)?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIC_INPUT_LEVEL_NAME => {
                let vals: Vec<u32> = self
                    .0
                    .mic_levels
                    .iter()
                    .map(|level| {
                        let pos = Self::MIC_LEVELS.iter().position(|l| level.eq(l)).unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            LINE_INPUT_LEVEL_NAME => {
                let vals: Vec<u32> = self
                    .0
                    .line_levels
                    .iter()
                    .map(|level| {
                        let pos = Self::LINE_LEVELS.iter().position(|l| level.eq(l)).unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIC_INPUT_LEVEL_NAME => {
                let mut params = self.0.clone();
                params
                    .mic_levels
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(level, &val)| {
                        let pos = val as usize;
                        Self::MIC_LEVELS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg = format!("Invalid index for mic input levels: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&l| *level = l)
                    })?;
                let res = T::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            LINE_INPUT_LEVEL_NAME => {
                let mut params = self.0.clone();
                params
                    .line_levels
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(level, &val)| {
                        let pos = val as usize;
                        Self::LINE_LEVELS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg = format!("Invalid index for line input levels: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&l| *level = l)
                    })?;
                let res = T::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
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
pub struct IoParamsCtl<T>(SaffireproIoParams, PhantomData<T>)
where
    T: SaffireproIoParamsSpecification
        + TcatExtensionSectionParamsOperation<SaffireproIoParams>
        + TcatExtensionSectionPartialMutableParamsOperation<SaffireproIoParams>;

fn optical_out_iface_mode_to_str(mode: &OpticalOutIfaceMode) -> &'static str {
    match mode {
        OpticalOutIfaceMode::Adat => "ADAT",
        OpticalOutIfaceMode::Spdif => "S/PDIF",
        OpticalOutIfaceMode::AesEbu => "AES/EBU",
    }
}

const ANALOG_OUT_0_1_PAD_NAME: &str = "analog-output-1/2-pad";
const OPTICAL_OUT_IFACE_MODE_NAME: &str = "optical-output-interface-mode";
const MIC_AMP_TRANSFORMER_NAME: &str = "mic-amp-transformer";

impl<T> IoParamsCtl<T>
where
    T: SaffireproIoParamsSpecification
        + TcatExtensionSectionParamsOperation<SaffireproIoParams>
        + TcatExtensionSectionPartialMutableParamsOperation<SaffireproIoParams>,
{
    const OPTICAL_OUT_IFACE_MODES: [OpticalOutIfaceMode; 3] = [
        OpticalOutIfaceMode::Adat,
        OpticalOutIfaceMode::Spdif,
        OpticalOutIfaceMode::AesEbu,
    ];

    fn cache(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res =
            T::cache_extension_whole_params(req, node, sections, caps, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, ANALOG_OUT_0_1_PAD_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<&str> = Self::OPTICAL_OUT_IFACE_MODES
            .iter()
            .take(if T::AESEBU_IS_SUPPORTED { 3 } else { 2 })
            .map(|mode| optical_out_iface_mode_to_str(mode))
            .collect();
        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OPTICAL_OUT_IFACE_MODE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MIC_AMP_TRANSFORMER_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 2, true)?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ANALOG_OUT_0_1_PAD_NAME => {
                let params = &self.0;
                elem_value.set_bool(&[params.analog_out_0_1_pad]);
                Ok(true)
            }
            OPTICAL_OUT_IFACE_MODE_NAME => {
                let params = &self.0;
                let pos = Self::OPTICAL_OUT_IFACE_MODES
                    .iter()
                    .position(|m| params.opt_out_iface_mode.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            MIC_AMP_TRANSFORMER_NAME => {
                let params = &self.0;
                elem_value.set_bool(&params.mic_amp_transformers);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ANALOG_OUT_0_1_PAD_NAME => {
                let mut params = self.0.clone();
                params.analog_out_0_1_pad = elem_value.boolean()[0];
                let res = T::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            OPTICAL_OUT_IFACE_MODE_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::OPTICAL_OUT_IFACE_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg =
                            format!("Invalid index of optical output interface mode: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&mode| params.opt_out_iface_mode = mode)?;
                let res = T::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            MIC_AMP_TRANSFORMER_NAME => {
                let mut params = self.0.clone();
                params
                    .mic_amp_transformers
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(transformer, val)| *transformer = val);
                let res = T::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
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
