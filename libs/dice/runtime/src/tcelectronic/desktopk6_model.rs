// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt};

use hinawa::FwReq;
use hinawa::{SndDice, SndUnitExt};

use core::card_cntr::*;
use core::elem_value_accessor::*;

use alsa_ctl_tlv_codec::items::DbInterval;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::tcelectronic::{*, desktop::*};

use crate::common_ctl::*;
use super::{fw_led_ctl::*, standalone_ctl::*};

#[derive(Default)]
pub struct Desktopk6Model{
    req: FwReq,
    sections: GeneralSections,
    segments: DesktopSegments,
    ctl: CommonCtl,
    meter_ctl: MeterCtl,
    panel_ctl: PanelCtl,
    mixer_ctl: MixerCtl,
    standalone_ctl: TcKonnektStandaloneCtl,
    hw_state_ctl: HwStateCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for Desktopk6Model {
    fn load(&mut self, unit: &mut SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let mut node = unit.get_node();

        self.sections = self.req.read_general_sections(&mut node, TIMEOUT_MS)?;
        let caps = self.req.read_clock_caps(&mut node, &self.sections, TIMEOUT_MS)?;
        let src_labels = self.req.read_clock_source_labels(&mut node, &self.sections, TIMEOUT_MS)?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.req.read_segment(&mut node, &mut self.segments.meter, TIMEOUT_MS)?;
        self.req.read_segment(&mut node, &mut self.segments.panel, TIMEOUT_MS)?;
        self.req.read_segment(&mut node, &mut self.segments.mixer, TIMEOUT_MS)?;
        self.req.read_segment(&mut node, &mut self.segments.config, TIMEOUT_MS)?;
        self.req.read_segment(&mut node, &mut self.segments.hw_state, TIMEOUT_MS)?;

        self.meter_ctl.load(&self.segments, card_cntr)?;
        self.panel_ctl.load(card_cntr)?;
        self.mixer_ctl.load(card_cntr)?;
        self.standalone_ctl.load(card_cntr)?;
        self.hw_state_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, unit: &mut SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read(unit, &mut self.req, &self.sections, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.meter_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.panel_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.standalone_ctl.read(&self.segments.config, elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_state_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndDice, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.write(unit, &mut self.req, &self.sections, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.panel_ctl.write(unit, &mut self.req, &mut self.segments, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.write(unit, &mut self.req, &mut self.segments, elem_id, old, new,
                                       TIMEOUT_MS)? {
            Ok(true)
        } else if self.standalone_ctl.write(unit, &mut self.req, &mut self.segments.config, elem_id, new,
                                            TIMEOUT_MS)? {
            Ok(true)
        } else if self.hw_state_ctl.write(unit, &mut self.req, &mut self.segments, elem_id, new,
                                          TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndDice, u32> for Desktopk6Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.panel_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.hw_state_ctl.0);
    }

    fn parse_notification(&mut self, unit: &mut SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &mut self.req, &self.sections, *msg, TIMEOUT_MS)?;

        self.req.parse_notification(&mut unit.get_node(), &mut self.segments.panel, TIMEOUT_MS, msg)?;
        self.req.parse_notification(&mut unit.get_node(), &mut self.segments.hw_state, TIMEOUT_MS, msg)?;

        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.panel_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_state_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<SndDice> for Desktopk6Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
    }

    fn measure_states(&mut self, unit: &mut SndDice) -> Result<(), Error> {
        self.ctl.measure_states(unit, &mut self.req, &self.sections, TIMEOUT_MS)?;

        self.req.read_segment(&mut unit.get_node(), &mut self.segments.meter, TIMEOUT_MS)?;

        Ok(())
    }

    fn measure_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Debug)]
pub struct MeterCtl(Vec<ElemId>);

impl MeterCtl {
    const ANALOG_IN_NAME: &'static str = "analog-input-meters";
    const MIXER_OUT_NAME: &'static str = "mixer-output-meters";
    const STREAM_IN_NAME: &'static str = "stream-input-meters";

    const METER_MIN: i32 = -1000;
    const METER_MAX: i32 = 0;
    const METER_STEP: i32 = 1;
    const METER_TLV: DbInterval = DbInterval{min: -9400, max: 0, linear: false, mute_avail: false};

    fn load(
        &mut self,
        segments: &DesktopSegments,
        card_cntr: &mut CardCntr
    ) -> Result<(), Error> {
        let labels = (0..segments.meter.data.analog_inputs.len())
            .map(|i| format!("Analog-input-{}", i))
            .collect::<Vec<_>>();
        self.add_meter_elem(card_cntr, Self::ANALOG_IN_NAME, &labels)?;

        let labels = (0..segments.meter.data.mixer_outputs.len())
            .map(|i| format!("Mixer-output-{}", i))
            .collect::<Vec<_>>();
        self.add_meter_elem(card_cntr, Self::MIXER_OUT_NAME, &labels)?;

        let labels = (0..segments.meter.data.stream_inputs.len())
            .map(|i| format!("Stream-input-{}", i))
            .collect::<Vec<_>>();
        self.add_meter_elem(card_cntr, Self::STREAM_IN_NAME, &labels)?;

        Ok(())
    }

    fn add_meter_elem<T: AsRef<str>>(
        &mut self,
        card_cntr:&mut CardCntr,
        name: &str,
        labels: &[T]
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::METER_MIN, Self::METER_MAX, Self::METER_STEP,
                                labels.len(), Some(&Into::<Vec<u32>>::into(Self::METER_TLV)), false)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))
    }

    fn read(
        &self,
        segments: &DesktopSegments,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::ANALOG_IN_NAME => {
                elem_value.set_int(&segments.meter.data.analog_inputs);
                Ok(true)
            }
            Self::MIXER_OUT_NAME => {
                elem_value.set_int(&segments.meter.data.mixer_outputs);
                Ok(true)
            }
            Self::STREAM_IN_NAME => {
                elem_value.set_int(&segments.meter.data.stream_inputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct PanelCtl{
    notified_elem_list: Vec<ElemId>,
    fw_led_ctl: FwLedCtl,
}

impl PanelCtl {
    const PANEL_BUTTON_COUNT_NAME: &'static str = "panel-button-count";
    const MIXER_OUT_VOL: &'static str = "mixer-output-volume";
    const PHONE_KNOB_VALUE_NAME: &'static str = "phone-knob-value";
    const MIX_KNOB_VALUE_NAME: &'static str = "mix-knob-value";
    const REVERB_LED_STATE_NAME: &'static str = "reverb-led-state";
    const REVERB_KNOB_VALUE_NAME: &'static str = "reverb-knob-value";

    const KNOB_MIN: i32 = -1000;
    const KNOB_MAX: i32 = 0;
    const KNOB_STEP: i32 = 1;

    const MIX_MIN: i32 = 0;
    const MIX_MAX: i32 = 1000;
    const MIX_STEP: i32 = 1;

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::PANEL_BUTTON_COUNT_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, 0, i32::MAX, 1, 1, None, false)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_OUT_VOL, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::KNOB_MIN, Self::KNOB_MAX, Self::KNOB_STEP,
                                1, None, false)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::PHONE_KNOB_VALUE_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::KNOB_MIN, Self::KNOB_MAX, Self::KNOB_STEP,
                                1, None, false)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MIX_KNOB_VALUE_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::MIX_MIN, Self::MIX_MAX, Self::MIX_STEP,
                                1, None, false)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::REVERB_LED_STATE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::REVERB_KNOB_VALUE_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::KNOB_MIN, Self::KNOB_MAX, Self::KNOB_STEP,
                                1, None, false)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        self.fw_led_ctl.load(card_cntr)?;
        self.notified_elem_list.extend_from_slice(&self.fw_led_ctl.0);

        Ok(())
    }

    fn read(
        &mut self,
        segments: &DesktopSegments,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::PANEL_BUTTON_COUNT_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(segments.panel.data.panel_button_count as i32)
                })
                .map(|_| true)
            }
            Self::MIXER_OUT_VOL => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(segments.panel.data.main_knob_value)
                })
                .map(|_| true)
            }
            Self::PHONE_KNOB_VALUE_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(segments.panel.data.phone_knob_value)
                })
                .map(|_| true)
            }
            Self::MIX_KNOB_VALUE_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(segments.panel.data.mix_knob_value as i32)
                })
                .map(|_| true)
            }
            Self::REVERB_LED_STATE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.panel.data.reverb_led_on)
                })
                .map(|_| true)
            }
            Self::REVERB_KNOB_VALUE_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(segments.panel.data.reverb_knob_value)
                })
                .map(|_| true)
            }
            _ => self.fw_led_ctl.read(&segments.panel, elem_id, elem_value),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        segments: &mut DesktopSegments,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::REVERB_LED_STATE_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    segments.panel.data.reverb_led_on = val;
                    req.write_segment(&mut unit.get_node(), &mut segments.panel, timeout_ms)
                })
                .map(|_| true)
            }
            _ => self.fw_led_ctl.write(unit, req, &mut segments.panel, elem_id, elem_value, timeout_ms),
        }
    }
}

fn hp_src_to_str(src: &DesktopHpSrc) -> &'static str {
    match src {
        DesktopHpSrc::Stream23 => "Stream-3/4",
        DesktopHpSrc::Mixer01 => "Mixer-out-1/2",
    }
}

#[derive(Default, Debug)]
struct MixerCtl;

impl MixerCtl {
    const MIXER_MIC_INST_SRC_LEVEL_NAME: &'static str = "mixer-mic-inst-source-level";
    const MIXER_MIC_INST_SRC_BALANCE_NAME: &'static str = "mixer-mic-inst-source-pan";
    const MIXER_MIC_INST_SRC_SEND_NAME: &'static str = "mixer-mic-inst-source-send";

    const MIXER_DUAL_INST_SRC_LEVEL_NAME: &'static str = "mixer-dual-inst-source-level";
    const MIXER_DUAL_INST_SRC_BALANCE_NAME: &'static str = "mixer-dual-inst-source-pan";
    const MIXER_DUAL_INST_SRC_SEND_NAME: &'static str = "mixer-dual-inst-source-send";

    const MIXER_STEREO_IN_SRC_LEVEL_NAME: &'static str = "mixer-stereo-input-source-level";
    const MIXER_STEREO_IN_SRC_BALANCE_NAME: &'static str = "mixer-stereo-input-source-pan";
    const MIXER_STEREO_IN_SRC_SEND_NAME: &'static str = "mixer-stereo-input-source-send";

    const HP_SRC_NAME: &'static str = "headphone-source";

    const LEVEL_MIN: i32 = -1000;
    const LEVEL_MAX: i32 = 0;
    const LEVEL_STEP: i32 = 1;
    const LEVEL_TLV: DbInterval = DbInterval{min: -9400, max: 0, linear: false, mute_avail: false};

    const BALANCE_MIN: i32 = -50;
    const BALANCE_MAX: i32 = 50;
    const BALANCE_STEP: i32 = 1;

    const HP_SRCS: [DesktopHpSrc;2] = [DesktopHpSrc::Stream23, DesktopHpSrc::Mixer01];

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_MIC_INST_SRC_LEVEL_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                        2, Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)), true)?;
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_MIC_INST_SRC_BALANCE_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, Self::BALANCE_MIN, Self::BALANCE_MAX, Self::BALANCE_STEP,
                                        2, None, true)?;
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_MIC_INST_SRC_SEND_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                        2, Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_DUAL_INST_SRC_LEVEL_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                        2, Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)), true)?;
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_DUAL_INST_SRC_BALANCE_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, Self::BALANCE_MIN, Self::BALANCE_MAX, Self::BALANCE_STEP,
                                        2, None, true)?;
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_DUAL_INST_SRC_SEND_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                        2, Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_STEREO_IN_SRC_LEVEL_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                        1, Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)), true)?;
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_STEREO_IN_SRC_BALANCE_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, Self::BALANCE_MIN, Self::BALANCE_MAX, Self::BALANCE_STEP,
                                        1, None, true)?;
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_STEREO_IN_SRC_SEND_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                        1, Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)), true)?;

        let labels: Vec<&str> = Self::HP_SRCS.iter().map(|s| hp_src_to_str(s)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::HP_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read(
        &mut self,
        segments: &DesktopSegments,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::MIXER_MIC_INST_SRC_LEVEL_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| {
                    Ok(segments.mixer.data.mic_inst_level[idx])
                })
                .map(|_| true)
            }
            Self::MIXER_MIC_INST_SRC_BALANCE_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| {
                    Ok(segments.mixer.data.mic_inst_pan[idx])
                })
                .map(|_| true)
            }
            Self::MIXER_MIC_INST_SRC_SEND_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| {
                    Ok(segments.mixer.data.mic_inst_send[idx])
                })
                .map(|_| true)
            }
            Self::MIXER_DUAL_INST_SRC_LEVEL_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| {
                    Ok(segments.mixer.data.dual_inst_level[idx])
                })
                .map(|_| true)
            }
            Self::MIXER_DUAL_INST_SRC_BALANCE_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| {
                    Ok(segments.mixer.data.dual_inst_pan[idx])
                })
                .map(|_| true)
            }
            Self::MIXER_DUAL_INST_SRC_SEND_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| {
                    Ok(segments.mixer.data.dual_inst_send[idx])
                })
                .map(|_| true)
            }
            Self::MIXER_STEREO_IN_SRC_LEVEL_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(segments.mixer.data.stereo_in_level)
                })
                .map(|_| true)
            }
            Self::MIXER_STEREO_IN_SRC_BALANCE_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(segments.mixer.data.stereo_in_pan)
                })
                .map(|_| true)
            }
            Self::MIXER_STEREO_IN_SRC_SEND_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(segments.mixer.data.stereo_in_send)
                })
                .map(|_| true)
            }
            Self::HP_SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::HP_SRCS.iter()
                        .position(|&s| s == segments.mixer.data.hp_src)
                        .expect("Programming error");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        segments: &mut DesktopSegments,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::MIXER_MIC_INST_SRC_LEVEL_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    segments.mixer.data.mic_inst_level[idx] = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.mixer, timeout_ms))
                .map(|_| true)
            }
            Self::MIXER_MIC_INST_SRC_BALANCE_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    segments.mixer.data.mic_inst_pan[idx] = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.mixer, timeout_ms))
                .map(|_| true)
            }
            Self::MIXER_MIC_INST_SRC_SEND_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    segments.mixer.data.mic_inst_send[idx] = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.mixer, timeout_ms))
                .map(|_| true)
            }
            Self::MIXER_DUAL_INST_SRC_LEVEL_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    segments.mixer.data.dual_inst_level[idx] = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.mixer, timeout_ms))
                .map(|_| true)
            }
            Self::MIXER_DUAL_INST_SRC_BALANCE_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    segments.mixer.data.dual_inst_pan[idx] = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.mixer, timeout_ms))
                .map(|_| true)
            }
            Self::MIXER_DUAL_INST_SRC_SEND_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    segments.mixer.data.dual_inst_send[idx] = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.mixer, timeout_ms))
                .map(|_| true)
            }
            Self::MIXER_STEREO_IN_SRC_LEVEL_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    segments.mixer.data.stereo_in_level = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.mixer, timeout_ms))
                .map(|_| true)
            }
            Self::MIXER_STEREO_IN_SRC_BALANCE_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    segments.mixer.data.stereo_in_pan = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.mixer, timeout_ms))
                .map(|_| true)
            }
            Self::MIXER_STEREO_IN_SRC_SEND_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    segments.mixer.data.stereo_in_send = val;
                    Ok(())
                })
                .and_then(|_| req.write_segment(&mut unit.get_node(), &mut segments.mixer, timeout_ms))
                .map(|_| true)
            }
            Self::HP_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    Self::HP_SRCS.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of headphone source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .and_then(|&s| {
                            segments.mixer.data.hp_src = s;
                            req.write_segment(&mut unit.get_node(), &mut segments.mixer, timeout_ms)
                        })
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

fn meter_target_to_str(target: &MeterTarget) -> &'static str {
    match target {
        MeterTarget::Input => "Input",
        MeterTarget::Pre => "Pre",
        MeterTarget::Post => "Post",
    }
}

fn input_scene_to_str(scene: &InputScene) -> &'static str {
    match scene {
        InputScene::MicInst => "Mic-inst",
        InputScene::DualInst => "Dual-inst",
        InputScene::StereoIn => "Stereo-in",
    }
}

#[derive(Default, Debug)]
struct HwStateCtl(Vec<ElemId>);

impl HwStateCtl {
    const METER_TARGET_NAME: &'static str = "meter-target";
    const MIXER_OUT_MONAURAL_NAME: &'static str = "mixer-out-monaural";
    const KNOB_ASSIGN_TO_HP_NAME: &'static str = "knob-assign-to-headphone";
    const MIXER_OUTPUT_DIM_ENABLE_NAME: &'static str = "mixer-output-dim-enable";
    const MIXER_OUTPUT_DIM_LEVEL_NAME: &'static str = "mixer-output-dim-level";
    const SCENE_NAME: &'static str = "scene-select";
    const REVERB_TO_MAIN_NAME: &'static str = "reverb-to-main";
    const REVERB_TO_HP_NAME: &'static str = "reverb-to-hp";
    const KNOB_BACKLIGHT_NAME: &'static str = "knob-backlight";
    const MIC_0_PHANTOM_NAME: &'static str = "mic-1-phantom";
    const MIC_0_BOOST_NAME: &'static str = "mic-1-boost";

    const METER_TARGETS: [MeterTarget;3] = [
        MeterTarget::Input,
        MeterTarget::Pre,
        MeterTarget::Post,
    ];

    const INPUT_SCENES: [InputScene;3] = [
        InputScene::MicInst,
        InputScene::DualInst,
        InputScene::StereoIn,
    ];

    const DIM_LEVEL_MIN: i32 = -1000;
    const DIM_LEVEL_MAX: i32 = -60;
    const DIM_LEVEL_STEP: i32 = 1;
    const DIM_LEVEL_TLV: DbInterval = DbInterval{min: -9400, max: -600, linear: false, mute_avail: false};

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::METER_TARGETS.iter()
            .map(|l| meter_target_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::METER_TARGET_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MIXER_OUT_MONAURAL_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::KNOB_ASSIGN_TO_HP_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_OUTPUT_DIM_ENABLE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_OUTPUT_DIM_LEVEL_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, Self::DIM_LEVEL_MIN, Self::DIM_LEVEL_MAX, Self::DIM_LEVEL_STEP,
                                        1, Some(&Into::<Vec<u32>>::into(Self::DIM_LEVEL_TLV)), true)?;

        let labels: Vec<&str> = Self::INPUT_SCENES.iter().map(|l| input_scene_to_str(l)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SCENE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::REVERB_TO_MAIN_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::REVERB_TO_HP_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::KNOB_BACKLIGHT_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MIC_0_PHANTOM_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MIC_0_BOOST_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(
        &mut self,
        segments: &DesktopSegments,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::METER_TARGET_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::METER_TARGETS.iter()
                        .position(|&t| t == segments.hw_state.data.meter_target)
                        .expect("Programming error");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::MIXER_OUT_MONAURAL_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.hw_state.data.mixer_output_monaural)
                })
                .map(|_| true)
            }
            Self::KNOB_ASSIGN_TO_HP_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.hw_state.data.knob_assign_to_hp)
                })
                .map(|_| true)
            }
            Self::MIXER_OUTPUT_DIM_ENABLE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.hw_state.data.mixer_output_dim_enabled)
                })
                .map(|_| true)
            }
            Self::MIXER_OUTPUT_DIM_LEVEL_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(segments.hw_state.data.mixer_output_dim_volume)
                })
                .map(|_| true)
            }
            Self::SCENE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::INPUT_SCENES.iter()
                        .position(|&s| s == segments.hw_state.data.input_scene)
                        .expect("Programming error");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::REVERB_TO_MAIN_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.hw_state.data.reverb_to_master)
                })
                .map(|_| true)
            }
            Self::REVERB_TO_HP_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.hw_state.data.reverb_to_hp)
                })
                .map(|_| true)
            }
            Self::KNOB_BACKLIGHT_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.hw_state.data.master_knob_backlight)
                })
                .map(|_| true)
            }
            Self::MIC_0_PHANTOM_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.hw_state.data.mic_0_phantom)
                })
                .map(|_| true)
            }
            Self::MIC_0_BOOST_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.hw_state.data.mic_0_boost)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        segments: &mut DesktopSegments,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::METER_TARGET_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::METER_TARGETS.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of meter target: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .and_then(|&target| {
                            segments.hw_state.data.meter_target = target;
                            req.write_segment(&mut unit.get_node(), &mut segments.hw_state, timeout_ms)
                        })
                })
                .map(|_| true)
            }
            Self::MIXER_OUT_MONAURAL_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    segments.hw_state.data.mixer_output_monaural = val;
                    req.write_segment(&mut unit.get_node(), &mut segments.hw_state, timeout_ms)
                })
                .map(|_| true)
            }
            Self::KNOB_ASSIGN_TO_HP_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    segments.hw_state.data.knob_assign_to_hp = val;
                    req.write_segment(&mut unit.get_node(), &mut segments.hw_state, timeout_ms)
                })
                .map(|_| true)
            }
            Self::MIXER_OUTPUT_DIM_ENABLE_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    segments.hw_state.data.mixer_output_dim_enabled = val;
                    req.write_segment(&mut unit.get_node(), &mut segments.hw_state, timeout_ms)
                })
                .map(|_| true)
            }
            Self::MIXER_OUTPUT_DIM_LEVEL_NAME=> {
                ElemValueAccessor::<i32>::get_val(elem_value, |val| {
                    segments.hw_state.data.mixer_output_dim_volume = val;
                    req.write_segment(&mut unit.get_node(), &mut segments.hw_state, timeout_ms)
                })
                .map(|_| true)
            }
            Self::SCENE_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::INPUT_SCENES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of input scene: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .and_then(|&scene| {
                            segments.hw_state.data.input_scene = scene;
                            req.write_segment(&mut unit.get_node(), &mut segments.hw_state, timeout_ms)
                        })
                })
                .map(|_| true)
            }
            Self::REVERB_TO_MAIN_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    segments.hw_state.data.reverb_to_master = val;
                    req.write_segment(&mut unit.get_node(), &mut segments.hw_state, timeout_ms)
                })
                .map(|_| true)
            }
            Self::REVERB_TO_HP_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    segments.hw_state.data.reverb_to_hp = val;
                    req.write_segment(&mut unit.get_node(), &mut segments.hw_state, timeout_ms)
                })
                .map(|_| true)
            }
            Self::KNOB_BACKLIGHT_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    segments.hw_state.data.master_knob_backlight = val;
                    req.write_segment(&mut unit.get_node(), &mut segments.hw_state, timeout_ms)
                })
                .map(|_| true)
            }
            Self::MIC_0_PHANTOM_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    segments.hw_state.data.mic_0_phantom = val;
                    req.write_segment(&mut unit.get_node(), &mut segments.hw_state, timeout_ms)
                })
                .map(|_| true)
            }
            Self::MIC_0_BOOST_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    segments.hw_state.data.mic_0_boost = val;
                    req.write_segment(&mut unit.get_node(), &mut segments.hw_state, timeout_ms)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
