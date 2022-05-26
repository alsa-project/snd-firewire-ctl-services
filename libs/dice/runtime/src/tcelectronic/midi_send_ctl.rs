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
    S: TcKonnektSegmentData,
    TcKonnektSegment<S>: TcKonnektSegmentSpec + TcKonnektNotifiedSegmentSpec,
    T: SegmentOperation<S>,
{
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;
    fn midi_sender(&self) -> &TcKonnektMidiSender;
    fn midi_sender_mut(&mut self) -> &mut TcKonnektMidiSender;

    fn load_midi_sender(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
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
        match elem_id.get_name().as_str() {
            NORMAL_EVENT_CH_NAME => {
                ElemValueAccessor::<u8>::set_val(elem_value, || Ok(self.midi_sender().normal.ch))
                    .map(|_| true)
            }
            NORMAL_EVENT_CC_NAME => {
                ElemValueAccessor::<u8>::set_val(elem_value, || Ok(self.midi_sender().normal.cc))
                    .map(|_| true)
            }
            PUSHED_EVENT_CH_NAME => {
                ElemValueAccessor::<u8>::set_val(elem_value, || Ok(self.midi_sender().pushed.ch))
                    .map(|_| true)
            }
            PUSHED_EVENT_CC_NAME => {
                ElemValueAccessor::<u8>::set_val(elem_value, || Ok(self.midi_sender().pushed.cc))
                    .map(|_| true)
            }
            EVENT_TO_PORT_NAME => ElemValueAccessor::<bool>::set_val(elem_value, || {
                Ok(self.midi_sender().send_to_port)
            })
            .map(|_| true),
            EVENT_TO_STREAM_NAME => ElemValueAccessor::<bool>::set_val(elem_value, || {
                Ok(self.midi_sender().send_to_stream)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write_midi_sender(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            NORMAL_EVENT_CH_NAME => ElemValueAccessor::<u8>::get_val(elem_value, |val| {
                self.state_write(unit, req, timeout_ms, |state| {
                    state.normal.ch = val;
                })
            })
            .map(|_| true),
            NORMAL_EVENT_CC_NAME => ElemValueAccessor::<u8>::get_val(elem_value, |val| {
                self.state_write(unit, req, timeout_ms, |state| {
                    state.normal.cc = val;
                })
            })
            .map(|_| true),
            PUSHED_EVENT_CH_NAME => ElemValueAccessor::<u8>::get_val(elem_value, |val| {
                self.state_write(unit, req, timeout_ms, |state| {
                    state.pushed.ch = val;
                })
            })
            .map(|_| true),
            PUSHED_EVENT_CC_NAME => ElemValueAccessor::<u8>::get_val(elem_value, |val| {
                self.state_write(unit, req, timeout_ms, |state| {
                    state.pushed.cc = val;
                })
            })
            .map(|_| true),
            EVENT_TO_PORT_NAME => ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                self.state_write(unit, req, timeout_ms, |state| {
                    state.send_to_port = val;
                })
            })
            .map(|_| true),
            EVENT_TO_STREAM_NAME => ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                self.state_write(unit, req, timeout_ms, |state| {
                    state.send_to_stream = val;
                })
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn state_write<F>(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
        cb: F,
    ) -> Result<(), Error>
    where
        F: Fn(&mut TcKonnektMidiSender),
    {
        cb(&mut self.midi_sender_mut());
        T::write_segment(req, &mut unit.1, self.segment_mut(), timeout_ms)
    }
}
