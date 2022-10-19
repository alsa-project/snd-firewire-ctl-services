// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {super::*, protocols::focusrite::spro24::*};

#[derive(Default)]
pub struct SPro24Model {
    req: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    common_ctl: CommonCtl<SPro24Protocol>,
    tcd22xx_ctls: Tcd22xxCtls<SPro24Protocol>,
    tcd22xx_ctl: SPro24Tcd22xxCtl,
    out_grp_ctl: OutGroupCtl<SPro24Protocol>,
    input_ctl: SaffireproInputCtl<SPro24Protocol>,
}

const TIMEOUT_MS: u32 = 20;

impl SPro24Model {
    pub fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        SPro24Protocol::read_general_sections(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .cache_whole_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.extension_sections =
            ProtocolExtension::read_extension_sections(&mut self.req, &mut unit.1, TIMEOUT_MS)?;

        self.tcd22xx_ctls.cache_whole_params(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            &self.sections.global.params,
            TIMEOUT_MS,
        )?;

        self.tcd22xx_ctl.cache(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            &self.sections.global.params,
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

        Ok(())
    }
}

impl CtlModel<(SndDice, FwNode)> for SPro24Model {
    fn load(&mut self, _: &mut (SndDice, FwNode), card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr, &self.sections)?;

        self.tcd22xx_ctls.load(card_cntr)?;

        self.tcd22xx_ctl
            .load(card_cntr, &self.sections.global.params)?;

        self.out_grp_ctl.load(card_cntr)?;
        self.input_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(&self.sections, elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read(elem_id, elem_value)? {
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
        } else if self.tcd22xx_ctls.write(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
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

impl NotifyModel<(SndDice, FwNode), u32> for SPro24Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.1);
        elem_id_list.extend_from_slice(&self.tcd22xx_ctls.notified_elem_id_list);
        self.tcd22xx_ctl.get_notified_elem_list(elem_id_list);
        elem_id_list.extend_from_slice(&self.out_grp_ctl.1);
    }

    fn parse_notification(&mut self, unit: &mut (SndDice, FwNode), msg: &u32) -> Result<(), Error> {
        self.common_ctl.parse_notification(
            &self.req,
            &unit.1,
            &mut self.sections,
            *msg,
            TIMEOUT_MS,
        )?;
        self.tcd22xx_ctls.parse_notification(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            &self.sections.global.params,
            TIMEOUT_MS,
            *msg,
        )?;
        self.tcd22xx_ctl.parse_notification(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            &self.sections.global.params,
            TIMEOUT_MS,
            *msg,
        )?;
        self.out_grp_ctl.parse_notification(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            *msg,
            TIMEOUT_MS,
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
        } else if self.tcd22xx_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_grp_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndDice, FwNode)> for SPro24Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.0);
        elem_id_list.extend_from_slice(&self.tcd22xx_ctls.measured_elem_id_list);
        self.tcd22xx_ctl.get_measured_elem_list(elem_id_list);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .cache_partial_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;
        self.tcd22xx_ctls.cache_partial_params(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;
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
        } else if self.tcd22xx_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default)]
struct SPro24Tcd22xxCtl(Tcd22xxCtl);

impl Tcd22xxCtlOperation<SPro24Protocol> for SPro24Tcd22xxCtl {
    fn tcd22xx_ctl(&self) -> &Tcd22xxCtl {
        &self.0
    }

    fn tcd22xx_ctl_mut(&mut self) -> &mut Tcd22xxCtl {
        &mut self.0
    }
}
