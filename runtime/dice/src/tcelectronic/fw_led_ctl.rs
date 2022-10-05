// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

const STATE_NAME: &str = "FireWire-LED-state";

pub fn firewire_led_state_to_str(state: &FireWireLedState) -> &'static str {
    match state {
        FireWireLedState::Off => "Off",
        FireWireLedState::On => "On",
        FireWireLedState::BlinkFast => "Blink-fast",
        FireWireLedState::BlinkSlow => "Blink-slow",
    }
}

pub trait FirewireLedCtlOperation<S, T>
where
    S: TcKonnektSegmentData + Clone,
    T: TcKonnektSegmentOperation<S> + TcKonnektMutableSegmentOperation<S>,
{
    fn segment(&self) -> &TcKonnektSegment<S>;
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;
    fn firewire_led(params: &S) -> &FireWireLedState;
    fn firewire_led_mut(params: &mut S) -> &mut FireWireLedState;

    const STATES: [FireWireLedState; 4] = [
        FireWireLedState::Off,
        FireWireLedState::On,
        FireWireLedState::BlinkFast,
        FireWireLedState::BlinkSlow,
    ];

    fn load_firewire_led(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let labels = Self::STATES
            .iter()
            .map(|s| firewire_led_state_to_str(s))
            .collect::<Vec<_>>();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, STATE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
    }

    fn read_firewire_led(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            STATE_NAME => {
                let params = &self.segment().data;
                let pos = Self::STATES
                    .iter()
                    .position(|s| Self::firewire_led(&params).eq(s))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_firewire_led(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            STATE_NAME => {
                let pos = elem_value.enumerated()[0] as usize;
                let s = Self::STATES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of FireWire LED: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let mut params = self.segment().data.clone();
                *Self::firewire_led_mut(&mut params) = s;
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
