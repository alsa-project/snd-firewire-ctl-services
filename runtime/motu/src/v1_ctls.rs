// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub(crate) use super::{protocols::version_1::*, v1_runtime::*};

#[derive(Default, Debug)]
pub(crate) struct V1ClkCtl<T: V1ClkOperation> {
    pub elem_id_list: Vec<ElemId>,
    rate: usize,
    source: usize,
    _phantom: PhantomData<T>,
}

fn clk_src_to_str(src: &V1ClkSrc) -> &'static str {
    match src {
        V1ClkSrc::Internal => "Internal",
        V1ClkSrc::Spdif => "S/PDIF",
        V1ClkSrc::WordClk => "Word-on-BNC",
        V1ClkSrc::AdatOpt => "Adat-on-opt",
        V1ClkSrc::AdatDsub => "Adat-on-Dsub",
        V1ClkSrc::AesebuXlr => "AES/EBU-on-XLR",
    }
}

const RATE_NAME: &str = "sampling- rate";
const SRC_NAME: &str = "clock-source";

impl<T: V1ClkOperation> V1ClkCtl<T> {
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::get_clk_rate(req, node, timeout_ms).map(|idx| self.rate = idx)?;
        T::get_clk_src(req, node, timeout_ms).map(|idx| self.source = idx)?;
        Ok(())
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = T::CLK_RATE_LABELS
            .iter()
            .map(|l| clk_rate_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = T::CLK_SRC_LABELS
            .iter()
            .map(|l| clk_src_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SRC_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            RATE_NAME => {
                elem_value.set_enum(&[self.rate as u32]);
                Ok(true)
            }
            SRC_NAME => {
                elem_value.set_enum(&[self.source as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        unit: &mut SndMotu,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            RATE_NAME => {
                let val = elem_value.enumerated()[0] as usize;
                unit.lock()?;
                let res = T::set_clk_rate(req, node, val, timeout_ms).map(|_| self.rate = val);
                unit.unlock()?;
                res.map(|_| true)
            }
            SRC_NAME => {
                let val = elem_value.enumerated()[0] as usize;
                unit.lock()?;
                let res = T::set_clk_src(req, node, val, timeout_ms).map(|_| self.source = val);
                unit.unlock()?;
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const MONITOR_INPUT_NAME: &str = "monitor-input";

pub trait V1MonitorInputCtlOperation<T: V1MonitorInputOperation> {
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = T::MONITOR_INPUT_MODES
            .iter()
            .map(|e| target_port_to_string(e))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_INPUT_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MONITOR_INPUT_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                T::get_monitor_input(req, &mut unit.1, timeout_ms).map(|idx| idx as u32)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MONITOR_INPUT_NAME => ElemValueAccessor::<u32>::get_val(new, |val| {
                T::set_monitor_input(req, &mut unit.1, val as usize, timeout_ms)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }
}
