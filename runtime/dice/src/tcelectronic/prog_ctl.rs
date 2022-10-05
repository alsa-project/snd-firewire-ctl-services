// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

const LOADED_NAME: &str = "loaded-program";

fn loaded_program_to_str(prog: &TcKonnektLoadedProgram) -> &str {
    match prog {
        TcKonnektLoadedProgram::P0 => "P1",
        TcKonnektLoadedProgram::P1 => "P2",
        TcKonnektLoadedProgram::P2 => "P3",
    }
}

pub trait ProgramCtlOperation<S, T>
where
    S: Clone,
    T: TcKonnektSegmentOperation<S> + TcKonnektMutableSegmentOperation<S>,
{
    const LOADED_PROGRAMS: &'static [TcKonnektLoadedProgram] = &[
        TcKonnektLoadedProgram::P0,
        TcKonnektLoadedProgram::P1,
        TcKonnektLoadedProgram::P2,
    ];

    fn segment(&self) -> &TcKonnektSegment<S>;
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;

    fn prog(params: &S) -> &TcKonnektLoadedProgram;
    fn prog_mut(params: &mut S) -> &mut TcKonnektLoadedProgram;

    fn load_prog(&self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let labels: Vec<&str> = Self::LOADED_PROGRAMS
            .iter()
            .map(|p| loaded_program_to_str(p))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, LOADED_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
    }

    fn read_prog(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            LOADED_NAME => {
                let params = &self.segment().data;
                let prog = Self::prog(&params);
                let pos = Self::LOADED_PROGRAMS
                    .iter()
                    .position(|p| prog.eq(p))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_prog(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            LOADED_NAME => {
                let mut params = self.segment().data.clone();
                let prog = Self::prog_mut(&mut params);
                let pos = elem_value.enumerated()[0] as usize;
                Self::LOADED_PROGRAMS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index for loaded programs: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&p| *prog = p)?;
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
