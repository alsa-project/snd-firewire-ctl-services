// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{common_ctl::*, tcd22xx_ctl::*, *},
    dice_protocols::{
        maudio::*,
        tcat::{extension::*, global_section::*, *},
    },
};

const TIMEOUT_MS: u32 = 20;

#[derive(Default)]
pub struct Pfire2626Model {
    req: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    ctl: CommonCtl,
    tcd22xx_ctl: Pfire2626Tcd22xxCtl,
    specific_ctl: Pfire2626SpecificCtl,
}

#[derive(Default)]
struct Pfire2626SpecificCtl([bool; Pfire2626Protocol::KNOB_COUNT]);

impl SpecificCtlOperation<Pfire2626Protocol> for Pfire2626SpecificCtl {
    fn state(&self) -> &[bool] {
        &self.0
    }

    fn state_mut(&mut self) -> &mut [bool] {
        &mut self.0
    }
}

impl CtlModel<(SndDice, FwNode)> for Pfire2626Model {
    fn load(
        &mut self,
        unit: &mut (SndDice, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.sections =
            GeneralProtocol::read_general_sections(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        let caps = ClockCaps::new(
            &Pfire2626Protocol::AVAIL_CLK_RATES,
            Pfire2626Protocol::AVAIL_CLK_SRCS,
        );
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
        self.specific_ctl.load(card_cntr)?;

        self.tcd22xx_ctl.cache(
            unit,
            &mut self.req,
            &self.sections,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;

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
        } else if self.specific_ctl.write(
            unit,
            &mut self.req,
            &self.extension_sections,
            elem_id,
            old,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for Pfire2626Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        self.tcd22xx_ctl.get_notified_elem_list(elem_id_list);
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
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndDice, FwNode)> for Pfire2626Model {
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
struct Pfire2626Tcd22xxCtl(Tcd22xxCtl);

impl Tcd22xxCtlOperation<Pfire2626Protocol> for Pfire2626Tcd22xxCtl {
    fn tcd22xx_ctl(&self) -> &Tcd22xxCtl {
        &self.0
    }

    fn tcd22xx_ctl_mut(&mut self) -> &mut Tcd22xxCtl {
        &mut self.0
    }
}

#[derive(Default)]
pub struct Pfire610Model {
    req: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    ctl: CommonCtl,
    tcd22xx_ctl: Pfire610Tcd22xxCtl,
    specific_ctl: Pfire610SpecificCtl,
}

#[derive(Default)]
struct Pfire610SpecificCtl([bool; Pfire610Protocol::KNOB_COUNT]);

impl SpecificCtlOperation<Pfire610Protocol> for Pfire610SpecificCtl {
    fn state(&self) -> &[bool] {
        &self.0
    }

    fn state_mut(&mut self) -> &mut [bool] {
        &mut self.0
    }
}

impl CtlModel<(SndDice, FwNode)> for Pfire610Model {
    fn load(
        &mut self,
        unit: &mut (SndDice, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.sections =
            GeneralProtocol::read_general_sections(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        let caps = ClockCaps::new(
            &Pfire610Protocol::AVAIL_CLK_RATES,
            Pfire610Protocol::AVAIL_CLK_SRCS,
        );
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
        self.specific_ctl.load(card_cntr)?;

        self.tcd22xx_ctl.cache(
            unit,
            &mut self.req,
            &self.sections,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;

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
        } else if self.specific_ctl.write(
            unit,
            &mut self.req,
            &self.extension_sections,
            elem_id,
            old,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for Pfire610Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        self.tcd22xx_ctl.get_notified_elem_list(elem_id_list);
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
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndDice, FwNode)> for Pfire610Model {
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
struct Pfire610Tcd22xxCtl(Tcd22xxCtl);

impl Tcd22xxCtlOperation<Pfire610Protocol> for Pfire610Tcd22xxCtl {
    fn tcd22xx_ctl(&self) -> &Tcd22xxCtl {
        &self.0
    }

    fn tcd22xx_ctl_mut(&mut self) -> &mut Tcd22xxCtl {
        &mut self.0
    }
}

fn opt_iface_b_mode_to_str(mode: &OptIfaceMode) -> &'static str {
    match mode {
        OptIfaceMode::Spdif => "SPDIF",
        OptIfaceMode::Adat => "ADAT",
    }
}

fn standalone_converter_mode_to_str(mode: &StandaloneConverterMode) -> &'static str {
    match mode {
        StandaloneConverterMode::AdDa => "A/D-D/A",
        StandaloneConverterMode::AdOnly => "A/D-only",
    }
}

const MASTER_KNOB_NAME: &str = "master-knob-target";
const OPT_IFACE_B_MODE_NAME: &str = "optical-iface-b-mode";
const STANDALONE_CONVERTER_MODE_NAME: &str = "standalone-converter-mode";

trait SpecificCtlOperation<T: PfireSpecificOperation> {
    fn state(&self) -> &[bool];
    fn state_mut(&mut self) -> &mut [bool];

    // MEMO: Both models support 'Output{id: DstBlkId::Ins0, count: 8}'.
    const MASTER_KNOB_TARGET_LABELS: [&'static str; 4] = [
        "analog-out-1/2",
        "analog-out-3/4",
        "analog-out-5/6",
        "analog-out-7/8",
    ];
    const OPT_IFACE_B_MODES: [OptIfaceMode; 2] = [OptIfaceMode::Adat, OptIfaceMode::Spdif];
    const STANDALONE_CONVERTER_MODES: [StandaloneConverterMode; 2] = [
        StandaloneConverterMode::AdDa,
        StandaloneConverterMode::AdOnly,
    ];

    fn load(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MASTER_KNOB_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, T::KNOB_COUNT, true)?;

        // NOTE: ClockSource::Tdif is used for second optical interface as 'ADAT_AUX'.
        if T::HAS_OPT_IFACE_B {
            let labels: Vec<&str> = Self::OPT_IFACE_B_MODES
                .iter()
                .map(|m| opt_iface_b_mode_to_str(m))
                .collect();
            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OPT_IFACE_B_MODE_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        }

        if T::SUPPORT_STANDALONE_CONVERTER {
            let labels: Vec<&str> = Self::STANDALONE_CONVERTER_MODES
                .iter()
                .map(|m| standalone_converter_mode_to_str(m))
                .collect();
            let elem_id =
                ElemId::new_by_name(ElemIfaceType::Card, 0, 0, STANDALONE_CONVERTER_MODE_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        }

        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MASTER_KNOB_NAME => {
                T::read_knob_assign(
                    req,
                    &mut unit.1,
                    sections,
                    &mut self.state_mut(),
                    timeout_ms,
                )?;
                elem_value.set_bool(&self.state());
                Ok(true)
            }
            OPT_IFACE_B_MODE_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let mode = T::read_opt_iface_b_mode(req, &mut unit.1, sections, timeout_ms)?;
                let pos = Self::OPT_IFACE_B_MODES
                    .iter()
                    .position(|m| mode.eq(m))
                    .unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            STANDALONE_CONVERTER_MODE_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let mode =
                    T::read_standalone_converter_mode(req, &mut unit.1, sections, timeout_ms)?;
                let pos = Self::STANDALONE_CONVERTER_MODES
                    .iter()
                    .position(|m| mode.eq(m))
                    .unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MASTER_KNOB_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, T::KNOB_COUNT, |idx, val| {
                    self.state_mut()[idx] = val;
                    Ok(())
                })?;
                T::write_knob_assign(req, &mut unit.1, sections, &self.state(), timeout_ms)?;
                Ok(true)
            }
            OPT_IFACE_B_MODE_NAME => ElemValueAccessor::<u32>::get_val(new, |val| {
                let &mode = Self::OPT_IFACE_B_MODES
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg =
                            format!("Invalid value for index of optical interface mode: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })?;
                T::write_opt_iface_b_mode(req, &mut unit.1, sections, mode, timeout_ms)
            })
            .map(|_| true),
            STANDALONE_CONVERTER_MODE_NAME => ElemValueAccessor::<u32>::get_val(new, |val| {
                let &mode = Self::STANDALONE_CONVERTER_MODES
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg = format!(
                            "Invalid value for index of standalone converter mode: {}",
                            val
                        );
                        Error::new(FileError::Inval, &msg)
                    })?;
                T::write_standalone_converter_mode(req, &mut unit.1, sections, mode, timeout_ms)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }
}
