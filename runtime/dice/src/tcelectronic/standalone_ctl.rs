// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

fn standalone_rate_to_str(rate: &TcKonnektStandaloneClockRate) -> &'static str {
    match rate {
        TcKonnektStandaloneClockRate::R44100 => "44100",
        TcKonnektStandaloneClockRate::R48000 => "48000",
        TcKonnektStandaloneClockRate::R88200 => "88200",
        TcKonnektStandaloneClockRate::R96000 => "96000",
    }
}

const RATE_NAME: &str = "standalone-clock-rate";

pub trait StandaloneCtlOperation<S, T>
where
    S: Clone,
    T: TcKonnektSegmentOperation<S> + TcKonnektMutableSegmentOperation<S>,
{
    fn segment(&self) -> &TcKonnektSegment<S>;
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;

    fn standalone_rate(params: &S) -> &TcKonnektStandaloneClockRate;
    fn standalone_rate_mut(params: &mut S) -> &mut TcKonnektStandaloneClockRate;

    const RATES: [TcKonnektStandaloneClockRate; 4] = [
        TcKonnektStandaloneClockRate::R44100,
        TcKonnektStandaloneClockRate::R48000,
        TcKonnektStandaloneClockRate::R88200,
        TcKonnektStandaloneClockRate::R96000,
    ];

    fn load_standalone_rate(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
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
            RATE_NAME => {
                let params = &self.segment().data;
                let rate = Self::standalone_rate(&params);
                let pos = Self::RATES.iter().position(|r| rate.eq(r)).unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_standalone_rate(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            RATE_NAME => {
                let pos = elem_value.enumerated()[0] as usize;
                let rate = Self::RATES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of clock rate: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let mut params = self.segment_mut().data.clone();
                *Self::standalone_rate_mut(&mut params) = rate;
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
