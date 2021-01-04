// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::Error;

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use hinawa::{FwNode, SndDice};

use dice_protocols::tcelectronic::*;
use dice_protocols::tcelectronic::fw_led::*;
use dice_protocols::tcelectronic::shell::*;

use core::card_cntr::*;
use core::elem_value_accessor::*;

use super::fw_led_ctl::*;

fn analog_jack_state_to_string(state: &ShellAnalogJackState) -> String {
    match state {
        ShellAnalogJackState::FrontSelected => "Front-selected",
        ShellAnalogJackState::FrontInserted => "Front-inserted",
        ShellAnalogJackState::FrontInsertedAttenuated => "Front-inserted-attenuated",
        ShellAnalogJackState::RearSelected => "Rear-selected",
        ShellAnalogJackState::RearInserted => "Rear-inserted",
    }.to_string()
}

#[derive(Default, Debug)]
pub struct HwStateCtl {
    pub notified_elem_list: Vec<ElemId>,
    fw_led_ctl: FwLedCtl,
}

impl<'a> HwStateCtl {
    // TODO: For Jack detection in ALSA applications.
    const ANALOG_JACK_STATE_NAME: &'a str = "analog-jack-state";

    const ANALOG_JACK_STATE_LABELS: &'a [ShellAnalogJackState] = &[
        ShellAnalogJackState::FrontSelected,
        ShellAnalogJackState::FrontInserted,
        ShellAnalogJackState::FrontInsertedAttenuated,
        ShellAnalogJackState::RearSelected,
        ShellAnalogJackState::RearInserted,
    ];

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels = Self::ANALOG_JACK_STATE_LABELS.iter()
            .map(|s| analog_jack_state_to_string(s))
            .collect::<Vec<_>>();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::ANALOG_JACK_STATE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, SHELL_ANALOG_JACK_STATE_COUNT, &labels, None, false)
            .map(|mut elem_id_list| self.notified_elem_list.append(&mut elem_id_list))?;

        self.fw_led_ctl.load(card_cntr)?;
        self.notified_elem_list.extend_from_slice(&self.fw_led_ctl.0);

        Ok(())
    }

    pub fn read<S>(&mut self, segment: &TcKonnektSegment<S>, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where S: TcKonnektSegmentData + AsRef<[ShellAnalogJackState]> + AsRef<FireWireLedState>,
    {
        match elem_id.get_name().as_str() {
            Self::ANALOG_JACK_STATE_NAME => {
                let analog_jack_states = AsRef::<[ShellAnalogJackState]>::as_ref(&segment.data);
                ElemValueAccessor::<u32>::set_vals(elem_value, analog_jack_states.len(), |idx| {
                    let pos = Self::ANALOG_JACK_STATE_LABELS.iter()
                        .position(|s| s.eq(&analog_jack_states[idx]))
                        .expect("Programming error");
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            _ => self.fw_led_ctl.read(segment, elem_id, elem_value),
        }
    }

    pub fn write<T, S>(&mut self, unit: &SndDice, proto: &T, segment: &mut TcKonnektSegment<S>,
                       elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where T: TcKonnektSegmentProtocol<FwNode, S>,
              S: TcKonnektSegmentData + AsMut<FireWireLedState>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
    {
        self.fw_led_ctl.write(unit, proto, segment, elem_id, elem_value, timeout_ms)
    }
}
