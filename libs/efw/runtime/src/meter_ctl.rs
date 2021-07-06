// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use core::card_cntr;

use alsactl::{ElemValueExt, ElemValueExtManual};

use efw_protocols::hw_info::*;

pub struct MeterCtl {
    pub measure_elems: Vec<alsactl::ElemId>,
    meters: Option<HwMeter>,
    midi_inputs: usize,
    midi_outputs: usize,
}

impl<'a> MeterCtl {
    const CLK_DETECT: &'a str = "clock-detect";
    const MIDI_IN_DETECT: &'a str = "midi-in-detect";
    const MIDI_OUT_DETECT: &'a str = "midi-out-detect";
    const INPUT_METERS: &'a str = "input-meter";
    const OUTPUT_METERS: &'a str = "output-meter";
    const GUITAR_STEREO_CONNECT: &'a str = "guitar-stereo-detect";
    const GUITAR_HEX_SIGNAL: &'a str = "guitar-hex-signal-detect";
    const GUITAR_CHARGE_STATE: &'a str = "guitar-charge-state-detect";

    const COEF_MIN: i32 = 0;
    const COEF_MAX: i32 = 0x007fffff;
    const COEF_STEP: i32 = 1;

    pub fn new() -> Self {
        MeterCtl {
            measure_elems: Vec::new(),
            meters: None,
            midi_inputs: 0,
            midi_outputs: 0,
        }
    }

    pub fn load(&mut self, hwinfo: &HwInfo, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        self.meters = Some(HwMeter::new(
            &hwinfo.clk_srcs,
            hwinfo.mixer_captures,
            hwinfo.mixer_playbacks,
        ));
        self.midi_inputs = hwinfo.midi_inputs;
        self.midi_outputs = hwinfo.midi_outputs;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card, 0, 0, Self::CLK_DETECT, 0);
        let elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, hwinfo.clk_srcs.len(), false)?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        if self.midi_inputs > 0 {
            let elem_id = alsactl::ElemId::new_by_name(
                alsactl::ElemIfaceType::Rawmidi, 0, 0, Self::MIDI_IN_DETECT, 0);

            let elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, self.midi_inputs, false)?;
            self.measure_elems.extend_from_slice(&elem_id_list);
        }

        if self.midi_outputs > 0 {
            let elem_id = alsactl::ElemId::new_by_name(
                alsactl::ElemIfaceType::Rawmidi, 0, 0, Self::MIDI_OUT_DETECT, 0);
            let elem_id_list =
                card_cntr.add_bool_elems(&elem_id, 1, self.midi_outputs, false)?;
            self.measure_elems.extend_from_slice(&elem_id_list);
        }

        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer, 0, 0, Self::INPUT_METERS, 0);
        let elem_id_list = card_cntr.add_int_elems(&elem_id, 1,
            Self::COEF_MIN, Self::COEF_MAX, Self::COEF_STEP,
            hwinfo.mixer_captures, None, false)?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer, 0, 0, Self::OUTPUT_METERS, 0);
        let elem_id_list = card_cntr.add_int_elems(&elem_id, 1,
            Self::COEF_MIN, Self::COEF_MAX, Self::COEF_STEP,
            hwinfo.mixer_playbacks, None, false)?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        let has_robot_guitar = hwinfo.caps.iter().find(|&e| *e == HwCap::RobotGuitar).is_some();
        if has_robot_guitar {
            let elem_id = alsactl::ElemId::new_by_name(
                alsactl::ElemIfaceType::Card, 0, 0, Self::GUITAR_STEREO_CONNECT, 0);
            let elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, 1, false)?;
            self.measure_elems.extend_from_slice(&elem_id_list);

            let elem_id = alsactl::ElemId::new_by_name(
                alsactl::ElemIfaceType::Card, 0, 0, Self::GUITAR_HEX_SIGNAL, 0);
            let elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, 1, false)?;
            self.measure_elems.extend_from_slice(&elem_id_list);
        }

        let has_guitar_charge = hwinfo.caps.iter().find(|&e| *e == HwCap::GuitarCharging).is_some();
        if has_guitar_charge {
            let elem_id = alsactl::ElemId::new_by_name(
                alsactl::ElemIfaceType::Card, 0, 0, Self::GUITAR_CHARGE_STATE, 0);
            let elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, 1, false)?;
            self.measure_elems.extend_from_slice(&elem_id_list);
        }

        Ok(())
    }

    pub fn measure_states(&mut self, unit: &mut hinawa::SndEfw, timeout_ms: u32) -> Result<(), Error> {
        match &mut self.meters {
            Some(meters) => unit.get_hw_meter(meters, timeout_ms),
            None => {
                let label = "Metering data is not prepared.";
                Err(Error::new(FileError::Nxio, &label))
            }
        }
    }

    pub fn measure_elem(&mut self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::CLK_DETECT => {
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
            Self::MIDI_IN_DETECT => {
                if let Some(meters) = &self.meters {
                    elem_value.set_bool(&meters.detected_midi_inputs[..self.midi_inputs]);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::MIDI_OUT_DETECT => {
                if let Some(meters) = &self.meters {
                    elem_value.set_bool(&meters.detected_midi_outputs[..self.midi_outputs]);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::INPUT_METERS => {
                if let Some(meters) = &self.meters {
                    elem_value.set_int(&meters.phys_input_meters);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::OUTPUT_METERS => {
                if let Some(meters) = &self.meters {
                    elem_value.set_int(&meters.phys_output_meters);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::GUITAR_STEREO_CONNECT => {
                if let Some(meters) = &self.meters {
                    elem_value.set_bool(&[meters.guitar_stereo_connect]);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::GUITAR_HEX_SIGNAL => {
                if let Some(meters) = &self.meters {
                    elem_value.set_bool(&[meters.guitar_hex_signal]);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::GUITAR_CHARGE_STATE => {
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
