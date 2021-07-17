// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use hinawa::{FwNode, SndDice, SndUnitExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use dice_protocols::tcelectronic::*;
use dice_protocols::tcelectronic::fw_led::*;
use dice_protocols::tcelectronic::shell::*;
use dice_protocols::tcelectronic::standalone::*;

use core::card_cntr::*;
use core::elem_value_accessor::*;

use super::fw_led_ctl::*;
use super::standalone_ctl::*;

fn analog_jack_state_to_string(state: &ShellAnalogJackState) -> String {
    match state {
        ShellAnalogJackState::FrontSelected => "Front-selected",
        ShellAnalogJackState::FrontInserted => "Front-inserted",
        ShellAnalogJackState::FrontInsertedAttenuated => "Front-inserted-attenuated",
        ShellAnalogJackState::RearSelected => "Rear-selected",
        ShellAnalogJackState::RearInserted => "Rear-inserted",
    }.to_string()
}

#[derive(Default, Debug)]
pub struct HwStateCtl {
    pub notified_elem_list: Vec<ElemId>,
    fw_led_ctl: FwLedCtl,
}

impl HwStateCtl {
    // TODO: For Jack detection in ALSA applications.
    const ANALOG_JACK_STATE_NAME: &'static str = "analog-jack-state";

    const ANALOG_JACK_STATE_LABELS: [ShellAnalogJackState;5] = [
        ShellAnalogJackState::FrontSelected,
        ShellAnalogJackState::FrontInserted,
        ShellAnalogJackState::FrontInsertedAttenuated,
        ShellAnalogJackState::RearSelected,
        ShellAnalogJackState::RearInserted,
    ];

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels = Self::ANALOG_JACK_STATE_LABELS.iter()
            .map(|s| analog_jack_state_to_string(s))
            .collect::<Vec<_>>();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::ANALOG_JACK_STATE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, SHELL_ANALOG_JACK_STATE_COUNT, &labels, None, false)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        self.fw_led_ctl.load(card_cntr)?;
        self.notified_elem_list.extend_from_slice(&self.fw_led_ctl.0);

        Ok(())
    }

    pub fn read<S>(&mut self, segment: &TcKonnektSegment<S>, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where S: TcKonnektSegmentData + AsRef<[ShellAnalogJackState]> + AsRef<FireWireLedState>,
    {
        match elem_id.get_name().as_str() {
            Self::ANALOG_JACK_STATE_NAME => {
                let analog_jack_states = AsRef::<[ShellAnalogJackState]>::as_ref(&segment.data);
                ElemValueAccessor::<u32>::set_vals(elem_value, analog_jack_states.len(), |idx| {
                    let pos = Self::ANALOG_JACK_STATE_LABELS.iter()
                        .position(|s| s.eq(&analog_jack_states[idx]))
                        .expect("Programming error");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            _ => self.fw_led_ctl.read(segment, elem_id, elem_value),
        }
    }

    pub fn write<T, S>(&mut self, unit: &SndDice, proto: &T, segment: &mut TcKonnektSegment<S>,
                       elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where T: TcKonnektSegmentProtocol<FwNode, S>,
              S: TcKonnektSegmentData + AsMut<FireWireLedState>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
    {
        self.fw_led_ctl.write(unit, proto, segment, elem_id, elem_value, timeout_ms)
    }
}

#[derive(Default, Debug)]
pub struct ShellMixerCtl{
    pub notified_elem_list: Vec<ElemId>,
    pub measured_elem_list: Vec<ElemId>,
}

impl ShellMixerCtl {
    const MIXER_STREAM_SRC_PAIR_GAIN_NAME: &'static str = "mixer-stream-source-gain";
    const MIXER_STREAM_SRC_PAIR_PAN_NAME: &'static str = "mixer-stream-source-pan";
    const MIXER_STREAM_SRC_PAIR_MUTE_NAME: &'static str = "mixer-stream-source-mute";
    const REVERB_STREAM_SRC_PAIR_GAIN_NAME: &'static str = "send-stream-source-gain";
    const MIXER_PHYS_SRC_STEREO_LINK_NAME: &'static str = "mixer-phys-source-link";

    const MIXER_PHYS_SRC_GAIN_NAME: &'static str = "mixer-phys-source-gain";
    const MIXER_PHYS_SRC_PAN_NAME: &'static str = "mixer-phys-source-pan";
    const MIXER_PHYS_SRC_MUTE_NAME: &'static str = "mixer-phys-source-mute";

    const REVERB_PHYS_SRC_GAIN_NAME: &'static str = "send-phys-source-gain";

    const MIXER_OUT_DIM_NAME: &'static str = "mixer-out-dim-enable";
    const MIXER_OUT_VOL_NAME: &'static str = "mixer-out-volume";
    const MIXER_OUT_DIM_VOL_NAME: &'static str = "mixer-out-dim-volume";

    const STREAM_IN_METER_NAME: &'static str = "stream-input-meters";
    const ANALOG_IN_METER_NAME: &'static str = "analog-input-meters";
    const DIGITAL_IN_METER_NAME: &'static str = "digital-input-meters";
    const MIXER_OUT_METER_NAME: &'static str = "mixer-output-meters";

    const LEVEL_MIN: i32 = -1000;
    const LEVEL_MAX: i32 = 0;
    const LEVEL_STEP: i32 = 1;
    const LEVEL_TLV: DbInterval = DbInterval{min: -9400, max: 0, linear: false, mute_avail: false};

    const PAN_MIN: i32 = -50;
    const PAN_MAX: i32 = 50;
    const PAN_STEP: i32 = 1;

    pub fn load<S, M>(&mut self, state_segment: &TcKonnektSegment<S>, meter_segment: &TcKonnektSegment<M>,
                      card_cntr: &mut CardCntr)
        -> Result<(), Error>
        where S: TcKonnektSegmentData + AsRef<ShellMixerState>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
              M: TcKonnektSegmentData + AsRef<ShellMixerMeter>,
              TcKonnektSegment<M>: TcKonnektSegmentSpec,
    {
        // For stream source of mixer.
        self.state_add_elem_level(card_cntr, Self::MIXER_STREAM_SRC_PAIR_GAIN_NAME, 1)?;
        self.state_add_elem_pan(card_cntr, Self::MIXER_STREAM_SRC_PAIR_PAN_NAME, 1)?;
        self.state_add_elem_bool(card_cntr, Self::MIXER_STREAM_SRC_PAIR_MUTE_NAME, 1)?;
        self.state_add_elem_level(card_cntr, Self::REVERB_STREAM_SRC_PAIR_GAIN_NAME, 1)?;

        // For physical sources of mixer.
        let state = state_segment.data.as_ref();
        let labels: Vec<String> = (0..state.analog.len())
            .map(|i| format!("Analog-{}/{}", i + 1, i + 2))
            .chain(
                (0..state.digital.len())
                .map(|i| format!("Digital-{}/{}", i + 1, i + 2))
            )
            .collect();
        self.state_add_elem_bool(card_cntr, Self::MIXER_PHYS_SRC_STEREO_LINK_NAME, labels.len())?;

        let labels: Vec<String> = (0..(state.analog.len() * 2))
            .map(|i| format!("Analog-{}", i + 1))
            .chain(
                (0..(state.digital.len() * 2))
                .map(|i| format!("Digital-{}", i + 1))
            )
            .collect();
        self.state_add_elem_level(card_cntr, Self::MIXER_PHYS_SRC_GAIN_NAME, labels.len())?;
        self.state_add_elem_pan(card_cntr, Self::MIXER_PHYS_SRC_PAN_NAME, labels.len())?;
        self.state_add_elem_bool(card_cntr, Self::MIXER_PHYS_SRC_MUTE_NAME, labels.len())?;
        self.state_add_elem_level(card_cntr, Self::REVERB_PHYS_SRC_GAIN_NAME, labels.len())?;

        // For output of mixer.
        self.state_add_elem_bool(card_cntr, Self::MIXER_OUT_DIM_NAME, 1)?;
        self.state_add_elem_level(card_cntr, Self::MIXER_OUT_VOL_NAME, 1)?;
        self.state_add_elem_level(card_cntr, Self::MIXER_OUT_DIM_VOL_NAME, 1)?;

        // For meter.
        let labels = (0..meter_segment.data.as_ref().stream_inputs.len())
            .map(|i| format!("Stream-input-{}", i))
            .collect::<Vec<_>>();
        self.meter_add_elem_level(card_cntr, Self::STREAM_IN_METER_NAME, labels.len())?;

        let labels = (0..meter_segment.data.as_ref().analog_inputs.len())
            .map(|i| format!("Analog-input-{}", i))
            .collect::<Vec<_>>();
        self.meter_add_elem_level(card_cntr, Self::ANALOG_IN_METER_NAME, labels.len())?;

        let labels = (0..meter_segment.data.as_ref().digital_inputs.len())
            .map(|i| format!("Digital-input-{}", i))
            .collect::<Vec<_>>();
        self.meter_add_elem_level(card_cntr, Self::DIGITAL_IN_METER_NAME, labels.len())?;

        let labels = (0..meter_segment.data.as_ref().main_outputs.len())
            .map(|i| format!("Mixer-output-{}", i))
            .collect::<Vec<_>>();
        self.meter_add_elem_level(card_cntr, Self::MIXER_OUT_METER_NAME, labels.len())?;

        Ok(())
    }

    fn state_add_elem_level(&mut self, card_cntr: &mut CardCntr, name: &str, value_count: usize)
        -> Result<(), Error>
    {
        let elem_id = alsactl::ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                value_count, Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)), true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))
    }

    fn state_add_elem_pan(&mut self, card_cntr: &mut CardCntr, name: &str, value_count: usize)
        -> Result<(), Error>
    {
        let elem_id = alsactl::ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::PAN_MIN, Self::PAN_MAX, Self::PAN_STEP,
                                value_count, None, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))
    }

    fn state_add_elem_bool(&mut self, card_cntr: &mut CardCntr, name: &str, value_count: usize)
        -> Result<(), Error>
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_bool_elems(&elem_id, 1, value_count, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))
    }

    fn meter_add_elem_level(&mut self, card_cntr: &mut CardCntr, name: &str, value_count: usize)
        -> Result<(), Error>
    {
        let elem_id = alsactl::ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::LEVEL_MIN, Self::LEVEL_MAX, Self::LEVEL_STEP,
                                value_count, Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)), false)
            .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))
    }

    pub fn read<S, M>(&self, state_segment: &TcKonnektSegment<S>, meter_segment: &TcKonnektSegment<M>,
                      elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where S: TcKonnektSegmentData + AsRef<ShellMixerState>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
              M: TcKonnektSegmentData + AsRef<ShellMixerMeter>,
              TcKonnektSegment<M>: TcKonnektSegmentSpec,
    {
        if self.read_notified_elem(state_segment, elem_id, elem_value)? {
            Ok(true)
        } else if self.read_measured_elem(meter_segment, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn state_read_phys_src<S, T, F>(segment: &TcKonnektSegment<S>, elem_value: &mut ElemValue, cb: F)
        -> Result<bool, Error>
        where S: TcKonnektSegmentData + AsRef<ShellMixerState>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
              F: Fn(&MonitorSrcParam) -> Result<T, Error>,
              T: Default + Copy + Eq,
              ElemValue: ElemValueAccessor<T>,
    {
        let state = segment.data.as_ref();
        let analog_count = segment.data.as_ref().analog.len();
        let digital_count = segment.data.as_ref().digital.len();
        let count = (analog_count + digital_count) * 2;
        ElemValueAccessor::<T>::set_vals(elem_value, count, |idx| {
            let i = idx / 2;
            let ch = idx % 2;
            let src_pair = if i < analog_count {
                &state.analog[i]
            } else {
                &state.digital[i - analog_count]
            };
            let param = if ch == 0 {
                &src_pair.left
            } else {
                &src_pair.right
            };
            cb(param)
        })
        .map(|_| true)
    }

    pub fn write<T, S>(&mut self, unit: &SndDice, proto: &T, segment: &mut TcKonnektSegment<S>,
                       elem_id: &ElemId, old: &ElemValue, new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where T: TcKonnektSegmentProtocol<FwNode, S>,
              S: TcKonnektSegmentData + AsRef<ShellMixerState> + AsMut<ShellMixerState>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
    {
        match elem_id.get_name().as_str() {
            Self::MIXER_STREAM_SRC_PAIR_GAIN_NAME => {
                Self::state_write(unit, proto, segment, new, timeout_ms, |state, val| {
                    state.stream.left.gain_to_mixer = val;
                    Ok(())
                })
            }
            Self::MIXER_STREAM_SRC_PAIR_PAN_NAME => {
                Self::state_write(unit, proto, segment, new, timeout_ms, |state, val| {
                    state.stream.left.pan_to_mixer = val;
                    Ok(())
                })
            }
            Self::MIXER_STREAM_SRC_PAIR_MUTE_NAME => {
                Self::state_write(unit, proto, segment, new, timeout_ms, |state, val| {
                    state.mutes.stream = val;
                    Ok(())
                })
            }
            Self::REVERB_STREAM_SRC_PAIR_GAIN_NAME => {
                Self::state_write(unit, proto, segment, new, timeout_ms, |state, val| {
                    state.stream.left.gain_to_send = val;
                    Ok(())
                })
            }
            Self::MIXER_PHYS_SRC_STEREO_LINK_NAME => {
                let analog_count = segment.data.as_ref().analog.len();
                let digital_count = segment.data.as_ref().digital.len();
                let count = analog_count + digital_count;
                ElemValueAccessor::<bool>::get_vals(new, old, count, |idx, val| {
                    if idx < analog_count {
                        segment.data.as_mut().analog[idx].stereo_link = val;
                    } else {
                        segment.data.as_mut().digital[idx - analog_count].stereo_link = val;
                    }
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), segment, timeout_ms))
                .map(|_| true)
            }
            Self::MIXER_PHYS_SRC_GAIN_NAME => {
                Self::state_write_phys_src(unit, proto, segment, new, old, timeout_ms, |param, val| {
                    param.gain_to_mixer = val;
                    Ok(())
                })
            }
            Self::MIXER_PHYS_SRC_PAN_NAME => {
                Self::state_write_phys_src(unit, proto, segment, new, old, timeout_ms, |param, val| {
                    param.pan_to_mixer = val;
                    Ok(())
                })
            }
            Self::MIXER_PHYS_SRC_MUTE_NAME => {
                let analog_count = segment.data.as_ref().mutes.analog.len();
                let digital_count = segment.data.as_ref().mutes.digital.len();
                let count = analog_count + digital_count;
                ElemValueAccessor::<bool>::get_vals(new, old, count, |idx, val| {
                    if idx < analog_count {
                        segment.data.as_mut().mutes.analog[idx] = val;
                    } else {
                        segment.data.as_mut().mutes.digital[idx - analog_count] = val;
                    };
                    Ok(())
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), segment, timeout_ms))
                .map(|_| true)
            }
            Self::REVERB_PHYS_SRC_GAIN_NAME => {
                Self::state_write_phys_src(unit, proto, segment, new, old, timeout_ms, |param, val| {
                    param.gain_to_send = val;
                    Ok(())
                })
            }
            Self::MIXER_OUT_DIM_NAME => {
                Self::state_write(unit, proto, segment, new, timeout_ms, |state, val| {
                    state.output_dim_enable = val;
                    Ok(())
                })
            }
            Self::MIXER_OUT_VOL_NAME => {
                Self::state_write(unit, proto, segment, new, timeout_ms, |state, val| {
                    state.output_volume = val;
                    Ok(())
                })
            }
            Self::MIXER_OUT_DIM_VOL_NAME => {
                Self::state_write(unit, proto, segment, new, timeout_ms, |state, val| {
                    state.output_dim_volume = val;
                    Ok(())
                })
            }
            _ => Ok(false),
        }
    }

    fn state_write<T, S, U, F>(unit: &SndDice, proto: &T, segment: &mut TcKonnektSegment<S>,
                            elem_value: &ElemValue, timeout_ms: u32, cb: F)
        -> Result<bool, Error>
        where T: TcKonnektSegmentProtocol<FwNode, S>,
              S: TcKonnektSegmentData + AsRef<ShellMixerState> + AsMut<ShellMixerState>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
              F: Fn(&mut ShellMixerState, U) -> Result<(), Error>,
              U: Default + Copy + Eq,
              ElemValue: ElemValueAccessor<U>,
    {
        ElemValueAccessor::<U>::get_val(elem_value, |val| {
            cb(segment.data.as_mut(), val)
        })
        .and_then(|_| proto.write_segment(&unit.get_node(), segment, timeout_ms))
        .map(|_| true)
    }

    fn state_write_phys_src<T, S, U, F>(unit: &SndDice, proto: &T, segment: &mut TcKonnektSegment<S>,
                                        new: &ElemValue, old: &ElemValue, timeout_ms: u32, cb: F)
        -> Result<bool, Error>
        where T: TcKonnektSegmentProtocol<FwNode, S>,
              S: TcKonnektSegmentData + AsRef<ShellMixerState> + AsMut<ShellMixerState>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
              F: Fn(&mut MonitorSrcParam, U) -> Result<(), Error>,
              U: Default + Copy + Eq,
              ElemValue: ElemValueAccessor<U>,
    {
        let analog_count = segment.data.as_ref().analog.len();
        let digital_count = segment.data.as_ref().digital.len();
        let count = (analog_count + digital_count) * 2;
        ElemValueAccessor::<U>::get_vals(new, old, count, |idx, val| {
            let i = idx / 2;
            let ch = idx % 2;
            let src_pair = if i < analog_count {
                &mut segment.data.as_mut().analog[i]
            } else {
                &mut segment.data.as_mut().digital[i - analog_count]
            };
            let param = if ch == 0 {
                &mut src_pair.left
            } else {
                &mut src_pair.right
            };
            cb(param, val)
        })
        .and_then(|_| proto.write_segment(&unit.get_node(), segment, timeout_ms))
        .map(|_| true)
    }

    pub fn read_notified_elem<S>(&self, segment: &TcKonnektSegment<S>, elem_id: &ElemId,
                                 elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where S: TcKonnektSegmentData + AsRef<ShellMixerState>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
    {
        match elem_id.get_name().as_str() {
            Self::MIXER_STREAM_SRC_PAIR_GAIN_NAME => {
                elem_value.set_int(&[segment.data.as_ref().stream.left.gain_to_mixer]);
                Ok(true)
            }
            Self::MIXER_STREAM_SRC_PAIR_PAN_NAME => {
                elem_value.set_int(&[segment.data.as_ref().stream.left.pan_to_mixer]);
                Ok(true)
            }
            Self::MIXER_STREAM_SRC_PAIR_MUTE_NAME => {
                elem_value.set_bool(&[segment.data.as_ref().mutes.stream]);
                Ok(true)
            }
            Self::REVERB_STREAM_SRC_PAIR_GAIN_NAME => {
                elem_value.set_int(&[segment.data.as_ref().stream.left.gain_to_send]);
                Ok(true)
            }
            Self::MIXER_PHYS_SRC_STEREO_LINK_NAME => {
                let state = segment.data.as_ref();
                let mut vals = Vec::with_capacity(state.analog.len() + state.digital.len());
                state.analog.iter()
                    .chain(state.digital.iter())
                    .for_each(|src| vals.push(src.stereo_link));
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::MIXER_PHYS_SRC_GAIN_NAME => {
                Self::state_read_phys_src(segment, elem_value, |param| {
                    Ok(param.gain_to_mixer)
                })
            }
            Self::MIXER_PHYS_SRC_PAN_NAME => {
                Self::state_read_phys_src(segment, elem_value, |param| {
                    Ok(param.pan_to_mixer)
                })
            }
            Self::MIXER_PHYS_SRC_MUTE_NAME => {
                let state = segment.data.as_ref();
                let mut vals = state.mutes.analog.clone();
                vals.extend_from_slice(&state.mutes.digital);
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::REVERB_PHYS_SRC_GAIN_NAME => {
                Self::state_read_phys_src(segment, elem_value, |param| {
                    Ok(param.gain_to_send)
                })
            }
            Self::MIXER_OUT_DIM_NAME => {
                elem_value.set_bool(&[segment.data.as_ref().output_dim_enable]);
                Ok(true)
            }
            Self::MIXER_OUT_VOL_NAME => {
                elem_value.set_int(&[segment.data.as_ref().output_volume]);
                Ok(true)
            }
            Self::MIXER_OUT_DIM_VOL_NAME => {
                elem_value.set_int(&[segment.data.as_ref().output_dim_volume]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn read_measured_elem<S>(&self, segment: &TcKonnektSegment<S>, elem_id: &ElemId,
                                 elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where S: TcKonnektSegmentData + AsRef<ShellMixerMeter>,
    {
        match elem_id.get_name().as_str() {
            Self::STREAM_IN_METER_NAME => {
                elem_value.set_int(&segment.data.as_ref().stream_inputs);
                Ok(true)
            }
            Self::ANALOG_IN_METER_NAME => {
                elem_value.set_int(&segment.data.as_ref().analog_inputs);
                Ok(true)
            }
            Self::DIGITAL_IN_METER_NAME => {
                elem_value.set_int(&segment.data.as_ref().digital_inputs);
                Ok(true)
            }
            Self::MIXER_OUT_METER_NAME => {
                elem_value.set_int(&segment.data.as_ref().main_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub struct ShellReverbReturnCtl(pub Vec<ElemId>);

impl ShellReverbReturnCtl {
    const USE_AS_PLUGIN_NAME: &'static str = "use-reverb-as-plugin";
    const GAIN_NAME: &'static str = "reverb-return-gain";
    const MUTE_NAME: &'static str = "reverb-return-mute";

    const GAIN_MIN: i32 = -1000;
    const GAIN_MAX: i32 = 0;
    const GAIN_STEP: i32 = 1;
    const GAIN_TLV: DbInterval = DbInterval{min: -7200, max: 0, linear: false, mute_avail: false};

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::USE_AS_PLUGIN_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::GAIN_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::GAIN_MIN, Self::GAIN_MAX, Self::GAIN_STEP,
                                1, Some(&Vec::<u32>::from(Self::GAIN_TLV)), true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MUTE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        Ok(())
    }

    pub fn read<S>(&self, segment: &TcKonnektSegment<S>, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where S: TcKonnektSegmentData + AsRef<ShellReverbReturn>,
    {
        match elem_id.get_name().as_str() {
            Self::USE_AS_PLUGIN_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segment.data.as_ref().plugin_mode)
                })
                .map(|_| true)
            }
            _ => self.read_notified_elem(segment, elem_id, elem_value),
        }
    }

    pub fn write<T, S>(&mut self, unit: &SndDice, proto: &T, segment: &mut TcKonnektSegment<S>,
                       elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where T: TcKonnektSegmentProtocol<FwNode, S>,
              S: TcKonnektSegmentData + AsMut<ShellReverbReturn>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
    {
        match elem_id.get_name().as_str() {
            Self::USE_AS_PLUGIN_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    segment.data.as_mut().plugin_mode = val;
                    proto.write_segment(&unit.get_node(), segment, timeout_ms)
                })
                .map(|_| true)
            }
            Self::GAIN_NAME => {
                ElemValueAccessor::<i32>::get_val(elem_value, |val| {
                    segment.data.as_mut().return_gain = val;
                    proto.write_segment(&unit.get_node(), segment, timeout_ms)
                })
                .map(|_| true)
            }
            Self::MUTE_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    segment.data.as_mut().return_mute = val;
                    proto.write_segment(&unit.get_node(), segment, timeout_ms)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub fn read_notified_elem<S>(&self, segment: &TcKonnektSegment<S>, elem_id: &ElemId,
                                 elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where S: TcKonnektSegmentData + AsRef<ShellReverbReturn>,
    {
        match elem_id.get_name().as_str() {
            Self::GAIN_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(segment.data.as_ref().return_gain)
                })
                .map(|_| true)
            }
            Self::MUTE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(segment.data.as_ref().return_mute)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

fn standalone_src_to_string(src: &ShellStandaloneClkSrc) -> String {
    match src {
        ShellStandaloneClkSrc::Optical => "Optical",
        ShellStandaloneClkSrc::Coaxial => "Coaxial",
        ShellStandaloneClkSrc::Internal => "Internal",
    }.to_string()
}

#[derive(Default, Debug)]
pub struct ShellStandaloneCtl(TcKonnektStandaloneCtl);

impl ShellStandaloneCtl {
    const SRC_NAME: &'static str = "standalone-clock-source";

    pub fn load<S>(&mut self, _: &TcKonnektSegment<S>, card_cntr: &mut CardCntr)
        -> Result<(), Error>
        where S: TcKonnektSegmentData + ShellStandaloneClkSpec,
    {
        let labels: Vec<String> = S::STANDALONE_CLOCK_SOURCES.iter()
            .map(|r| standalone_src_to_string(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        self.0.load(card_cntr)?;

        Ok(())
    }

    pub fn read<S>(&mut self, segment: &TcKonnektSegment<S>, elem_id: &ElemId, elem_value: &ElemValue)
        -> Result<bool, Error>
        where S: TcKonnektSegmentData + AsRef<ShellStandaloneClkSrc> + ShellStandaloneClkSpec +
                 AsRef<TcKonnektStandaloneClkRate>,
    {
        match elem_id.get_name().as_str() {
            Self::SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let src = segment.data.as_ref();
                    let pos = S::STANDALONE_CLOCK_SOURCES.iter()
                        .position(|s| s.eq(&src))
                        .expect("Programming error");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            _ => self.0.read(segment, elem_id, elem_value),
        }
    }

    pub fn write<T, S>(&mut self, unit: &SndDice, proto: &T, segment: &mut TcKonnektSegment<S>,
                       elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where T: TcKonnektSegmentProtocol<FwNode, S>,
              S: TcKonnektSegmentData + AsMut<ShellStandaloneClkSrc> + ShellStandaloneClkSpec +
                 AsMut<TcKonnektStandaloneClkRate>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec
    {
        match elem_id.get_name().as_str() {
            Self::SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    S::STANDALONE_CLOCK_SOURCES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of clock source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| *segment.data.as_mut() = s)
                })
                .and_then(|_| proto.write_segment(&unit.get_node(), segment, timeout_ms))
                .map(|_| true)
            }
            _ => self.0.write(unit, proto, segment, elem_id, elem_value, timeout_ms),
        }
    }
}

fn mixer_stream_src_pair_to_string(src: &ShellMixerStreamSrcPair) -> String {
    match src {
        ShellMixerStreamSrcPair::Stream01 => "Stream-1/2",
        ShellMixerStreamSrcPair::Stream23 => "Stream-3/4",
        ShellMixerStreamSrcPair::Stream45 => "Stream-5/6",
        ShellMixerStreamSrcPair::Stream67 => "Stream-7/8",
        ShellMixerStreamSrcPair::Stream89 => "Stream-9/10",
        ShellMixerStreamSrcPair::Stream1011 => "Stream-11/12",
        ShellMixerStreamSrcPair::Stream1213 => "Stream-13/14",
    }.to_string()
}

#[derive(Default, Debug)]
pub struct MixerStreamSrcPairCtl;

impl MixerStreamSrcPairCtl {
    const MIXER_STREAM_SRC_NAME: &'static str = "mixer-stream-soruce";
    const MIXER_STREAM_SRC_PAIRS: [ShellMixerStreamSrcPair;7] = [
        ShellMixerStreamSrcPair::Stream01,
        ShellMixerStreamSrcPair::Stream23,
        ShellMixerStreamSrcPair::Stream45,
        ShellMixerStreamSrcPair::Stream67,
        ShellMixerStreamSrcPair::Stream89,
        ShellMixerStreamSrcPair::Stream1011,
        ShellMixerStreamSrcPair::Stream1213,
    ];

    pub fn load<S>(&mut self, _: &TcKonnektSegment<S>, card_cntr: &mut CardCntr)
        -> Result<(), Error>
        where S: TcKonnektSegmentData + ShellMixerStreamSrcPairSpec,
    {
        let labels: Vec<String> = Self::MIXER_STREAM_SRC_PAIRS.iter()
            .take(S::MAXIMUM_STREAM_SRC_PAIR_COUNT)
            .map(|s| mixer_stream_src_pair_to_string(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MIXER_STREAM_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    pub fn read<S>(&mut self, segment: &TcKonnektSegment<S>, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where S: TcKonnektSegmentData + AsRef<ShellMixerStreamSrcPair> + ShellMixerStreamSrcPairSpec,
    {
        match elem_id.get_name().as_str() {
            Self::MIXER_STREAM_SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::MIXER_STREAM_SRC_PAIRS.iter()
                        .take(S::MAXIMUM_STREAM_SRC_PAIR_COUNT)
                        .position(|s| s == segment.data.as_ref())
                        .expect("Programming error...");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub fn write<T, S>(&mut self, unit: &SndDice, proto: &T, segment: &mut TcKonnektSegment<S>,
                       elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where T: TcKonnektSegmentProtocol<FwNode, S>,
              S: TcKonnektSegmentData + AsMut<ShellMixerStreamSrcPair> + ShellMixerStreamSrcPairSpec,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
    {
        match elem_id.get_name().as_str() {
            Self::MIXER_STREAM_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::MIXER_STREAM_SRC_PAIRS.iter()
                        .take(S::MAXIMUM_STREAM_SRC_PAIR_COUNT)
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of stream src pair: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .and_then(|&s| {
                            *segment.data.as_mut() = s;
                            proto.write_segment(&unit.get_node(), segment, timeout_ms)
                        })
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

pub fn phys_out_src_to_string(src: &ShellPhysOutSrc) -> String {
    match src {
        ShellPhysOutSrc::Stream => "Stream-input",
        ShellPhysOutSrc::Analog01 => "Analog-input-1/2",
        ShellPhysOutSrc::MixerOut01 => "Mixer-output-1/2",
        ShellPhysOutSrc::MixerSend01 => "Mixer-send/1/2",
    }.to_string()
}

pub const PHYS_OUT_SRCS: [ShellPhysOutSrc;4] = [
    ShellPhysOutSrc::Stream,
    ShellPhysOutSrc::Analog01,
    ShellPhysOutSrc::MixerOut01,
    ShellPhysOutSrc::MixerSend01,
];

#[derive(Default, Debug)]
pub struct ShellCoaxIfaceCtl;

impl ShellCoaxIfaceCtl {
    const OUT_SRC_NAME: &'static str = "coaxial-output-source";

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = PHYS_OUT_SRCS.iter()
            .map(|s| phys_out_src_to_string(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OUT_SRC_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|_| ())
    }

    pub fn read<S>(&mut self, segment: &TcKonnektSegment<S>, elem_id: &ElemId, elem_value: &ElemValue)
        -> Result<bool, Error>
        where for<'b> S: TcKonnektSegmentData + AsRef<ShellCoaxOutPairSrc>,
    {
        match elem_id.get_name().as_str() {
            Self::OUT_SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = PHYS_OUT_SRCS.iter()
                        .position(|s| s.eq(&segment.data.as_ref().0))
                        .expect("Programming error");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub fn write<T, S>(&mut self, unit: &SndDice, proto: &T, segment: &mut TcKonnektSegment<S>,
                       elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where T: TcKonnektSegmentProtocol<FwNode, S>,
              for<'b> S: TcKonnektSegmentData + AsMut<ShellCoaxOutPairSrc>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
    {
        match elem_id.get_name().as_str() {
            Self::OUT_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    PHYS_OUT_SRCS.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of clock rate: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .and_then(|&s| {
                            segment.data.as_mut().0 = s;
                            proto.write_segment(&unit.get_node(), segment, timeout_ms)
                        })
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

fn opt_in_fmt_to_string(fmt: &ShellOptInputIfaceFormat) -> String {
    match fmt {
        ShellOptInputIfaceFormat::Adat0to7 => "ADAT-1:8",
        ShellOptInputIfaceFormat::Adat0to5Spdif01 => "ADAT-1:6+S/PDIF-1/2",
        ShellOptInputIfaceFormat::Toslink01Spdif01 => "TOSLINK-1/2+S/PDIF-1/2",
    }.to_string()
}

fn opt_out_fmt_to_string(fmt: &ShellOptOutputIfaceFormat) -> String {
    match fmt {
        ShellOptOutputIfaceFormat::Adat => "ADAT",
        ShellOptOutputIfaceFormat::Spdif => "S/PDIF",
    }.to_string()
}

#[derive(Default, Debug)]
pub struct ShellOptIfaceCtl;

impl ShellOptIfaceCtl {
    const IN_FMT_NAME: &'static str = "optical-input-format";
    const OUT_FMT_NAME: &'static str = "optical-output-format";
    const OUT_SRC_NAME: &'static str = "optical-output-source";

    const IN_FMTS: [ShellOptInputIfaceFormat;3] = [
        ShellOptInputIfaceFormat::Adat0to7,
        ShellOptInputIfaceFormat::Adat0to5Spdif01,
        ShellOptInputIfaceFormat::Toslink01Spdif01,
    ];

    const OUT_FMTS: [ShellOptOutputIfaceFormat;2] = [
        ShellOptOutputIfaceFormat::Adat,
        ShellOptOutputIfaceFormat::Spdif,
    ];

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = Self::IN_FMTS.iter()
            .map(|s| opt_in_fmt_to_string(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::IN_FMT_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<String> = Self::OUT_FMTS.iter()
            .map(|s| opt_out_fmt_to_string(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OUT_FMT_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<String> = PHYS_OUT_SRCS.iter()
            .map(|s| phys_out_src_to_string(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OUT_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    pub fn read<S>(&mut self, segment: &TcKonnektSegment<S>, elem_id: &ElemId, elem_value: &ElemValue)
        -> Result<bool, Error>
        where for<'b> S: TcKonnektSegmentData + AsRef<ShellOptIfaceConfig>,
    {
        match elem_id.get_name().as_str() {
            Self::IN_FMT_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let state = segment.data.as_ref();
                    let pos = Self::IN_FMTS.iter()
                        .position(|f| f.eq(&state.input_format))
                        .expect("Programming error");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::OUT_FMT_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let state = segment.data.as_ref();
                    let pos = Self::OUT_FMTS.iter()
                        .position(|f| f.eq(&state.output_format))
                        .expect("Programming error");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::OUT_SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let state = segment.data.as_ref();
                    let pos = PHYS_OUT_SRCS.iter()
                        .position(|s| s.eq(&state.output_source.0))
                        .expect("Programming error");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub fn write<T, S>(&mut self, unit: &SndDice, proto: &T, segment: &mut TcKonnektSegment<S>,
                       elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where T: TcKonnektSegmentProtocol<FwNode, S>,
              for<'b> S: TcKonnektSegmentData + AsMut<ShellOptIfaceConfig>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
    {
        match elem_id.get_name().as_str() {
            Self::IN_FMT_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::IN_FMTS.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of clock rate: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .and_then(|&f| {
                            let mut state = segment.data.as_mut();
                            state.input_format = f;
                            proto.write_segment(&unit.get_node(), segment, timeout_ms)
                        })
                })
                .map(|_| true)
            }
            Self::OUT_FMT_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::OUT_FMTS.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of clock rate: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .and_then(|&f| {
                            let mut state = segment.data.as_mut();
                            state.output_format = f;
                            proto.write_segment(&unit.get_node(), segment, timeout_ms)
                        })
                })
                .map(|_| true)
            }
            Self::OUT_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    PHYS_OUT_SRCS.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of clock rate: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .and_then(|&s| {
                            let mut state = segment.data.as_mut();
                            state.output_source.0 = s;
                            proto.write_segment(&unit.get_node(), segment, timeout_ms)
                        })
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub struct ShellKnobCtl{
    pub notified_elem_list: Vec<ElemId>,
}

impl ShellKnobCtl {
    const TARGET_NAME: &'static str = "knob-target";

    const K8_TARGETS: [&'static str;4] = [
        "Analog-1",
        "Analog-2",
        "S/PDIF-1/2",
        "Configurable",
    ];
    const K24D_KLIVE_TARGETS: [&'static str;4] = [
        "Analog-1",
        "Analog-2",
        "Analog-3/4",
        "Configurable",
    ];
    const ITWIN_TARGETS: [&'static str;4] = [
        "Channel-strip-1",
        "Channel-strip-2",
        "Reverb-1/2",
        "Mixer-1/2",
    ];
    const TARGET_COUNT: u32 = 4;

    pub fn load<S>(&mut self, _: &TcKonnektSegment<S>, card_cntr: &mut CardCntr)
        -> Result<(), Error>
        where S: TcKonnektSegmentData + ShellKnobTargetSpec,
    {
        let labels = if S::HAS_SPDIF {
            Self::K8_TARGETS
        } else if S::HAS_EFFECTS {
            Self::ITWIN_TARGETS
        } else {
            Self::K24D_KLIVE_TARGETS
        };
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::TARGET_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))
    }

    pub fn read<S>(&mut self, segment: &TcKonnektSegment<S>, elem_id: &ElemId, elem_value: &ElemValue)
        -> Result<bool, Error>
        where S: TcKonnektSegmentData + AsRef<ShellKnobTarget>,
    {
        match elem_id.get_name().as_str() {
            Self::TARGET_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let state = segment.data.as_ref();
                    if state.0 >= Self::TARGET_COUNT {
                        let msg = format!("Unexpected value for index of program: {}", state.0);
                        Err(Error::new(FileError::Io, &msg))
                    } else {
                        Ok(state.0)
                    }
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub fn write<T, S>(&mut self, unit: &SndDice, proto: &T, segment: &mut TcKonnektSegment<S>,
                       elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where T: TcKonnektSegmentProtocol<FwNode, S>,
              for<'b> S: TcKonnektSegmentData + AsMut<ShellKnobTarget>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
    {
        match elem_id.get_name().as_str() {
            Self::TARGET_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    if val >= Self::TARGET_COUNT {
                        let msg = format!("Invalid value for index of program: {}", val);
                        Err(Error::new(FileError::Io, &msg))
                    } else {
                        segment.data.as_mut().0 = val;
                        proto.write_segment(&unit.get_node(), segment, timeout_ms)
                    }
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub struct ShellKnob2Ctl;

impl ShellKnob2Ctl {
    const KNOB2_NAME: &'static str = "configurable-knob-target";

    const K8_LABELS: [&'static str;2] = [
        "Stream-input-1/2",
        "Mixer-1/2",
    ];
    const K24D_LABELS: [&'static str;8] = [
        "Digital-1/2",
        "Digital-3/4",
        "Digital-5/6",
        "Digital-7/8",
        "Stream",
        "Reverb-1/2",
        "Mixer-1/2",
        "Tune-pitch-tone",
    ];
    const KLIVE_LABELS: [&'static str;9] = [
        "Digital-1/2",
        "Digital-3/4",
        "Digital-5/6",
        "Digital-7/8",
        "Stream",
        "Reverb-1/2",
        "Mixer-1/2",
        "Tune-pitch-tone",
        "Midi-send",
    ];

    pub fn load<S>(&mut self, _: &TcKonnektSegment<S>, card_cntr: &mut CardCntr)
        -> Result<(), Error>
        where S: TcKonnektSegmentData + ShellKnob2TargetSpec,
    {
        let labels = if S::KNOB2_TARGET_COUNT == 9 {
            &Self::KLIVE_LABELS[..]
        } else if S::KNOB2_TARGET_COUNT == 8 {
            &Self::K24D_LABELS[..]
        } else if S::KNOB2_TARGET_COUNT == 2 {
            &Self::K8_LABELS[..]
        } else {
            unreachable!();
        };
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::KNOB2_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|_| ())
    }

    pub fn read<S>(&mut self, segment: &TcKonnektSegment<S>, elem_id: &ElemId, elem_value: &ElemValue)
        -> Result<bool, Error>
        where S: TcKonnektSegmentData + AsRef<ShellKnob2Target> + ShellKnob2TargetSpec,
    {
        match elem_id.get_name().as_str() {
            Self::KNOB2_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let state = segment.data.as_ref();
                    if state.0 >= S::KNOB2_TARGET_COUNT as u32 {
                        let msg = format!("Unexpected value for index of program: {}", state.0);
                        Err(Error::new(FileError::Io, &msg))
                    } else {
                        Ok(state.0)
                    }
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub fn write<T, S>(&mut self, unit: &SndDice, proto: &T, segment: &mut TcKonnektSegment<S>,
                       elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where T: TcKonnektSegmentProtocol<FwNode, S>,
              for<'b> S: TcKonnektSegmentData + AsMut<ShellKnob2Target> + ShellKnob2TargetSpec,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
    {
        match elem_id.get_name().as_str() {
            Self::KNOB2_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    if val >= S::KNOB2_TARGET_COUNT as u32 {
                        let msg = format!("Invalid value for index of program: {}", val);
                        Err(Error::new(FileError::Io, &msg))
                    } else {
                        segment.data.as_mut().0 = val;
                        proto.write_segment(&unit.get_node(), segment, timeout_ms)
                    }
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
