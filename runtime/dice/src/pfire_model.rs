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
    T: Tcd22xxSpecOperation
        + Tcd22xxRouterOperation
        + Tcd22xxMixerOperation
        + PfireSpecificOperation
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>,
{
    req: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    common_ctl: PfireCommonCtl<T>,
    tcd22xx_ctl: PfireTcd22xxCtl<T>,
    specific_ctl: PfireSpecificCtl<T>,
}

impl<T> PfireModel<T>
where
    T: Tcd22xxSpecOperation
        + Tcd22xxRouterOperation
        + Tcd22xxMixerOperation
        + PfireSpecificOperation
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>,
{
    pub fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        T::read_general_sections(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .whole_cache(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        Ok(())
    }
}

impl<T> CtlModel<(SndDice, FwNode)> for PfireModel<T>
where
    T: Tcd22xxSpecOperation
        + Tcd22xxRouterOperation
        + Tcd22xxMixerOperation
        + PfireSpecificOperation
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>,
{
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

impl<T> NotifyModel<(SndDice, FwNode), u32> for PfireModel<T>
where
    T: Tcd22xxSpecOperation
        + Tcd22xxRouterOperation
        + Tcd22xxMixerOperation
        + PfireSpecificOperation
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>,
{
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

impl<T> MeasureModel<(SndDice, FwNode)> for PfireModel<T>
where
    T: Tcd22xxSpecOperation
        + Tcd22xxRouterOperation
        + Tcd22xxMixerOperation
        + PfireSpecificOperation
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>,
{
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
pub struct PfireCommonCtl<T>(Vec<ElemId>, Vec<ElemId>, PhantomData<T>)
where
    T: PfireSpecificOperation // to avoid implementation candidates.
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>;

impl<T> CommonCtlOperation<T> for PfireCommonCtl<T> where
    T: PfireSpecificOperation // to avoid implementation candidates.
        + TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>
{
}

#[derive(Debug)]
pub struct PfireSpecificCtl<T: PfireSpecificOperation>(Vec<bool>, PhantomData<T>);

impl<T: PfireSpecificOperation> Default for PfireSpecificCtl<T> {
    fn default() -> Self {
        Self(vec![Default::default(); T::KNOB_COUNT], Default::default())
    }
}

impl<T: PfireSpecificOperation> SpecificCtlOperation<T> for PfireSpecificCtl<T> {
    fn state(&self) -> &[bool] {
        &self.0
    }

    fn state_mut(&mut self) -> &mut [bool] {
        &mut self.0
    }
}

#[derive(Default, Debug)]
pub struct PfireTcd22xxCtl<T>(Tcd22xxCtl, PhantomData<T>)
where
    T: Tcd22xxSpecOperation
        + Tcd22xxRouterOperation
        + Tcd22xxMixerOperation
        + PfireSpecificOperation; // to avoid implementation candidates.

impl<T> Tcd22xxCtlOperation<T> for PfireTcd22xxCtl<T>
where
    T: Tcd22xxSpecOperation
        + Tcd22xxRouterOperation
        + Tcd22xxMixerOperation
        + PfireSpecificOperation, // to avoid implementation candidates.
{
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

pub trait SpecificCtlOperation<T: PfireSpecificOperation> {
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
