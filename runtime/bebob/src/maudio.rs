// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
pub mod audiophile_model;
pub mod fw410_model;
pub mod ozonic_model;
pub mod profirelightbridge_model;
pub mod solo_model;
pub mod special_model;

use {
    super::{common_ctls::*, *},
    protocols::{maudio::normal::*, *},
    ta1394_avc_general::*,
};

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

trait MaudioNormalMeterCtlOperation<O: MaudioNormalMeterProtocol> {
    fn state(&self) -> &MaudioNormalMeter;
    fn state_mut(&mut self) -> &mut MaudioNormalMeter;

    fn load_meter(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
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

    fn cache_meter(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        avc: &BebobAvc,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let meter = self.state_mut();
        let switch_state = meter.switch;

        let res = O::read_meter(req, node, meter, timeout_ms);
        debug!(params = ?meter, res = ?res);
        res?;

        if switch_state != meter.switch {
            let state = meter.switch.unwrap();
            let mut op = AudiophileLedSwitch::new(state);
            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;
        }

        Ok(())
    }

    fn read_meter(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        let meter = self.state();

        match elem_id.name().as_str() {
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
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            SWITCH_NAME => {
                if O::HAS_SWITCH {
                    if let Some(switch) = &mut self.state_mut().switch {
                        let pos = elem_value.enumerated()[0] as usize;
                        let &state = SWITCH_LIST.iter().nth(pos).ok_or_else(|| {
                            let msg = format!("Invalid index for LED switch: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })?;
                        let mut op = AudiophileLedSwitch::new(state);
                        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
                            .map(|_| *switch = state)?;
                        Ok(true)
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

    fn state(&self) -> &MaudioNormalMixerParameters;
    fn state_mut(&mut self) -> &mut MaudioNormalMixerParameters;

    fn load_src_state(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
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
            self.state_mut()
                .0
                .iter_mut()
                .enumerate()
                .for_each(|(dst_idx, srcs)| {
                    let src_idx = Self::SRC_COUNT - Self::DST_COUNT + dst_idx;
                    srcs[src_idx] = true;
                });
        }

        Ok(())
    }

    fn cache(&mut self, avc: &BebobAvc, timeout_ms: u32) -> Result<(), Error> {
        // NOTE: Due to quirk of the ASIC, it should be avoided to request AV/C status request
        // for mixer parameters. Alternatively, initiate AV/C control request for them.
        let mut params = self.state().clone();
        params.0.iter_mut().for_each(|levels| {
            levels.iter_mut().for_each(|level| *level = !(*level));
        });
        let res = O::update(avc, self.state(), &mut params, timeout_ms);
        debug!(params = ?self.state(), ?res);
        res
    }

    fn read_src_state(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if elem_id.name().as_str() == Self::MIXER_NAME {
            let dst_idx = elem_id.index() as usize;
            elem_value.set_bool(&self.state().0[dst_idx]);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write_src_state(
        &mut self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if elem_id.name().as_str() == Self::MIXER_NAME {
            let dst_idx = elem_id.index() as usize;
            let mut params = self.state().clone();
            let vals = &new.boolean()[..Self::SRC_COUNT];
            params.0[dst_idx].copy_from_slice(&vals);
            let res = O::update(avc, &params, self.state_mut(), timeout_ms);
            debug!(params = ?self.state(), ?res);
            res.map(|_| true)
        } else {
            Ok(false)
        }
    }
}
