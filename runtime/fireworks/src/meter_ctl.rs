// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

const CLK_DETECT_NAME: &str = "clock-detect";
const MIDI_IN_DETECT_NAME: &str = "midi-in-detect";
const MIDI_OUT_DETECT_NAME: &str = "midi-out-detect";
const INPUT_METERS_NAME: &str = "input-meter";
const OUTPUT_METERS_NAME: &str = "output-meter";
const GUITAR_STEREO_CONNECT_NAME: &str = "guitar-stereo-detect";
const GUITAR_HEX_SIGNAL_NAME: &str = "guitar-hex-signal-detect";
const GUITAR_CHARGE_STATE_NAME: &str = "guitar-charge-state-detect";

#[derive(Debug)]
pub(crate) struct HwMeterCtl<T>(pub Vec<ElemId>, HwMeter, PhantomData<T>)
where
    T: EfwHardwareSpecification + EfwWhollyCachableParamsOperation<SndEfw, HwMeter>;

impl<T> Default for HwMeterCtl<T>
where
    T: EfwHardwareSpecification + EfwWhollyCachableParamsOperation<SndEfw, HwMeter>,
{
    fn default() -> Self {
        HwMeterCtl(
            Default::default(),
            T::create_hardware_meter(),
            Default::default(),
        )
    }
}

impl<T> HwMeterCtl<T>
where
    T: EfwHardwareSpecification + EfwWhollyCachableParamsOperation<SndEfw, HwMeter>,
{
    const COEF_MIN: i32 = 0;
    const COEF_MAX: i32 = 0x007fffff;
    const COEF_STEP: i32 = 1;

    pub(crate) fn cache(&mut self, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        T::cache_wholly(unit, &mut self.1, timeout_ms)
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_DETECT_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, self.1.detected_clk_srcs.len(), false)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        if T::MIDI_INPUT_COUNT > 0 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Rawmidi, 0, 0, MIDI_IN_DETECT_NAME, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, T::MIDI_INPUT_COUNT, false)
                .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;
        }

        if T::MIDI_OUTPUT_COUNT > 0 {
            let elem_id =
                ElemId::new_by_name(ElemIfaceType::Rawmidi, 0, 0, MIDI_OUT_DETECT_NAME, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, T::MIDI_OUTPUT_COUNT, false)
                .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;
        }

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_METERS_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::COEF_MIN,
                Self::COEF_MAX,
                Self::COEF_STEP,
                T::phys_input_count(),
                None,
                false,
            )
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_METERS_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::COEF_MIN,
                Self::COEF_MAX,
                Self::COEF_STEP,
                T::phys_output_count(),
                None,
                false,
            )
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let has_robot_guitar = T::CAPABILITIES
            .iter()
            .find(|e| HwCap::RobotGuitar.eq(e))
            .is_some();
        if has_robot_guitar {
            let elem_id =
                ElemId::new_by_name(ElemIfaceType::Card, 0, 0, GUITAR_STEREO_CONNECT_NAME, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, 1, false)
                .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, GUITAR_HEX_SIGNAL_NAME, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, 1, false)
                .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;
        }

        let has_guitar_charge = T::CAPABILITIES
            .iter()
            .find(|e| HwCap::GuitarCharging.eq(e))
            .is_some();
        if has_guitar_charge {
            let elem_id =
                ElemId::new_by_name(ElemIfaceType::Card, 0, 0, GUITAR_CHARGE_STATE_NAME, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, 1, false)
                .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;
        }

        Ok(())
    }

    pub(crate) fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_DETECT_NAME => {
                let vals: Vec<bool> = self
                    .1
                    .detected_clk_srcs
                    .iter()
                    .map(|(_, detected)| *detected)
                    .collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            MIDI_IN_DETECT_NAME => {
                elem_value.set_bool(&self.1.detected_midi_inputs[..T::MIDI_INPUT_COUNT]);
                Ok(true)
            }
            MIDI_OUT_DETECT_NAME => {
                elem_value.set_bool(&self.1.detected_midi_outputs[..T::MIDI_OUTPUT_COUNT]);
                Ok(true)
            }
            INPUT_METERS_NAME => {
                elem_value.set_int(&self.1.phys_input_meters);
                Ok(true)
            }
            OUTPUT_METERS_NAME => {
                elem_value.set_int(&self.1.phys_output_meters);
                Ok(true)
            }
            GUITAR_STEREO_CONNECT_NAME => {
                elem_value.set_bool(&[self.1.guitar_stereo_connect]);
                Ok(true)
            }
            GUITAR_HEX_SIGNAL_NAME => {
                elem_value.set_bool(&[self.1.guitar_hex_signal]);
                Ok(true)
            }
            GUITAR_CHARGE_STATE_NAME => {
                elem_value.set_bool(&[self.1.guitar_charging]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
