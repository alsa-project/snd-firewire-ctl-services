// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

const NORMAL_EVENT_CH_NAME: &str = "midi-normal-event-channel";
const NORMAL_EVENT_CC_NAME: &str = "midi-normal-event-cc";
const PUSHED_EVENT_CH_NAME: &str = "midi-pushed-event-channel";
const PUSHED_EVENT_CC_NAME: &str = "midi-pushed-event-cc";
const EVENT_TO_PORT_NAME: &str = "midi-event-to-port";
const EVENT_TO_STREAM_NAME: &str = "midi-event-to-stream";

pub trait MidiSendCtlOperation<S, T>
where
    S: Clone,
    T: TcKonnektSegmentOperation<S> + TcKonnektMutableSegmentOperation<S>,
{
    fn segment(&self) -> &TcKonnektSegment<S>;
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;

    fn midi_sender(params: &S) -> &TcKonnektMidiSender;
    fn midi_sender_mut(params: &mut S) -> &mut TcKonnektMidiSender;

    fn load_midi_sender(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Rawmidi, 0, 0, NORMAL_EVENT_CH_NAME, 0);
        let _ = card_cntr.add_bytes_elems(&elem_id, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Rawmidi, 0, 0, NORMAL_EVENT_CC_NAME, 0);
        let _ = card_cntr.add_bytes_elems(&elem_id, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Rawmidi, 0, 0, PUSHED_EVENT_CH_NAME, 0);
        let _ = card_cntr.add_bytes_elems(&elem_id, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Rawmidi, 0, 0, PUSHED_EVENT_CC_NAME, 0);
        let _ = card_cntr.add_bytes_elems(&elem_id, 1, 1, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Rawmidi, 0, 0, EVENT_TO_PORT_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Rawmidi, 0, 0, EVENT_TO_STREAM_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read_midi_sender(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            NORMAL_EVENT_CH_NAME => {
                let params = &self.segment().data;
                let sender = Self::midi_sender(&params);
                elem_value.set_bytes(&[sender.normal.ch]);
                Ok(true)
            }
            NORMAL_EVENT_CC_NAME => {
                let params = &self.segment().data;
                let sender = Self::midi_sender(&params);
                elem_value.set_bytes(&[sender.normal.cc]);
                Ok(true)
            }
            PUSHED_EVENT_CH_NAME => {
                let params = &self.segment().data;
                let sender = Self::midi_sender(&params);
                elem_value.set_bytes(&[sender.pushed.ch]);
                Ok(true)
            }
            PUSHED_EVENT_CC_NAME => {
                let params = &self.segment().data;
                let sender = Self::midi_sender(&params);
                elem_value.set_bytes(&[sender.pushed.cc]);
                Ok(true)
            }
            EVENT_TO_PORT_NAME => {
                let params = &self.segment().data;
                let sender = Self::midi_sender(&params);
                elem_value.set_bool(&[sender.send_to_port]);
                Ok(true)
            }
            EVENT_TO_STREAM_NAME => {
                let params = &self.segment().data;
                let sender = Self::midi_sender(&params);
                elem_value.set_bool(&[sender.send_to_stream]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_midi_sender(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            NORMAL_EVENT_CH_NAME => {
                let mut params = self.segment().data.clone();
                let mut sender = Self::midi_sender_mut(&mut params);
                sender.normal.ch = elem_value.bytes()[0];
                T::update_partial_segment(req, &node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            NORMAL_EVENT_CC_NAME => {
                let mut params = self.segment().data.clone();
                let mut sender = Self::midi_sender_mut(&mut params);
                sender.normal.cc = elem_value.bytes()[0];
                T::update_partial_segment(req, &node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            PUSHED_EVENT_CH_NAME => {
                let mut params = self.segment().data.clone();
                let mut sender = Self::midi_sender_mut(&mut params);
                sender.pushed.ch = elem_value.bytes()[0];
                T::update_partial_segment(req, &node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            PUSHED_EVENT_CC_NAME => {
                let mut params = self.segment().data.clone();
                let mut sender = Self::midi_sender_mut(&mut params);
                sender.pushed.cc = elem_value.bytes()[0];
                T::update_partial_segment(req, &node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            EVENT_TO_PORT_NAME => {
                let mut params = self.segment().data.clone();
                let mut sender = Self::midi_sender_mut(&mut params);
                sender.send_to_port = elem_value.boolean()[0];
                T::update_partial_segment(req, &node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            EVENT_TO_STREAM_NAME => {
                let mut params = self.segment().data.clone();
                let mut sender = Self::midi_sender_mut(&mut params);
                sender.send_to_stream = elem_value.boolean()[0];
                T::update_partial_segment(req, &node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
