// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::Error;

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use hinawa::{FwNode, SndDice, SndUnitExt};

use dice_protocols::tcat::extension::*;
use dice_protocols::focusrite::*;

use core::card_cntr::*;
use core::elem_value_accessor::*;

#[derive(Default, Debug)]
pub struct OutGroupCtl(Vec<ElemId>);

impl<'a> OutGroupCtl {
    const VOL_NAME: &'a str = "output-group-volume";
    const VOL_HWCTL_NAME: &'a str = "output-group-volume-hwctl";
    const VOL_MUTE_NAME: &'a str = "output-group-volume-mute";
    const MUTE_NAME: &'a str = "output-group-mute";
    const DIM_NAME: &'a str = "output-group-dim";
    const DIM_HWCTL_NAME: &'a str= "output-group-dim-hwctl";
    const MUTE_HWCTL_NAME: &'a str = "output-group-mute-hwctl";

    const LEVEL_MIN: i32 = 0x00;
    const LEVEL_MAX: i32 = 0x7f;
    const LEVEL_STEP: i32 = 0x01;

    pub fn load<T, S>(&mut self, card_cntr: &mut CardCntr, unit: &SndDice, proto: &T,
                      sections: &ExtensionSections, state: &mut S, timeout_ms: u32)
        -> Result<(), Error>
        where T: FocusriteSaffireOutGroupProtocol<FwNode, S>,
              S: OutGroupSpec + AsRef<OutGroupState> + AsMut<OutGroupState>,
    {
        let node = unit.get_node();
        proto.read_out_group_mute(&node, sections, state, timeout_ms)?;
        proto.read_out_group_dim(&node, sections, state, timeout_ms)?;
        proto.read_out_group_vols(&node, sections, state, timeout_ms)?;
        proto.read_out_group_vol_mute_hwctls(&node, sections, state, timeout_ms)?;
        proto.read_out_group_dim_mute_hwctls(&node, sections, state, timeout_ms)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MUTE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::DIM_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let output_count = S::ENTRY_COUNT;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::VOL_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                output_count, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::VOL_MUTE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, output_count, true)?;

        if S::HAS_VOL_HWCTL {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::VOL_HWCTL_NAME, 0);
            card_cntr.add_bool_elems(&elem_id, 1, output_count, true)?;
        }

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::DIM_HWCTL_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, output_count, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MUTE_HWCTL_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, output_count, true)?;

        Ok(())
    }

    pub fn read<S>(&mut self, state: &S, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where S: AsRef<OutGroupState>,
    {
        match elem_id.get_name().as_str() {
            Self::VOL_MUTE_NAME => {
                elem_value.set_bool(&state.as_ref().vol_mutes);
                Ok(true)
            }
            Self::VOL_HWCTL_NAME => {
                elem_value.set_bool(&state.as_ref().vol_hwctls);
                Ok(true)
            }
            Self::DIM_HWCTL_NAME => {
                elem_value.set_bool(&state.as_ref().dim_hwctls);
                Ok(true)
            }
            Self::MUTE_HWCTL_NAME => {
                elem_value.set_bool(&state.as_ref().mute_hwctls);
                Ok(true)
            }
            _ => self.read_notified_elem(state, elem_id, elem_value),
        }
    }

    pub fn write<T, S>(&mut self, unit: &SndDice, proto: &T, sections: &ExtensionSections,
                       state: &mut S, elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where T: FocusriteSaffireOutGroupProtocol<FwNode, S>,
              S: OutGroupSpec + AsRef<OutGroupState> + AsMut<OutGroupState>,
    {
        match elem_id.get_name().as_str() {
            Self::MUTE_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    proto.write_out_group_mute(&unit.get_node(), sections, state, val, timeout_ms)
                })
                .map(|_| true)
            }
            Self::DIM_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    proto.write_out_group_dim(&unit.get_node(), sections, state, val, timeout_ms)
                })
                .map(|_| true)
            }
            Self::VOL_NAME => {
                let mut vals = vec![0i32;state.as_ref().vols.len()];
                elem_value.get_int(&mut vals);
                let vols: Vec<i8> = vals.iter()
                    .map(|&v| (Self::LEVEL_MAX - v) as i8)
                    .collect();
                proto.write_out_group_vols(&unit.get_node(), sections, state, &vols, timeout_ms)
                .map(|_| true)
            }
            Self::VOL_MUTE_NAME => {
                let mut vol_mutes = state.as_ref().vol_mutes.clone();
                let vol_hwctls = state.as_ref().vol_hwctls.clone();
                elem_value.get_bool(&mut vol_mutes);
                proto.write_out_group_vol_mute_hwctls(&unit.get_node(), sections, state, &vol_mutes,
                                                      &vol_hwctls, timeout_ms)
                .map(|_| true)
            }
            Self::VOL_HWCTL_NAME => {
                let vol_mutes = state.as_ref().vol_mutes.clone();
                let mut vol_hwctls = state.as_ref().vol_hwctls.clone();
                elem_value.get_bool(&mut vol_hwctls);
                proto.write_out_group_vol_mute_hwctls(&unit.get_node(), sections, state, &vol_mutes,
                                                      &vol_hwctls, timeout_ms)
                .map(|_| true)
            }
            Self::DIM_HWCTL_NAME => {
                let mute_hwctls = state.as_ref().mute_hwctls.clone();
                let mut dim_hwctls = state.as_ref().dim_hwctls.clone();
                elem_value.get_bool(&mut dim_hwctls);
                proto.write_out_group_dim_mute_hwctls(&unit.get_node(), sections, state,
                                                      &dim_hwctls, &mute_hwctls, timeout_ms)?;
                Ok(true)
            }
            Self::MUTE_HWCTL_NAME => {
                let mut mute_hwctls = state.as_ref().mute_hwctls.clone();
                let dim_hwctls = state.as_ref().dim_hwctls.clone();
                elem_value.get_bool(&mut mute_hwctls);
                proto.write_out_group_dim_mute_hwctls(&unit.get_node(), sections, state,
                                                      &dim_hwctls, &mute_hwctls, timeout_ms)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn get_notified_elem_list(&self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.0);
    }

    pub fn parse_notification<T, S>(&mut self, unit: &SndDice, proto: &T, sections: &ExtensionSections,
                                    state: &mut S, msg: u32, timeout_ms: u32)
        -> Result<(), Error>
        where T: FocusriteSaffireOutGroupProtocol<FwNode, S>,
              S: OutGroupSpec + AsRef<OutGroupState> + AsMut<OutGroupState>,
    {
        if msg.has_dim_mute_change() {
            let node = unit.get_node();
            proto.read_out_group_mute(&node, sections, state, timeout_ms)?;
            proto.read_out_group_dim(&node, sections, state, timeout_ms)?;
        }

        if msg.has_vol_change() {
            proto.read_out_group_knob_value(&unit.get_node(), sections, state, timeout_ms)?;

            let vol = state.as_ref().hw_knob_value;
            let hwctls = state.as_ref().vol_hwctls.clone();
            state.as_mut().vols.iter_mut()
                .zip(hwctls.iter())
                .filter(|(_, &hwctl)| hwctl)
                .for_each(|(v, _)| *v = vol);
        }

        Ok(())
    }

    pub fn read_notified_elem<S>(&self, state: &S, elem_id: &ElemId, elem_value: &ElemValue)
        -> Result<bool, Error>
        where S: AsRef<OutGroupState>,
    {
        match elem_id.get_name().as_str() {
            Self::MUTE_NAME => {
                elem_value.set_bool(&[state.as_ref().mute_enabled]);
                Ok(true)
            }
            Self::DIM_NAME => {
                elem_value.set_bool(&[state.as_ref().dim_enabled]);
                Ok(true)
            }
            Self::VOL_NAME => {
                let vols: Vec<i32> = state.as_ref().vols.iter()
                    .map(|&v| Self::LEVEL_MAX - (v as i32))
                    .collect();
                elem_value.set_int(&vols);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
