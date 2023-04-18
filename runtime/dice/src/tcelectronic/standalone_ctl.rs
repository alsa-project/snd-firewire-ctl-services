// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

const RATE_NAME: &str = "standalone-clock-rate";

const RATES: &[TcKonnektStandaloneClockRate] = &[
    TcKonnektStandaloneClockRate::R44100,
    TcKonnektStandaloneClockRate::R48000,
    TcKonnektStandaloneClockRate::R88200,
    TcKonnektStandaloneClockRate::R96000,
];

fn standalone_rate_to_str(rate: &TcKonnektStandaloneClockRate) -> &str {
    match rate {
        TcKonnektStandaloneClockRate::R44100 => "44100",
        TcKonnektStandaloneClockRate::R48000 => "48000",
        TcKonnektStandaloneClockRate::R88200 => "88200",
        TcKonnektStandaloneClockRate::R96000 => "96000",
    }
}

pub fn load_standalone_rate<T, U>(card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error>
where
    T: TcKonnektSegmentOperation<U> + TcKonnektMutableSegmentOperation<U>,
    U: Debug + Clone + AsRef<TcKonnektStandaloneClockRate> + AsMut<TcKonnektStandaloneClockRate>,
{
    let labels: Vec<&str> = RATES.iter().map(|r| standalone_rate_to_str(r)).collect();
    let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, RATE_NAME, 0);
    card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
}

pub fn read_standalone_rate<T, U>(
    segment: &TcKonnektSegment<U>,
    elem_id: &ElemId,
    elem_value: &ElemValue,
) -> Result<bool, Error>
where
    T: TcKonnektSegmentOperation<U> + TcKonnektMutableSegmentOperation<U>,
    U: Debug + Clone + AsRef<TcKonnektStandaloneClockRate> + AsMut<TcKonnektStandaloneClockRate>,
{
    match elem_id.name().as_str() {
        RATE_NAME => {
            let params = segment.data.as_ref();
            let pos = RATES.iter().position(|r| params.eq(r)).unwrap();
            elem_value.set_enum(&[pos as u32]);
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub fn write_standalone_rate<T, U>(
    segment: &mut TcKonnektSegment<U>,
    req: &FwReq,
    node: &FwNode,
    elem_id: &ElemId,
    elem_value: &ElemValue,
    timeout_ms: u32,
) -> Result<bool, Error>
where
    T: TcKonnektSegmentOperation<U> + TcKonnektMutableSegmentOperation<U>,
    U: Debug + Clone + AsRef<TcKonnektStandaloneClockRate> + AsMut<TcKonnektStandaloneClockRate>,
{
    match elem_id.name().as_str() {
        RATE_NAME => {
            let mut data = segment.data.clone();
            let pos = elem_value.enumerated()[0] as usize;
            RATES
                .iter()
                .nth(pos)
                .ok_or_else(|| {
                    let msg = format!("Invalid index of clock rate: {}", pos);
                    Error::new(FileError::Inval, &msg)
                })
                .map(|&rate| {
                    *data.as_mut() = rate;
                })?;
            let res = T::update_partial_segment(req, node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        _ => Ok(false),
    }
}
