// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::{Error, FileError};

use hinawa::{SndDice, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use dice_protocols::tcelectronic::{*, standalone::*};

use core::card_cntr::*;
use core::elem_value_accessor::*;

fn standalone_rate_to_string(rate: &TcKonnektStandaloneClkRate) -> String {
    match rate {
        TcKonnektStandaloneClkRate::R44100 => "44100",
        TcKonnektStandaloneClkRate::R48000 => "48000",
        TcKonnektStandaloneClkRate::R88200 => "88200",
        TcKonnektStandaloneClkRate::R96000 => "96000",
    }.to_string()
}

#[derive(Default, Debug)]
pub struct TcKonnektStandaloneCtl;

impl TcKonnektStandaloneCtl {
    const RATE_NAME: &'static str = "standalone-clock-rate";

    const RATES: [TcKonnektStandaloneClkRate;4] = [
        TcKonnektStandaloneClkRate::R44100,
        TcKonnektStandaloneClkRate::R48000,
        TcKonnektStandaloneClkRate::R88200,
        TcKonnektStandaloneClkRate::R96000,
    ];

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = Self::RATES.iter()
            .map(|r| standalone_rate_to_string(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::RATE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    pub fn read<S>(
        &mut self,
        segment: &TcKonnektSegment<S>,
        elem_id: &ElemId,
        elem_value: &ElemValue
    ) -> Result<bool, Error>
        where for<'b> S: TcKonnektSegmentData + AsRef<TcKonnektStandaloneClkRate>,
    {
        match elem_id.get_name().as_str() {
            Self::RATE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let rate = segment.data.as_ref();
                    let pos = Self::RATES.iter()
                        .position(|r| r.eq(rate))
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
              for<'b> S: TcKonnektSegmentData + AsMut<TcKonnektStandaloneClkRate>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec
    {
        match elem_id.get_name().as_str() {
            Self::RATE_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::RATES.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of clock rate: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&r| *segment.data.as_mut() = r)
                })
                .and_then(|_| proto.write_segment(&mut unit.get_node(), segment, timeout_ms))
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
