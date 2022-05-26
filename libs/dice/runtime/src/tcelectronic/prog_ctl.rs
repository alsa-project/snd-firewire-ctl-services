// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

const LOADED_NAME: &str = "loaded-program";

pub trait ProgramCtlOperation<S, T>
where
    S: TcKonnektSegmentData,
    TcKonnektSegment<S>: TcKonnektSegmentSpec + TcKonnektNotifiedSegmentSpec,
    T: SegmentOperation<S>,
{
    const PROG_LABELS: [&'static str; 3] = ["P1", "P2", "P3"];

    fn segment_mut(&mut self) -> &mut TcKonnektSegment<S>;
    fn prog(&self) -> &TcKonnektLoadedProgram;
    fn prog_mut(&mut self) -> &mut TcKonnektLoadedProgram;

    fn load_prog(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let labels: Vec<String> = Self::PROG_LABELS.iter().map(|l| l.to_string()).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, LOADED_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
    }

    fn read_prog(&mut self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            LOADED_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                if self.prog().0 >= Self::PROG_LABELS.len() as u32 {
                    let msg = format!("Unexpected index of program: {}", self.prog().0);
                    Err(Error::new(FileError::Io, &msg))
                } else {
                    Ok(self.prog().0)
                }
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write_prog(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            LOADED_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    if val >= Self::PROG_LABELS.len() as u32 {
                        let msg = format!("Invalid value for index of program: {}", val);
                        Err(Error::new(FileError::Io, &msg))
                    } else {
                        self.prog_mut().0 = val;
                        Ok(())
                    }
                })?;
                T::write_segment(req, &mut unit.1, self.segment_mut(), timeout_ms).map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
