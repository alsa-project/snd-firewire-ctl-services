// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

fn standalone_rate_to_str(rate: &TcKonnektStandaloneClkRate) -> &'static str {
    match rate {
        TcKonnektStandaloneClkRate::R44100 => "44100",
        TcKonnektStandaloneClkRate::R48000 => "48000",
        TcKonnektStandaloneClkRate::R88200 => "88200",
        TcKonnektStandaloneClkRate::R96000 => "96000",
    }
}

const RATE_NAME: &str = "standalone-clock-rate";

pub trait StandaloneCtlOperation<S, T>
where
    S: TcKonnektSegmentData,
    TcKonnektSegment<S>: TcKonnektSegmentSpec,
    T: SegmentOperation<S>,
{
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;
    fn standalone_rate(&self) -> &TcKonnektStandaloneClkRate;
    fn standalone_rate_mut(&mut self) -> &mut TcKonnektStandaloneClkRate;

    const RATES: [TcKonnektStandaloneClkRate; 4] = [
        TcKonnektStandaloneClkRate::R44100,
        TcKonnektStandaloneClkRate::R48000,
        TcKonnektStandaloneClkRate::R88200,
        TcKonnektStandaloneClkRate::R96000,
    ];

    fn load_standalone_rate(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::RATES
            .iter()
            .map(|r| standalone_rate_to_str(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, RATE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        Ok(())
    }

    fn read_standalone_rate(
        &mut self,
        elem_id: &ElemId,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            RATE_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let pos = Self::RATES
                    .iter()
                    .position(|r| self.standalone_rate().eq(r))
                    .unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write_standalone_rate(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            RATE_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::RATES
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index of clock rate: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&r| *self.standalone_rate_mut() = r)
                })?;
                T::write_segment(req, &mut unit.1, self.segment_mut(), timeout_ms).map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
