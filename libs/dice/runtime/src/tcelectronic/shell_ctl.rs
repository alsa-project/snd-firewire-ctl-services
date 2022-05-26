// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, dice_protocols::tcelectronic::shell::*};

fn analog_jack_state_to_str(state: &ShellAnalogJackState) -> &'static str {
    match state {
        ShellAnalogJackState::FrontSelected => "Front-selected",
        ShellAnalogJackState::FrontInserted => "Front-inserted",
        ShellAnalogJackState::FrontInsertedAttenuated => "Front-inserted-attenuated",
        ShellAnalogJackState::RearSelected => "Rear-selected",
        ShellAnalogJackState::RearInserted => "Rear-inserted",
    }
}

// TODO: For Jack detection in ALSA applications.
const ANALOG_JACK_STATE_NAME: &str = "analog-jack-state";

pub trait ShellHwStateCtlOperation<S, T>: FirewireLedCtlOperation<S, T>
where
    S: TcKonnektSegmentData,
    TcKonnektSegment<S>: TcKonnektSegmentSpec + TcKonnektNotifiedSegmentSpec,
    T: SegmentOperation<S>,
{
    fn hw_state(&self) -> &ShellHwState;
    fn hw_state_mut(&mut self) -> &mut ShellHwState;

    const ANALOG_JACK_STATE_LABELS: [ShellAnalogJackState; 5] = [
        ShellAnalogJackState::FrontSelected,
        ShellAnalogJackState::FrontInserted,
        ShellAnalogJackState::FrontInsertedAttenuated,
        ShellAnalogJackState::RearSelected,
        ShellAnalogJackState::RearInserted,
    ];

    fn load_hw_state(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let mut notified_elem_id_list = Vec::new();

        let labels = Self::ANALOG_JACK_STATE_LABELS
            .iter()
            .map(|s| analog_jack_state_to_str(s))
            .collect::<Vec<_>>();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, ANALOG_JACK_STATE_NAME, 0);
        card_cntr
            .add_enum_elems(
                &elem_id,
                1,
                SHELL_ANALOG_JACK_STATE_COUNT,
                &labels,
                None,
                false,
            )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        self.load_firewire_led(card_cntr)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn read_hw_state(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.read_firewire_led(elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.get_name().as_str() {
                ANALOG_JACK_STATE_NAME => ElemValueAccessor::<u32>::set_vals(
                    elem_value,
                    SHELL_ANALOG_JACK_STATE_COUNT,
                    |idx| {
                        let pos = Self::ANALOG_JACK_STATE_LABELS
                            .iter()
                            .position(|s| self.hw_state().analog_jack_states[idx].eq(s))
                            .unwrap();
                        Ok(pos as u32)
                    },
                )
                .map(|_| true),
                _ => Ok(false),
            }
        }
    }

    fn write_hw_state(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        self.write_firewire_led(unit, req, elem_id, elem_value, timeout_ms)
    }
}

const MIXER_STREAM_SRC_PAIR_GAIN_NAME: &str = "mixer-stream-source-gain";
const MIXER_STREAM_SRC_PAIR_PAN_NAME: &str = "mixer-stream-source-pan";
const MIXER_STREAM_SRC_PAIR_MUTE_NAME: &str = "mixer-stream-source-mute";
const REVERB_STREAM_SRC_PAIR_GAIN_NAME: &str = "send-stream-source-gain";
const MIXER_PHYS_SRC_STEREO_LINK_NAME: &str = "mixer-phys-source-link";

const MIXER_PHYS_SRC_GAIN_NAME: &str = "mixer-phys-source-gain";
const MIXER_PHYS_SRC_PAN_NAME: &str = "mixer-phys-source-pan";
const MIXER_PHYS_SRC_MUTE_NAME: &str = "mixer-phys-source-mute";

const REVERB_PHYS_SRC_GAIN_NAME: &str = "send-phys-source-gain";

const MIXER_OUT_DIM_NAME: &str = "mixer-out-dim-enable";
const MIXER_OUT_VOL_NAME: &str = "mixer-out-volume";
const MIXER_OUT_DIM_VOL_NAME: &str = "mixer-out-dim-volume";

const STREAM_IN_METER_NAME: &str = "stream-input-meters";
const ANALOG_IN_METER_NAME: &str = "analog-input-meters";
const DIGITAL_IN_METER_NAME: &str = "digital-input-meters";
const MIXER_OUT_METER_NAME: &str = "mixer-output-meters";

pub trait ShellMixerCtlOperation<S, T, U>
where
    S: TcKonnektSegmentData,
    T: TcKonnektSegmentData,
    TcKonnektSegment<S>: TcKonnektSegmentSpec + TcKonnektNotifiedSegmentSpec,
    TcKonnektSegment<T>: TcKonnektSegmentSpec,
    U: SegmentOperation<S> + SegmentOperation<T>,
{
    const LEVEL_MIN: i32 = -1000;
    const LEVEL_MAX: i32 = 0;
    const LEVEL_STEP: i32 = 1;
    const LEVEL_TLV: DbInterval = DbInterval {
        min: -9400,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    const PAN_MIN: i32 = -50;
    const PAN_MAX: i32 = 50;
    const PAN_STEP: i32 = 1;

    fn state_segment(&self) -> &TcKonnektSegment<S>;
    fn state_segment_mut(&mut self) -> &mut TcKonnektSegment<S>;
    fn meter_segment_mut(&mut self) -> &mut TcKonnektSegment<T>;
    fn state(&self) -> &ShellMixerState;
    fn state_mut(&mut self) -> &mut ShellMixerState;
    fn meter(&self) -> &ShellMixerMeter;
    fn meter_mut(&mut self) -> &mut ShellMixerMeter;
    fn enabled(&self) -> bool;

    fn load_mixer(
        &mut self,
        card_cntr: &mut CardCntr,
    ) -> Result<(Vec<ElemId>, Vec<ElemId>), Error> {
        let mut notified_elem_id_list = Vec::new();

        // For stream source of mixer.
        Self::state_add_elem_level(
            card_cntr,
            &mut notified_elem_id_list,
            MIXER_STREAM_SRC_PAIR_GAIN_NAME,
            1,
        )?;
        Self::state_add_elem_pan(
            card_cntr,
            &mut notified_elem_id_list,
            MIXER_STREAM_SRC_PAIR_PAN_NAME,
            1,
        )?;
        Self::state_add_elem_bool(
            card_cntr,
            &mut notified_elem_id_list,
            MIXER_STREAM_SRC_PAIR_MUTE_NAME,
            1,
        )?;
        Self::state_add_elem_level(
            card_cntr,
            &mut notified_elem_id_list,
            REVERB_STREAM_SRC_PAIR_GAIN_NAME,
            1,
        )?;

        // For physical sources of mixer.
        let labels: Vec<String> = (0..self.state().analog.len())
            .map(|i| format!("Analog-{}/{}", i + 1, i + 2))
            .chain((0..self.state().digital.len()).map(|i| format!("Digital-{}/{}", i + 1, i + 2)))
            .collect();
        Self::state_add_elem_bool(
            card_cntr,
            &mut notified_elem_id_list,
            MIXER_PHYS_SRC_STEREO_LINK_NAME,
            labels.len(),
        )?;

        let labels: Vec<String> = (0..(self.state().analog.len() * 2))
            .map(|i| format!("Analog-{}", i + 1))
            .chain((0..(self.state().digital.len() * 2)).map(|i| format!("Digital-{}", i + 1)))
            .collect();
        Self::state_add_elem_level(
            card_cntr,
            &mut notified_elem_id_list,
            MIXER_PHYS_SRC_GAIN_NAME,
            labels.len(),
        )?;
        Self::state_add_elem_pan(
            card_cntr,
            &mut notified_elem_id_list,
            MIXER_PHYS_SRC_PAN_NAME,
            labels.len(),
        )?;
        Self::state_add_elem_bool(
            card_cntr,
            &mut notified_elem_id_list,
            MIXER_PHYS_SRC_MUTE_NAME,
            labels.len(),
        )?;
        Self::state_add_elem_level(
            card_cntr,
            &mut notified_elem_id_list,
            REVERB_PHYS_SRC_GAIN_NAME,
            labels.len(),
        )?;

        // For output of mixer.
        Self::state_add_elem_bool(card_cntr, &mut notified_elem_id_list, MIXER_OUT_DIM_NAME, 1)?;
        Self::state_add_elem_level(card_cntr, &mut notified_elem_id_list, MIXER_OUT_VOL_NAME, 1)?;
        Self::state_add_elem_level(
            card_cntr,
            &mut notified_elem_id_list,
            MIXER_OUT_DIM_VOL_NAME,
            1,
        )?;

        // For meter.
        let mut measured_elem_id_list = Vec::new();
        let labels = (0..self.meter().stream_inputs.len())
            .map(|i| format!("Stream-input-{}", i))
            .collect::<Vec<_>>();
        Self::meter_add_elem_level(
            card_cntr,
            &mut measured_elem_id_list,
            STREAM_IN_METER_NAME,
            labels.len(),
        )?;

        let labels = (0..self.meter().analog_inputs.len())
            .map(|i| format!("Analog-input-{}", i))
            .collect::<Vec<_>>();
        Self::meter_add_elem_level(
            card_cntr,
            &mut measured_elem_id_list,
            ANALOG_IN_METER_NAME,
            labels.len(),
        )?;

        let labels = (0..self.meter().digital_inputs.len())
            .map(|i| format!("Digital-input-{}", i))
            .collect::<Vec<_>>();
        Self::meter_add_elem_level(
            card_cntr,
            &mut measured_elem_id_list,
            DIGITAL_IN_METER_NAME,
            labels.len(),
        )?;

        let labels = (0..self.meter().main_outputs.len())
            .map(|i| format!("Mixer-output-{}", i))
            .collect::<Vec<_>>();
        Self::meter_add_elem_level(
            card_cntr,
            &mut measured_elem_id_list,
            MIXER_OUT_METER_NAME,
            labels.len(),
        )?;

        Ok((notified_elem_id_list, measured_elem_id_list))
    }

    fn read_mixer(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_mixer_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_mixer_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn state_read_phys_src<V, F>(&self, elem_value: &mut ElemValue, cb: F) -> Result<bool, Error>
    where
        F: Fn(&MonitorSrcParam) -> Result<V, Error>,
        V: Default + Copy + Eq,
        ElemValue: ElemValueAccessor<V>,
    {
        let analog_count = self.state().analog.len();
        let digital_count = self.state().digital.len();
        let count = (analog_count + digital_count) * 2;

        ElemValueAccessor::<V>::set_vals(elem_value, count, |idx| {
            let i = idx / 2;
            let ch = idx % 2;
            let src_pair = if i < analog_count {
                &self.state().analog[i]
            } else {
                &self.state().digital[i - analog_count]
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

    fn write_mixer(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIXER_STREAM_SRC_PAIR_GAIN_NAME => {
                self.state_write(unit, req, new, timeout_ms, |state, val| {
                    state.stream.left.gain_to_mixer = val;
                    Ok(())
                })
            }
            MIXER_STREAM_SRC_PAIR_PAN_NAME => {
                self.state_write(unit, req, new, timeout_ms, |state, val| {
                    state.stream.left.pan_to_mixer = val;
                    Ok(())
                })
            }
            MIXER_STREAM_SRC_PAIR_MUTE_NAME => {
                self.state_write(unit, req, new, timeout_ms, |state, val| {
                    state.mutes.stream = val;
                    Ok(())
                })
            }
            REVERB_STREAM_SRC_PAIR_GAIN_NAME => {
                self.state_write(unit, req, new, timeout_ms, |state, val| {
                    state.stream.left.gain_to_send = val;
                    Ok(())
                })
            }
            MIXER_PHYS_SRC_STEREO_LINK_NAME => {
                let analog_count = self.state().analog.len();
                let digital_count = self.state().digital.len();
                let count = analog_count + digital_count;

                ElemValueAccessor::<bool>::get_vals(new, old, count, |idx, val| {
                    if idx < analog_count {
                        self.state_mut().analog[idx].stereo_link = val;
                    } else {
                        self.state_mut().digital[idx - analog_count].stereo_link = val;
                    }
                    Ok(())
                })?;

                U::write_segment(
                    req,
                    &mut unit.get_node(),
                    self.state_segment_mut(),
                    timeout_ms,
                )
                .map(|_| true)
            }
            MIXER_PHYS_SRC_GAIN_NAME => {
                self.state_write_phys_src(unit, req, new, old, timeout_ms, |param, val| {
                    param.gain_to_mixer = val;
                    Ok(())
                })
            }
            MIXER_PHYS_SRC_PAN_NAME => {
                self.state_write_phys_src(unit, req, new, old, timeout_ms, |param, val| {
                    param.pan_to_mixer = val;
                    Ok(())
                })
            }
            MIXER_PHYS_SRC_MUTE_NAME => {
                let analog_count = self.state().mutes.analog.len();
                let digital_count = self.state().mutes.digital.len();
                let count = analog_count + digital_count;

                ElemValueAccessor::<bool>::get_vals(new, old, count, |idx, val| {
                    if idx < analog_count {
                        self.state_mut().mutes.analog[idx] = val;
                    } else {
                        self.state_mut().mutes.digital[idx - analog_count] = val;
                    };
                    Ok(())
                })?;

                U::write_segment(
                    req,
                    &mut unit.get_node(),
                    self.state_segment_mut(),
                    timeout_ms,
                )
                .map(|_| true)
            }
            REVERB_PHYS_SRC_GAIN_NAME => {
                self.state_write_phys_src(unit, req, new, old, timeout_ms, |param, val| {
                    param.gain_to_send = val;
                    Ok(())
                })
            }
            MIXER_OUT_DIM_NAME => self.state_write(unit, req, new, timeout_ms, |state, val| {
                state.output_dim_enable = val;
                Ok(())
            }),
            MIXER_OUT_VOL_NAME => self.state_write(unit, req, new, timeout_ms, |state, val| {
                state.output_volume = val;
                Ok(())
            }),
            MIXER_OUT_DIM_VOL_NAME => self.state_write(unit, req, new, timeout_ms, |state, val| {
                state.output_dim_volume = val;
                Ok(())
            }),
            _ => Ok(false),
        }
    }

    fn state_write<V, F>(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_value: &ElemValue,
        timeout_ms: u32,
        cb: F,
    ) -> Result<bool, Error>
    where
        F: Fn(&mut ShellMixerState, V) -> Result<(), Error>,
        V: Default + Copy + Eq,
        ElemValue: ElemValueAccessor<V>,
    {
        ElemValueAccessor::<V>::get_val(elem_value, |val| cb(self.state_mut(), val))?;
        U::write_segment(
            req,
            &mut unit.get_node(),
            self.state_segment_mut(),
            timeout_ms,
        )
        .map(|_| true)
    }

    fn state_write_phys_src<V, F>(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        new: &ElemValue,
        old: &ElemValue,
        timeout_ms: u32,
        cb: F,
    ) -> Result<bool, Error>
    where
        F: Fn(&mut MonitorSrcParam, V) -> Result<(), Error>,
        V: Default + Copy + Eq,
        ElemValue: ElemValueAccessor<V>,
    {
        let analog_count = self.state().analog.len();
        let digital_count = self.state().digital.len();
        let count = (analog_count + digital_count) * 2;

        ElemValueAccessor::<V>::get_vals(new, old, count, |idx, val| {
            let i = idx / 2;
            let ch = idx % 2;
            let src_pair = if i < analog_count {
                &mut self.state_mut().analog[i]
            } else {
                &mut self.state_mut().digital[i - analog_count]
            };
            let param = if ch == 0 {
                &mut src_pair.left
            } else {
                &mut src_pair.right
            };
            cb(param, val)
        })?;
        U::write_segment(
            req,
            &mut unit.get_node(),
            self.state_segment_mut(),
            timeout_ms,
        )
        .map(|_| true)
    }

    fn read_mixer_notified_elem(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIXER_STREAM_SRC_PAIR_GAIN_NAME => {
                elem_value.set_int(&[self.state().stream.left.gain_to_mixer]);
                Ok(true)
            }
            MIXER_STREAM_SRC_PAIR_PAN_NAME => {
                elem_value.set_int(&[self.state().stream.left.pan_to_mixer]);
                Ok(true)
            }
            MIXER_STREAM_SRC_PAIR_MUTE_NAME => {
                elem_value.set_bool(&[self.state().mutes.stream]);
                Ok(true)
            }
            REVERB_STREAM_SRC_PAIR_GAIN_NAME => {
                elem_value.set_int(&[self.state().stream.left.gain_to_send]);
                Ok(true)
            }
            MIXER_PHYS_SRC_STEREO_LINK_NAME => {
                let vals: Vec<bool> = self
                    .state()
                    .analog
                    .iter()
                    .chain(self.state().digital.iter())
                    .map(|src| src.stereo_link)
                    .collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            MIXER_PHYS_SRC_GAIN_NAME => {
                self.state_read_phys_src(elem_value, |param| Ok(param.gain_to_mixer))
            }
            MIXER_PHYS_SRC_PAN_NAME => {
                self.state_read_phys_src(elem_value, |param| Ok(param.pan_to_mixer))
            }
            MIXER_PHYS_SRC_MUTE_NAME => {
                let mut vals = self.state().mutes.analog.clone();
                vals.extend_from_slice(&self.state().mutes.digital);
                elem_value.set_bool(&vals);
                Ok(true)
            }
            REVERB_PHYS_SRC_GAIN_NAME => {
                self.state_read_phys_src(elem_value, |param| Ok(param.gain_to_send))
            }
            MIXER_OUT_DIM_NAME => {
                elem_value.set_bool(&[self.state().output_dim_enable]);
                Ok(true)
            }
            MIXER_OUT_VOL_NAME => {
                elem_value.set_int(&[self.state().output_volume]);
                Ok(true)
            }
            MIXER_OUT_DIM_VOL_NAME => {
                elem_value.set_int(&[self.state().output_dim_volume]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn read_mixer_measured_elem(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            STREAM_IN_METER_NAME => {
                elem_value.set_int(&self.meter().stream_inputs);
                Ok(true)
            }
            ANALOG_IN_METER_NAME => {
                elem_value.set_int(&self.meter().analog_inputs);
                Ok(true)
            }
            DIGITAL_IN_METER_NAME => {
                elem_value.set_int(&self.meter().digital_inputs);
                Ok(true)
            }
            MIXER_OUT_METER_NAME => {
                elem_value.set_int(&self.meter().main_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn state_add_elem_level(
        card_cntr: &mut CardCntr,
        notified_elem_id_list: &mut Vec<ElemId>,
        name: &str,
        value_count: usize,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::LEVEL_MIN,
                Self::LEVEL_MAX,
                Self::LEVEL_STEP,
                value_count,
                Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
                true,
            )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
    }

    fn state_add_elem_pan(
        card_cntr: &mut CardCntr,
        notified_elem_id_list: &mut Vec<ElemId>,
        name: &str,
        value_count: usize,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::PAN_MIN,
                Self::PAN_MAX,
                Self::PAN_STEP,
                value_count,
                None,
                true,
            )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
    }

    fn state_add_elem_bool(
        card_cntr: &mut CardCntr,
        notified_elem_id_list: &mut Vec<ElemId>,
        name: &str,
        value_count: usize,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, value_count, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))
    }

    fn meter_add_elem_level(
        card_cntr: &mut CardCntr,
        measured_elem_id_list: &mut Vec<ElemId>,
        name: &str,
        value_count: usize,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::LEVEL_MIN,
                Self::LEVEL_MAX,
                Self::LEVEL_STEP,
                value_count,
                Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
                false,
            )
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))
    }
}

const USE_AS_PLUGIN_NAME: &str = "use-reverb-as-plugin";
const GAIN_NAME: &str = "reverb-return-gain";
const MUTE_NAME: &str = "reverb-return-mute";

pub trait ShellReverbReturnCtlOperation<S, T>
where
    S: TcKonnektSegmentData,
    TcKonnektSegment<S>: TcKonnektSegmentSpec + TcKonnektNotifiedSegmentSpec,
    T: SegmentOperation<S>,
{
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;
    fn reverb_return(&self) -> &ShellReverbReturn;
    fn reverb_return_mut(&mut self) -> &mut ShellReverbReturn;

    const GAIN_MIN: i32 = -1000;
    const GAIN_MAX: i32 = 0;
    const GAIN_STEP: i32 = 1;
    const GAIN_TLV: DbInterval = DbInterval {
        min: -7200,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    fn load_reverb_return(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let mut notified_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, USE_AS_PLUGIN_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::GAIN_MIN,
                Self::GAIN_MAX,
                Self::GAIN_STEP,
                1,
                Some(&Vec::<u32>::from(Self::GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn read_reverb_return(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            USE_AS_PLUGIN_NAME => ElemValueAccessor::<bool>::set_val(elem_value, || {
                Ok(self.reverb_return().plugin_mode)
            })
            .map(|_| true),
            _ => self.read_reverb_return_notified_elem(elem_id, elem_value),
        }
    }

    fn write_reverb_return(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            USE_AS_PLUGIN_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    self.reverb_return_mut().plugin_mode = val;
                    Ok(())
                })?;
                T::write_segment(req, &mut unit.get_node(), self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            GAIN_NAME => {
                ElemValueAccessor::<i32>::get_val(elem_value, |val| {
                    self.reverb_return_mut().return_gain = val;
                    Ok(())
                })?;
                T::write_segment(req, &mut unit.get_node(), self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            MUTE_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    self.reverb_return_mut().return_mute = val;
                    Ok(())
                })?;
                T::write_segment(req, &mut unit.get_node(), self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn read_reverb_return_notified_elem(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            GAIN_NAME => ElemValueAccessor::<i32>::set_val(elem_value, || {
                Ok(self.reverb_return().return_gain)
            })
            .map(|_| true),
            MUTE_NAME => ElemValueAccessor::<bool>::set_val(elem_value, || {
                Ok(self.reverb_return().return_mute)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }
}

fn standalone_src_to_str(src: &ShellStandaloneClkSrc) -> &'static str {
    match src {
        ShellStandaloneClkSrc::Optical => "Optical",
        ShellStandaloneClkSrc::Coaxial => "Coaxial",
        ShellStandaloneClkSrc::Internal => "Internal",
    }
}

const SRC_NAME: &str = "standalone-clock-source";

pub trait ShellStandaloneCtlOperation<S, T>: StandaloneCtlOperation<S, T>
where
    S: TcKonnektSegmentData + ShellStandaloneClkSpec,
    TcKonnektSegment<S>: TcKonnektSegmentSpec,
    T: SegmentOperation<S>,
{
    fn standalone_src(&self) -> &ShellStandaloneClkSrc;
    fn standalone_src_mut(&mut self) -> &mut ShellStandaloneClkSrc;

    fn load_standalone(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = S::STANDALONE_CLOCK_SOURCES
            .iter()
            .map(|r| standalone_src_to_str(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        self.load_standalone_rate(card_cntr)?;

        Ok(())
    }

    fn read_standalone(&mut self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            SRC_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let pos = S::STANDALONE_CLOCK_SOURCES
                    .iter()
                    .position(|s| self.standalone_src().eq(s))
                    .unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            _ => self.read_standalone_rate(elem_id, elem_value),
        }
    }

    fn write_standalone(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    S::STANDALONE_CLOCK_SOURCES
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of clock source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| *self.standalone_src_mut() = s)
                })?;
                T::write_segment(req, &mut unit.get_node(), self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => self.write_standalone_rate(unit, req, elem_id, elem_value, timeout_ms),
        }
    }
}

fn mixer_stream_src_pair_to_str(src: &ShellMixerStreamSrcPair) -> &'static str {
    match src {
        ShellMixerStreamSrcPair::Stream01 => "Stream-1/2",
        ShellMixerStreamSrcPair::Stream23 => "Stream-3/4",
        ShellMixerStreamSrcPair::Stream45 => "Stream-5/6",
        ShellMixerStreamSrcPair::Stream67 => "Stream-7/8",
        ShellMixerStreamSrcPair::Stream89 => "Stream-9/10",
        ShellMixerStreamSrcPair::Stream1011 => "Stream-11/12",
        ShellMixerStreamSrcPair::Stream1213 => "Stream-13/14",
    }
}

const MIXER_STREAM_SRC_NAME: &str = "mixer-stream-soruce";

pub trait ShellMixerStreamSrcCtlOperation<S, T>
where
    S: TcKonnektSegmentData + ShellMixerStreamSrcPairSpec,
    TcKonnektSegment<S>: TcKonnektSegmentSpec + TcKonnektNotifiedSegmentSpec,
    T: SegmentOperation<S>,
{
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;
    fn mixer_stream_src(&self) -> &ShellMixerStreamSrcPair;
    fn mixer_stream_src_mut(&mut self) -> &mut ShellMixerStreamSrcPair;

    const MIXER_STREAM_SRC_PAIRS: [ShellMixerStreamSrcPair; 7] = [
        ShellMixerStreamSrcPair::Stream01,
        ShellMixerStreamSrcPair::Stream23,
        ShellMixerStreamSrcPair::Stream45,
        ShellMixerStreamSrcPair::Stream67,
        ShellMixerStreamSrcPair::Stream89,
        ShellMixerStreamSrcPair::Stream1011,
        ShellMixerStreamSrcPair::Stream1213,
    ];

    fn load_mixer_stream_src(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::MIXER_STREAM_SRC_PAIRS
            .iter()
            .take(S::MAXIMUM_STREAM_SRC_PAIR_COUNT)
            .map(|s| mixer_stream_src_pair_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_STREAM_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read_mixer_stream_src(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIXER_STREAM_SRC_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let pos = Self::MIXER_STREAM_SRC_PAIRS
                    .iter()
                    .take(S::MAXIMUM_STREAM_SRC_PAIR_COUNT)
                    .position(|s| self.mixer_stream_src().eq(s))
                    .unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write_mixer_stream_src(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIXER_STREAM_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::MIXER_STREAM_SRC_PAIRS
                        .iter()
                        .take(S::MAXIMUM_STREAM_SRC_PAIR_COUNT)
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index of stream src pair: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| *self.mixer_stream_src_mut() = s)
                })?;
                T::write_segment(req, &mut unit.get_node(), self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

pub fn phys_out_src_to_str(src: &ShellPhysOutSrc) -> &'static str {
    match src {
        ShellPhysOutSrc::Stream => "Stream-input",
        ShellPhysOutSrc::Analog01 => "Analog-input-1/2",
        ShellPhysOutSrc::MixerOut01 => "Mixer-output-1/2",
        ShellPhysOutSrc::MixerSend01 => "Mixer-send/1/2",
    }
}

pub const PHYS_OUT_SRCS: [ShellPhysOutSrc; 4] = [
    ShellPhysOutSrc::Stream,
    ShellPhysOutSrc::Analog01,
    ShellPhysOutSrc::MixerOut01,
    ShellPhysOutSrc::MixerSend01,
];

const COAX_OUT_SRC_NAME: &str = "coaxial-output-source";

pub trait ShellCoaxIfaceCtlOperation<S, T>
where
    S: TcKonnektSegmentData,
    TcKonnektSegment<S>: TcKonnektSegmentSpec + TcKonnektNotifiedSegmentSpec,
    T: SegmentOperation<S>,
{
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;
    fn coax_out_src(&self) -> &ShellCoaxOutPairSrc;
    fn coax_out_src_mut(&mut self) -> &mut ShellCoaxOutPairSrc;

    fn load_coax_out_src(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = PHYS_OUT_SRCS
            .iter()
            .map(|s| phys_out_src_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, COAX_OUT_SRC_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|_| ())
    }

    fn read_coax_out_src(
        &mut self,
        elem_id: &ElemId,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            COAX_OUT_SRC_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let pos = PHYS_OUT_SRCS
                    .iter()
                    .position(|s| self.coax_out_src().0.eq(s))
                    .unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write_coax_out_src(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            COAX_OUT_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    PHYS_OUT_SRCS
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of clock rate: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| self.coax_out_src_mut().0 = s)
                })?;
                T::write_segment(req, &mut unit.get_node(), self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

fn opt_in_fmt_to_str(fmt: &ShellOptInputIfaceFormat) -> &'static str {
    match fmt {
        ShellOptInputIfaceFormat::Adat0to7 => "ADAT-1:8",
        ShellOptInputIfaceFormat::Adat0to5Spdif01 => "ADAT-1:6+S/PDIF-1/2",
        ShellOptInputIfaceFormat::Toslink01Spdif01 => "TOSLINK-1/2+S/PDIF-1/2",
    }
}

fn opt_out_fmt_to_str(fmt: &ShellOptOutputIfaceFormat) -> &'static str {
    match fmt {
        ShellOptOutputIfaceFormat::Adat => "ADAT",
        ShellOptOutputIfaceFormat::Spdif => "S/PDIF",
    }
}

const OPT_IN_FMT_NAME: &str = "optical-input-format";
const OPT_OUT_FMT_NAME: &str = "optical-output-format";
const OPT_OUT_SRC_NAME: &str = "optical-output-source";

pub trait ShellOptIfaceCtl<S, T>
where
    S: TcKonnektSegmentData,
    TcKonnektSegment<S>: TcKonnektSegmentSpec + TcKonnektNotifiedSegmentSpec,
    T: SegmentOperation<S>,
{
    const IN_FMTS: [ShellOptInputIfaceFormat; 3] = [
        ShellOptInputIfaceFormat::Adat0to7,
        ShellOptInputIfaceFormat::Adat0to5Spdif01,
        ShellOptInputIfaceFormat::Toslink01Spdif01,
    ];

    const OUT_FMTS: [ShellOptOutputIfaceFormat; 2] = [
        ShellOptOutputIfaceFormat::Adat,
        ShellOptOutputIfaceFormat::Spdif,
    ];

    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;
    fn opt_iface_config(&self) -> &ShellOptIfaceConfig;
    fn opt_iface_config_mut(&mut self) -> &mut ShellOptIfaceConfig;

    fn load_opt_iface_config(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::IN_FMTS.iter().map(|s| opt_in_fmt_to_str(s)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OPT_IN_FMT_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = Self::OUT_FMTS
            .iter()
            .map(|s| opt_out_fmt_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OPT_OUT_FMT_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = PHYS_OUT_SRCS
            .iter()
            .map(|s| phys_out_src_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OPT_OUT_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read_opt_iface_config(
        &mut self,
        elem_id: &ElemId,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OPT_IN_FMT_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let pos = Self::IN_FMTS
                    .iter()
                    .position(|f| self.opt_iface_config().input_format.eq(f))
                    .unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            OPT_OUT_FMT_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let pos = Self::OUT_FMTS
                    .iter()
                    .position(|f| self.opt_iface_config().output_format.eq(f))
                    .unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            OPT_OUT_SRC_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let pos = PHYS_OUT_SRCS
                    .iter()
                    .position(|s| self.opt_iface_config().output_source.0.eq(s))
                    .unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write_opt_iface_config(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OPT_IN_FMT_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::IN_FMTS
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index of optical input format: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&f| self.opt_iface_config_mut().input_format = f)
                })?;
                T::write_segment(req, &mut unit.get_node(), self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            OPT_OUT_FMT_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::OUT_FMTS
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index of optical output format: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&f| self.opt_iface_config_mut().output_format = f)
                })?;
                T::write_segment(req, &mut unit.get_node(), self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            OPT_OUT_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    PHYS_OUT_SRCS
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index of optical output source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| self.opt_iface_config_mut().output_source.0 = s)
                })?;
                T::write_segment(req, &mut unit.get_node(), self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const TARGET_NAME: &str = "knob-target";

pub trait ShellKnobCtlOperation<S, T>
where
    S: TcKonnektSegmentData,
    TcKonnektSegment<S>: TcKonnektSegmentSpec + TcKonnektNotifiedSegmentSpec,
    T: SegmentOperation<S>,
{
    const TARGETS: [&'static str; 4];

    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;
    fn knob_target(&self) -> &ShellKnobTarget;
    fn knob_target_mut(&mut self) -> &mut ShellKnobTarget;

    fn load_knob_target(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, TARGET_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &Self::TARGETS, None, true)
    }

    fn read_knob_target(
        &mut self,
        elem_id: &ElemId,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            TARGET_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let state = self.knob_target();
                if state.0 >= Self::TARGETS.len() as u32 {
                    let msg = format!("Unexpected index of program: {}", state.0);
                    Err(Error::new(FileError::Io, &msg))
                } else {
                    Ok(state.0)
                }
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write_knob_target(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            TARGET_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    if val >= Self::TARGETS.len() as u32 {
                        let msg = format!("Invalid index of program: {}", val);
                        Err(Error::new(FileError::Io, &msg))
                    } else {
                        self.knob_target_mut().0 = val;
                        Ok(())
                    }
                })?;
                T::write_segment(req, &mut unit.get_node(), self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const KNOB2_NAME: &str = "configurable-knob-target";

pub trait ShellKnob2CtlOperation<S, T>
where
    S: TcKonnektSegmentData,
    TcKonnektSegment<S>: TcKonnektSegmentSpec + TcKonnektNotifiedSegmentSpec,
    T: SegmentOperation<S>,
{
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;
    fn knob2_target(&self) -> &ShellKnob2Target;
    fn knob2_target_mut(&mut self) -> &mut ShellKnob2Target;

    const TARGETS: &'static [&'static str];

    fn load_knob2_target(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, KNOB2_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &Self::TARGETS, None, true)
    }

    fn read_knob2_target(
        &mut self,
        elem_id: &ElemId,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            KNOB2_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let state = self.knob2_target();
                if state.0 >= Self::TARGETS.len() as u32 {
                    let msg = format!("Invalid index of program: {}", state.0);
                    Err(Error::new(FileError::Io, &msg))
                } else {
                    Ok(state.0)
                }
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write_knob2_target(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            KNOB2_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    if val >= Self::TARGETS.len() as u32 {
                        let msg = format!("Invalid value for index of program: {}", val);
                        Err(Error::new(FileError::Io, &msg))
                    } else {
                        self.knob2_target_mut().0 = val;
                        Ok(())
                    }
                })?;
                T::write_segment(req, &mut unit.get_node(), self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
