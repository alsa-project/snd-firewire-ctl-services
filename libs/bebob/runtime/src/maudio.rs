// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
pub mod ozonic_model;
pub mod solo_model;
pub mod audiophile_model;
pub mod fw410_model;
pub mod profirelightbridge_model;
pub mod special_model;

mod common_proto;


use glib::{Error, FileError};

use hinawa::{FwNode, FwReq};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;
use core::elem_value_accessor::*;

use ta1394::*;

use bebob_protocols::{*, maudio::normal::*};

use crate::model::{IN_METER_NAME, OUT_METER_NAME};

const STREAM_INPUT_METER_NAME: &str = "stream-input-meters";
const HP_METER_NAME: &str = "headphone-meters";
const AUX_OUT_METER_NAME: &str = "aux-output-meters";
const ROTARY_NAME: &str = "rotary";
const SWITCH_NAME: &str = "switch";
const SYNC_STATUS_NAME: &str = "sync status";

const SWITCH_LIST: [AudiophileSwitchState; 3] = [
    AudiophileSwitchState::Off,
    AudiophileSwitchState::A,
    AudiophileSwitchState::B,
];

const METER_TLV: DbInterval = DbInterval {
    min: -14400,
    max: 0,
    linear: false,
    mute_avail: false,
};

fn audiophile_switch_state_to_string(state: &AudiophileSwitchState) -> String {
    match state {
        AudiophileSwitchState::Off => "OFF",
        AudiophileSwitchState::A => "A",
        AudiophileSwitchState::B => "B",
    }
    .to_string()
}

trait MaudioNormalMeterCtlOperation<O>: AsMut<MaudioNormalMeter> + AsRef<MaudioNormalMeter>
where
    O: MaudioNormalMeterProtocol,
{
    fn load_meter(
        &mut self,
        card_cntr: &mut CardCntr,
        req: &FwReq,
        node: &FwNode,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        O::read_meter(req, node, self.as_mut(), timeout_ms)?;

        let mut measure_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, IN_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                O::LEVEL_MIN,
                O::LEVEL_MAX,
                O::LEVEL_STEP,
                O::PHYS_INPUT_COUNT,
                Some(&Into::<Vec<u32>>::into(METER_TLV)),
                false,
            )
            .map(|mut elem_id_list| measure_elem_id_list.append(&mut elem_id_list))?;

        if O::STREAM_INPUT_COUNT > 0 {
            let elem_id =
                ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, STREAM_INPUT_METER_NAME, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    O::LEVEL_MIN,
                    O::LEVEL_MAX,
                    O::LEVEL_STEP,
                    O::PHYS_INPUT_COUNT,
                    Some(&Into::<Vec<u32>>::into(METER_TLV)),
                    false,
                )
                .map(|mut elem_id_list| measure_elem_id_list.append(&mut elem_id_list))?;
        }

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                O::LEVEL_MIN,
                O::LEVEL_MAX,
                O::LEVEL_STEP,
                O::PHYS_OUTPUT_COUNT,
                Some(&Into::<Vec<u32>>::into(METER_TLV)),
                false,
            )
            .map(|mut elem_id_list| measure_elem_id_list.append(&mut elem_id_list))?;

        if O::STREAM_INPUT_COUNT == 0 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, HP_METER_NAME, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    O::LEVEL_MIN,
                    O::LEVEL_MAX,
                    O::LEVEL_STEP,
                    2,
                    Some(&Into::<Vec<u32>>::into(METER_TLV)),
                    false,
                )
                .map(|mut elem_id_list| measure_elem_id_list.append(&mut elem_id_list))?;

            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, AUX_OUT_METER_NAME, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    O::LEVEL_MIN,
                    O::LEVEL_MAX,
                    O::LEVEL_STEP,
                    2,
                    Some(&Into::<Vec<u32>>::into(METER_TLV)),
                    false,
                )
                .map(|mut elem_id_list| measure_elem_id_list.append(&mut elem_id_list))?;
        }

        if O::ROTARY_COUNT > 0 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ROTARY_NAME, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    O::ROTARY_MIN,
                    O::ROTARY_MAX,
                    O::ROTARY_STEP,
                    O::ROTARY_COUNT,
                    None,
                    false,
                )
                .map(|mut elem_id_list| measure_elem_id_list.append(&mut elem_id_list))?;
        }

        if O::HAS_SWITCH {
            let labels: Vec<String> = SWITCH_LIST
                .iter()
                .map(|s| audiophile_switch_state_to_string(s))
                .collect();

            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SWITCH_NAME, 0);
            card_cntr
                .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
                .map(|mut elem_id_list| measure_elem_id_list.append(&mut elem_id_list))?;
        }

        if O::HAS_SYNC_STATUS {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SYNC_STATUS_NAME, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, 1, false)
                .map(|mut elem_id_list| measure_elem_id_list.append(&mut elem_id_list))?;
        }

        Ok(measure_elem_id_list)
    }

    fn measure_meter(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        avc: &BebobAvc,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let meter = self.as_mut();
        let switch_state = meter.switch;

        O::read_meter(req, node, meter, timeout_ms)?;

        if switch_state != meter.switch {
            let state = meter.switch.unwrap();
            let mut op = AudiophileLedSwitch::new(state);
            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
        }

        Ok(())
    }

    fn read_meter(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        let meter = self.as_ref();

        match elem_id.get_name().as_str() {
            IN_METER_NAME => {
                elem_value.set_int(&meter.phys_inputs);
                Ok(true)
            }
            STREAM_INPUT_METER_NAME => {
                if let Some(stream_inputs) = &meter.stream_inputs {
                    elem_value.set_int(stream_inputs);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            OUT_METER_NAME => {
                elem_value.set_int(&meter.phys_outputs);
                Ok(true)
            }
            HP_METER_NAME => {
                if let Some(headphone) = &meter.headphone {
                    elem_value.set_int(headphone);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            AUX_OUT_METER_NAME => {
                if let Some(aux_output) = &meter.aux_output {
                    elem_value.set_int(aux_output);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            ROTARY_NAME => {
                if let Some(rotaries) = &meter.rotaries {
                    elem_value.set_int(rotaries);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            SWITCH_NAME => {
                if let Some(switch) = &meter.switch {
                    let val = match switch {
                        AudiophileSwitchState::Off => 0,
                        AudiophileSwitchState::A => 1,
                        AudiophileSwitchState::B => 2,
                    };
                    elem_value.set_enum(&[val]);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            SYNC_STATUS_NAME => {
                if let Some(sync_status) = &meter.sync_status {
                    elem_value.set_bool(&[*sync_status]);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

    fn write_meter(
        &mut self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            SWITCH_NAME => {
                if O::HAS_SWITCH {
                    if let Some(switch) = &mut self.as_mut().switch {
                        ElemValueAccessor::<u32>::get_val(new, |val| {
                            let state = SWITCH_LIST
                                .iter()
                                .nth(val as usize)
                                .ok_or_else(|| {
                                    let msg = format!("Invalid index for LED switch: {}", val);
                                    Error::new(FileError::Inval, &msg)
                                })
                                .map(|s| *s)?;
                            let mut op = AudiophileLedSwitch::new(state);
                            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
                            *switch = state;
                            Ok(())
                        })
                        .map(|_| true)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }
}

pub trait MaudioNormalMixerCtlOperation<O: MaudioNormalMixerOperation> {
    const MIXER_NAME: &'static str;

    const DST_LABELS: &'static [&'static str];
    const SRC_LABELS: &'static [&'static str];

    const DST_COUNT: usize = O::DST_FUNC_BLOCK_ID_LIST.len();
    const SRC_COUNT: usize = O::SRC_FUNC_BLOCK_ID_LIST.len();

    fn load_src_state(
        &self,
        card_cntr: &mut CardCntr,
        avc: &BebobAvc,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(
            Self::DST_COUNT,
            Self::DST_LABELS.len(),
            "Programming error for count of destination: {}",
            Self::MIXER_NAME
        );
        assert_eq!(
            Self::SRC_COUNT,
            Self::SRC_LABELS.len(),
            "Programming error for count of source: {}",
            Self::MIXER_NAME
        );

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, Self::DST_COUNT, Self::SRC_COUNT, true)
            .map(|_| ())?;

        // For convenicence, make connection between mixer destination and stream source.
        if Self::DST_COUNT > 1 {
            (0..Self::DST_COUNT).try_for_each(|dst_idx| {
                let src_idx = Self::SRC_COUNT - Self::DST_COUNT + dst_idx;
                O::write_mixer_src(avc, dst_idx, src_idx, true, timeout_ms)
            })?;
        }

        Ok(())
    }

    fn read_src_state(
        &self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if elem_id.get_name().as_str() == Self::MIXER_NAME {
            let dst_idx = elem_id.get_index() as usize;
            ElemValueAccessor::<bool>::set_vals(elem_value, Self::SRC_COUNT, |src_idx| {
                O::read_mixer_src(avc, dst_idx, src_idx, timeout_ms)
            })
            .map(|_| true)
        } else {
            Ok(false)
        }
    }

    fn write_src_state(
        &self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if elem_id.get_name().as_str() == Self::MIXER_NAME {
            let dst_idx = elem_id.get_index() as usize;
            ElemValueAccessor::<bool>::get_vals(new, old, Self::SRC_COUNT, |src_idx, val| {
                O::write_mixer_src(avc, dst_idx, src_idx, val, timeout_ms)
            })
            .map(|_| true)
        } else {
            Ok(false)
        }
    }
}
