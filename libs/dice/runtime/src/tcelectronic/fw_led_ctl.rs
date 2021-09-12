// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use hinawa::{SndDice, SndUnitExt};

use dice_protocols::tcelectronic::*;
use dice_protocols::tcelectronic::fw_led::*;

use core::card_cntr::*;
use core::elem_value_accessor::*;

pub fn firewire_led_state_to_string(state: &FireWireLedState) -> String {
    match state {
        FireWireLedState::Off => "Off",
        FireWireLedState::On => "On",
        FireWireLedState::BlinkFast => "Blink-fast",
        FireWireLedState::BlinkSlow => "Blink-slow",
    }.to_string()
}

#[derive(Default, Debug)]
pub struct FwLedCtl(pub Vec<ElemId>);

impl FwLedCtl {
    const STATE_NAME: &'static str = "FireWire-LED-state";

    const STATES: [FireWireLedState;4] = [
        FireWireLedState::Off,
        FireWireLedState::On,
        FireWireLedState::BlinkFast,
        FireWireLedState::BlinkSlow,
    ];

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels = Self::STATES.iter()
            .map(|s| firewire_led_state_to_string(s))
            .collect::<Vec<_>>();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::STATE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        Ok(())
    }

    pub fn read<S>(
        &mut self,
        segment: &TcKonnektSegment<S>,
        elem_id: &ElemId,
        elem_value: &mut ElemValue
    ) -> Result<bool, Error>
        where S: TcKonnektSegmentData + AsRef<FireWireLedState>,
    {
        match elem_id.get_name().as_str() {
            Self::STATE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::STATES.iter()
                        .position(|s| s.eq(segment.data.as_ref()))
                        .expect("Programming error");
                    Ok(pos as u32)
                })
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
    ) -> Result<bool, Error>
        where T: TcKonnektSegmentProtocol<S>,
              S: TcKonnektSegmentData + AsMut<FireWireLedState>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
    {
        match elem_id.get_name().as_str() {
            Self::STATE_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::STATES.iter()
                        .nth(val as usize)
                        .ok_or_else(||{
                            let msg = format!("Invalid index of FireWire LED: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| *segment.data.as_mut() = s)
                })
                .and_then(|_| proto.write_segment(&mut unit.get_node(), segment, timeout_ms))
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
