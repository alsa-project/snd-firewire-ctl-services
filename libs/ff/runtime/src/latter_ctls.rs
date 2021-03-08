// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt};
use hinawa::{FwNode, SndUnit, SndUnitExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;

use ff_protocols::latter::*;

#[derive(Default, Debug)]
pub struct FfLatterMeterCtl<V>
    where V: RmeFfLatterMeterSpec + AsRef<FfLatterMeterState> + AsMut<FfLatterMeterState>,
{
    state: V,
    measured_elem_list: Vec<ElemId>,
}

impl<'a, V> FfLatterMeterCtl<V>
    where V: RmeFfLatterMeterSpec + AsRef<FfLatterMeterState> + AsMut<FfLatterMeterState>,
{
    const LINE_INPUT_METER: &'a str = "meter:line-input";
    const MIC_INPUT_METER: &'a str = "meter:mic-input";
    const SPDIF_INPUT_METER: &'a str = "meter:spdif-input";
    const ADAT_INPUT_METER: &'a str = "meter:adat-input";
    const STREAM_INPUT_METER: &'a str = "meter:stream-input";
    const LINE_OUTPUT_METER: &'a str = "meter:line-output";
    const HP_OUTPUT_METER: &'a str = "meter:hp-output";
    const SPDIF_OUTPUT_METER: &'a str = "meter:spdif-output";
    const ADAT_OUTPUT_METER: &'a str = "meter:adat-output";

    const LEVEL_MIN: i32 = 0x0;
    const LEVEL_MAX: i32 = 0x07fffff0;
    const LEVEL_STEP: i32 = 0x10;
    const LEVEL_TLV: DbInterval = DbInterval{min: -9003, max: 600, linear: false, mute_avail: false};

    pub fn load<U>(&mut self, unit: &SndUnit, proto: &U, timeout_ms: u32, card_cntr: &mut CardCntr)
        -> Result<(), Error>
        where U: RmeFfLatterMeterProtocol<FwNode, V>,
    {
        proto.read_meter(&unit.get_node(), &mut self.state, timeout_ms)?;

        [
            (Self::LINE_INPUT_METER, V::LINE_INPUT_COUNT),
            (Self::MIC_INPUT_METER, V::MIC_INPUT_COUNT),
            (Self::SPDIF_INPUT_METER, V::SPDIF_INPUT_COUNT),
            (Self::ADAT_INPUT_METER, V::ADAT_INPUT_COUNT),
            (Self::STREAM_INPUT_METER, V::STREAM_INPUT_COUNT),
            (Self::LINE_OUTPUT_METER, V::LINE_OUTPUT_COUNT),
            (Self::HP_OUTPUT_METER, V::HP_OUTPUT_COUNT),
            (Self::SPDIF_OUTPUT_METER, V::SPDIF_OUTPUT_COUNT),
            (Self::ADAT_OUTPUT_METER, V::ADAT_OUTPUT_COUNT),
        ].iter()
            .try_for_each(|(name, count)| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, name, 0);
                card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP, *count,
                                        Some(&Vec::<u32>::from(&Self::LEVEL_TLV)), false)
                    .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))
            })?;

        Ok(())
    }

    pub fn get_measured_elem_list(&self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.measured_elem_list);
    }

    pub fn measure_states<U>(&mut self, unit: &SndUnit, proto: &U, timeout_ms: u32)
        -> Result<(), Error>
        where U: RmeFfLatterMeterProtocol<FwNode, V>,
    {
        proto.read_meter(&unit.get_node(), &mut self.state, timeout_ms)
    }

    pub fn read_measured_elem(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::LINE_INPUT_METER => {
                elem_value.set_int(&self.state.as_ref().line_inputs);
                Ok(true)
            }
            Self::MIC_INPUT_METER => {
                elem_value.set_int(&self.state.as_ref().mic_inputs);
                Ok(true)
            }
            Self::SPDIF_INPUT_METER => {
                elem_value.set_int(&self.state.as_ref().spdif_inputs);
                Ok(true)
            }
            Self::ADAT_INPUT_METER => {
                elem_value.set_int(&self.state.as_ref().adat_inputs);
                Ok(true)
            }
            Self::STREAM_INPUT_METER => {
                elem_value.set_int(&self.state.as_ref().stream_inputs);
                Ok(true)
            }
            Self::LINE_OUTPUT_METER => {
                elem_value.set_int(&self.state.as_ref().line_outputs);
                Ok(true)
            }
            Self::HP_OUTPUT_METER => {
                elem_value.set_int(&self.state.as_ref().hp_outputs);
                Ok(true)
            }
            Self::SPDIF_OUTPUT_METER => {
                elem_value.set_int(&self.state.as_ref().spdif_outputs);
                Ok(true)
            }
            Self::ADAT_OUTPUT_METER => {
                elem_value.set_int(&self.state.as_ref().adat_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
