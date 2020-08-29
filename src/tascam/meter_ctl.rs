// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use alsactl::ElemValueExt;

use crate::card_cntr;

use super::common_ctl::CommonCtl;

pub struct MeterCtl<'a> {
    clk_src_labels: &'a [&'a str],

    analog_out_count: u32,
    has_adat: bool,
    has_solo: bool,

    pub measure_elems: Vec<alsactl::ElemId>,

    monitor: i32,
    solo: i32,
    inputs: [i32; 18],
    outputs: [i32; 18],
    rate: u32,
    src: u32,
    monitor_meters: [i32; 2],
    analog_mixer_meters: [i32; 2],
    monitor_mode: u32,
}

impl<'a> MeterCtl<'a> {
    const MONITOR_ROTARY_NAME: &'a str = "monitor-rotary";
    const SOLO_ROTARY_NAME: &'a str = "solo-rotary";
    const INPUT_METER_NAME: &'a str = "input-meters";
    const OUTPUT_METER_NAME: &'a str = "output-meters";
    const DETECTED_CLK_SRC_NAME: &'a str = "detected-clock-source";
    const DETECTED_CLK_RATE_NAME: &'a str = "detected-clock-rate";
    const MONITOR_METER_NAME: &'a str = "monitor-meters";
    const ANALOG_MIXER_METER_NAME: &'a str = "analog-mixer-meters";
    const MONITOR_MODE_NAME: &'a str = "monitor-mode";

    const MONITOR_MODE_LABELS: &'a [&'a str] = &["computer", "inputs", "both"];

    const ROTARY_MIN: i32 = 0;
    const ROTARY_MAX: i32 = 1023;
    const ROTARY_STEP: i32 = 2;

    const METER_MIN: i32 = 0;
    const METER_MAX: i32 = 0x7fffff00;
    const METER_STEP: i32 = 0xff;

    pub fn new(clk_src_labels: &'a [&'a str], analog_out_count: u32, has_adat: bool, has_solo: bool) -> Self {
        MeterCtl {
            clk_src_labels,
            analog_out_count,
            has_adat,
            has_solo,
            measure_elems: Vec::new(),
            monitor: 0,
            solo: 0,
            inputs: [0; 18],
            outputs: [0; 18],
            monitor_meters: [0; 2],
            analog_mixer_meters: [0; 2],
            rate: 0,
            src: 0,
            monitor_mode: 0,
        }
    }

    pub fn parse_states(&mut self, states: &[u32; 64]) {
        let monitor = (states[5] & 0x0000ffff) as i32;
        if (self.monitor - monitor).abs() > Self::ROTARY_STEP {
            self.monitor = monitor;
        }

        let solo = ((states[4] >> 16) & 0x0000ffff) as i32;
        if (self.solo - solo).abs() > Self::ROTARY_STEP {
            self.solo = solo;
        }

        self.inputs.iter_mut().enumerate().for_each(|(i, input)| {
            *input = states[i + 16] as i32;
        });
        self.outputs.iter_mut().enumerate().for_each(|(i, output)| {
            *output = states[i + 34] as i32;
        });
        let bits = (states[52] & 0x0000000f) as u8;
        if bits > 0 && bits < 5 {
            self.src = (bits - 1) as u32;
        }

        let bits = ((states[52] >> 8) & 0x000000ff) as u8;
        self.rate = match bits {
            0x01 => 0,
            0x02 => 1,
            0x81 => 2,
            0x82 => 3,
            _ => 0,
        };

        self.monitor_meters.iter_mut().enumerate().for_each(|(i, m)| {
            *m = states[i + 54] as i32;
        });

        self.analog_mixer_meters.iter_mut().enumerate().for_each(|(i, m)| {
            *m = states[i + 57] as i32;
        });

        if states[59] > 0 && states[59] < 4 {
            self.monitor_mode = states[59] - 1;
        }
    }

    pub fn load(&mut self, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        // For volume of monitor knob.
        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::MONITOR_ROTARY_NAME,
            0,
        );
        let elem_id_list = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::ROTARY_MIN,
            Self::ROTARY_MAX,
            Self::ROTARY_STEP,
            1,
            None,
            false,
        )?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        // For For volume of solo knob.
        if self.has_solo {
            let elem_id = alsactl::ElemId::new_by_name(
                alsactl::ElemIfaceType::Mixer,
                0,
                0,
                Self::SOLO_ROTARY_NAME,
                0,
            );
            let elem_id_list = card_cntr.add_int_elems(
                &elem_id,
                1,
                Self::ROTARY_MIN,
                Self::ROTARY_MAX,
                Self::ROTARY_STEP,
                1,
                None,
                false,
            )?;
            self.measure_elems.extend_from_slice(&elem_id_list);
        }

        // For meters of inputs.
        let mut inputs = 10;
        if self.has_adat {
            inputs += 8;
        }
        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::INPUT_METER_NAME,
            0,
        );
        let elem_id_list = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::METER_MIN,
            Self::METER_MAX,
            Self::METER_STEP,
            inputs,
            None,
            false,
        )?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        // For meters of outputs.
        let mut outputs = 2 + self.analog_out_count as usize;
        if self.has_adat {
            outputs += 8;
        }
        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::OUTPUT_METER_NAME,
            0,
        );
        let elem_id_list = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::METER_MIN,
            Self::METER_MAX,
            Self::METER_STEP,
            outputs,
            None,
            false,
        )?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        // For meters of monitors.
        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::MONITOR_METER_NAME,
            0,
        );
        let elem_id_list = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::METER_MIN,
            Self::METER_MAX,
            Self::METER_STEP,
            2,
            None,
            false,
        )?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        // For meters of mixer for analog inputs.
        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::ANALOG_MIXER_METER_NAME,
            0,
        );
        let elem_id_list = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::METER_MIN,
            Self::METER_MAX,
            Self::METER_STEP,
            2,
            None,
            false,
        )?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        // For detection of clock source.
        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::DETECTED_CLK_SRC_NAME,
            0,
        );
        let elem_id_list =
            card_cntr.add_enum_elems(&elem_id, 1, 1, self.clk_src_labels, None, false)?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        // For detection of clock rate.
        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::DETECTED_CLK_RATE_NAME,
            0,
        );
        let elem_id_list =
            card_cntr.add_enum_elems(&elem_id, 1, 1, CommonCtl::CLK_RATE_LABELS, None, false)?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        // For mode of monitor.
        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::MONITOR_MODE_NAME,
            0,
        );
        let elem_id_list =
            card_cntr.add_enum_elems(&elem_id, 1, 1, Self::MONITOR_MODE_LABELS, None, false)?;
        self.measure_elems.extend_from_slice(&elem_id_list);

        Ok(())
    }

    pub fn read(
        &mut self,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::MONITOR_ROTARY_NAME => {
                elem_value.set_int(&[self.monitor]);
                Ok(true)
            }
            Self::SOLO_ROTARY_NAME => {
                elem_value.set_int(&[self.solo]);
                Ok(true)
            }
            Self::INPUT_METER_NAME => {
                let mut vals = Vec::new();
                // For Analog inputs.
                (0..8).for_each(|i| {
                    vals.push(self.inputs[i]);
                });
                if self.has_adat {
                    // For ADAT inputs.
                    (0..8).for_each(|i| {
                        vals.push(self.inputs[i + 8]);
                    });
                }
                // For S/PDIF inputs.
                (0..2).for_each(|i| {
                    vals.push(self.inputs[i + 10]);
                });
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::OUTPUT_METER_NAME => {
                let mut vals = Vec::new();
                (0..(self.analog_out_count as usize)).for_each(|i| {
                    vals.push(self.outputs[i]);
                });
                if self.has_adat {
                    (0..8).for_each(|i| {
                        vals.push(self.outputs[8 + i]);
                    });
                }
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::DETECTED_CLK_SRC_NAME => {
                elem_value.set_enum(&[self.src]);
                Ok(true)
            }
            Self::DETECTED_CLK_RATE_NAME => {
                elem_value.set_enum(&[self.rate]);
                Ok(true)
            }
            Self::MONITOR_METER_NAME => {
                elem_value.set_int(&self.monitor_meters);
                Ok(true)
            }
            Self::ANALOG_MIXER_METER_NAME => {
                elem_value.set_int(&self.analog_mixer_meters);
                Ok(true)
            }
            Self::MONITOR_MODE_NAME => {
                elem_value.set_enum(&[self.monitor_mode]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
