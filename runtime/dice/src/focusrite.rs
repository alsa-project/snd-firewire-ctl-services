// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub mod liquids56_model;
pub mod spro14_model;
pub mod spro24_model;
pub mod spro24dsp_model;
pub mod spro26_model;
pub mod spro40_model;

use {
    super::{tcd22xx_ctl::*, *},
    protocols::{focusrite::*, tcat::extension::*},
    std::marker::PhantomData,
};

const VOL_NAME: &str = "output-group-volume";
const VOL_HWCTL_NAME: &str = "output-group-volume-hwctl";
const VOL_MUTE_NAME: &str = "output-group-volume-mute";
const MUTE_NAME: &str = "output-group-mute";
const DIM_NAME: &str = "output-group-dim";
const DIM_HWCTL_NAME: &str = "output-group-dim-hwctl";
const MUTE_HWCTL_NAME: &str = "output-group-mute-hwctl";

trait OutGroupCtlOperation<T: SaffireproOutGroupOperation> {
    fn state(&self) -> &OutGroupState;
    fn state_mut(&mut self) -> &mut OutGroupState;

    const LEVEL_MIN: i32 = 0x00;
    const LEVEL_MAX: i32 = 0x7f;
    const LEVEL_STEP: i32 = 0x01;

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        let mut state = T::create_out_group_state();
        T::read_out_group_mute(req, &mut unit.1, sections, &mut state, timeout_ms)?;
        T::read_out_group_dim(req, &mut unit.1, sections, &mut state, timeout_ms)?;
        T::read_out_group_vols(req, &mut unit.1, sections, &mut state, timeout_ms)?;
        T::read_out_group_vol_mute_hwctls(req, &mut unit.1, sections, &mut state, timeout_ms)?;
        T::read_out_group_dim_mute_hwctls(req, &mut unit.1, sections, &mut state, timeout_ms)?;

        *self.state_mut() = state;

        let mut notified_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, DIM_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

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
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, VOL_MUTE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, T::ENTRY_COUNT, true)?;

        if T::HAS_VOL_HWCTL {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, VOL_HWCTL_NAME, 0);
            card_cntr.add_bool_elems(&elem_id, 1, T::ENTRY_COUNT, true)?;
        }

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, DIM_HWCTL_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, T::ENTRY_COUNT, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MUTE_HWCTL_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, T::ENTRY_COUNT, true)?;

        Ok(notified_elem_id_list)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            VOL_MUTE_NAME => {
                elem_value.set_bool(&self.state().vol_mutes);
                Ok(true)
            }
            VOL_HWCTL_NAME => {
                elem_value.set_bool(&self.state().vol_hwctls);
                Ok(true)
            }
            DIM_HWCTL_NAME => {
                elem_value.set_bool(&self.state().dim_hwctls);
                Ok(true)
            }
            MUTE_HWCTL_NAME => {
                elem_value.set_bool(&self.state().mute_hwctls);
                Ok(true)
            }
            _ => self.read_notified_elem(elem_id, elem_value),
        }
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
            MUTE_NAME => ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                T::write_out_group_mute(
                    req,
                    &mut unit.1,
                    sections,
                    self.state_mut(),
                    val,
                    timeout_ms,
                )
            })
            .map(|_| true),
            DIM_NAME => ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                T::write_out_group_dim(
                    req,
                    &mut unit.1,
                    sections,
                    self.state_mut(),
                    val,
                    timeout_ms,
                )
            })
            .map(|_| true),
            VOL_NAME => {
                let vals = &elem_value.int()[..T::ENTRY_COUNT];
                let vols: Vec<i8> = vals.iter().map(|&v| (Self::LEVEL_MAX - v) as i8).collect();
                T::write_out_group_vols(
                    req,
                    &mut unit.1,
                    sections,
                    self.state_mut(),
                    &vols,
                    timeout_ms,
                )
                .map(|_| true)
            }
            VOL_MUTE_NAME => {
                let vol_mutes = &elem_value.boolean()[..T::ENTRY_COUNT];
                let vol_hwctls = self.state().vol_hwctls.clone();
                T::write_out_group_vol_mute_hwctls(
                    req,
                    &mut unit.1,
                    sections,
                    self.state_mut(),
                    &vol_mutes,
                    &vol_hwctls,
                    timeout_ms,
                )
                .map(|_| true)
            }
            VOL_HWCTL_NAME => {
                let vol_hwctls = &elem_value.boolean()[..T::ENTRY_COUNT];
                let vol_mutes = vec![false; T::ENTRY_COUNT];
                T::write_out_group_vol_mute_hwctls(
                    req,
                    &mut unit.1,
                    sections,
                    self.state_mut(),
                    &vol_mutes,
                    &vol_hwctls,
                    timeout_ms,
                )
                .map(|_| true)
            }
            DIM_HWCTL_NAME => {
                let dim_hwctls = &elem_value.boolean()[..T::ENTRY_COUNT];
                let mute_hwctls = self.state().mute_hwctls.clone();
                T::write_out_group_dim_mute_hwctls(
                    req,
                    &mut unit.1,
                    sections,
                    self.state_mut(),
                    &dim_hwctls,
                    &mute_hwctls,
                    timeout_ms,
                )?;
                Ok(true)
            }
            MUTE_HWCTL_NAME => {
                let mute_hwctls = &elem_value.boolean()[..T::ENTRY_COUNT];
                let dim_hwctls = self.state().dim_hwctls.clone();
                T::write_out_group_dim_mute_hwctls(
                    req,
                    &mut unit.1,
                    sections,
                    self.state_mut(),
                    &dim_hwctls,
                    &mute_hwctls,
                    timeout_ms,
                )?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn parse_notification(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        sections: &ExtensionSections,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if T::has_dim_mute_change(msg) {
            let state = self.state_mut();
            T::read_out_group_mute(req, &mut unit.1, sections, state, timeout_ms)?;
            T::read_out_group_dim(req, &mut unit.1, sections, state, timeout_ms)?;
        }

        if T::has_vol_change(msg) {
            let state = self.state_mut();
            T::read_out_group_knob_value(req, &mut unit.1, sections, state, timeout_ms)?;

            let vol = state.hw_knob_value;
            let hwctls = state.vol_hwctls.clone();
            state
                .vols
                .iter_mut()
                .zip(hwctls)
                .filter(|(_, hwctl)| *hwctl)
                .for_each(|(v, _)| *v = vol);
        }

        Ok(())
    }

    fn read_notified_elem(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MUTE_NAME => {
                elem_value.set_bool(&[self.state().mute_enabled]);
                Ok(true)
            }
            DIM_NAME => {
                elem_value.set_bool(&[self.state().dim_enabled]);
                Ok(true)
            }
            VOL_NAME => {
                let vols: Vec<i32> = self
                    .state()
                    .vols
                    .iter()
                    .map(|&v| Self::LEVEL_MAX - (v as i32))
                    .collect();
                elem_value.set_int(&vols);
                Ok(true)
            }
            _ => Ok(false),
        }
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
    T: SaffireproInputOperation;

impl<T: SaffireproInputOperation> SaffireproInputCtl<T> {
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
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::cache_whole_input_params(req, node, sections, &mut self.0, timeout_ms);
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
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
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
                let res = T::update_partial_input_params(
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
                let res = T::update_partial_input_params(
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
