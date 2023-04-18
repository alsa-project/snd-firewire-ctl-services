// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, protocols::tcelectronic::shell::*};

// TODO: For Jack detection in ALSA applications.
const ANALOG_JACK_STATE_NAME: &str = "analog-jack-state";

const ANALOG_JACK_STATE_LABELS: &[ShellAnalogJackState] = &[
    ShellAnalogJackState::FrontSelected,
    ShellAnalogJackState::FrontInserted,
    ShellAnalogJackState::FrontInsertedAttenuated,
    ShellAnalogJackState::RearSelected,
    ShellAnalogJackState::RearInserted,
];

fn analog_jack_state_to_str(state: &ShellAnalogJackState) -> &'static str {
    match state {
        ShellAnalogJackState::FrontSelected => "Front-selected",
        ShellAnalogJackState::FrontInserted => "Front-inserted",
        ShellAnalogJackState::FrontInsertedAttenuated => "Front-inserted-attenuated",
        ShellAnalogJackState::RearSelected => "Rear-selected",
        ShellAnalogJackState::RearInserted => "Rear-inserted",
    }
}

pub fn load_hw_state<T, U>(card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error>
where
    T: TcKonnektSegmentOperation<U> + TcKonnektMutableSegmentOperation<U>,
    U: Debug
        + Clone
        + AsRef<ShellHwState>
        + AsMut<ShellHwState>
        + AsRef<FireWireLedState>
        + AsMut<FireWireLedState>,
{
    let mut elem_id_list = Vec::new();

    let labels: Vec<&str> = ANALOG_JACK_STATE_LABELS
        .iter()
        .map(|s| analog_jack_state_to_str(s))
        .collect();
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
        .map(|mut list| elem_id_list.append(&mut list))?;

    load_firewire_led::<T, U>(card_cntr).map(|mut list| elem_id_list.append(&mut list))?;

    Ok(elem_id_list)
}

pub fn read_hw_state<T, U>(
    segment: &TcKonnektSegment<U>,
    elem_id: &ElemId,
    elem_value: &mut ElemValue,
) -> Result<bool, Error>
where
    T: TcKonnektSegmentOperation<U> + TcKonnektMutableSegmentOperation<U>,
    U: Debug
        + Clone
        + AsRef<ShellHwState>
        + AsMut<ShellHwState>
        + AsRef<FireWireLedState>
        + AsMut<FireWireLedState>,
{
    match elem_id.name().as_str() {
        ANALOG_JACK_STATE_NAME => {
            let params: &ShellHwState = segment.data.as_ref();
            let vals: Vec<u32> = params
                .analog_jack_states
                .iter()
                .map(|state| {
                    let pos = ANALOG_JACK_STATE_LABELS
                        .iter()
                        .position(|s| state.eq(s))
                        .unwrap();
                    pos as u32
                })
                .collect();
            elem_value.set_enum(&vals);
            Ok(true)
        }
        _ => read_firewire_led::<T, U>(segment, elem_id, elem_value),
    }
}

pub fn write_hw_state<T, U>(
    segment: &mut TcKonnektSegment<U>,
    req: &mut FwReq,
    node: &mut FwNode,
    elem_id: &ElemId,
    elem_value: &ElemValue,
    timeout_ms: u32,
) -> Result<bool, Error>
where
    T: TcKonnektSegmentOperation<U> + TcKonnektMutableSegmentOperation<U>,
    U: Debug
        + Clone
        + AsRef<ShellHwState>
        + AsMut<ShellHwState>
        + AsRef<FireWireLedState>
        + AsMut<FireWireLedState>,
{
    write_firewire_led::<T, U>(segment, req, node, elem_id, elem_value, timeout_ms)
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

pub fn load_mixer<T, U>(
    _: &TcKonnektSegment<U>,
    card_cntr: &mut CardCntr,
) -> Result<Vec<ElemId>, Error>
where
    T: ShellMixerStateSpecification
        + TcKonnektSegmentOperation<U>
        + TcKonnektMutableSegmentOperation<U>,
    U: Debug + Clone + AsRef<ShellMixerState> + AsMut<ShellMixerState>,
{
    let mut elem_id_list = Vec::new();

    // For stream source.
    let elem_id = ElemId::new_by_name(
        ElemIfaceType::Mixer,
        0,
        0,
        MIXER_STREAM_SRC_PAIR_GAIN_NAME,
        0,
    );
    card_cntr
        .add_int_elems(
            &elem_id,
            1,
            LEVEL_MIN,
            LEVEL_MAX,
            LEVEL_STEP,
            1,
            Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
            true,
        )
        .map(|mut list| elem_id_list.append(&mut list))?;

    let elem_id = ElemId::new_by_name(
        ElemIfaceType::Mixer,
        0,
        0,
        MIXER_STREAM_SRC_PAIR_PAN_NAME,
        0,
    );
    card_cntr
        .add_int_elems(&elem_id, 1, PAN_MIN, PAN_MAX, PAN_STEP, 1, None, true)
        .map(|mut list| elem_id_list.append(&mut list))?;

    let elem_id = ElemId::new_by_name(
        ElemIfaceType::Mixer,
        0,
        0,
        MIXER_STREAM_SRC_PAIR_MUTE_NAME,
        0,
    );
    card_cntr
        .add_bool_elems(&elem_id, 1, 1, true)
        .map(|mut list| elem_id_list.append(&mut list))?;

    let elem_id = ElemId::new_by_name(
        ElemIfaceType::Mixer,
        0,
        0,
        REVERB_STREAM_SRC_PAIR_GAIN_NAME,
        0,
    );
    card_cntr
        .add_int_elems(
            &elem_id,
            1,
            LEVEL_MIN,
            LEVEL_MAX,
            LEVEL_STEP,
            1,
            Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
            true,
        )
        .map(|mut list| elem_id_list.append(&mut list))?;

    // For physical sources of mixer.
    let labels: Vec<String> = (0..T::analog_input_pair_count())
        .map(|i| format!("Analog-{}/{}", i + 1, i + 2))
        .chain((0..T::digital_input_pair_count()).map(|i| format!("Digital-{}/{}", i + 1, i + 2)))
        .collect();
    let elem_id = ElemId::new_by_name(
        ElemIfaceType::Mixer,
        0,
        0,
        MIXER_PHYS_SRC_STEREO_LINK_NAME,
        0,
    );
    card_cntr
        .add_bool_elems(&elem_id, 1, labels.len(), true)
        .map(|mut list| elem_id_list.append(&mut list))?;

    let labels: Vec<String> = (0..(T::analog_input_pair_count() * 2))
        .map(|i| format!("Analog-{}", i + 1))
        .chain((0..(T::digital_input_pair_count() * 2)).map(|i| format!("Digital-{}", i + 1)))
        .collect();
    let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_PHYS_SRC_GAIN_NAME, 0);
    card_cntr
        .add_int_elems(
            &elem_id,
            1,
            LEVEL_MIN,
            LEVEL_MAX,
            LEVEL_STEP,
            1,
            Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
            true,
        )
        .map(|mut list| elem_id_list.append(&mut list))?;

    let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_PHYS_SRC_PAN_NAME, 0);
    card_cntr
        .add_int_elems(
            &elem_id,
            1,
            PAN_MIN,
            PAN_MAX,
            PAN_STEP,
            labels.len(),
            None,
            true,
        )
        .map(|mut list| elem_id_list.append(&mut list))?;

    let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_PHYS_SRC_MUTE_NAME, 0);
    card_cntr
        .add_bool_elems(&elem_id, 1, labels.len(), true)
        .map(|mut list| elem_id_list.append(&mut list))?;

    let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_PHYS_SRC_GAIN_NAME, 0);
    card_cntr
        .add_int_elems(
            &elem_id,
            1,
            LEVEL_MIN,
            LEVEL_MAX,
            LEVEL_STEP,
            labels.len(),
            Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
            true,
        )
        .map(|mut list| elem_id_list.append(&mut list))?;

    // For output of mixer.
    let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUT_DIM_NAME, 0);
    card_cntr
        .add_bool_elems(&elem_id, 1, 1, true)
        .map(|mut list| elem_id_list.append(&mut list))?;

    let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUT_VOL_NAME, 0);
    card_cntr
        .add_int_elems(
            &elem_id,
            1,
            LEVEL_MIN,
            LEVEL_MAX,
            LEVEL_STEP,
            1,
            Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
            true,
        )
        .map(|mut list| elem_id_list.append(&mut list))?;

    let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUT_DIM_VOL_NAME, 0);
    card_cntr
        .add_int_elems(
            &elem_id,
            1,
            LEVEL_MIN,
            LEVEL_MAX,
            LEVEL_STEP,
            1,
            Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
            true,
        )
        .map(|mut list| elem_id_list.append(&mut list))?;

    Ok(elem_id_list)
}

pub fn read_mixer<T, U>(
    segment: &TcKonnektSegment<U>,
    elem_id: &ElemId,
    elem_value: &mut ElemValue,
) -> Result<bool, Error>
where
    T: ShellMixerStateSpecification
        + TcKonnektSegmentOperation<U>
        + TcKonnektMutableSegmentOperation<U>,
    U: Debug + Clone + AsRef<ShellMixerState> + AsMut<ShellMixerState>,
{
    match elem_id.name().as_str() {
        MIXER_STREAM_SRC_PAIR_GAIN_NAME => {
            let params = segment.data.as_ref();
            elem_value.set_int(&[params.stream.params[0].gain_to_mixer]);
            Ok(true)
        }
        MIXER_STREAM_SRC_PAIR_PAN_NAME => {
            let params = segment.data.as_ref();
            elem_value.set_int(&[params.stream.params[0].pan_to_mixer]);
            Ok(true)
        }
        MIXER_STREAM_SRC_PAIR_MUTE_NAME => {
            let params = segment.data.as_ref();
            elem_value.set_bool(&[params.mutes.stream]);
            Ok(true)
        }
        REVERB_STREAM_SRC_PAIR_GAIN_NAME => {
            let params = segment.data.as_ref();
            elem_value.set_int(&[params.stream.params[0].gain_to_send]);
            Ok(true)
        }
        MIXER_PHYS_SRC_STEREO_LINK_NAME => {
            let params = segment.data.as_ref();
            let vals: Vec<bool> = phys_src_pair_iter(params)
                .map(|pair| pair.stereo_link)
                .collect();
            elem_value.set_bool(&vals);
            Ok(true)
        }
        MIXER_PHYS_SRC_GAIN_NAME => {
            let params = segment.data.as_ref();
            let vals: Vec<i32> = phys_src_params_iter(params)
                .map(|params| params.gain_to_mixer)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        }
        MIXER_PHYS_SRC_PAN_NAME => {
            let params = segment.data.as_ref();
            let vals: Vec<i32> = phys_src_params_iter(params)
                .map(|params| params.pan_to_mixer)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        }
        MIXER_PHYS_SRC_MUTE_NAME => {
            let params = segment.data.as_ref();
            let mut vals = params.mutes.analog.clone();
            vals.extend_from_slice(&params.mutes.digital);
            elem_value.set_bool(&vals);
            Ok(true)
        }
        REVERB_PHYS_SRC_GAIN_NAME => {
            let params = segment.data.as_ref();
            let vals: Vec<i32> = phys_src_params_iter(params)
                .map(|params| params.gain_to_send)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        }
        MIXER_OUT_DIM_NAME => {
            let params = segment.data.as_ref();
            elem_value.set_bool(&[params.output_dim_enable]);
            Ok(true)
        }
        MIXER_OUT_VOL_NAME => {
            let params = segment.data.as_ref();
            elem_value.set_int(&[params.output_volume]);
            Ok(true)
        }
        MIXER_OUT_DIM_VOL_NAME => {
            let params = segment.data.as_ref();
            elem_value.set_int(&[params.output_dim_volume]);
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub fn write_mixer<T, U>(
    segment: &mut TcKonnektSegment<U>,
    req: &FwReq,
    node: &FwNode,
    elem_id: &ElemId,
    elem_value: &ElemValue,
    timeout_ms: u32,
) -> Result<bool, Error>
where
    T: ShellMixerStateSpecification
        + TcKonnektSegmentOperation<U>
        + TcKonnektMutableSegmentOperation<U>,
    U: Debug + Clone + AsRef<ShellMixerState> + AsMut<ShellMixerState>,
{
    match elem_id.name().as_str() {
        MIXER_STREAM_SRC_PAIR_GAIN_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            params.stream.params[0].gain_to_mixer = elem_value.int()[0];
            let res = T::update_partial_segment(req, node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        MIXER_STREAM_SRC_PAIR_PAN_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            params.stream.params[0].pan_to_mixer = elem_value.int()[0];
            let res = T::update_partial_segment(req, node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        MIXER_STREAM_SRC_PAIR_MUTE_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            params.mutes.stream = elem_value.boolean()[0];
            let res = T::update_partial_segment(req, node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        REVERB_STREAM_SRC_PAIR_GAIN_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            params.stream.params[0].gain_to_send = elem_value.int()[0];
            let res = T::update_partial_segment(req, node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        MIXER_PHYS_SRC_STEREO_LINK_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            phys_src_pair_iter_mut(params)
                .zip(elem_value.boolean())
                .for_each(|(pair, val)| pair.stereo_link = val);
            let res = T::update_partial_segment(req, node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        MIXER_PHYS_SRC_GAIN_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            phys_src_params_iter_mut(params)
                .zip(elem_value.int())
                .for_each(|(p, &val)| p.gain_to_mixer = val);
            let res = T::update_partial_segment(req, node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        MIXER_PHYS_SRC_PAN_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            phys_src_params_iter_mut(params)
                .zip(elem_value.int())
                .for_each(|(p, &val)| p.pan_to_mixer = val);
            let res = T::update_partial_segment(req, node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        MIXER_PHYS_SRC_MUTE_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            phys_src_pair_iter_mut(params)
                .zip(elem_value.boolean())
                .for_each(|(pair, val)| pair.stereo_link = val);
            let res = T::update_partial_segment(req, node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        REVERB_PHYS_SRC_GAIN_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            phys_src_params_iter_mut(params)
                .zip(elem_value.int())
                .for_each(|(p, &val)| p.gain_to_send = val);
            let res = T::update_partial_segment(req, node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        MIXER_OUT_DIM_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            params.output_dim_enable = elem_value.boolean()[0];
            let res = T::update_partial_segment(req, node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        MIXER_OUT_VOL_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            params.output_volume = elem_value.int()[0];
            let res = T::update_partial_segment(req, node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        MIXER_OUT_DIM_VOL_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            params.output_dim_volume = elem_value.int()[0];
            let res = T::update_partial_segment(req, node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        _ => Ok(false),
    }
}

#[derive(Default, Debug)]
pub struct MixerMeterCtl<T, U>
where
    T: ShellMixerMeterSpecification + TcKonnektSegmentOperation<U>,
    TcKonnektSegment<U>: Default,
    U: Debug + AsRef<ShellMixerMeter> + AsMut<ShellMixerMeter>,
{
    pub elem_id_list: Vec<ElemId>,
    segment: TcKonnektSegment<U>,
    _phantom: PhantomData<T>,
}

const METER_MIN: i32 = -1000;
const METER_MAX: i32 = 0;
const METER_STEP: i32 = 1;
const METER_TLV: DbInterval = DbInterval {
    min: -9400,
    max: 0,
    linear: false,
    mute_avail: false,
};

impl<T, U> MixerMeterCtl<T, U>
where
    T: ShellMixerMeterSpecification + TcKonnektSegmentOperation<U>,
    TcKonnektSegment<U>: Default,
    U: Debug + AsRef<ShellMixerMeter> + AsMut<ShellMixerMeter>,
{
    pub fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache_whole_segment(req, node, &mut self.segment, timeout_ms);
        debug!(params = ?self.segment.data, ?res);
        res
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        [
            (STREAM_IN_METER_NAME, T::STREAM_INPUT_COUNT, "stream-input"),
            (ANALOG_IN_METER_NAME, T::ANALOG_INPUT_COUNT, "analog-input"),
            (
                DIGITAL_IN_METER_NAME,
                T::DIGITAL_INPUT_COUNT,
                "digital-input",
            ),
            (MIXER_OUT_METER_NAME, T::MAIN_OUTPUT_COUNT, "mixer-output"),
        ]
        .iter()
        .try_for_each(|&(name, count, label)| {
            let labels: Vec<String> = (0..count).map(|i| format!("{}-{}", label, i)).collect();

            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    METER_MIN,
                    METER_MAX,
                    METER_STEP,
                    labels.len(),
                    Some(&Into::<Vec<u32>>::into(METER_TLV)),
                    false,
                )
                .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
        })
    }

    pub fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            STREAM_IN_METER_NAME => {
                let params = self.segment.data.as_ref();
                elem_value.set_int(&params.stream_inputs);
                Ok(true)
            }
            ANALOG_IN_METER_NAME => {
                let params = self.segment.data.as_ref();
                elem_value.set_int(&params.analog_inputs);
                Ok(true)
            }
            DIGITAL_IN_METER_NAME => {
                let params = self.segment.data.as_ref();
                elem_value.set_int(&params.digital_inputs);
                Ok(true)
            }
            MIXER_OUT_METER_NAME => {
                let params = self.segment.data.as_ref();
                elem_value.set_int(&params.main_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

const USE_AS_PLUGIN_NAME: &str = "use-reverb-as-plugin";
const GAIN_NAME: &str = "reverb-return-gain";
const MUTE_NAME: &str = "reverb-return-mute";

const GAIN_MIN: i32 = -1000;
const GAIN_MAX: i32 = 0;
const GAIN_STEP: i32 = 1;
const GAIN_TLV: DbInterval = DbInterval {
    min: -7200,
    max: 0,
    linear: false,
    mute_avail: false,
};

pub fn load_reverb_return<T, U>(card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error>
where
    T: TcKonnektSegmentOperation<U> + TcKonnektMutableSegmentOperation<U>,
    U: Debug + Clone + AsRef<ShellReverbReturn> + AsMut<ShellReverbReturn>,
{
    let mut elem_id_list = Vec::new();

    let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, USE_AS_PLUGIN_NAME, 0);
    card_cntr
        .add_bool_elems(&elem_id, 1, 1, true)
        .map(|mut list| elem_id_list.append(&mut list))?;

    let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, GAIN_NAME, 0);
    card_cntr
        .add_int_elems(
            &elem_id,
            1,
            GAIN_MIN,
            GAIN_MAX,
            GAIN_STEP,
            1,
            Some(&Vec::<u32>::from(GAIN_TLV)),
            true,
        )
        .map(|mut list| elem_id_list.append(&mut list))?;

    let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MUTE_NAME, 0);
    card_cntr
        .add_bool_elems(&elem_id, 1, 1, true)
        .map(|mut list| elem_id_list.append(&mut list))?;

    Ok(elem_id_list)
}

pub fn read_reverb_return<T, U>(
    segment: &TcKonnektSegment<U>,
    elem_id: &ElemId,
    elem_value: &mut ElemValue,
) -> Result<bool, Error>
where
    T: TcKonnektSegmentOperation<U> + TcKonnektMutableSegmentOperation<U>,
    U: Debug + Clone + AsRef<ShellReverbReturn> + AsMut<ShellReverbReturn>,
{
    match elem_id.name().as_str() {
        USE_AS_PLUGIN_NAME => {
            let params = segment.data.as_ref();
            elem_value.set_bool(&[params.plugin_mode]);
            Ok(true)
        }
        GAIN_NAME => {
            let params = segment.data.as_ref();
            elem_value.set_int(&[params.return_gain]);
            Ok(true)
        }
        MUTE_NAME => {
            let params = segment.data.as_ref();
            elem_value.set_bool(&[params.return_mute]);
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub fn write_reverb_return<T, U>(
    segment: &mut TcKonnektSegment<U>,
    req: &FwReq,
    node: &FwNode,
    elem_id: &ElemId,
    elem_value: &ElemValue,
    timeout_ms: u32,
) -> Result<bool, Error>
where
    T: TcKonnektSegmentOperation<U> + TcKonnektMutableSegmentOperation<U>,
    U: Debug + Clone + AsRef<ShellReverbReturn> + AsMut<ShellReverbReturn>,
{
    match elem_id.name().as_str() {
        USE_AS_PLUGIN_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            params.plugin_mode = elem_value.boolean()[0];
            let res = T::update_partial_segment(req, node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        GAIN_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            params.return_gain = elem_value.int()[0];
            let res = T::update_partial_segment(req, node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        MUTE_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            params.return_mute = elem_value.boolean()[0];
            let res = T::update_partial_segment(req, node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        _ => Ok(false),
    }
}

const SRC_NAME: &str = "standalone-clock-source";

fn standalone_src_to_str(src: &ShellStandaloneClockSource) -> &'static str {
    match src {
        ShellStandaloneClockSource::Optical => "Optical",
        ShellStandaloneClockSource::Coaxial => "Coaxial",
        ShellStandaloneClockSource::Internal => "Internal",
    }
}

pub fn load_standalone<T, U>(card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error>
where
    T: TcKonnektSegmentOperation<U>
        + TcKonnektMutableSegmentOperation<U>
        + ShellStandaloneClockSpecification,
    U: Debug
        + Clone
        + AsRef<ShellStandaloneClockSource>
        + AsMut<ShellStandaloneClockSource>
        + AsRef<TcKonnektStandaloneClockRate>
        + AsMut<TcKonnektStandaloneClockRate>,
{
    let mut elem_id_list = Vec::new();

    let labels: Vec<&str> = T::STANDALONE_CLOCK_SOURCES
        .iter()
        .map(|r| standalone_src_to_str(r))
        .collect();
    let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SRC_NAME, 0);
    card_cntr
        .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
        .map(|mut list| elem_id_list.append(&mut list))?;

    load_standalone_rate::<T, U>(card_cntr).map(|mut list| elem_id_list.append(&mut list))?;

    Ok(elem_id_list)
}

pub fn read_standalone<T, U>(
    segment: &TcKonnektSegment<U>,
    elem_id: &ElemId,
    elem_value: &ElemValue,
) -> Result<bool, Error>
where
    T: ShellStandaloneClockSpecification
        + TcKonnektSegmentOperation<U>
        + TcKonnektMutableSegmentOperation<U>,
    U: Debug
        + Clone
        + AsRef<ShellStandaloneClockSource>
        + AsMut<ShellStandaloneClockSource>
        + AsRef<TcKonnektStandaloneClockRate>
        + AsMut<TcKonnektStandaloneClockRate>,
{
    match elem_id.name().as_str() {
        SRC_NAME => {
            let params: &ShellStandaloneClockSource = segment.data.as_ref();
            let pos = T::STANDALONE_CLOCK_SOURCES
                .iter()
                .position(|s| params.eq(s))
                .unwrap();
            elem_value.set_enum(&[pos as u32]);
            Ok(true)
        }
        _ => read_standalone_rate::<T, U>(segment, elem_id, elem_value),
    }
}

pub fn write_standalone<T, U>(
    segment: &mut TcKonnektSegment<U>,
    req: &FwReq,
    node: &FwNode,
    elem_id: &ElemId,
    elem_value: &ElemValue,
    timeout_ms: u32,
) -> Result<bool, Error>
where
    T: ShellStandaloneClockSpecification
        + TcKonnektSegmentOperation<U>
        + TcKonnektMutableSegmentOperation<U>,
    U: Debug
        + Clone
        + AsRef<ShellStandaloneClockSource>
        + AsMut<ShellStandaloneClockSource>
        + AsRef<TcKonnektStandaloneClockRate>
        + AsMut<TcKonnektStandaloneClockRate>,
{
    match elem_id.name().as_str() {
        SRC_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            let pos = elem_value.enumerated()[0] as usize;
            T::STANDALONE_CLOCK_SOURCES
                .iter()
                .nth(pos)
                .ok_or_else(|| {
                    let msg = format!("Invalid value for index of clock source: {}", pos);
                    Error::new(FileError::Inval, &msg)
                })
                .map(|&src| *params = src)?;
            let res = T::update_partial_segment(req, node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        _ => write_standalone_rate::<T, U>(segment, req, node, elem_id, elem_value, timeout_ms),
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
    S: Clone + Debug,
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
                let res =
                    T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms);
                debug!(params = ?self.segment().data, ?res);
                res.map(|_| true)
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
    S: Clone + Debug,
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
                let res =
                    T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms);
                debug!(params = ?self.segment().data);
                res.map(|_| true)
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
    S: Clone + Debug,
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
                let res =
                    T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms);
                debug!(params = ?self.segment().data);
                res.map(|_| true)
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
                let res =
                    T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms);
                debug!(params = ?self.segment().data);
                res.map(|_| true)
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
                let res =
                    T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms);
                debug!(params = ?self.segment().data);
                res.map(|_| true)
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
    S: Clone + Debug,
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
                let res =
                    T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms);
                debug!(params = ?self.segment().data);
                res.map(|_| true)
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
    S: Clone + Debug,
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
                let res =
                    T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms);
                debug!(params = ?self.segment().data);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
