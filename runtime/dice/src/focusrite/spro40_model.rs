// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {super::*, protocols::focusrite::spro40::*};

#[derive(Default)]
pub struct SPro40Model {
    req: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    common_ctl: CommonCtl,
    tcd22xx_ctl: SPro40Tcd22xxCtl,
    out_grp_ctl: OutGroupCtl<SPro40Protocol>,
    specific_ctl: SpecificCtl,
}

const TIMEOUT_MS: u32 = 20;

impl SPro40Model {
    pub fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        SPro40Protocol::read_general_sections(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .whole_cache(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        Ok(())
    }
}

impl CtlModel<(SndDice, FwNode)> for SPro40Model {
    fn load(
        &mut self,
        unit: &mut (SndDice, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.common_ctl.load(card_cntr, &self.sections).map(
            |(measured_elem_id_list, notified_elem_id_list)| {
                self.common_ctl.0 = measured_elem_id_list;
                self.common_ctl.1 = notified_elem_id_list;
            },
        )?;

        self.extension_sections =
            ProtocolExtension::read_extension_sections(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.tcd22xx_ctl.load(
            unit,
            &mut self.req,
            &self.extension_sections,
            &self.sections.global.params.avail_rates,
            &self.sections.global.params.avail_sources,
            &self.sections.global.params.clock_source_labels,
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

        self.out_grp_ctl.load(card_cntr)?;
        self.specific_ctl.load(card_cntr)?;

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
        } else if self.specific_ctl.read(
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
        } else if self.specific_ctl.write(
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

impl NotifyModel<(SndDice, FwNode), u32> for SPro40Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.1);
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
        self.tcd22xx_ctl.parse_notification(
            unit,
            &mut self.req,
            &self.sections,
            &self.extension_sections,
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
        } else if self.tcd22xx_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_grp_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndDice, FwNode)> for SPro40Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.0);
        self.tcd22xx_ctl.get_measured_elem_list(elem_id_list);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .measure(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;
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

#[derive(Default, Debug)]
struct CommonCtl(Vec<ElemId>, Vec<ElemId>);

impl CommonCtlOperation<SPro40Protocol> for CommonCtl {}

#[derive(Default)]
struct SPro40Tcd22xxCtl(Tcd22xxCtl);

impl Tcd22xxCtlOperation<SPro40Protocol> for SPro40Tcd22xxCtl {
    fn tcd22xx_ctl(&self) -> &Tcd22xxCtl {
        &self.0
    }

    fn tcd22xx_ctl_mut(&mut self) -> &mut Tcd22xxCtl {
        &mut self.0
    }
}

fn opt_out_iface_mode_to_string(mode: &OptOutIfaceMode) -> String {
    match mode {
        OptOutIfaceMode::Adat => "ADAT",
        OptOutIfaceMode::Spdif => "S/PDIF",
    }
    .to_string()
}

#[derive(Default, Debug)]
struct SpecificCtl;

impl SpecificCtl {
    const ANALOG_OUT_0_1_PAD_NAME: &'static str = "analog-output-1/2-pad";
    const OPT_OUT_IFACE_MODE_NAME: &'static str = "optical-output-interface-mode";

    const OPT_OUT_IFACE_MODES: [OptOutIfaceMode; 2] =
        [OptOutIfaceMode::Adat, OptOutIfaceMode::Spdif];

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::ANALOG_OUT_0_1_PAD_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::OPT_OUT_IFACE_MODES
            .iter()
            .map(|mode| opt_out_iface_mode_to_string(mode))
            .collect();
        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OPT_OUT_IFACE_MODE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::ANALOG_OUT_0_1_PAD_NAME => {
                let enabled = SPro40Protocol::read_analog_out_0_1_pad(
                    req,
                    &mut unit.1,
                    sections,
                    timeout_ms,
                )?;
                elem_value.set_bool(&[enabled]);
                Ok(true)
            }
            Self::OPT_OUT_IFACE_MODE_NAME => {
                let mode = SPro40Protocol::read_opt_out_iface_mode(
                    req,
                    &mut unit.1,
                    sections,
                    timeout_ms,
                )?;
                let pos = Self::OPT_OUT_IFACE_MODES
                    .iter()
                    .position(|m| m.eq(&mode))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::ANALOG_OUT_0_1_PAD_NAME => {
                let val = elem_value.boolean()[0];
                SPro40Protocol::write_analog_out_0_1_pad(
                    req,
                    &mut unit.1,
                    sections,
                    val,
                    timeout_ms,
                )
                .map(|_| true)
            }
            Self::OPT_OUT_IFACE_MODE_NAME => {
                let val = elem_value.enumerated()[0];
                let &mode = Self::OPT_OUT_IFACE_MODES
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg =
                            format!("Invalid index of optical output interface mode: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })?;
                SPro40Protocol::write_opt_out_iface_mode(
                    req,
                    &mut unit.1,
                    sections,
                    mode,
                    timeout_ms,
                )
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
