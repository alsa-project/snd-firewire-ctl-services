// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

const NORMAL_EVENT_CH_NAME: &str = "midi-normal-event-channel";
const NORMAL_EVENT_CC_NAME: &str = "midi-normal-event-cc";
const PUSHED_EVENT_CH_NAME: &str = "midi-pushed-event-channel";
const PUSHED_EVENT_CC_NAME: &str = "midi-pushed-event-cc";
const EVENT_TO_PORT_NAME: &str = "midi-event-to-port";
const EVENT_TO_STREAM_NAME: &str = "midi-event-to-stream";

pub fn load_midi_sender<T, U>(card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error>
where
    T: TcKonnektSegmentOperation<U> + TcKonnektMutableSegmentOperation<U>,
    U: Debug + Clone + AsRef<TcKonnektMidiSender> + AsMut<TcKonnektMidiSender>,
{
    let mut elem_id_list = Vec::new();

    let elem_id = ElemId::new_by_name(ElemIfaceType::Rawmidi, 0, 0, NORMAL_EVENT_CH_NAME, 0);
    card_cntr
        .add_bytes_elems(&elem_id, 1, 1, None, true)
        .map(|mut list| elem_id_list.append(&mut list))?;

    let elem_id = ElemId::new_by_name(ElemIfaceType::Rawmidi, 0, 0, NORMAL_EVENT_CC_NAME, 0);
    card_cntr
        .add_bytes_elems(&elem_id, 1, 1, None, true)
        .map(|mut list| elem_id_list.append(&mut list))?;

    let elem_id = ElemId::new_by_name(ElemIfaceType::Rawmidi, 0, 0, PUSHED_EVENT_CH_NAME, 0);
    card_cntr
        .add_bytes_elems(&elem_id, 1, 1, None, true)
        .map(|mut list| elem_id_list.append(&mut list))?;

    let elem_id = ElemId::new_by_name(ElemIfaceType::Rawmidi, 0, 0, PUSHED_EVENT_CC_NAME, 0);
    card_cntr
        .add_bytes_elems(&elem_id, 1, 1, None, true)
        .map(|mut list| elem_id_list.append(&mut list))?;

    let elem_id = ElemId::new_by_name(ElemIfaceType::Rawmidi, 0, 0, EVENT_TO_PORT_NAME, 0);
    card_cntr
        .add_bool_elems(&elem_id, 1, 1, true)
        .map(|mut list| elem_id_list.append(&mut list))?;

    let elem_id = ElemId::new_by_name(ElemIfaceType::Rawmidi, 0, 0, EVENT_TO_STREAM_NAME, 0);
    card_cntr
        .add_bool_elems(&elem_id, 1, 1, true)
        .map(|mut list| elem_id_list.append(&mut list))?;

    Ok(elem_id_list)
}

pub fn read_midi_sender<T, U>(
    segment: &TcKonnektSegment<U>,
    elem_id: &ElemId,
    elem_value: &mut ElemValue,
) -> Result<bool, Error>
where
    T: TcKonnektSegmentOperation<U> + TcKonnektMutableSegmentOperation<U>,
    U: Debug + Clone + AsRef<TcKonnektMidiSender> + AsMut<TcKonnektMidiSender>,
{
    match elem_id.name().as_str() {
        NORMAL_EVENT_CH_NAME => {
            let params = segment.data.as_ref();
            elem_value.set_bytes(&[params.normal.ch]);
            Ok(true)
        }
        NORMAL_EVENT_CC_NAME => {
            let params = segment.data.as_ref();
            elem_value.set_bytes(&[params.normal.cc]);
            Ok(true)
        }
        PUSHED_EVENT_CH_NAME => {
            let params = segment.data.as_ref();
            elem_value.set_bytes(&[params.pushed.ch]);
            Ok(true)
        }
        PUSHED_EVENT_CC_NAME => {
            let params = segment.data.as_ref();
            elem_value.set_bytes(&[params.pushed.cc]);
            Ok(true)
        }
        EVENT_TO_PORT_NAME => {
            let params = segment.data.as_ref();
            elem_value.set_bool(&[params.send_to_port]);
            Ok(true)
        }
        EVENT_TO_STREAM_NAME => {
            let params = segment.data.as_ref();
            elem_value.set_bool(&[params.send_to_stream]);
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub fn write_midi_sender<T, U>(
    segment: &mut TcKonnektSegment<U>,
    req: &FwReq,
    node: &FwNode,
    elem_id: &ElemId,
    elem_value: &ElemValue,
    timeout_ms: u32,
) -> Result<bool, Error>
where
    T: TcKonnektSegmentOperation<U> + TcKonnektMutableSegmentOperation<U>,
    U: Debug + Clone + AsRef<TcKonnektMidiSender> + AsMut<TcKonnektMidiSender>,
{
    match elem_id.name().as_str() {
        NORMAL_EVENT_CH_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            params.normal.ch = elem_value.bytes()[0];
            let res = T::update_partial_segment(req, &node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        NORMAL_EVENT_CC_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            params.normal.cc = elem_value.bytes()[0];
            let res = T::update_partial_segment(req, &node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        PUSHED_EVENT_CH_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            params.pushed.ch = elem_value.bytes()[0];
            let res = T::update_partial_segment(req, &node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        PUSHED_EVENT_CC_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            params.pushed.cc = elem_value.bytes()[0];
            let res = T::update_partial_segment(req, &node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        EVENT_TO_PORT_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            params.send_to_port = elem_value.boolean()[0];
            let res = T::update_partial_segment(req, &node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        EVENT_TO_STREAM_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            params.send_to_stream = elem_value.boolean()[0];
            let res = T::update_partial_segment(req, &node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        _ => Ok(false),
    }
}
