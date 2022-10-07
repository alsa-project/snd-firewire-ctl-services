// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, protocols::tcelectronic::shell::*};

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
    S: Clone,
    T: TcKonnektSegmentOperation<S> + TcKonnektMutableSegmentOperation<S>,
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
            match elem_id.name().as_str() {
                ANALOG_JACK_STATE_NAME => {
                    let vals: Vec<u32> = self
                        .hw_state()
                        .analog_jack_states
                        .iter()
                        .map(|state| {
                            let pos = Self::ANALOG_JACK_STATE_LABELS
                                .iter()
                                .position(|s| state.eq(s))
                                .unwrap();
                            pos as u32
                        })
                        .collect();
                    elem_value.set_enum(&vals);
                    Ok(true)
                }
                _ => Ok(false),
            }
        }
    }

    fn write_hw_state(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        self.write_firewire_led(req, node, elem_id, elem_value, timeout_ms)
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

fn phys_src_pair_iter(state: &ShellMixerState) -> impl Iterator<Item = &ShellMonitorSrcPair> {
    state.analog.iter().chain(state.digital.iter())
}

fn phys_src_pair_iter_mut(
    state: &mut ShellMixerState,
) -> impl Iterator<Item = &mut ShellMonitorSrcPair> {
    state.analog.iter_mut().chain(state.digital.iter_mut())
}

fn phys_src_params_iter(state: &ShellMixerState) -> impl Iterator<Item = &MonitorSrcParam> {
    phys_src_pair_iter(state).flat_map(|pair| pair.params.iter())
}

fn phys_src_params_iter_mut(
    state: &mut ShellMixerState,
) -> impl Iterator<Item = &mut MonitorSrcParam> {
    phys_src_pair_iter_mut(state).flat_map(|pair| pair.params.iter_mut())
}

pub trait ShellMixerStateCtlOperation<S, T, U>
where
    S: Clone,
    U: TcKonnektSegmentOperation<S>
        + TcKonnektSegmentOperation<T>
        + TcKonnektMutableSegmentOperation<S>
        + TcKonnektNotifiedSegmentOperation<S>,
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

    fn segment(&self) -> &TcKonnektSegment<S>;
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;

    fn state(params: &S) -> &ShellMixerState;
    fn state_mut(params: &mut S) -> &mut ShellMixerState;

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
        let state = Self::state(&self.segment().data);
        let labels: Vec<String> = (0..state.analog.len())
            .map(|i| format!("Analog-{}/{}", i + 1, i + 2))
            .chain((0..state.digital.len()).map(|i| format!("Digital-{}/{}", i + 1, i + 2)))
            .collect();
        Self::state_add_elem_bool(
            card_cntr,
            &mut notified_elem_id_list,
            MIXER_PHYS_SRC_STEREO_LINK_NAME,
            labels.len(),
        )?;

        let labels: Vec<String> = (0..(state.analog.len() * 2))
            .map(|i| format!("Analog-{}", i + 1))
            .chain((0..(state.digital.len() * 2)).map(|i| format!("Digital-{}", i + 1)))
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

        Ok((notified_elem_id_list, Vec::new()))
    }

    fn read_mixer(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_STREAM_SRC_PAIR_GAIN_NAME => {
                let params = &self.segment().data;
                let state = Self::state(params);
                elem_value.set_int(&[state.stream.params[0].gain_to_mixer]);
                Ok(true)
            }
            MIXER_STREAM_SRC_PAIR_PAN_NAME => {
                let params = &self.segment().data;
                let state = Self::state(params);
                elem_value.set_int(&[state.stream.params[0].pan_to_mixer]);
                Ok(true)
            }
            MIXER_STREAM_SRC_PAIR_MUTE_NAME => {
                let params = &self.segment().data;
                let state = Self::state(params);
                elem_value.set_bool(&[state.mutes.stream]);
                Ok(true)
            }
            REVERB_STREAM_SRC_PAIR_GAIN_NAME => {
                let params = &self.segment().data;
                let state = Self::state(params);
                elem_value.set_int(&[state.stream.params[0].gain_to_send]);
                Ok(true)
            }
            MIXER_PHYS_SRC_STEREO_LINK_NAME => {
                let params = &self.segment().data;
                let state = Self::state(params);
                let vals: Vec<bool> = phys_src_pair_iter(state)
                    .map(|pair| pair.stereo_link)
                    .collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            MIXER_PHYS_SRC_GAIN_NAME => {
                let params = &self.segment().data;
                let state = Self::state(params);
                let vals: Vec<i32> = phys_src_params_iter(state)
                    .map(|params| params.gain_to_mixer)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MIXER_PHYS_SRC_PAN_NAME => {
                let params = &self.segment().data;
                let state = Self::state(params);
                let vals: Vec<i32> = phys_src_params_iter(state)
                    .map(|params| params.pan_to_mixer)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MIXER_PHYS_SRC_MUTE_NAME => {
                let params = &self.segment().data;
                let state = Self::state(params);
                let mut vals = state.mutes.analog.clone();
                vals.extend_from_slice(&state.mutes.digital);
                elem_value.set_bool(&vals);
                Ok(true)
            }
            REVERB_PHYS_SRC_GAIN_NAME => {
                let params = &self.segment().data;
                let state = Self::state(params);
                let vals: Vec<i32> = phys_src_params_iter(state)
                    .map(|params| params.gain_to_send)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MIXER_OUT_DIM_NAME => {
                let params = &self.segment().data;
                let state = Self::state(params);
                elem_value.set_bool(&[state.output_dim_enable]);
                Ok(true)
            }
            MIXER_OUT_VOL_NAME => {
                let params = &self.segment().data;
                let state = Self::state(params);
                elem_value.set_int(&[state.output_volume]);
                Ok(true)
            }
            MIXER_OUT_DIM_VOL_NAME => {
                let params = &self.segment().data;
                let state = Self::state(params);
                elem_value.set_int(&[state.output_dim_volume]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_mixer(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_STREAM_SRC_PAIR_GAIN_NAME => {
                let mut params = self.segment().data.clone();
                let state = Self::state_mut(&mut params);
                state.stream.params[0].gain_to_mixer = elem_value.int()[0];
                U::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            MIXER_STREAM_SRC_PAIR_PAN_NAME => {
                let mut params = self.segment().data.clone();
                let state = Self::state_mut(&mut params);
                state.stream.params[0].pan_to_mixer = elem_value.int()[0];
                U::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            MIXER_STREAM_SRC_PAIR_MUTE_NAME => {
                let mut params = self.segment().data.clone();
                let state = Self::state_mut(&mut params);
                state.mutes.stream = elem_value.boolean()[0];
                U::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            REVERB_STREAM_SRC_PAIR_GAIN_NAME => {
                let mut params = self.segment().data.clone();
                let state = Self::state_mut(&mut params);
                state.stream.params[0].gain_to_send = elem_value.int()[0];
                U::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            MIXER_PHYS_SRC_STEREO_LINK_NAME => {
                let mut params = self.segment().data.clone();
                let state = Self::state_mut(&mut params);
                phys_src_pair_iter_mut(state)
                    .zip(elem_value.boolean())
                    .for_each(|(pair, val)| pair.stereo_link = val);
                U::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            MIXER_PHYS_SRC_GAIN_NAME => {
                let mut params = self.segment().data.clone();
                let state = Self::state_mut(&mut params);
                phys_src_params_iter_mut(state)
                    .zip(elem_value.int())
                    .for_each(|(p, &val)| p.gain_to_mixer = val);
                U::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            MIXER_PHYS_SRC_PAN_NAME => {
                let mut params = self.segment().data.clone();
                let state = Self::state_mut(&mut params);
                phys_src_params_iter_mut(state)
                    .zip(elem_value.int())
                    .for_each(|(p, &val)| p.pan_to_mixer = val);
                U::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            MIXER_PHYS_SRC_MUTE_NAME => {
                let mut params = self.segment().data.clone();
                let state = Self::state_mut(&mut params);
                phys_src_pair_iter_mut(state)
                    .zip(elem_value.boolean())
                    .for_each(|(pair, val)| pair.stereo_link = val);
                U::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            REVERB_PHYS_SRC_GAIN_NAME => {
                let mut params = self.segment().data.clone();
                let state = Self::state_mut(&mut params);
                phys_src_params_iter_mut(state)
                    .zip(elem_value.int())
                    .for_each(|(p, &val)| p.gain_to_send = val);
                U::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            MIXER_OUT_DIM_NAME => {
                let mut params = self.segment().data.clone();
                let state = Self::state_mut(&mut params);
                state.output_dim_enable = elem_value.boolean()[0];
                U::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            MIXER_OUT_VOL_NAME => {
                let mut params = self.segment().data.clone();
                let state = Self::state_mut(&mut params);
                state.output_volume = elem_value.int()[0];
                U::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            MIXER_OUT_DIM_VOL_NAME => {
                let mut params = self.segment().data.clone();
                let state = Self::state_mut(&mut params);
                state.output_dim_volume = elem_value.int()[0];
                U::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
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
}

pub trait ShellMixerMeterCtlOperation<S, T>
where
    T: TcKonnektSegmentOperation<S>,
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

    fn meter(&self) -> &ShellMixerMeter;

    fn segment(&self) -> &TcKonnektSegment<S>;
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        T::cache_whole_segment(req, node, self.segment_mut(), timeout_ms)
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let mut measured_elem_id_list = Vec::new();

        let meter = self.meter();

        [
            (
                STREAM_IN_METER_NAME,
                meter.stream_inputs.len(),
                "stream-input",
            ),
            (
                ANALOG_IN_METER_NAME,
                meter.analog_inputs.len(),
                "analog-input",
            ),
            (
                DIGITAL_IN_METER_NAME,
                meter.digital_inputs.len(),
                "digital-input",
            ),
            (
                MIXER_OUT_METER_NAME,
                meter.main_outputs.len(),
                "mixer-output",
            ),
        ]
        .iter()
        .try_for_each(|&(name, count, label)| {
            let labels: Vec<String> = (0..count).map(|i| format!("{}-{}", label, i)).collect();

            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    Self::LEVEL_MIN,
                    Self::LEVEL_MAX,
                    Self::LEVEL_STEP,
                    labels.len(),
                    Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
                    false,
                )
                .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))
        })
        .map(|_| measured_elem_id_list)
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            STREAM_IN_METER_NAME => {
                let meter = self.meter();
                elem_value.set_int(&meter.stream_inputs);
                Ok(true)
            }
            ANALOG_IN_METER_NAME => {
                let meter = self.meter();
                elem_value.set_int(&meter.analog_inputs);
                Ok(true)
            }
            DIGITAL_IN_METER_NAME => {
                let meter = self.meter();
                elem_value.set_int(&meter.digital_inputs);
                Ok(true)
            }
            MIXER_OUT_METER_NAME => {
                let meter = self.meter();
                elem_value.set_int(&meter.main_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

const USE_AS_PLUGIN_NAME: &str = "use-reverb-as-plugin";
const GAIN_NAME: &str = "reverb-return-gain";
const MUTE_NAME: &str = "reverb-return-mute";

pub trait ShellReverbReturnCtlOperation<S, T>
where
    S: Clone,
    T: TcKonnektSegmentOperation<S> + TcKonnektMutableSegmentOperation<S>,
{
    fn segment(&self) -> &TcKonnektSegment<S>;
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;

    fn reverb_return(params: &S) -> &ShellReverbReturn;
    fn reverb_return_mut(params: &mut S) -> &mut ShellReverbReturn;

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
        match elem_id.name().as_str() {
            USE_AS_PLUGIN_NAME => {
                let params = &self.segment().data;
                let state = Self::reverb_return(&params);
                elem_value.set_bool(&[state.plugin_mode]);
                Ok(true)
            }
            GAIN_NAME => {
                let params = &self.segment().data;
                let state = Self::reverb_return(&params);
                elem_value.set_int(&[state.return_gain]);
                Ok(true)
            }
            MUTE_NAME => {
                let params = &self.segment().data;
                let state = Self::reverb_return(&params);
                elem_value.set_bool(&[state.return_mute]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_reverb_return(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            USE_AS_PLUGIN_NAME => {
                let mut params = self.segment().data.clone();
                let state = Self::reverb_return_mut(&mut params);
                state.plugin_mode = elem_value.boolean()[0];
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            GAIN_NAME => {
                let mut params = self.segment().data.clone();
                let state = Self::reverb_return_mut(&mut params);
                state.return_gain = elem_value.int()[0];
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            MUTE_NAME => {
                let mut params = self.segment().data.clone();
                let state = Self::reverb_return_mut(&mut params);
                state.return_mute = elem_value.boolean()[0];
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

fn standalone_src_to_str(src: &ShellStandaloneClockSource) -> &'static str {
    match src {
        ShellStandaloneClockSource::Optical => "Optical",
        ShellStandaloneClockSource::Coaxial => "Coaxial",
        ShellStandaloneClockSource::Internal => "Internal",
    }
}

const SRC_NAME: &str = "standalone-clock-source";

pub trait ShellStandaloneCtlOperation<S, T>: StandaloneCtlOperation<S, T>
where
    S: Clone + Debug,
    T: TcKonnektSegmentOperation<S>
        + TcKonnektMutableSegmentOperation<S>
        + ShellStandaloneClockSpecification,
{
    fn standalone_src(params: &S) -> &ShellStandaloneClockSource;
    fn standalone_src_mut(params: &mut S) -> &mut ShellStandaloneClockSource;

    fn load_standalone(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = T::STANDALONE_CLOCK_SOURCES
            .iter()
            .map(|r| standalone_src_to_str(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        self.load_standalone_rate(card_cntr)?;

        Ok(())
    }

    fn read_standalone(&mut self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            SRC_NAME => {
                let params = &self.segment().data;
                let src = Self::standalone_src(&params);
                let pos = T::STANDALONE_CLOCK_SOURCES
                    .iter()
                    .position(|s| src.eq(s))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => self.read_standalone_rate(elem_id, elem_value),
        }
    }

    fn write_standalone(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            SRC_NAME => {
                let mut params = self.segment().data.clone();
                let src = Self::standalone_src_mut(&mut params);
                let pos = elem_value.enumerated()[0] as usize;
                T::STANDALONE_CLOCK_SOURCES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of clock source: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&s| *src = s)?;
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => self.write_standalone_rate(req, node, elem_id, elem_value, timeout_ms),
        }
    }
}

fn mixer_stream_src_pair_to_str(src: &ShellMixerStreamSourcePair) -> &'static str {
    match src {
        ShellMixerStreamSourcePair::Stream0_1 => "Stream-1/2",
        ShellMixerStreamSourcePair::Stream2_3 => "Stream-3/4",
        ShellMixerStreamSourcePair::Stream4_5 => "Stream-5/6",
        ShellMixerStreamSourcePair::Stream6_7 => "Stream-7/8",
        ShellMixerStreamSourcePair::Stream8_9 => "Stream-9/10",
        ShellMixerStreamSourcePair::Stream10_11 => "Stream-11/12",
        ShellMixerStreamSourcePair::Stream12_13 => "Stream-13/14",
    }
}

const MIXER_STREAM_SRC_NAME: &str = "mixer-stream-soruce";

pub trait ShellMixerStreamSrcCtlOperation<S, T>
where
    S: Clone,
    T: TcKonnektSegmentOperation<S>
        + TcKonnektMutableSegmentOperation<S>
        + ShellMixerStreamSourcePairSpecification,
{
    fn segment(&self) -> &TcKonnektSegment<S>;
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;

    fn mixer_stream_src(params: &S) -> &ShellMixerStreamSourcePair;
    fn mixer_stream_src_mut(params: &mut S) -> &mut ShellMixerStreamSourcePair;

    fn load_mixer_stream_src(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = T::MIXER_STREAM_SOURCE_PAIRS
            .iter()
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
        match elem_id.name().as_str() {
            MIXER_STREAM_SRC_NAME => {
                let params = &self.segment().data;
                let pair = Self::mixer_stream_src(&params);
                let pos = T::MIXER_STREAM_SOURCE_PAIRS
                    .iter()
                    .position(|p| pair.eq(p))
                    .ok_or_else(|| {
                        let msg =
                            format!("Unexpected value for mixer stream source pair: {:?}", pair);
                        Error::new(FileError::Io, &msg)
                    })?;
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_mixer_stream_src(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_STREAM_SRC_NAME => {
                let mut params = self.segment().data.clone();
                let pair = Self::mixer_stream_src_mut(&mut params);
                let pos = elem_value.enumerated()[0] as usize;
                T::MIXER_STREAM_SOURCE_PAIRS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!(
                            "Invalid index {} of knob0, should be less than {}",
                            pos,
                            T::MIXER_STREAM_SOURCE_PAIRS.len()
                        );
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&p| *pair = p)?;
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
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
    S: Clone,
    T: TcKonnektSegmentOperation<S> + TcKonnektMutableSegmentOperation<S>,
{
    fn segment(&self) -> &TcKonnektSegment<S>;
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;
    fn coax_out_src(params: &S) -> &ShellCoaxOutPairSrc;
    fn coax_out_src_mut(params: &mut S) -> &mut ShellCoaxOutPairSrc;

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
        match elem_id.name().as_str() {
            COAX_OUT_SRC_NAME => {
                let params = &self.segment().data;
                let src = Self::coax_out_src(params);
                let pos = PHYS_OUT_SRCS.iter().position(|s| src.0.eq(s)).unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_coax_out_src(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            COAX_OUT_SRC_NAME => {
                let mut params = self.segment().data.clone();
                let src = Self::coax_out_src_mut(&mut params);
                let pos = elem_value.enumerated()[0] as usize;
                PHYS_OUT_SRCS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of clock rate: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&s| src.0 = s)?;
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
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
    S: Clone,
    T: TcKonnektSegmentOperation<S> + TcKonnektMutableSegmentOperation<S>,
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

    fn segment(&self) -> &TcKonnektSegment<S>;
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;

    fn opt_iface_config(params: &S) -> &ShellOptIfaceConfig;
    fn opt_iface_config_mut(params: &mut S) -> &mut ShellOptIfaceConfig;

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
        match elem_id.name().as_str() {
            OPT_IN_FMT_NAME => {
                let params = &self.segment().data;
                let config = Self::opt_iface_config(&params);
                let pos = Self::IN_FMTS
                    .iter()
                    .position(|f| config.input_format.eq(f))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            OPT_OUT_FMT_NAME => {
                let params = &self.segment().data;
                let config = Self::opt_iface_config(&params);
                let pos = Self::OUT_FMTS
                    .iter()
                    .position(|f| config.output_format.eq(f))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            OPT_OUT_SRC_NAME => {
                let params = &self.segment().data;
                let config = Self::opt_iface_config(&params);
                let pos = PHYS_OUT_SRCS
                    .iter()
                    .position(|s| config.output_source.0.eq(s))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_opt_iface_config(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OPT_IN_FMT_NAME => {
                let mut params = self.segment().data.clone();
                let config = Self::opt_iface_config_mut(&mut params);
                let pos = elem_value.enumerated()[0] as usize;
                Self::IN_FMTS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of optical input format: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&f| config.input_format = f)?;
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            OPT_OUT_FMT_NAME => {
                let mut params = self.segment().data.clone();
                let config = Self::opt_iface_config_mut(&mut params);
                let pos = elem_value.enumerated()[0] as usize;
                Self::OUT_FMTS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of optical output format: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&f| config.output_format = f)?;
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            OPT_OUT_SRC_NAME => {
                let mut params = self.segment().data.clone();
                let config = Self::opt_iface_config_mut(&mut params);
                let pos = elem_value.enumerated()[0] as usize;
                PHYS_OUT_SRCS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of optical output source: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&s| config.output_source.0 = s)?;
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const TARGET_NAME: &str = "knob-target";

fn knob0_target_to_str(target: &ShellKnob0Target) -> &str {
    match target {
        ShellKnob0Target::Analog0 => "Analog-1",
        ShellKnob0Target::Analog1 => "Analog-2",
        ShellKnob0Target::Analog2_3 => "Analog-3/4",
        ShellKnob0Target::Spdif0_1 => "S/PDIF-1/2",
        ShellKnob0Target::ChannelStrip0 => "Channel-strip-1",
        ShellKnob0Target::ChannelStrip1 => "Channel-strip-2",
        ShellKnob0Target::Reverb => "Reverb",
        ShellKnob0Target::Mixer => "Mixer",
        ShellKnob0Target::Configurable => "Configurable",
    }
}

pub trait ShellKnob0CtlOperation<S, T>
where
    S: Clone,
    T: TcKonnektSegmentOperation<S>
        + TcKonnektMutableSegmentOperation<S>
        + ShellKnob0TargetSpecification,
{
    fn segment(&self) -> &TcKonnektSegment<S>;
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;

    fn knob0_target(params: &S) -> &ShellKnob0Target;
    fn knob0_target_mut(params: &mut S) -> &mut ShellKnob0Target;

    fn load_knob0_target(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let labels: Vec<&str> = T::KNOB0_TARGETS
            .iter()
            .map(|t| knob0_target_to_str(t))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, TARGET_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
    }

    fn read_knob0_target(
        &mut self,
        elem_id: &ElemId,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            TARGET_NAME => {
                let params = &self.segment().data;
                let target = Self::knob0_target(&params);
                let pos = T::KNOB0_TARGETS
                    .iter()
                    .position(|t| target.eq(t))
                    .ok_or_else(|| {
                        let msg = format!("Unexpected value for knob0: {:?}", target);
                        Error::new(FileError::Io, &msg)
                    })?;
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_knob0_target(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            TARGET_NAME => {
                let mut params = self.segment().data.clone();
                let target = Self::knob0_target_mut(&mut params);
                let pos = elem_value.enumerated()[0] as usize;
                T::KNOB0_TARGETS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!(
                            "Invalid index {} of knob0, should be less than {}",
                            pos,
                            T::KNOB0_TARGETS.len()
                        );
                        Error::new(FileError::Io, &msg)
                    })
                    .map(|&t| *target = t)?;
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const KNOB1_NAME: &str = "configurable-knob-target";

fn knob1_target_to_str(target: &ShellKnob1Target) -> &'static str {
    match target {
        ShellKnob1Target::Digital0_1 => "Digital-1/2",
        ShellKnob1Target::Digital2_3 => "Digital-3/4",
        ShellKnob1Target::Digital4_5 => "Digital-5/6",
        ShellKnob1Target::Digital6_7 => "Digital-7/8",
        ShellKnob1Target::Stream => "Stream",
        ShellKnob1Target::Reverb => "Reverb",
        ShellKnob1Target::Mixer => "Mixer",
        ShellKnob1Target::TunerPitchTone => "Tune-pitch/tone",
        ShellKnob1Target::MidiSend => "Midi-send",
    }
}

pub trait ShellKnob1CtlOperation<S, T>
where
    S: Clone,
    T: TcKonnektSegmentOperation<S>
        + TcKonnektMutableSegmentOperation<S>
        + TcKonnektNotifiedSegmentOperation<S>
        + ShellKnob1TargetSpecification,
{
    fn segment(&self) -> &TcKonnektSegment<S>;
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;

    fn knob1_target(params: &S) -> &ShellKnob1Target;
    fn knob1_target_mut(params: &mut S) -> &mut ShellKnob1Target;

    fn load_knob1_target(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let labels: Vec<&str> = T::KNOB1_TARGETS
            .iter()
            .map(|target| knob1_target_to_str(target))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, KNOB1_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
    }

    fn read_knob1_target(
        &mut self,
        elem_id: &ElemId,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            KNOB1_NAME => {
                let params = &self.segment().data;
                let target = Self::knob1_target(&params);
                let pos = T::KNOB1_TARGETS
                    .iter()
                    .position(|t| target.eq(t))
                    .ok_or_else(|| {
                        let msg = format!("Unexpected value for knob1: {:?}", target);
                        Error::new(FileError::Io, &msg)
                    })?;
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_knob1_target(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            KNOB1_NAME => {
                let mut params = self.segment().data.clone();
                let target = Self::knob1_target_mut(&mut params);
                let pos = elem_value.enumerated()[0] as usize;
                T::KNOB1_TARGETS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!(
                            "Invalid index {} of knob1, should be less than {}",
                            pos,
                            T::KNOB1_TARGETS.len()
                        );
                        Error::new(FileError::Io, &msg)
                    })
                    .map(|&t| *target = t)?;
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
