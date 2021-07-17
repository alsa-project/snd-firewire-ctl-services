// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::{Error, FileError};

use hinawa::{FwNode, SndDice, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use dice_protocols::tcelectronic::{*, prog::*};

use core::card_cntr::*;
use core::elem_value_accessor::*;

#[derive(Default, Debug)]
pub struct TcKonnektProgramCtl(pub Vec<ElemId>);

impl TcKonnektProgramCtl {
    const LOADED_NAME: &'static str = "loaded-program";

    const PROG_LABELS: [&'static str;3] = ["P1", "P2", "P3"];

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = Self::PROG_LABELS.iter()
            .map(|l| l.to_string())
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::LOADED_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))
    }

    pub fn read<S>(&mut self, segment: &TcKonnektSegment<S>, elem_id: &ElemId, elem_value: &ElemValue)
        -> Result<bool, Error>
        where S: TcKonnektSegmentData + AsRef<TcKonnektLoadedProgram>,
    {
        match elem_id.get_name().as_str() {
            Self::LOADED_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let state = segment.data.as_ref();
                    if state.0 >= Self::PROG_LABELS.len() as u32 {
                        let msg = format!("Unexpected value for index of program: {}", state.0);
                        Err(Error::new(FileError::Io, &msg))
                    } else {
                        Ok(state.0)
                    }
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub fn write<T, S>(&mut self, unit: &SndDice, proto: &T, segment: &mut TcKonnektSegment<S>,
                       elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where T: TcKonnektSegmentProtocol<FwNode, S>,
              for<'b> S: TcKonnektSegmentData + AsMut<TcKonnektLoadedProgram>,
              TcKonnektSegment<S>: TcKonnektSegmentSpec,
    {
        match elem_id.get_name().as_str() {
            Self::LOADED_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    if val >= Self::PROG_LABELS.len() as u32 {
                        let msg = format!("Invalid value for index of program: {}", val);
                        Err(Error::new(FileError::Io, &msg))
                    } else {
                        segment.data.as_mut().0 = val;
                        proto.write_segment(&unit.get_node(), segment, timeout_ms)
                    }
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
