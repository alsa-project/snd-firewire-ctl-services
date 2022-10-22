// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {super::*, protocols::focusrite::spro14::*};

#[derive(Default)]
pub struct SPro14Model {
    req: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    common_ctl: CommonCtl<SPro14Protocol>,
    tcd22xx_ctl: SPro14Tcd22xxCtl,
    out_grp_ctl: OutGroupCtl<SPro14Protocol>,
    input_ctl: SaffireproInputCtl<SPro14Protocol>,
}

const TIMEOUT_MS: u32 = 20;

impl SPro14Model {
    pub fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        SPro14Protocol::read_general_sections(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .cache_whole_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        Ok(())
    }
}

impl CtlModel<(SndDice, FwNode)> for SPro14Model {
    fn load(
        &mut self,
        unit: &mut (SndDice, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.common_ctl.load(card_cntr, &self.sections)?;

        self.extension_sections =
            ProtocolExtension::read_extension_sections(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.tcd22xx_ctl.load(
            unit,
            &mut self.req,
            &self.extension_sections,
            &self.sections.global.params,
            TIMEOUT_MS,
            card_cntr,
        )?;

        self.tcd22xx_ctl.cache(
            unit,
            &mut self.req,
            &self.sections,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;

        self.out_grp_ctl.cache(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;

        self.input_ctl.cache(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;

        self.out_grp_ctl.load(card_cntr)?;
        self.input_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(&self.sections, elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read(
            unit,
            &mut self.req,
            &self.extension_sections,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.out_grp_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.write(
            &unit.0,
            &self.req,
            &unit.1,
            &mut self.sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.tcd22xx_ctl.write(
            unit,
            &mut self.req,
            &self.extension_sections,
            elem_id,
            old,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.out_grp_ctl.write(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.input_ctl.write(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for SPro14Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.1);
        self.tcd22xx_ctl.get_notified_elem_list(elem_id_list);
    }

    fn parse_notification(&mut self, unit: &mut (SndDice, FwNode), msg: &u32) -> Result<(), Error> {
        self.common_ctl.parse_notification(
            &self.req,
            &unit.1,
            &mut self.sections,
            *msg,
            TIMEOUT_MS,
        )?;
        self.tcd22xx_ctl.parse_notification(
            unit,
            &mut self.req,
            &self.sections,
            &self.extension_sections,
            TIMEOUT_MS,
            *msg,
        )?;
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(&self.sections, elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndDice, FwNode)> for SPro14Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.0);
        self.tcd22xx_ctl.get_measured_elem_list(elem_id_list);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .cache_partial_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;
        self.tcd22xx_ctl.measure_states(
            unit,
            &mut self.req,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;
        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(&self.sections, elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default)]
struct SPro14Tcd22xxCtl(Tcd22xxCtl);

impl Tcd22xxCtlOperation<SPro14Protocol> for SPro14Tcd22xxCtl {
    fn tcd22xx_ctl(&self) -> &Tcd22xxCtl {
        &self.0
    }

    fn tcd22xx_ctl_mut(&mut self) -> &mut Tcd22xxCtl {
        &mut self.0
    }
}
