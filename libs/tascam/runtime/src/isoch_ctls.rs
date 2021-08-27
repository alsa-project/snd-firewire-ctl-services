// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use glib::Error;

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt};

use core::card_cntr::*;

use tascam_protocols::isoch::*;

const MONITOR_ROTARY_NAME: &str = "monitor-rotary";
const SOLO_ROTARY_NAME: &str = "solo-rotary";
const INPUT_METER_NAME: &str = "input-meters";
const OUTPUT_METER_NAME: &str = "output-meters";
const DETECTED_CLK_SRC_NAME: &str = "detected-clock-source";
const DETECTED_CLK_RATE_NAME: &str = "detected-clock-rate";
const MONITOR_METER_NAME: &str = "monitor-meters";
const ANALOG_MIXER_METER_NAME: &str = "analog-mixer-meters";
const MONITOR_MODE_NAME: &str = "monitor-mode";

fn clk_src_to_str(src: &Option<ClkSrc>) -> &'static str {
    match src {
        Some(ClkSrc::Internal) => "Internal",
        Some(ClkSrc::Wordclock) => "Word-clock",
        Some(ClkSrc::Spdif) => "S/PDIF",
        Some(ClkSrc::Adat) => "ADAT",
        None => "N/A",
    }
}

fn clk_rate_to_str(rate: &Option<ClkRate>) -> &'static str {
    match rate {
        Some(ClkRate::R44100) => "44100",
        Some(ClkRate::R48000) => "48000",
        Some(ClkRate::R88200) => "88200",
        Some(ClkRate::R96000) => "i96000",
        None => "N/A",
    }
}

fn monitor_mode_to_str(mode: &MonitorMode) -> &'static str {
    match mode {
        MonitorMode::Computer => "computer",
        MonitorMode::Inputs => "input",
        MonitorMode::Both => "both",
    }
}

pub trait IsochMeterCtl<T: IsochMeterOperation>:
    AsMut<IsochMeterState> + AsRef<IsochMeterState>
{
    const INPUT_LABELS: &'static [&'static str];
    const OUTPUT_LABELS: &'static [&'static str];

    const CLK_SRCS: [Option<ClkSrc>; 5] = [
        Some(ClkSrc::Internal),
        Some(ClkSrc::Wordclock),
        Some(ClkSrc::Spdif),
        Some(ClkSrc::Adat),
        None,
    ];

    const CLK_RATES: [Option<ClkRate>; 5] = [
        Some(ClkRate::R44100),
        Some(ClkRate::R48000),
        Some(ClkRate::R88200),
        Some(ClkRate::R96000),
        None,
    ];

    const MONITOR_MODES: [MonitorMode; 3] = [
        MonitorMode::Computer,
        MonitorMode::Inputs,
        MonitorMode::Both,
    ];

    fn parse_state(&mut self, image: &[u32]) -> Result<(), Error> {
        T::parse_meter_state(self.as_mut(), image)
    }

    fn load_state(
        &mut self,
        card_cntr: &mut CardCntr,
        image: &[u32],
    ) -> Result<Vec<ElemId>, Error> {
        assert_eq!(Self::INPUT_LABELS.len(), T::INPUT_COUNT);
        assert_eq!(Self::OUTPUT_LABELS.len(), T::OUTPUT_COUNT);

        let mut measured_elem_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_ROTARY_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::ROTARY_MIN as i32,
                T::ROTARY_MAX as i32,
                T::ROTARY_STEP as i32,
                1,
                None,
                false,
            )
            .map(|mut elem_id_list| measured_elem_list.append(&mut elem_id_list))?;

        if T::HAS_SOLO {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, SOLO_ROTARY_NAME, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    T::ROTARY_MIN as i32,
                    T::ROTARY_MAX as i32,
                    T::ROTARY_STEP as i32,
                    1,
                    None,
                    false,
                )
                .map(|mut elem_id_list| measured_elem_list.append(&mut elem_id_list))?;
        }

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                T::INPUT_COUNT,
                None,
                false,
            )
            .map(|mut elem_id_list| measured_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                T::OUTPUT_COUNT,
                None,
                false,
            )
            .map(|mut elem_id_list| measured_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                2,
                None,
                false,
            )
            .map(|mut elem_id_list| measured_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ANALOG_MIXER_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                2,
                None,
                false,
            )
            .map(|mut elem_id_list| measured_elem_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::CLK_SRCS.iter().map(|s| clk_src_to_str(s)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, DETECTED_CLK_SRC_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| measured_elem_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::CLK_RATES.iter().map(|s| clk_rate_to_str(s)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, DETECTED_CLK_RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| measured_elem_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::MONITOR_MODES
            .iter()
            .map(|s| monitor_mode_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| measured_elem_list.append(&mut elem_id_list))?;

        *self.as_mut() = T::create_meter_state();
        self.parse_state(image)?;

        Ok(measured_elem_list)
    }

    fn read_state(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MONITOR_ROTARY_NAME => {
                elem_value.set_int(&[self.as_ref().monitor as i32]);
                Ok(true)
            }
            SOLO_ROTARY_NAME => {
                elem_value.set_int(&[self.as_ref().solo.unwrap() as i32]);
                Ok(true)
            }
            INPUT_METER_NAME => {
                let vals: Vec<i32> = self.as_ref().inputs.iter().map(|&l| l as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            OUTPUT_METER_NAME => {
                let vals: Vec<i32> = self.as_ref().outputs.iter().map(|&l| l as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            DETECTED_CLK_SRC_NAME => {
                let pos = Self::CLK_SRCS
                    .iter()
                    .position(|s| s.eq(&self.as_ref().src))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            DETECTED_CLK_RATE_NAME => {
                let pos = Self::CLK_RATES
                    .iter()
                    .position(|r| r.eq(&self.as_ref().rate))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            MONITOR_METER_NAME => {
                let vals: Vec<i32> = self
                    .as_ref()
                    .monitor_meters
                    .iter()
                    .map(|&l| l as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            ANALOG_MIXER_METER_NAME => {
                let vals: Vec<i32> = self
                    .as_ref()
                    .analog_mixer_meters
                    .iter()
                    .map(|&l| l as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MONITOR_MODE_NAME => {
                let pos = Self::MONITOR_MODES
                    .iter()
                    .position(|m| m.eq(&self.as_ref().monitor_mode))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
