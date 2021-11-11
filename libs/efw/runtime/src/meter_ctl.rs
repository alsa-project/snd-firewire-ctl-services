// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    glib::{Error, FileError},
    hinawa::SndEfw,
    alsactl::{ElemId, ElemIfaceType, ElemValueExt, ElemValueExtManual, ElemValue},
    core::card_cntr::*,
    efw_protocols::hw_info::*,
};

#[derive(Default)]
pub struct MeterCtl {
    pub measure_elems: Vec<ElemId>,
    meters: Option<HwMeter>,
    midi_inputs: usize,
    midi_outputs: usize,
}

const CLK_DETECT_NAME: &str = "clock-detect";
const MIDI_IN_DETECT_NAME: &str = "midi-in-detect";
const MIDI_OUT_DETECT_NAME: &str = "midi-out-detect";
const INPUT_METERS_NAME: &str = "input-meter";
const OUTPUT_METERS_NAME: &str = "output-meter";
const GUITAR_STEREO_CONNECT_NAME: &str = "guitar-stereo-detect";
const GUITAR_HEX_SIGNAL_NAME: &str = "guitar-hex-signal-detect";
const GUITAR_CHARGE_STATE_NAME: &str = "guitar-charge-state-detect";

impl MeterCtl {
    const COEF_MIN: i32 = 0;
    const COEF_MAX: i32 = 0x007fffff;
    const COEF_STEP: i32 = 1;

    pub fn load(&mut self, hwinfo: &HwInfo, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.meters = Some(HwMeter::new(
            &hwinfo.clk_srcs,
            hwinfo.mixer_captures,
            hwinfo.mixer_playbacks,
        ));
        self.midi_inputs = hwinfo.midi_inputs;
        self.midi_outputs = hwinfo.midi_outputs;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_DETECT_NAME, 0);
        let elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, hwinfo.clk_srcs.len(), false)?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        if self.midi_inputs > 0 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Rawmidi, 0, 0, MIDI_IN_DETECT_NAME, 0);

            let elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, self.midi_inputs, false)?;
            self.measure_elems.extend_from_slice(&elem_id_list);
        }

        if self.midi_outputs > 0 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Rawmidi, 0, 0, MIDI_OUT_DETECT_NAME, 0);
            let elem_id_list =
                card_cntr.add_bool_elems(&elem_id, 1, self.midi_outputs, false)?;
            self.measure_elems.extend_from_slice(&elem_id_list);
        }

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_METERS_NAME, 0);
        let elem_id_list = card_cntr.add_int_elems(&elem_id, 1,
            Self::COEF_MIN, Self::COEF_MAX, Self::COEF_STEP,
            hwinfo.mixer_captures, None, false)?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_METERS_NAME, 0);
        let elem_id_list = card_cntr.add_int_elems(&elem_id, 1,
            Self::COEF_MIN, Self::COEF_MAX, Self::COEF_STEP,
            hwinfo.mixer_playbacks, None, false)?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        let has_robot_guitar = hwinfo.caps.iter().find(|&e| *e == HwCap::RobotGuitar).is_some();
        if has_robot_guitar {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, GUITAR_STEREO_CONNECT_NAME, 0);
            let elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, 1, false)?;
            self.measure_elems.extend_from_slice(&elem_id_list);

            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, GUITAR_HEX_SIGNAL_NAME, 0);
            let elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, 1, false)?;
            self.measure_elems.extend_from_slice(&elem_id_list);
        }

        let has_guitar_charge = hwinfo.caps.iter().find(|&e| *e == HwCap::GuitarCharging).is_some();
        if has_guitar_charge {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, GUITAR_CHARGE_STATE_NAME, 0);
            let elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, 1, false)?;
            self.measure_elems.extend_from_slice(&elem_id_list);
        }

        Ok(())
    }

    pub fn measure_states(&mut self, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        match &mut self.meters {
            Some(meters) => unit.get_hw_meter(meters, timeout_ms),
            None => {
                let label = "Metering data is not prepared.";
                Err(Error::new(FileError::Nxio, &label))
            }
        }
    }

    pub fn measure_elem(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            CLK_DETECT_NAME => {
                if let Some(meters) = &self.meters {
                    let vals: Vec<bool> = meters
                        .detected_clk_srcs
                        .iter()
                        .map(|(_, detected)| *detected)
                        .collect();
                    elem_value.set_bool(&vals);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            MIDI_IN_DETECT_NAME => {
                if let Some(meters) = &self.meters {
                    elem_value.set_bool(&meters.detected_midi_inputs[..self.midi_inputs]);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            MIDI_OUT_DETECT_NAME => {
                if let Some(meters) = &self.meters {
                    elem_value.set_bool(&meters.detected_midi_outputs[..self.midi_outputs]);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            INPUT_METERS_NAME => {
                if let Some(meters) = &self.meters {
                    elem_value.set_int(&meters.phys_input_meters);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            OUTPUT_METERS_NAME => {
                if let Some(meters) = &self.meters {
                    elem_value.set_int(&meters.phys_output_meters);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            GUITAR_STEREO_CONNECT_NAME => {
                if let Some(meters) = &self.meters {
                    elem_value.set_bool(&[meters.guitar_stereo_connect]);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            GUITAR_HEX_SIGNAL_NAME => {
                if let Some(meters) = &self.meters {
                    elem_value.set_bool(&[meters.guitar_hex_signal]);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            GUITAR_CHARGE_STATE_NAME => {
                if let Some(meters) = &self.meters {
                    elem_value.set_bool(&[meters.guitar_charging]);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }
}
