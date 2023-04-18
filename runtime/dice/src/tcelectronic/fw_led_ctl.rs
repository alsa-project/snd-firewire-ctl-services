// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

const STATE_NAME: &str = "FireWire-LED-state";

const STATES: &[FireWireLedState] = &[
    FireWireLedState::Off,
    FireWireLedState::On,
    FireWireLedState::BlinkFast,
    FireWireLedState::BlinkSlow,
];

fn firewire_led_state_to_str(state: &FireWireLedState) -> &'static str {
    match state {
        FireWireLedState::Off => "Off",
        FireWireLedState::On => "On",
        FireWireLedState::BlinkFast => "Blink-fast",
        FireWireLedState::BlinkSlow => "Blink-slow",
    }
}

pub fn load_firewire_led<T, U>(card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error>
where
    T: TcKonnektSegmentOperation<U> + TcKonnektMutableSegmentOperation<U>,
    U: Debug + Clone + AsRef<FireWireLedState> + AsMut<FireWireLedState>,
{
    let labels = STATES
        .iter()
        .map(|s| firewire_led_state_to_str(s))
        .collect::<Vec<_>>();
    let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, STATE_NAME, 0);
    card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
}

pub fn read_firewire_led<T, U>(
    segment: &TcKonnektSegment<U>,
    elem_id: &ElemId,
    elem_value: &mut ElemValue,
) -> Result<bool, Error>
where
    T: TcKonnektSegmentOperation<U> + TcKonnektMutableSegmentOperation<U>,
    U: Debug + Clone + AsRef<FireWireLedState> + AsMut<FireWireLedState>,
{
    match elem_id.name().as_str() {
        STATE_NAME => {
            let params = segment.data.as_ref();
            let pos = STATES.iter().position(|s| params.eq(s)).unwrap();
            elem_value.set_enum(&[pos as u32]);
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub fn write_firewire_led<T, U>(
    segment: &mut TcKonnektSegment<U>,
    req: &mut FwReq,
    node: &mut FwNode,
    elem_id: &ElemId,
    elem_value: &ElemValue,
    timeout_ms: u32,
) -> Result<bool, Error>
where
    T: TcKonnektSegmentOperation<U> + TcKonnektMutableSegmentOperation<U>,
    U: Debug + Clone + AsRef<FireWireLedState> + AsMut<FireWireLedState>,
{
    match elem_id.name().as_str() {
        STATE_NAME => {
            let pos = elem_value.enumerated()[0] as usize;
            let mut data = segment.data.clone();
            let params = data.as_mut();
            STATES
                .iter()
                .nth(pos)
                .ok_or_else(|| {
                    let msg = format!("Invalid index of FireWire LED: {}", pos);
                    Error::new(FileError::Inval, &msg)
                })
                .map(|&s| *params = s)?;
            let res = T::update_partial_segment(req, node, &data, segment, timeout_ms);
            debug!(params = ?segment, ?res);
            res.map(|_| true)
        }
        _ => Ok(false),
    }
}
