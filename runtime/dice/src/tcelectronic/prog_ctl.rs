// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

const LOADED_NAME: &str = "loaded-program";

pub trait ProgramCtlOperation<S, T>
where
    S: TcKonnektSegmentData + Clone,
    T: TcKonnektSegmentOperation<S> + TcKonnektMutableSegmentOperation<S>,
{
    const PROG_LABELS: [&'static str; 3] = ["P1", "P2", "P3"];

    fn segment(&self) -> &TcKonnektSegment<S>;
    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;

    fn prog(params: &S) -> &TcKonnektLoadedProgram;
    fn prog_mut(params: &mut S) -> &mut TcKonnektLoadedProgram;

    fn load_prog(&self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let labels: Vec<String> = Self::PROG_LABELS.iter().map(|l| l.to_string()).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, LOADED_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
    }

    fn read_prog(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            LOADED_NAME => {
                let params = &self.segment().data;
                let prog = Self::prog(&params);
                if prog.0 >= Self::PROG_LABELS.len() as u32 {
                    let msg = format!("Unexpected index of program: {}", prog.0);
                    Err(Error::new(FileError::Io, &msg))?;
                }
                elem_value.set_enum(&[prog.0]);
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
                let val = elem_value.enumerated()[0];
                if val >= Self::PROG_LABELS.len() as u32 {
                    let msg = format!("Invalid value for index of program: {}", val);
                    Err(Error::new(FileError::Io, &msg))?;
                }
                let mut params = self.segment().data.clone();
                Self::prog_mut(&mut params).0 = val;
                T::update_partial_segment(req, node, &params, self.segment_mut(), timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
