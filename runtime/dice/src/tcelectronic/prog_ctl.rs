// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

const LOADED_NAME: &str = "loaded-program";

const LOADED_PROGRAMS: &[TcKonnektLoadedProgram] = &[
    TcKonnektLoadedProgram::P0,
    TcKonnektLoadedProgram::P1,
    TcKonnektLoadedProgram::P2,
];

fn loaded_program_to_str(prog: &TcKonnektLoadedProgram) -> &str {
    match prog {
        TcKonnektLoadedProgram::P0 => "P1",
        TcKonnektLoadedProgram::P1 => "P2",
        TcKonnektLoadedProgram::P2 => "P3",
    }
}

pub fn load_prog<T, U>(card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error>
where
    T: TcKonnektSegmentOperation<U> + TcKonnektMutableSegmentOperation<U>,
    U: Debug + Clone + AsRef<TcKonnektLoadedProgram> + AsMut<TcKonnektLoadedProgram>,
{
    let labels: Vec<&str> = LOADED_PROGRAMS
        .iter()
        .map(|p| loaded_program_to_str(p))
        .collect();
    let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, LOADED_NAME, 0);
    card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
}

pub fn read_prog<T, U>(
    segment: &TcKonnektSegment<U>,
    elem_id: &ElemId,
    elem_value: &ElemValue,
) -> Result<bool, Error>
where
    T: TcKonnektSegmentOperation<U> + TcKonnektMutableSegmentOperation<U>,
    U: Debug + Clone + AsRef<TcKonnektLoadedProgram> + AsMut<TcKonnektLoadedProgram>,
{
    match elem_id.name().as_str() {
        LOADED_NAME => {
            let params = segment.data.as_ref();
            let pos = LOADED_PROGRAMS.iter().position(|p| params.eq(p)).unwrap();
            elem_value.set_enum(&[pos as u32]);
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub fn write_prog<T, U>(
    segment: &mut TcKonnektSegment<U>,
    req: &FwReq,
    node: &FwNode,
    elem_id: &ElemId,
    elem_value: &ElemValue,
    timeout_ms: u32,
) -> Result<bool, Error>
where
    T: TcKonnektSegmentOperation<U> + TcKonnektMutableSegmentOperation<U>,
    U: Debug + Clone + AsRef<TcKonnektLoadedProgram> + AsMut<TcKonnektLoadedProgram>,
{
    match elem_id.name().as_str() {
        LOADED_NAME => {
            let mut data = segment.data.clone();
            let params = data.as_mut();
            let pos = elem_value.enumerated()[0] as usize;
            LOADED_PROGRAMS
                .iter()
                .nth(pos)
                .ok_or_else(|| {
                    let msg = format!("Invalid index for loaded programs: {}", pos);
                    Error::new(FileError::Inval, &msg)
                })
                .map(|&prog| *params = prog)?;
            let res = T::update_partial_segment(req, node, &data, segment, timeout_ms);
            debug!(params = ?segment.data, ?res);
            res.map(|_| true)
        }
        _ => Ok(false),
    }
}
