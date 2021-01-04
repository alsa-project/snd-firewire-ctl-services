// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::Error;

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use hinawa::{FwNode, SndDice, SndUnitExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use dice_protocols::tcelectronic::*;
use dice_protocols::tcelectronic::fw_led::*;
use dice_protocols::tcelectronic::shell::*;

use core::card_cntr::*;
use core::elem_value_accessor::*;

use super::fw_led_ctl::*;

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

impl<'a> HwStateCtl {
    // TODO: For Jack detection in ALSA applications.
    const ANALOG_JACK_STATE_NAME: &'a str = "analog-jack-state";

    const ANALOG_JACK_STATE_LABELS: &'a [ShellAnalogJackState] = &[
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

impl<'a> ShellMixerCtl {
    const MIXER_STREAM_SRC_PAIR_GAIN_NAME: &'a str = "mixer-stream-source-gain";
    const MIXER_STREAM_SRC_PAIR_PAN_NAME: &'a str = "mixer-stream-source-pan";
    const MIXER_STREAM_SRC_PAIR_MUTE_NAME: &'a str = "mixer-stream-source-mute";
    const REVERB_STREAM_SRC_PAIR_GAIN_NAME: &'a str = "send-stream-source-gain";
    const MIXER_PHYS_SRC_STEREO_LINK_NAME: &'a str = "mixer-phys-source-link";

    const MIXER_PHYS_SRC_GAIN_NAME: &'a str = "mixer-phys-source-gain";
    const MIXER_PHYS_SRC_PAN_NAME: &'a str = "mixer-phys-source-pan";
    const MIXER_PHYS_SRC_MUTE_NAME: &'a str = "mixer-phys-source-mute";

    const REVERB_PHYS_SRC_GAIN_NAME: &'a str = "send-phys-source-gain";

    const MIXER_OUT_DIM_NAME: &'a str = "mixer-out-dim-enable";
    const MIXER_OUT_VOL_NAME: &'a str = "mixer-out-volume";
    const MIXER_OUT_DIM_VOL_NAME: &'a str = "mixer-out-dim-volume";

    const STREAM_IN_METER_NAME: &'a str = "stream-input-meters";
    const ANALOG_IN_METER_NAME: &'a str = "analog-input-meters";
    const DIGITAL_IN_METER_NAME: &'a str = "digital-input-meters";
    const MIXER_OUT_METER_NAME: &'a str = "mixer-output-meters";

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

impl<'a> ShellReverbReturnCtl {
    const USE_AS_PLUGIN_NAME: &'a str = "use-reverb-as-plugin";
    const GAIN_NAME: &'a str = "reverb-return-gain";
    const MUTE_NAME: &'a str = "reverb-return-mute";

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
