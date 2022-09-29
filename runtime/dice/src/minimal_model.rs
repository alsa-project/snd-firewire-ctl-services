// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

#[derive(Default)]
pub struct MinimalModel {
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl,
}

const TIMEOUT_MS: u32 = 20;

impl MinimalModel {
    pub fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        GeneralProtocol::read_general_sections(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .whole_cache(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        Ok(())
    }
}

impl CtlModel<(SndDice, FwNode)> for MinimalModel {
    fn load(&mut self, _: &mut (SndDice, FwNode), card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr, &self.sections).map(
            |(measured_elem_id_list, notified_elem_id_list)| {
                self.common_ctl.0 = measured_elem_id_list;
                self.common_ctl.1 = notified_elem_id_list;
            },
        )
    }

    fn read(
        &mut self,
        _: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.common_ctl.read(&self.sections, elem_id, elem_value)
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        self.common_ctl.write(
            &unit.0,
            &self.req,
            &unit.1,
            &mut self.sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for MinimalModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.1);
    }

    fn parse_notification(&mut self, unit: &mut (SndDice, FwNode), msg: &u32) -> Result<(), Error> {
        self.common_ctl
            .parse_notification(&self.req, &unit.1, &mut self.sections, *msg, TIMEOUT_MS)
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.common_ctl.read(&self.sections, elem_id, elem_value)
    }
}

impl MeasureModel<(SndDice, FwNode)> for MinimalModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.0);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .measure(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)
    }

    fn measure_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.common_ctl.read(&self.sections, elem_id, elem_value)
    }
}

#[derive(Default, Debug)]
struct GeneralProtocol;

impl TcatOperation for GeneralProtocol {}

impl TcatGlobalSectionSpecification for GeneralProtocol {}

#[derive(Default, Debug)]
struct CommonCtl(Vec<ElemId>, Vec<ElemId>);

impl CommonCtlOperation<GeneralProtocol> for CommonCtl {}
