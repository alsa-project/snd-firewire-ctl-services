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
    S: TcKonnektSegmentData,
    TcKonnektSegment<S>: TcKonnektSegmentSpec,
    T: SegmentOperation<S>,
{
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;
    fn firewire_led(&self) -> &FireWireLedState;
    fn firewire_led_mut(&mut self) -> &mut FireWireLedState;

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
        match elem_id.get_name().as_str() {
            STATE_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let pos = Self::STATES
                    .iter()
                    .position(|s| self.firewire_led().eq(s))
                    .unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write_firewire_led(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            STATE_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::STATES
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index of FireWire LED: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| *self.firewire_led_mut() = s)
                })?;
                T::write_segment(req, &mut unit.get_node(), self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
