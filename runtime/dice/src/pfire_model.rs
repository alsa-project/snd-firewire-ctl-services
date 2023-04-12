// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{tcd22xx_ctl::*, *},
    protocols::{
        maudio::*,
        tcat::{extension::*, tcd22xx_spec::*},
    },
    std::marker::PhantomData,
};

const TIMEOUT_MS: u32 = 20;

pub type Pfire2626Model = PfireModel<Pfire2626Protocol>;
pub type Pfire610Model = PfireModel<Pfire610Protocol>;

#[derive(Default)]
pub struct PfireModel<T>
where
    T: Tcd22xxSpecification
        + Tcd22xxOperation
        + PfireSpecificSpecification
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>
        + TcatExtensionOperation
        + TcatExtensionSectionParamsOperation<PfireSpecificParams>
        + TcatExtensionSectionPartialMutableParamsOperation<PfireSpecificParams>,
{
    req: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    common_ctl: CommonCtl<T>,
    tcd22xx_ctls: Tcd22xxCtls<T>,
    specific_ctl: PfireSpecificCtl<T>,
}

impl<T> PfireModel<T>
where
    T: Tcd22xxSpecification
        + Tcd22xxOperation
        + PfireSpecificSpecification
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>
        + TcatExtensionOperation
        + TcatExtensionSectionParamsOperation<PfireSpecificParams>
        + TcatExtensionSectionPartialMutableParamsOperation<PfireSpecificParams>,
{
    pub(crate) fn store_configuration(&mut self, node: &FwNode) -> Result<(), Error> {
        self.tcd22xx_ctls.store_configuration(
            &mut self.req,
            node,
            &self.extension_sections,
            TIMEOUT_MS,
        )
    }
}

impl<T> CtlModel<(SndDice, FwNode)> for PfireModel<T>
where
    T: Tcd22xxSpecification
        + Tcd22xxOperation
        + PfireSpecificSpecification
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>
        + TcatExtensionOperation
        + TcatExtensionSectionParamsOperation<PfireSpecificParams>
        + TcatExtensionSectionPartialMutableParamsOperation<PfireSpecificParams>,
{
    fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        T::read_general_sections(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .cache_whole_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        T::read_extension_sections(&self.req, &unit.1, &mut self.extension_sections, TIMEOUT_MS)?;

        self.tcd22xx_ctls.cache_whole_params(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            &self.common_ctl.global_params,
            TIMEOUT_MS,
        )?;

        self.specific_ctl.cache(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            &self.tcd22xx_ctls.caps,
            TIMEOUT_MS,
        )?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr)?;

        self.tcd22xx_ctls.load(card_cntr)?;

        self.specific_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.specific_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        _: &ElemValue,
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
        } else if self.specific_ctl.write(
            &mut self.req,
            &mut unit.1,
            &self.extension_sections,
            &self.tcd22xx_ctls.caps,
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

impl<T> NotifyModel<(SndDice, FwNode), u32> for PfireModel<T>
where
    T: Tcd22xxSpecification
        + Tcd22xxOperation
        + PfireSpecificSpecification
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>
        + TcatExtensionOperation
        + TcatExtensionSectionParamsOperation<PfireSpecificParams>
        + TcatExtensionSectionPartialMutableParamsOperation<PfireSpecificParams>,
{
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_id_list);
        elem_id_list.extend_from_slice(&self.tcd22xx_ctls.notified_elem_id_list);
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
            &self.common_ctl.global_params,
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
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<T> MeasureModel<(SndDice, FwNode)> for PfireModel<T>
where
    T: Tcd22xxSpecification
        + Tcd22xxOperation
        + PfireSpecificSpecification
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>
        + TcatExtensionOperation
        + TcatExtensionSectionParamsOperation<PfireSpecificParams>
        + TcatExtensionSectionPartialMutableParamsOperation<PfireSpecificParams>,
{
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.measured_elem_id_list);
        elem_id_list.extend_from_slice(&self.tcd22xx_ctls.measured_elem_id_list);
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
        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Debug)]
pub struct PfireSpecificCtl<T: PfireSpecificSpecification>(PfireSpecificParams, PhantomData<T>)
where
    T: PfireSpecificSpecification
        + TcatExtensionSectionParamsOperation<PfireSpecificParams>
        + TcatExtensionSectionPartialMutableParamsOperation<PfireSpecificParams>;

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

impl<T> PfireSpecificCtl<T>
where
    T: PfireSpecificSpecification
        + TcatExtensionSectionParamsOperation<PfireSpecificParams>
        + TcatExtensionSectionPartialMutableParamsOperation<PfireSpecificParams>,
{
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

    fn cache(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res =
            T::cache_extension_whole_params(req, node, &sections, caps, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MASTER_KNOB_NAME, 0);
        let _ =
            card_cntr.add_bool_elems(&elem_id, 1, Self::MASTER_KNOB_TARGET_LABELS.len(), true)?;

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

    fn read(&mut self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MASTER_KNOB_NAME => {
                elem_value.set_bool(&self.0.knob_assigns);
                Ok(true)
            }
            OPT_IFACE_B_MODE_NAME => {
                let pos = Self::OPT_IFACE_B_MODES
                    .iter()
                    .position(|m| self.0.opt_iface_b_mode.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            STANDALONE_CONVERTER_MODE_NAME => {
                let pos = Self::STANDALONE_CONVERTER_MODES
                    .iter()
                    .position(|m| self.0.standalone_mode.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MASTER_KNOB_NAME => {
                let mut params = self.0.clone();
                params
                    .knob_assigns
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(assign, val)| *assign = val);
                let res = T::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            OPT_IFACE_B_MODE_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::OPT_IFACE_B_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg =
                            format!("Invalid value for index of optical interface mode: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&mode| params.opt_iface_b_mode = mode)?;
                let res = T::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            STANDALONE_CONVERTER_MODE_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::STANDALONE_CONVERTER_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!(
                            "Invalid value for index of standalone converter mode: {}",
                            pos
                        );
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&mode| params.standalone_mode = mode)?;
                let res = T::update_extension_partial_params(
                    req,
                    node,
                    sections,
                    caps,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
