// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use hinawa::{SndDice, SndUnitExt};

use dice_protocols::tcelectronic::{*, midi_send::*};

use core::card_cntr::*;
use core::elem_value_accessor::*;

#[derive(Default, Debug)]
pub struct MidiSendCtl;

const NORMAL_EVENT_CH_NAME: &str = "midi-normal-event-channel";
const NORMAL_EVENT_CC_NAME: &str = "midi-normal-event-cc";
const PUSHED_EVENT_CH_NAME: &str = "midi-pushed-event-channel";
const PUSHED_EVENT_CC_NAME: &str = "midi-pushed-event-cc";
const EVENT_TO_PORT_NAME: &str = "midi-event-to-port";
const EVENT_TO_STREAM_NAME: &str = "midi-event-to-stream";

impl MidiSendCtl {
    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
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

    pub fn read<S>(
        &self,
        segment: &TcKonnektSegment<S>,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error>
        where S: TcKonnektSegmentData + AsRef<TcKonnektMidiSender>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
    {
        match elem_id.get_name().as_str() {
            NORMAL_EVENT_CH_NAME => {
                ElemValueAccessor::<u8>::set_val(elem_value, || Ok(segment.data.as_ref().normal.ch))
                .map(|_| true)
            }
            NORMAL_EVENT_CC_NAME => {
                ElemValueAccessor::<u8>::set_val(elem_value, || Ok(segment.data.as_ref().normal.cc))
                .map(|_| true)
            }
            PUSHED_EVENT_CH_NAME => {
                ElemValueAccessor::<u8>::set_val(elem_value, || Ok(segment.data.as_ref().pushed.ch))
                .map(|_| true)
            }
            PUSHED_EVENT_CC_NAME => {
                ElemValueAccessor::<u8>::set_val(elem_value, || Ok(segment.data.as_ref().pushed.cc))
                .map(|_| true)
            }
            EVENT_TO_PORT_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || Ok(segment.data.as_ref().send_to_port))
                .map(|_| true)
            }
            EVENT_TO_STREAM_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || Ok(segment.data.as_ref().send_to_stream))
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub fn write<T, S>(
        &mut self,
        unit: &mut SndDice,
        proto: &mut T,
        segment: &mut TcKonnektSegment<S>,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32
    )
        -> Result<bool, Error>
        where T: TcKonnektSegmentProtocol<S>,
              S: TcKonnektSegmentData + AsMut<TcKonnektMidiSender>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
    {
        match elem_id.get_name().as_str() {
            NORMAL_EVENT_CH_NAME => {
                ElemValueAccessor::<u8>::get_val(elem_value, |val| {
                    Self::state_write(unit, proto, segment, timeout_ms, |state| {
                        state.normal.ch = val;
                    })
                })
                .map(|_| true)
            }
            NORMAL_EVENT_CC_NAME => {
                ElemValueAccessor::<u8>::get_val(elem_value, |val| {
                    Self::state_write(unit, proto, segment, timeout_ms, |state| {
                        state.normal.cc = val;
                    })
                })
                .map(|_| true)
            }
            PUSHED_EVENT_CH_NAME => {
                ElemValueAccessor::<u8>::get_val(elem_value, |val| {
                    Self::state_write(unit, proto, segment, timeout_ms, |state| {
                        state.pushed.ch = val;
                    })
                })
                .map(|_| true)
            }
            PUSHED_EVENT_CC_NAME => {
                ElemValueAccessor::<u8>::get_val(elem_value, |val| {
                    Self::state_write(unit, proto, segment, timeout_ms, |state| {
                        state.pushed.cc = val;
                    })
                })
                .map(|_| true)
            }
            EVENT_TO_PORT_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    Self::state_write(unit, proto, segment, timeout_ms, |state| {
                        state.send_to_port = val;
                    })
                })
                .map(|_| true)
            }
            EVENT_TO_STREAM_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    Self::state_write(unit, proto, segment, timeout_ms, |state| {
                        state.send_to_stream = val;
                    })
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn state_write<T, S, F>(
        unit: &mut SndDice,
        proto: &mut T,
        segment: &mut TcKonnektSegment<S>,
        timeout_ms: u32,
        cb: F
    ) -> Result<(), Error>
        where T: TcKonnektSegmentProtocol<S>,
              S: TcKonnektSegmentData + AsMut<TcKonnektMidiSender>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
              F: Fn(&mut TcKonnektMidiSender)
    {
        cb(&mut segment.data.as_mut());
        proto.write_segment(&mut unit.get_node(), segment, timeout_ms)
    }
}
