// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2024 Takashi Sakamoto

use {super::*, protocols::weiss::avc::*};

#[derive(Default)]
pub struct WeissMan301Model {
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl<WeissMan301Protocol>,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<(SndDice, FwNode)> for WeissMan301Model {
    fn cache(&mut self, (_, node): &mut (SndDice, FwNode)) -> Result<(), Error> {
        WeissMan301Protocol::read_general_sections(
            &self.req,
            &node,
            &mut self.sections,
            TIMEOUT_MS,
        )?;

        self.common_ctl
            .cache_whole_params(&self.req, &node, &mut self.sections, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr)?;
        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        (unit, node): &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.write(
            &unit,
            &self.req,
            &node,
            &mut self.sections,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for WeissMan301Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_id_list);
    }

    fn parse_notification(
        &mut self,
        (_, node): &mut (SndDice, FwNode),
        msg: &u32,
    ) -> Result<(), Error> {
        self.common_ctl
            .parse_notification(&self.req, &node, &mut self.sections, *msg, TIMEOUT_MS)
    }
}

impl MeasureModel<(SndDice, FwNode)> for WeissMan301Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.measured_elem_id_list);
    }

    fn measure_states(&mut self, (_, node): &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .cache_partial_params(&self.req, &node, &mut self.sections, TIMEOUT_MS)
    }
}
