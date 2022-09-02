// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {super::*, protocols::focusrite::spro24::*};

#[derive(Default)]
pub struct SPro24Model {
    req: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    ctl: CommonCtl,
    tcd22xx_ctl: SPro24Tcd22xxCtl,
    out_grp_ctl: OutGroupCtl,
    input_ctl: InputCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<(SndDice, FwNode)> for SPro24Model {
    fn load(
        &mut self,
        unit: &mut (SndDice, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.sections =
            GeneralProtocol::read_general_sections(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        let caps = GlobalSectionProtocol::read_clock_caps(
            &mut self.req,
            &mut unit.1,
            &self.sections,
            TIMEOUT_MS,
        )?;
        let src_labels = GlobalSectionProtocol::read_clock_source_labels(
            &mut self.req,
            &mut unit.1,
            &self.sections,
            TIMEOUT_MS,
        )?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.extension_sections =
            ProtocolExtension::read_extension_sections(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.tcd22xx_ctl.load(
            unit,
            &mut self.req,
            &self.extension_sections,
            &caps,
            &src_labels,
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

        let _ = self.out_grp_ctl.load(
            card_cntr,
            unit,
            &mut self.req,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;

        self.input_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.ctl.read(
            unit,
            &mut self.req,
            &self.sections,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
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
        } else if self.input_ctl.read(
            unit,
            &mut self.req,
            &self.extension_sections,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
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
        if self.ctl.write(
            unit,
            &mut self.req,
            &self.sections,
            elem_id,
            old,
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
            unit,
            &mut self.req,
            &self.extension_sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.input_ctl.write(
            unit,
            &mut self.req,
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
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        self.tcd22xx_ctl.get_notified_elem_list(elem_id_list);
        elem_id_list.extend_from_slice(&self.out_grp_ctl.1);
    }

    fn parse_notification(&mut self, unit: &mut (SndDice, FwNode), msg: &u32) -> Result<(), Error> {
        self.ctl
            .parse_notification(unit, &mut self.req, &self.sections, *msg, TIMEOUT_MS)?;
        self.tcd22xx_ctl.parse_notification(
            unit,
            &mut self.req,
            &self.sections,
            &self.extension_sections,
            TIMEOUT_MS,
            *msg,
        )?;
        self.out_grp_ctl.parse_notification(
            unit,
            &mut self.req,
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
        if self.ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_grp_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndDice, FwNode)> for SPro24Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        self.tcd22xx_ctl.get_measured_elem_list(elem_id_list);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.ctl
            .measure_states(unit, &mut self.req, &self.sections, TIMEOUT_MS)?;
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
        if self.ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.measure_elem(elem_id, elem_value)? {
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

#[derive(Default)]
struct OutGroupCtl(OutGroupState, Vec<ElemId>);

impl OutGroupCtlOperation<SPro24Protocol> for OutGroupCtl {
    fn state(&self) -> &OutGroupState {
        &self.0
    }

    fn state_mut(&mut self) -> &mut OutGroupState {
        &mut self.0
    }
}

#[derive(Default)]
struct InputCtl;

impl SaffireproInputCtlOperation<SPro24Protocol> for InputCtl {}
