// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use hinawa::FwReq;
use hinawa::{SndDice, SndUnitExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;
use core::elem_value_accessor::*;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::tcelectronic::*;
use dice_protocols::tcelectronic::studio::*;

use crate::common_ctl::*;
use super::ch_strip_ctl::*;
use super::reverb_ctl::*;
use super::fw_led_ctl::*;
use super::standalone_ctl::*;
use super::midi_send_ctl::*;
use super::prog_ctl::*;

#[derive(Default)]
pub struct Studiok48Model{
    proto: Studiok48Proto,
    sections: GeneralSections,
    segments: StudioSegments,
    ctl: CommonCtl,
    ch_strip_ctl: ChStripCtl,
    reverb_ctl: ReverbCtl,
    hw_state_ctl: HwStateCtl,
    phys_out_ctl: PhysOutCtl,
    mixer_ctl: MixerCtl,
    config_ctl: ConfigCtl,
    remote_ctl: RemoteCtl,
    lineout_ctl: LineoutCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for Studiok48Model {
    fn load(&mut self, unit: &SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let node = unit.get_node();

        self.sections = self.proto.read_general_sections(&node, TIMEOUT_MS)?;
        let caps = self.proto.read_clock_caps(&node, &self.sections, TIMEOUT_MS)?;
        let src_labels = self.proto.read_clock_source_labels(&node, &self.sections, TIMEOUT_MS)?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.ch_strip_ctl.load(unit, &self.proto, &mut self.segments.ch_strip_state,
                               &mut self.segments.ch_strip_meter, TIMEOUT_MS, card_cntr)?;
        self.reverb_ctl.load(unit, &self.proto, &mut self.segments.reverb_state, &mut self.segments.reverb_meter,
                             TIMEOUT_MS, card_cntr)?;

        let node = unit.get_node();
        self.proto.read_segment(&node, &mut self.segments.hw_state, TIMEOUT_MS)?;
        self.proto.read_segment(&node, &mut self.segments.phys_out, TIMEOUT_MS)?;
        self.proto.read_segment(&node, &mut self.segments.mixer_state, TIMEOUT_MS)?;
        self.proto.read_segment(&node, &mut self.segments.config, TIMEOUT_MS)?;
        self.proto.read_segment(&node, &mut self.segments.remote, TIMEOUT_MS)?;
        self.proto.read_segment(&node, &mut self.segments.out_level, TIMEOUT_MS)?;

        self.hw_state_ctl.load(card_cntr)?;
        self.phys_out_ctl.load(card_cntr)?;
        self.mixer_ctl.load(&self.segments, card_cntr)?;
        self.config_ctl.load(card_cntr)?;
        self.remote_ctl.load(card_cntr)?;
        self.lineout_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, unit: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read(unit, &self.proto, &self.sections, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.ch_strip_ctl.read(&self.segments.ch_strip_state, &self.segments.ch_strip_meter,
                                         elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read(&self.segments.reverb_state, &self.segments.reverb_meter,
                                       elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_state_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_out_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.config_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.remote_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.lineout_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &SndDice, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.write(unit, &self.proto, &self.sections, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.ch_strip_ctl.write(unit, &self.proto, &mut self.segments.ch_strip_state, elem_id,
                                          old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.reverb_ctl.write(unit, &self.proto, &mut self.segments.reverb_state, elem_id,
                                        new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.hw_state_ctl.write(unit, &self.proto, &mut self.segments, elem_id,
                                          new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.phys_out_ctl.write(unit, &self.proto, &mut self.segments, elem_id, old, new,
                                          TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.write(unit, &self.proto, &mut self.segments, elem_id, old, new,
                                       TIMEOUT_MS)? {
            Ok(true)
        } else if self.config_ctl.write(unit, &self.proto, &mut self.segments, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.remote_ctl.write(unit, &self.proto, &mut self.segments, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.lineout_ctl.write(unit, &self.proto, &mut self.segments, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndDice, u32> for Studiok48Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.ch_strip_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.reverb_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.hw_state_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.phys_out_ctl.0);
        elem_id_list.extend_from_slice(&self.mixer_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.remote_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, unit: &SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &self.proto, &self.sections, *msg, TIMEOUT_MS)?;

        let node = unit.get_node();
        self.proto.parse_notification(&node, &mut self.segments.ch_strip_state, TIMEOUT_MS, *msg)?;
        self.proto.parse_notification(&node, &mut self.segments.reverb_state, TIMEOUT_MS, *msg)?;
        self.proto.parse_notification(&node, &mut self.segments.hw_state, TIMEOUT_MS, *msg)?;
        self.proto.parse_notification(&node, &mut self.segments.phys_out, TIMEOUT_MS, msg)?;
        self.proto.parse_notification(&node, &mut self.segments.mixer_state, TIMEOUT_MS, msg)?;
        self.proto.parse_notification(&node, &mut self.segments.remote, TIMEOUT_MS, msg)?;

        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_ctl.read_notified_elem(&self.segments.ch_strip_state, elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read_notified_elem(&self.segments.reverb_state, elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_state_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_out_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.remote_ctl.read(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<hinawa::SndDice> for Studiok48Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.ch_strip_ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.reverb_ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.mixer_ctl.measured_elem_list);
    }

    fn measure_states(&mut self, unit: &SndDice) -> Result<(), Error> {
        self.ctl.measure_states(unit, &self.proto, &self.sections, TIMEOUT_MS)?;
        self.ch_strip_ctl.measure_states(unit, &self.proto, &self.segments.ch_strip_state,
                                         &mut self.segments.ch_strip_meter, TIMEOUT_MS)?;
        self.reverb_ctl.measure_states(unit, &self.proto, &self.segments.reverb_state,
                                       &mut self.segments.reverb_meter, TIMEOUT_MS)?;

        let node = unit.get_node();
        self.proto.read_segment(&node, &mut self.segments.mixer_meter, TIMEOUT_MS)?;

        Ok(())
    }

    fn measure_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_ctl.read_measured_elem(&self.segments.ch_strip_meter, elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read_measured_elem(&self.segments.reverb_meter, elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read_measured_elem(&self.segments, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Debug)]
struct Studiok48Proto(FwReq);

impl AsRef<FwReq> for Studiok48Proto {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

fn analog_jack_state_to_string(state: &StudioAnalogJackState) -> String {
    match state {
        StudioAnalogJackState::FrontSelected => "front-selected",
        StudioAnalogJackState::FrontInserted => "front-inserted",
        StudioAnalogJackState::RearSelected => "rear-selected",
        StudioAnalogJackState::RearInserted => "rear-inserted",
    }.to_string()
}

#[derive(Default, Debug)]
struct HwStateCtl {
    notified_elem_list: Vec<ElemId>,
    fw_led_ctl: FwLedCtl,
}

impl<'a> HwStateCtl {
    // TODO: For Jack detection in ALSA applications.
    const ANALOG_JACK_STATE_NAME: &'a str = "analog-jack-state";
    const HP_JACK_STATE_NAME: &'a str = "headphone-jack-state";
    const VALID_MASTER_LEVEL_NAME: &'a str = "valid-master-level";

    const ANALOG_JACK_STATES: &'a [StudioAnalogJackState] = &[
        StudioAnalogJackState::FrontSelected,
        StudioAnalogJackState::FrontInserted,
        StudioAnalogJackState::RearSelected,
        StudioAnalogJackState::RearInserted,
    ];

    fn load(&mut self, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        let labels = Self::ANALOG_JACK_STATES.iter()
            .map(|s| analog_jack_state_to_string(s))
            .collect::<Vec<_>>();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::ANALOG_JACK_STATE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, STUDIO_ANALOG_JACK_STATE_COUNT, &labels, None, false)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::HP_JACK_STATE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 2, false)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::VALID_MASTER_LEVEL_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        self.fw_led_ctl.load(card_cntr)
            .map(|_| self.notified_elem_list.extend_from_slice(&self.fw_led_ctl.0))?;

        Ok(())
    }

    fn read(&mut self, segments: &StudioSegments, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.fw_led_ctl.read(&segments.hw_state, elem_id, elem_value)? {
            Ok(true)
        } else if self.read_notified_elem(&segments, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &SndDice, proto: &Studiok48Proto, segments: &mut StudioSegments,
                 elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        self.fw_led_ctl.write(unit, proto, &mut segments.hw_state, elem_id, elem_value, timeout_ms)
    }

    fn read_notified_elem(&mut self, segments: &StudioSegments, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::ANALOG_JACK_STATE_NAME => {
                let analog_jack_states = &segments.hw_state.data.analog_jack_states;
                ElemValueAccessor::<u32>::set_vals(elem_value, analog_jack_states.len(), |idx| {
                    let pos = Self::ANALOG_JACK_STATES.iter()
                        .position(|s| s.eq(&analog_jack_states[idx]))
                        .expect("Programming error");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::HP_JACK_STATE_NAME => {
                elem_value.set_bool(&segments.hw_state.data.hp_state);
                Ok(true)
            }
            Self::VALID_MASTER_LEVEL_NAME => {
                elem_value.set_bool(&[segments.hw_state.data.valid_master_level]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

fn src_pair_entry_to_string(entry: &SrcEntry) -> String {
    match entry {
        SrcEntry::Unused => "Unused".to_string(),
        SrcEntry::Analog(ch) => format!("Analog-{}", ch + 1),
        SrcEntry::Spdif(ch) => format!("S/PDIF-{}", ch + 1),
        SrcEntry::Adat(ch) => format!("ADAT-{}", ch + 1),
        SrcEntry::StreamA(ch) => format!("Stream-A-{}", ch + 1),
        SrcEntry::StreamB(ch) => format!("Stream-B-{}", ch + 1),
        SrcEntry::Mixer(ch) => {
            if *ch < 2 {
                format!("Mixer-{}", ch + 1)
            } else if *ch < 6 {
                format!("Aux-{}", ch - 1)
            } else {
                format!("Reverb-{}", ch - 5)
            }
        }
    }
}

#[derive(Default, Debug)]
struct PhysOutCtl(pub Vec<ElemId>);

impl<'a> PhysOutCtl {
    const MASTER_OUT_DIM_NAME: &'a str = "master-out-dim";
    const MASTER_OUT_VOL_NAME: &'a str = "master-out-volume";
    const MASTER_OUT_DIM_VOL_NAME: &'a str = "master-out-dim-volume";

    const OUT_STEREO_LINK_NAME: &'a str = "output-stereo-link";
    const OUT_MUTE_NAME: &'a str = "output-mute";
    const OUT_SRC_NAME: &'a str = "output-source";
    const OUT_GRP_SRC_ENABLE_NAME: &'a str = "output-group-source-enable";
    const OUT_GRP_SRC_TRIM_NAME: &'a str = "output-group-source-trim";
    const OUT_GRP_SRC_DELAY_NAME: &'a str = "output-group-source-delay";

    const PHYS_OUT_SRCS: &'a [SrcEntry] = &[
        SrcEntry::Unused,
        SrcEntry::Analog(0), SrcEntry::Analog(1), SrcEntry::Analog(2), SrcEntry::Analog(3),
        SrcEntry::Analog(4), SrcEntry::Analog(5), SrcEntry::Analog(6), SrcEntry::Analog(7),
        SrcEntry::Analog(8), SrcEntry::Analog(9), SrcEntry::Analog(10), SrcEntry::Analog(11),
        SrcEntry::Spdif(0), SrcEntry::Spdif(1),
        SrcEntry::Adat(0), SrcEntry::Adat(1), SrcEntry::Adat(2), SrcEntry::Adat(3),
        SrcEntry::Adat(4), SrcEntry::Adat(5), SrcEntry::Adat(6), SrcEntry::Adat(7),
        SrcEntry::StreamA(0), SrcEntry::StreamA(1), SrcEntry::StreamA(2), SrcEntry::StreamA(3),
        SrcEntry::StreamA(4), SrcEntry::StreamA(5), SrcEntry::StreamA(6), SrcEntry::StreamA(7),
        SrcEntry::StreamA(8), SrcEntry::StreamA(9), SrcEntry::StreamA(10), SrcEntry::StreamA(11),
        SrcEntry::StreamA(12), SrcEntry::StreamA(13), SrcEntry::StreamA(14), SrcEntry::StreamA(15),
        SrcEntry::StreamB(0), SrcEntry::StreamB(1), SrcEntry::StreamB(2), SrcEntry::StreamB(3),
        SrcEntry::StreamB(4), SrcEntry::StreamB(5), SrcEntry::StreamB(6), SrcEntry::StreamB(7),
        SrcEntry::StreamB(8), SrcEntry::StreamB(9), SrcEntry::StreamB(10), SrcEntry::StreamB(11),
        SrcEntry::Mixer(0), SrcEntry::Mixer(1),
        SrcEntry::Mixer(2), SrcEntry::Mixer(3), SrcEntry::Mixer(4), SrcEntry::Mixer(5),
        SrcEntry::Mixer(6), SrcEntry::Mixer(7),
    ];

    const VOL_MIN: i32 = -1000;
    const VOL_MAX: i32 = 0;
    const VOL_STEP: i32 = 1;
    const VOL_TLV: DbInterval = DbInterval{min: -7200, max: 0, linear: false, mute_avail: false};

    const TRIM_MIN: i32 = -20;
    const TRIM_MAX: i32 = 0;
    const TRIM_STEP: i32 = 1;

    const DELAY_MIN: i32 = 0;
    const DELAY_MAX: i32 = 30;
    const DELAY_STEP: i32 = 1;

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        // For master output.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MASTER_OUT_DIM_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MASTER_OUT_VOL_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::VOL_MIN, Self::VOL_MAX, Self::VOL_STEP,
                                1, Some(&Into::<Vec<u32>>::into(Self::VOL_TLV)), true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MASTER_OUT_DIM_VOL_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::VOL_MIN, Self::VOL_MAX, Self::VOL_STEP,
                                1, Some(&Into::<Vec<u32>>::into(Self::VOL_TLV)), true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        // For source of output pair.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::OUT_STEREO_LINK_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, STUDIO_PHYS_OUT_PAIR_COUNT, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::OUT_MUTE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, STUDIO_PHYS_OUT_PAIR_COUNT * 2, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let labels: Vec<String> = Self::PHYS_OUT_SRCS.iter()
            .map(|src| src_pair_entry_to_string(src))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::OUT_SRC_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, STUDIO_PHYS_OUT_PAIR_COUNT * 2, &labels, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::OUT_GRP_SRC_ENABLE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, STUDIO_PHYS_OUT_PAIR_COUNT * 2, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::OUT_GRP_SRC_TRIM_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::TRIM_MIN, Self::TRIM_MAX, Self::TRIM_STEP,
                                STUDIO_PHYS_OUT_PAIR_COUNT * 2, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::OUT_GRP_SRC_DELAY_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::DELAY_MIN, Self::DELAY_MAX, Self::DELAY_STEP,
                                STUDIO_PHYS_OUT_PAIR_COUNT * 2, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, segments: &StudioSegments, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MASTER_OUT_DIM_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.phys_out.data.master_out.dim_enabled)
                })
                .map(|_| true)
            }
            Self::MASTER_OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(segments.phys_out.data.master_out.vol)
                })
                .map(|_| true)
            }
            Self::MASTER_OUT_DIM_VOL_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(segments.phys_out.data.master_out.dim_vol)
                })
                .map(|_| true)
            }
            Self::OUT_STEREO_LINK_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, STUDIO_PHYS_OUT_PAIR_COUNT, |idx| {
                    Ok(segments.phys_out.data.out_pair_srcs[idx].stereo_link)
                })
                .map(|_| true)
            }
            Self::OUT_MUTE_NAME => {
                elem_value.set_bool(&segments.phys_out.data.out_mutes);
                Ok(true)
            }
            Self::OUT_SRC_NAME => {
                Self::read_out_src_param(segments, elem_value, |param| {
                     let pos = Self::PHYS_OUT_SRCS.iter()
                        .position(|s| s.eq(&param.src))
                         .expect("Programming error");
                     Ok(pos as u32)
                 })
                .map(|_| true)
            }
            Self::OUT_GRP_SRC_ENABLE_NAME => {
                elem_value.set_bool(&segments.phys_out.data.out_assign_to_grp);
                Ok(true)
            }
            Self::OUT_GRP_SRC_TRIM_NAME => {
                Self::read_out_src_param(segments, elem_value, |param| {
                    Ok(param.vol)
                })
            }
            Self::OUT_GRP_SRC_DELAY_NAME => {
                Self::read_out_src_param(segments, elem_value, |param| {
                    Ok(param.delay)
                })
            }
            _ => Ok(false),
        }
    }

    fn read_out_src_param<T, F>(segments: &StudioSegments, elem_value: &mut ElemValue, cb: F)
        -> Result<bool, Error>
        where F: Fn(&PhysOutSrcParam) -> Result<T, Error>,
              T: Default + Copy + Eq,
              ElemValue: ElemValueAccessor<T>,
    {
        ElemValueAccessor::<T>::set_vals(elem_value, STUDIO_PHYS_OUT_PAIR_COUNT * 2, |idx| {
            let i = idx / 2;
            let ch = idx % 2;
            let param = if ch == 0 {
                &segments.phys_out.data.out_pair_srcs[i].left
            } else {
                &segments.phys_out.data.out_pair_srcs[i].right
            };
            cb(param)
        })
        .map(|_| true)
    }

    fn write(&mut self, unit: &SndDice, proto: &Studiok48Proto, segments: &mut StudioSegments,
             elem_id: &ElemId, old: &ElemValue, new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MASTER_OUT_DIM_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    segments.phys_out.data.master_out.dim_enabled = val;
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            Self::MASTER_OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    segments.phys_out.data.master_out.vol = val;
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            Self::MASTER_OUT_DIM_VOL_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    segments.phys_out.data.master_out.dim_vol = val;
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            Self::OUT_STEREO_LINK_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, STUDIO_PHYS_OUT_PAIR_COUNT, |idx, val| {
                    segments.phys_out.data.out_pair_srcs[idx].stereo_link = val;
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            Self::OUT_MUTE_NAME => {
                new.get_bool(&mut segments.phys_out.data.out_mutes);
                proto.write_segment(&unit.get_node(), &mut segments.phys_out, timeout_ms)
                    .map(|_| true)
            }
            Self::OUT_SRC_NAME => {
                Self::write_out_src_param(unit, proto, segments, new, old, timeout_ms, |param, val: u32| {
                     Self::PHYS_OUT_SRCS.iter()
                         .nth(val as usize)
                         .ok_or_else(|| {
                             let msg = format!("Invalid value for index of source of output: {}", val);
                             Error::new(FileError::Inval, &msg)
                         })
                        .map(|&s| param.src = s)
                 })
            }
            Self::OUT_GRP_SRC_ENABLE_NAME => {
                new.get_bool(&mut segments.phys_out.data.out_assign_to_grp);
                proto.write_segment(&unit.get_node(), &mut segments.phys_out, timeout_ms)
                    .map(|_| true)
            }
            Self::OUT_GRP_SRC_TRIM_NAME => {
                Self::write_out_src_param(unit, proto, segments, new, old, timeout_ms, |param, val| {
                    param.vol = val;
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            Self::OUT_GRP_SRC_DELAY_NAME => {
                Self::write_out_src_param(unit, proto, segments, new, old, timeout_ms, |param, val| {
                    param.delay = val;
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.phys_out, timeout_ms))
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write_out_src_param<T, F>(unit: &SndDice, proto: &Studiok48Proto, segments: &mut StudioSegments,
                                 new: &ElemValue, old: &ElemValue, timeout_ms: u32, cb: F)
        -> Result<bool, Error>
        where F: Fn(&mut PhysOutSrcParam, T) -> Result<(), Error>,
              T: Default + Copy + Eq,
              ElemValue: ElemValueAccessor<T>,
    {
        ElemValueAccessor::<T>::get_vals(new, old, STUDIO_PHYS_OUT_PAIR_COUNT * 2, |idx, val| {
            let i = idx / 2;
            let ch = idx % 2;
            let param = if ch == 0 {
                &mut segments.phys_out.data.out_pair_srcs[i].left
            } else {
                &mut segments.phys_out.data.out_pair_srcs[i].right
            };
            cb(param, val)
        })
        .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.phys_out, timeout_ms))
        .map(|_| true)
    }
}

fn src_pair_mode_to_string(entry: &MonitorSrcPairMode) -> String {
    match entry {
        MonitorSrcPairMode::Inactive => "Inactive",
        MonitorSrcPairMode::Active => "Active",
        MonitorSrcPairMode::Fixed => "Fixed",
    }.to_string()
}

#[derive(Default, Debug)]
pub struct MixerCtl{
    notified_elem_list: Vec<ElemId>,
    measured_elem_list: Vec<ElemId>,
}

impl<'a> MixerCtl {
    const SRC_PAIR_MODE_NAME: &'a str = "mixer-input-mode";
    const SRC_ENTRY_NAME: &'a str = "mixer-input-source";
    const SRC_STEREO_LINK_NAME: &'a str = "mixer-input-stereo-link";
    const SRC_GAIN_NAME: &'a str = "mixer-input-gain";
    const SRC_PAN_NAME: &'a str = "mixer-input-pan";
    const REVERB_SRC_GAIN_NAME: &'a str = "reverb-input-gain";
    const AUX01_SRC_GAIN_NAME: &'a str = "aux-1/2-input-gain";
    const AUX23_SRC_GAIN_NAME: &'a str = "aux-3/4-input-gain";
    const SRC_MUTE_NAME: &'a str = "mixer-input-mute";

    const OUT_DIM_NAME: &'a str = "mixer-output-dim";
    const OUT_VOL_NAME: &'a str = "mixer-output-volume";
    const OUT_DIM_VOL_NAME: &'a str = "mixer-output-dim-volume";
    const REVERB_RETURN_MUTE_NAME: &'a str = "reverb-return-mute";
    const REVERB_RETURN_GAIN_NAME: &'a str = "reverb-return-gain";

    const POST_FADER_NAME: &'a str = "mixer-post-fader";

    const CH_STRIP_AS_PLUGIN_NAME: &'a str = "channel-strip-as-plugin";
    const CH_STRIP_SRC_NAME: &'a str = "channel-strip-source";
    const CH_STRIP_23_AT_MID_RATE: &'a str = "channel-strip-3/4-at-mid-rate";

    const MIXER_ENABLE_NAME: &'a str = "mixer-direct-monitoring";

    const MIXER_INPUT_METER_NAME: &'a str = "mixer-input-meter";
    const MIXER_OUTPUT_METER_NAME: &'a str = "mixer-output-meter";
    const AUX_OUTPUT_METER_NAME: &'a str = "aux-output-meter";

    const SRC_PAIR_MODES: [MonitorSrcPairMode;3] = [
        MonitorSrcPairMode::Inactive,
        MonitorSrcPairMode::Active,
        MonitorSrcPairMode::Fixed,
    ];

    const SRC_PAIR_ENTRIES: &'a [SrcEntry] = &[
        SrcEntry::Unused,
        SrcEntry::Analog(0), SrcEntry::Analog(1), SrcEntry::Analog(2), SrcEntry::Analog(3),
        SrcEntry::Analog(4), SrcEntry::Analog(5), SrcEntry::Analog(6), SrcEntry::Analog(7),
        SrcEntry::Analog(8), SrcEntry::Analog(9), SrcEntry::Analog(10), SrcEntry::Analog(11),
        SrcEntry::Spdif(0), SrcEntry::Spdif(1),
        SrcEntry::Adat(0), SrcEntry::Adat(1), SrcEntry::Adat(2), SrcEntry::Adat(3),
        SrcEntry::Adat(4), SrcEntry::Adat(5), SrcEntry::Adat(6), SrcEntry::Adat(7),
        SrcEntry::StreamA(0), SrcEntry::StreamA(1), SrcEntry::StreamA(2), SrcEntry::StreamA(3),
        SrcEntry::StreamA(4), SrcEntry::StreamA(5), SrcEntry::StreamA(6), SrcEntry::StreamA(7),
        SrcEntry::StreamA(8), SrcEntry::StreamA(9), SrcEntry::StreamA(10), SrcEntry::StreamA(11),
        SrcEntry::StreamA(12), SrcEntry::StreamA(13), SrcEntry::StreamA(14), SrcEntry::StreamA(15),
        SrcEntry::StreamB(0), SrcEntry::StreamB(1), SrcEntry::StreamB(2), SrcEntry::StreamB(3),
        SrcEntry::StreamB(4), SrcEntry::StreamB(5), SrcEntry::StreamB(6), SrcEntry::StreamB(7),
        SrcEntry::StreamB(8), SrcEntry::StreamB(9), SrcEntry::StreamB(10), SrcEntry::StreamB(11),
    ];

    const OUT_LABELS: [&'a str;3] = ["Main-1/2", "Aux-1/2", "Aux-3/4"];
    const SEND_TARGET_LABELS: [&'a str;3] = ["Reverb-1/2", "Aux-1/2", "Aux-3/4"];

    const LEVEL_MIN: i32 = -1000;
    const LEVEL_MAX: i32 = 0;
    const LEVEL_STEP: i32 = 1;
    const LEVEL_TLV: DbInterval = DbInterval{min: -7200, max: 0, linear: false, mute_avail: false};

    const PAN_MIN: i32 = -50;
    const PAN_MAX: i32 = 50;
    const PAN_STEP: i32 = 1;

    fn load(&mut self, segments: &StudioSegments, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let state = &segments.mixer_state.data;
        let labels: Vec<String> = (0..state.src_pairs.len())
            .map(|i| format!("Mixer-source-{}/{}", i + 1, i + 2))
            .collect();
        let item_labels: Vec<String> = Self::SRC_PAIR_MODES.iter()
            .map(|m| src_pair_mode_to_string(m))
            .collect();
        self.state_add_elem_enum(card_cntr, Self::SRC_PAIR_MODE_NAME, 1, labels.len(), &item_labels)?;
        self.state_add_elem_bool(card_cntr, Self::SRC_STEREO_LINK_NAME, 1, labels.len())?;

        let labels: Vec<String> = (0..(state.src_pairs.len() * 2))
            .map(|i| format!("Mixer-source-{}", i + 1))
            .collect();
        let item_labels: Vec<String> = Self::SRC_PAIR_ENTRIES.iter()
            .map(|s| src_pair_entry_to_string(s))
            .collect();
        self.state_add_elem_enum(card_cntr, Self::SRC_ENTRY_NAME, 1, labels.len(), &item_labels)?;
        self.state_add_elem_level(card_cntr, Self::SRC_GAIN_NAME, 1, labels.len())?;
        self.state_add_elem_pan(card_cntr, Self::SRC_PAN_NAME, 1, labels.len())?;
        self.state_add_elem_level(card_cntr, Self::REVERB_SRC_GAIN_NAME, 1, labels.len())?;
        self.state_add_elem_level(card_cntr, Self::AUX01_SRC_GAIN_NAME, 1, labels.len())?;
        self.state_add_elem_level(card_cntr, Self::AUX23_SRC_GAIN_NAME, 1, labels.len())?;
        self.state_add_elem_bool(card_cntr, Self::SRC_MUTE_NAME, 1, labels.len())?;

        let labels = &Self::OUT_LABELS;
        self.state_add_elem_bool(card_cntr, Self::REVERB_RETURN_MUTE_NAME, 1, labels.len())?;
        self.state_add_elem_level(card_cntr, Self::REVERB_RETURN_GAIN_NAME, 1, labels.len())?;
        self.state_add_elem_bool(card_cntr, Self::OUT_DIM_NAME, 1, labels.len())?;
        self.state_add_elem_level(card_cntr, Self::OUT_VOL_NAME, 1, labels.len())?;
        self.state_add_elem_level(card_cntr, Self::OUT_DIM_VOL_NAME, 1, labels.len())?;

        let labels = &Self::SEND_TARGET_LABELS;
        self.state_add_elem_bool(card_cntr, Self::POST_FADER_NAME, 1, labels.len())?;

        let labels: Vec<String> = (0..2)
            .map(|i| format!("Channel-strip-{}/{}", i + 1, i + 2))
            .collect();
        self.state_add_elem_bool(card_cntr, Self::CH_STRIP_AS_PLUGIN_NAME, 1, labels.len())?;
        let labels: Vec<String> = (0..4)
            .map(|i| format!("Channel-strip-{}", i))
            .collect();
        self.state_add_elem_enum(card_cntr, Self::CH_STRIP_SRC_NAME, 1, labels.len(), &item_labels)?; 
        self.state_add_elem_bool(card_cntr, Self::CH_STRIP_23_AT_MID_RATE, 1, 1)?;

        self.state_add_elem_bool(card_cntr, Self::MIXER_ENABLE_NAME, 1, 1)?;

        // For metering.
        let meter = &segments.mixer_meter.data;
        let labels: Vec<String> = (0..meter.src_inputs.len())
            .map(|i| format!("mixer-input-{}", i))
            .collect();
        self.meter_add_elem_level(card_cntr, Self::MIXER_INPUT_METER_NAME, labels.len())?;

        let labels: Vec<String> = (0..meter.mixer_outputs.len())
            .map(|i| format!("mixer-output-{}", i))
            .collect();
        self.meter_add_elem_level(card_cntr, Self::MIXER_OUTPUT_METER_NAME, labels.len())?;

        let labels: Vec<String> = (0..meter.mixer_outputs.len())
            .map(|i| format!("aux-output-{}", i))
            .collect();
        self.meter_add_elem_level(card_cntr, Self::AUX_OUTPUT_METER_NAME, labels.len())?;

        Ok(())
    }

    fn state_add_elem_enum<T: AsRef<str>>(&mut self, card_cntr: &mut CardCntr, name: &str,
                                          count: usize, value_count: usize, labels: &[T])
        -> Result<(), Error>
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_enum_elems(&elem_id, count, value_count, &labels, None, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))
    }

    fn state_add_elem_bool(&mut self, card_cntr: &mut CardCntr, name: &str, count: usize, value_count: usize)
        -> Result<(), Error>
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_bool_elems(&elem_id, count, value_count, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))
    }

    fn state_add_elem_level(&mut self, card_cntr: &mut CardCntr, name: &str, count: usize, value_count: usize)
        -> Result<(), Error>
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(&elem_id, count, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                value_count, Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)), true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))
    }

    fn state_add_elem_pan(&mut self, card_cntr: &mut CardCntr, name: &str, count: usize, value_count: usize)
        -> Result<(), Error>
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(&elem_id, count, Self::PAN_MIN, Self::PAN_MAX, Self::PAN_STEP, value_count,
                                None, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))
    }

    fn meter_add_elem_level(&mut self, card_cntr: &mut CardCntr, name: &str, value_count: usize)
        -> Result<(), Error>
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                value_count, Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)), false)
            .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))
    }

    fn read(&mut self, segments: &StudioSegments, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.read_notified_elem(segments, elem_id, elem_value)? {
            Ok(true)
        } else if self.read_measured_elem(segments, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn state_read_src_param<T, F>(segments: &StudioSegments, elem_value: &ElemValue, cb: F)
        -> Result<bool, Error>
        where T: Default + Copy + Eq,
              F: Fn(&MonitorSrcParam) -> Result<T, Error>,
              ElemValue: ElemValueAccessor<T>,
    {
        let count = segments.mixer_state.data.src_pairs.len() * 2;
        ElemValueAccessor::<T>::set_vals(elem_value, count, |idx| {
            let i = idx / 2;
            let ch = idx % 2;
            let param = if ch == 0 {
                &segments.mixer_state.data.src_pairs[i].left
            } else {
                &segments.mixer_state.data.src_pairs[i].right
            };
            cb(param)
        })
        .map(|_| true)
    }

    fn state_read_out_pair<T, F>(segments: &StudioSegments, elem_value: &ElemValue, cb: F)
        -> Result<bool, Error>
        where T: Copy + Default + Eq + PartialEq,
              F: Fn(&OutPair) -> Result<T, Error>,
              ElemValue: ElemValueAccessor<T>,
    {
        ElemValueAccessor::<T>::set_vals(elem_value, Self::OUT_LABELS.len(), |idx| {
            cb(&segments.mixer_state.data.mixer_out[idx])
        })
        .map(|_| true)
    }

    fn write(&mut self, unit: &SndDice, proto: &Studiok48Proto, segments: &mut StudioSegments,
             elem_id: &ElemId, old: &ElemValue, new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::SRC_PAIR_MODE_NAME => {
                let state = &mut segments.mixer_state.data;
                ElemValueAccessor::<u32>::get_vals(new, old, state.src_pairs.len(), |idx, val| {
                    if let Some(m) = Self::SRC_PAIR_MODES.iter().nth(val as usize) {
                        if state.src_pairs[idx].mode != MonitorSrcPairMode::Fixed {
                            if *m != MonitorSrcPairMode::Fixed {
                                state.src_pairs[idx].mode = *m;
                                Ok(())
                            } else {
                                let msg = format!("The fixed mode is not newly available: {}", idx);
                                Err(Error::new(FileError::Inval, &msg))
                            }
                        } else {
                            let msg = format!("The source of mixer is immutable: {}", idx);
                            Err(Error::new(FileError::Inval, &msg))
                        }
                    } else {
                        let msg = format!("Invalid value for index of mixer source: {}", val);
                        Err(Error::new(FileError::Inval, &msg))
                    }
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.mixer_state, timeout_ms))
                .map(|_| true)
            }
            Self::SRC_ENTRY_NAME => {
                Self::state_write_src_param(unit, proto, segments, new, old, timeout_ms, |param, val: u32| {
                    Self::SRC_PAIR_ENTRIES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of mixer source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| param.src = s)
                })
            }
            Self::SRC_STEREO_LINK_NAME => {
                let pair_count = segments.mixer_state.data.src_pairs.len();
                ElemValueAccessor::<bool>::get_vals(new, old, pair_count, |idx, val| {
                    segments.mixer_state.data.src_pairs[idx].stereo_link = val;
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.mixer_state, timeout_ms))
                .map(|_| true)
            }
            Self::SRC_GAIN_NAME => {
                Self::state_write_src_param(unit, proto, segments, new, old, timeout_ms, |param, val| {
                    param.gain_to_main = val;
                    Ok(())
                })
            }
            Self::SRC_PAN_NAME => {
                Self::state_write_src_param(unit, proto, segments, new, old, timeout_ms, |param, val| {
                    param.pan_to_main = val;
                    Ok(())
                })
            }
            Self::REVERB_SRC_GAIN_NAME => {
                Self::state_write_src_param(unit, proto, segments, new, old, timeout_ms, |param, val| {
                    param.gain_to_reverb = val;
                    Ok(())
                })
            }
            Self::AUX01_SRC_GAIN_NAME => {
                Self::state_write_src_param(unit, proto, segments, new, old, timeout_ms, |param, val| {
                    param.gain_to_aux0 = val;
                    Ok(())
                })
            }
            Self::AUX23_SRC_GAIN_NAME => {
                Self::state_write_src_param(unit, proto, segments, new, old, timeout_ms, |param, val| {
                    param.gain_to_aux1 = val;
                    Ok(())
                })
            }
            Self::SRC_MUTE_NAME => {
                new.get_bool(&mut segments.mixer_state.data.mutes);
                proto.write_segment(&unit.get_node(), &mut segments.mixer_state, timeout_ms)
                    .map(|_| true)
            }
            Self::OUT_DIM_NAME => {
                Self::state_write_out_pair(unit, proto, segments, new, old, timeout_ms, |pair, val| {
                    pair.dim_enabled = val;
                    Ok(())
                })
            }
            Self::OUT_VOL_NAME=> {
                Self::state_write_out_pair(unit, proto, segments, new, old, timeout_ms, |pair, val| {
                    pair.vol = val;
                    Ok(())
                })
            }
            Self::OUT_DIM_VOL_NAME => {
                Self::state_write_out_pair(unit, proto, segments, new, old, timeout_ms, |pair, val| {
                    pair.dim_vol = val;
                    Ok(())
                })
            }
            Self::REVERB_RETURN_MUTE_NAME => {
                new.get_bool(&mut segments.mixer_state.data.reverb_return_mute);
                proto.write_segment(&unit.get_node(), &mut segments.mixer_state, timeout_ms)
                    .map(|_| true)
            }
            Self::REVERB_RETURN_GAIN_NAME => {
                new.get_int(&mut segments.mixer_state.data.reverb_return_gain);
                proto.write_segment(&unit.get_node(), &mut segments.mixer_state, timeout_ms)
                    .map(|_| true)
            }
            Self::CH_STRIP_AS_PLUGIN_NAME => {
                new.get_bool(&mut segments.mixer_state.data.ch_strip_as_plugin);
                proto.write_segment(&unit.get_node(), &mut segments.mixer_state, timeout_ms)
                    .map(|_| true)
            }
            Self::CH_STRIP_SRC_NAME => {
                let count = segments.mixer_state.data.ch_strip_src.len();
                ElemValueAccessor::<u32>::get_vals(new, old, count, |idx, val| {
                    Self::SRC_PAIR_ENTRIES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of ch strip source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| segments.mixer_state.data.ch_strip_src[idx] = s)
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.mixer_state, timeout_ms))
                .map(|_| true)
            }
            Self::CH_STRIP_23_AT_MID_RATE => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    segments.mixer_state.data.ch_strip_23_at_mid_rate = val;
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.mixer_state, timeout_ms))
                .map(|_| true)
            }
            Self::POST_FADER_NAME => {
                new.get_bool(&mut segments.mixer_state.data.post_fader);
                proto.write_segment(&unit.get_node(), &mut segments.mixer_state, timeout_ms)
                    .map(|_| true)
            }
            Self::MIXER_ENABLE_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    segments.mixer_state.data.enabled = val;
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.mixer_state, timeout_ms))
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn state_write_src_param<T, F>(unit: &SndDice, proto: &Studiok48Proto, segments: &mut StudioSegments,
                                  new: &ElemValue, old: &ElemValue, timeout_ms: u32, cb: F)
        -> Result<bool, Error>
        where T: Default + Copy + Eq,
              F: Fn(&mut MonitorSrcParam, T) -> Result<(), Error>,
              ElemValue: ElemValueAccessor<T>,
    {
        let count = segments.mixer_state.data.src_pairs.len() * 2;
        ElemValueAccessor::<T>::get_vals(new, old, count, |idx, val| {
            let i = idx / 2;
            let ch = idx % 2;
            let param = if ch == 0 {
                &mut segments.mixer_state.data.src_pairs[i].left
            } else {
                &mut segments.mixer_state.data.src_pairs[i].right
            };
            cb(param, val)
        })
        .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.mixer_state, timeout_ms))
        .map(|_| true)
    }

    fn state_write_out_pair<T, F>(unit: &SndDice, proto: &Studiok48Proto, segments: &mut StudioSegments,
                                  new: &ElemValue, old: &ElemValue, timeout_ms: u32, cb: F)
        -> Result<bool, Error>
        where T: Default + Copy + Eq,
              F: Fn(&mut OutPair, T) -> Result<(), Error>,
              ElemValue: ElemValueAccessor<T>,
    {
        ElemValueAccessor::<T>::get_vals(new, old, Self::OUT_LABELS.len(), |idx, val| {
            cb(&mut segments.mixer_state.data.mixer_out[idx], val)
        })
        .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.mixer_state, timeout_ms))
        .map(|_| true)
    }

    fn read_notified_elem(&self, segments: &StudioSegments, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::SRC_PAIR_MODE_NAME => {
                let pair_count = segments.mixer_state.data.src_pairs.len();
                ElemValueAccessor::<u32>::set_vals(elem_value, pair_count, |idx| {
                    let pos = Self::SRC_PAIR_MODES.iter()
                        .position(|m| m.eq(&segments.mixer_state.data.src_pairs[idx].mode))
                        .expect("Programming error...");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::SRC_STEREO_LINK_NAME => {
                let pair_count = segments.mixer_state.data.src_pairs.len();
                ElemValueAccessor::<bool>::set_vals(elem_value, pair_count, |idx| {
                    Ok(segments.mixer_state.data.src_pairs[idx].stereo_link)
                })
                .map(|_| true)
            }
            Self::SRC_ENTRY_NAME => {
                Self::state_read_src_param(segments, elem_value, |param| {
                    let pos = Self::SRC_PAIR_ENTRIES.iter()
                        .position(|m| m.eq(&param.src))
                        .expect("Programming error...");
                    Ok(pos as u32)
                })
            }
            Self::SRC_GAIN_NAME => {
                Self::state_read_src_param(segments, elem_value, |param| Ok(param.gain_to_main))
            }
            Self::SRC_PAN_NAME => {
                Self::state_read_src_param(segments, elem_value, |param| Ok(param.pan_to_main))
            }
            Self::REVERB_SRC_GAIN_NAME => {
                Self::state_read_src_param(segments, elem_value, |param| Ok(param.gain_to_reverb))
            }
            Self::AUX01_SRC_GAIN_NAME => {
                Self::state_read_src_param(segments, elem_value, |param| Ok(param.gain_to_aux0))
            }
            Self::AUX23_SRC_GAIN_NAME => {
                Self::state_read_src_param(segments, elem_value, |param| Ok(param.gain_to_aux1))
            }
            Self::SRC_MUTE_NAME => {
                elem_value.set_bool(&segments.mixer_state.data.mutes);
                Ok(true)
            }
            Self::OUT_DIM_NAME => {
                Self::state_read_out_pair(segments, elem_value, |pair| Ok(pair.dim_enabled))
            }
            Self::OUT_VOL_NAME => {
                Self::state_read_out_pair(segments, elem_value, |pair| Ok(pair.vol))
            }
            Self::OUT_DIM_VOL_NAME => {
                Self::state_read_out_pair(segments, elem_value, |pair| Ok(pair.dim_vol))
            }
            Self::REVERB_RETURN_MUTE_NAME => {
                elem_value.set_bool(&segments.mixer_state.data.reverb_return_mute);
                Ok(true)
            }
            Self::REVERB_RETURN_GAIN_NAME => {
                elem_value.set_int(&segments.mixer_state.data.reverb_return_gain);
                Ok(true)
            }
            Self::POST_FADER_NAME => {
                elem_value.set_bool(&segments.mixer_state.data.post_fader);
                Ok(true)
            }
            Self::CH_STRIP_AS_PLUGIN_NAME => {
                elem_value.set_bool(&segments.mixer_state.data.ch_strip_as_plugin);
                Ok(true)
            }
            Self::CH_STRIP_SRC_NAME => {
                let count = segments.mixer_state.data.ch_strip_src.len();
                ElemValueAccessor::<u32>::set_vals(elem_value, count, |idx| {
                    let pos = Self::SRC_PAIR_ENTRIES.iter()
                        .position(|s| s.eq(&segments.mixer_state.data.ch_strip_src[idx]))
                        .expect("Programming error...");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::CH_STRIP_23_AT_MID_RATE => {
                elem_value.set_bool(&[segments.mixer_state.data.ch_strip_23_at_mid_rate]);
                Ok(true)
            }
            Self::MIXER_ENABLE_NAME => {
                elem_value.set_bool(&[segments.mixer_state.data.enabled]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn read_measured_elem(&self, segments: &StudioSegments, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MIXER_INPUT_METER_NAME => {
                elem_value.set_int(&segments.mixer_meter.data.src_inputs);
                Ok(true)
            }
            Self::MIXER_OUTPUT_METER_NAME => {
                elem_value.set_int(&segments.mixer_meter.data.mixer_outputs);
                Ok(true)
            }
            Self::AUX_OUTPUT_METER_NAME => {
                elem_value.set_int(&segments.mixer_meter.data.aux_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

fn opt_iface_mode_to_string(mode: &OptIfaceMode) -> String {
    match mode {
        OptIfaceMode::Adat => "ADAT",
        OptIfaceMode::Spdif => "S/PDIF",
    }.to_string()
}

fn standalone_clk_src_to_string(src: &StudioStandaloneClkSrc) -> String {
    match src {
        StudioStandaloneClkSrc::Adat => "ADAT",
        StudioStandaloneClkSrc::SpdifOnOpt01 => "S/PDIF-opt-1/2",
        StudioStandaloneClkSrc::SpdifOnOpt23 => "S/PDIF-opt-3/4",
        StudioStandaloneClkSrc::SpdifOnCoax => "S/PDIF-coax",
        StudioStandaloneClkSrc::WordClock => "Word-clock",
        StudioStandaloneClkSrc::Internal => "Internal",
    }.to_string()
}

#[derive(Default, Debug)]
pub struct ConfigCtl{
    standalone_rate: TcKonnektStandaloneCtl,
    midi_send: MidiSendCtl,
}

impl<'a> ConfigCtl {
    const OPT_IFACE_MODE_NAME: &'a str = "opt-iface-mode";
    const STANDALONE_CLK_SRC_NAME: &'a str = "standalone-clock-source";
    const CLOCK_RECOVERY_NAME: &'a str = "clock-recovery";

    const OPT_IFACE_MODES: [OptIfaceMode;2] = [OptIfaceMode::Adat, OptIfaceMode::Spdif];

    const STANDALONE_CLK_SRCS: [StudioStandaloneClkSrc;6] = [
        StudioStandaloneClkSrc::Adat,
        StudioStandaloneClkSrc::SpdifOnOpt01,
        StudioStandaloneClkSrc::SpdifOnOpt23,
        StudioStandaloneClkSrc::SpdifOnCoax,
        StudioStandaloneClkSrc::WordClock,
        StudioStandaloneClkSrc::Internal,
    ];

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = Self::OPT_IFACE_MODES.iter()
            .map(|m| opt_iface_mode_to_string(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OPT_IFACE_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<String> = Self::STANDALONE_CLK_SRCS.iter()
            .map(|r| standalone_clk_src_to_string(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::STANDALONE_CLK_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        self.standalone_rate.load(card_cntr)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::CLOCK_RECOVERY_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        self.midi_send.load(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, segments: &StudioSegments, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::OPT_IFACE_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::OPT_IFACE_MODES.iter()
                        .position(|m| m.eq(&segments.config.data.opt_iface_mode))
                        .expect("Programming error...");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::STANDALONE_CLK_SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::STANDALONE_CLK_SRCS.iter()
                        .position(|s| s.eq(&segments.config.data.standalone_src))
                        .expect("Programming error...");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::CLOCK_RECOVERY_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || Ok(segments.config.data.clock_recovery))
                .map(|_| true)
            }
            _ => {
                if self.standalone_rate.read(&segments.config, elem_id, elem_value)? {
                    Ok(true)
                } else if self.midi_send.read(&segments.config, elem_id, elem_value)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn write(&mut self, unit: &SndDice, proto: &Studiok48Proto, segments: &mut StudioSegments,
             elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::OPT_IFACE_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::OPT_IFACE_MODES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of standalone clock source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&m| segments.config.data.opt_iface_mode = m)
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.config, timeout_ms))
                .map(|_| true)
            }
            Self::STANDALONE_CLK_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::STANDALONE_CLK_SRCS.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of standalone clock source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| segments.config.data.standalone_src = s)
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.config, timeout_ms))
                .map(|_| true)
            }
            Self::CLOCK_RECOVERY_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    segments.config.data.clock_recovery = val;
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.config, timeout_ms))
                .map(|_| true)
            }
            _ => {
                if self.standalone_rate.write(unit, proto, &mut segments.config, elem_id, elem_value,
                                              timeout_ms)? {
                    Ok(true)
                } else if self.midi_send.write(unit, proto, &mut segments.config, elem_id, elem_value,
                                               timeout_ms)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }
}

fn effect_button_mode_to_string(mode: &RemoteEffectButtonMode) -> String {
    match mode {
        RemoteEffectButtonMode::Reverb => "Reverb",
        RemoteEffectButtonMode::Midi => "Midi",
    }.to_string()
}

fn knob_push_mode_to_string(mode: &KnobPushMode) -> String {
    match mode {
        KnobPushMode::Pan => "Pan",
        KnobPushMode::GainToReverb => "Reverb",
        KnobPushMode::GainToAux0 => "Aux-1/2",
        KnobPushMode::GainToAux1 => "Aux-3/4",
    }.to_string()
}

#[derive(Default, Debug)]
pub struct RemoteCtl{
    notified_elem_list: Vec<ElemId>,
    prog_ctl: TcKonnektProgramCtl,
}

impl<'a> RemoteCtl {
    const USER_ASSIGN_NAME: &'a str = "remote-user-assign";
    const EFFECT_BUTTON_MODE_NAME: &'a str = "remote-effect-button-mode";
    const FALLBACK_TO_MASTER_ENABLE_NAME: &'a str = "remote-fallback-to-master-enable";
    const FALLBACK_TO_MASTER_DURATION_NAME: &'a str = "remote-fallback-to-master-duration";
    const KNOB_PUSH_MODE_NAME: &'a str = "remote-knob-push-mode";

    const EFFECT_BUTTON_MODES: [RemoteEffectButtonMode;2] = [
        RemoteEffectButtonMode::Reverb,
        RemoteEffectButtonMode::Midi,
    ];

    // NOTE: by milisecond.
    const DURATION_MIN: i32 = 10;
    const DURATION_MAX: i32 = 1000;
    const DURATION_STEP: i32 = 1;

    const KNOB_PUSH_MODES: [KnobPushMode;4] = [
        KnobPushMode::Pan,
        KnobPushMode::GainToReverb,
        KnobPushMode::GainToAux0,
        KnobPushMode::GainToAux1,
    ];

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.prog_ctl.load(card_cntr)?;
        self.notified_elem_list.extend_from_slice(&self.prog_ctl.0);

        let labels: Vec<String> = MixerCtl::SRC_PAIR_ENTRIES.iter()
            .map(|src| src_pair_entry_to_string(src))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::USER_ASSIGN_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, STUDIO_REMOTE_USER_ASSIGN_COUNT, &labels, None, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        let labels: Vec<String> = Self::EFFECT_BUTTON_MODES.iter()
            .map(|m| effect_button_mode_to_string(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EFFECT_BUTTON_MODE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::FALLBACK_TO_MASTER_ENABLE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::FALLBACK_TO_MASTER_DURATION_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::DURATION_MIN, Self::DURATION_MAX, Self::DURATION_STEP,
                                1, None, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        let labels: Vec<String> = Self::KNOB_PUSH_MODES.iter()
            .map(|m| knob_push_mode_to_string(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::KNOB_PUSH_MODE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, segments: &StudioSegments, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.read_notified_elem(segments, elem_id, elem_value)
    }

    fn write(&mut self, unit: &SndDice, proto: &Studiok48Proto, segments: &mut StudioSegments,
             elem_id: &ElemId, old: &ElemValue, new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::USER_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, STUDIO_REMOTE_USER_ASSIGN_COUNT, |idx, val| {
                    MixerCtl::SRC_PAIR_ENTRIES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of source of user assignment: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| segments.remote.data.user_assigns[idx] = s)
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.remote, timeout_ms))
                .map(|_| true)
            }
            Self::EFFECT_BUTTON_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    Self::EFFECT_BUTTON_MODES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of source of user assignment: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&m| segments.remote.data.effect_button_mode = m)
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.remote, timeout_ms))
                .map(|_| true)
            }
            Self::FALLBACK_TO_MASTER_ENABLE_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    segments.remote.data.fallback_to_master_enable = val;
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.remote, timeout_ms))
                .map(|_| true)
            }
            Self::FALLBACK_TO_MASTER_DURATION_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    segments.remote.data.fallback_to_master_duration = val as u32;
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.remote, timeout_ms))
                .map(|_| true)
            }
            Self::KNOB_PUSH_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    Self::KNOB_PUSH_MODES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of source of user assignment: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&m| segments.remote.data.knob_push_mode = m)
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.remote, timeout_ms))
                .map(|_| true)
            }
            _ => self.prog_ctl.write(unit, proto, &mut segments.remote, elem_id, new, timeout_ms)
        }
    }

    fn read_notified_elem(&mut self, segments: &StudioSegments, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::USER_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, STUDIO_REMOTE_USER_ASSIGN_COUNT, |idx| {
                    let pos = MixerCtl::SRC_PAIR_ENTRIES.iter()
                        .position(|p| p.eq(&segments.remote.data.user_assigns[idx]))
                        .expect("Programming error...");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::EFFECT_BUTTON_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::EFFECT_BUTTON_MODES.iter()
                        .position(|m| m.eq(&segments.remote.data.effect_button_mode))
                        .expect("Programming error");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::FALLBACK_TO_MASTER_ENABLE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segments.remote.data.fallback_to_master_enable)
                })
                .map(|_| true)
            }
            Self::FALLBACK_TO_MASTER_DURATION_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(segments.remote.data.fallback_to_master_duration as i32)
                })
                .map(|_| true)
            }
            Self::KNOB_PUSH_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::KNOB_PUSH_MODES.iter()
                        .position(|m| m.eq(&segments.remote.data.knob_push_mode))
                        .expect("Programming error");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            _ => self.prog_ctl.read(&segments.remote, elem_id, elem_value),
        }
    }
}

fn nominal_signal_level_to_string(level: &NominalSignalLevel) -> String {
    match level {
        NominalSignalLevel::Professional => "+4dBu",
        NominalSignalLevel::Consumer => "-10dBV",
    }.to_string()
}

#[derive(Default, Debug)]
pub struct LineoutCtl;

impl<'a> LineoutCtl {
    const LINE_OUT_45_LEVEL_NAME: &'a str = "line-out-5/6-level";
    const LINE_OUT_67_LEVEL_NAME: &'a str = "line-out-7/8-level";
    const LINE_OUT_89_LEVEL_NAME: &'a str = "line-out-9/10-level";
    const LINE_OUT_1011_LEVEL_NAME: &'a str = "line-out-11/12-level";

    const NOMINAL_SIGNAL_LEVELS: [NominalSignalLevel;2] = [
        NominalSignalLevel::Professional,
        NominalSignalLevel::Consumer,
    ];

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = Self::NOMINAL_SIGNAL_LEVELS.iter()
            .map(|m| nominal_signal_level_to_string(m))
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::LINE_OUT_45_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::LINE_OUT_67_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::LINE_OUT_89_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::LINE_OUT_1011_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read(&mut self, segments: &StudioSegments, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::LINE_OUT_45_LEVEL_NAME => {
                Self::read_as_index(elem_value, segments.out_level.data.line_45)
            }
            Self::LINE_OUT_67_LEVEL_NAME => {
                Self::read_as_index(elem_value, segments.out_level.data.line_67)
            }
            Self::LINE_OUT_89_LEVEL_NAME => {
                Self::read_as_index(elem_value, segments.out_level.data.line_89)
            }
            Self::LINE_OUT_1011_LEVEL_NAME => {
                Self::read_as_index(elem_value, segments.out_level.data.line_1011)
            }
            _ => Ok(false),
        }
    }

    fn read_as_index(elem_value: &mut ElemValue, level: NominalSignalLevel)
        -> Result<bool, Error>
    {
        ElemValueAccessor::<u32>::set_val(elem_value, || {
            let pos = Self::NOMINAL_SIGNAL_LEVELS.iter()
                .position(|l| l.eq(&level))
                .expect("Programming error.");
            Ok(pos as u32)
        })
        .map(|_| true)
    }

    fn write(&mut self, unit: &SndDice, proto: &Studiok48Proto, segments: &mut StudioSegments,
             elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::LINE_OUT_45_LEVEL_NAME => {
                Self::write_as_index(unit, proto, segments, elem_value, timeout_ms, |data, level| {
                    data.line_45 = level
                })
            }
            Self::LINE_OUT_67_LEVEL_NAME => {
                Self::write_as_index(unit, proto, segments, elem_value, timeout_ms, |data, level| {
                    data.line_67 = level
                })
            }
            Self::LINE_OUT_89_LEVEL_NAME => {
                Self::write_as_index(unit, proto, segments, elem_value, timeout_ms, |data, level| {
                    data.line_89 = level
                })
            }
            Self::LINE_OUT_1011_LEVEL_NAME => {
                Self::write_as_index(unit, proto, segments, elem_value, timeout_ms, |data, level| {
                    data.line_1011 = level
                })
            }
            _ => Ok(false),
        }
    }

    fn write_as_index<F>(unit: &SndDice, proto: &Studiok48Proto, segments: &mut StudioSegments,
                         elem_value: &ElemValue, timeout_ms: u32, cb: F)
        -> Result<bool, Error>
        where F: Fn(&mut StudioLineOutLevel, NominalSignalLevel),
    {
        ElemValueAccessor::<u32>::get_val(elem_value, |val| {
            Self::NOMINAL_SIGNAL_LEVELS.iter()
                .nth(val as usize)
                .ok_or_else(|| {
                    let msg = format!("Invalid value for index of nominal level: {}", val);
                    Error::new(FileError::Inval, &msg)
                })
                .map(|&l| cb(&mut segments.out_level.data, l))
        })
        .and_then(|_| proto.write_segment(&unit.get_node(), &mut segments.out_level, timeout_ms))
        .map(|_| true)
    }
}
