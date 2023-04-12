// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub(crate) use super::{protocols::version_1::*, v1_runtime::*};

#[derive(Default, Debug)]
pub(crate) struct V1ClkCtl<T>
where
    T: MotuVersion1ClockSpecification
        + MotuWhollyCacheableParamsOperation<Version1ClockParameters>
        + MotuWhollyUpdatableParamsOperation<Version1ClockParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: Version1ClockParameters,
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

impl<T> V1ClkCtl<T>
where
    T: MotuVersion1ClockSpecification
        + MotuWhollyCacheableParamsOperation<Version1ClockParameters>
        + MotuWhollyUpdatableParamsOperation<Version1ClockParameters>,
{
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::cache_wholly(req, node, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = T::CLK_RATES.iter().map(|l| clk_rate_to_str(l)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = T::CLK_SRCS.iter().map(|l| clk_src_to_str(l)).collect();
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
                let pos = T::CLK_RATES
                    .iter()
                    .position(|r| self.params.rate.eq(r))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            SRC_NAME => {
                let pos = T::CLK_SRCS
                    .iter()
                    .position(|s| self.params.source.eq(s))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
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
                let mut params = self.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                T::CLK_RATES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index for rate of media clock: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&r| params.rate = r)?;
                unit.lock()?;
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                unit.unlock()?;
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            SRC_NAME => {
                let mut params = self.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                T::CLK_SRCS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index for source of sampling clock: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&s| params.source = s)?;
                unit.lock()?;
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                unit.unlock()?;
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct V1MonitorInputCtl<T>
where
    T: MotuVersion1MonitorInputSpecification
        + MotuWhollyCacheableParamsOperation<Version1MonitorInputParameters>
        + MotuWhollyUpdatableParamsOperation<Version1MonitorInputParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: Version1MonitorInputParameters,
    _phantom: PhantomData<T>,
}

const MONITOR_INPUT_NAME: &str = "monitor-input";

impl<T> V1MonitorInputCtl<T>
where
    T: MotuVersion1MonitorInputSpecification
        + MotuWhollyCacheableParamsOperation<Version1MonitorInputParameters>
        + MotuWhollyUpdatableParamsOperation<Version1MonitorInputParameters>,
{
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::cache_wholly(req, node, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = T::MONITOR_INPUT_MODES
            .iter()
            .map(|e| target_port_to_string(e))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_INPUT_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MONITOR_INPUT_NAME => {
                let pos = T::MONITOR_INPUT_MODES
                    .iter()
                    .position(|m| self.params.0.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MONITOR_INPUT_NAME => {
                let mut params = self.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                T::MONITOR_INPUT_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of monitor input mode: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&mode| params.0 = mode)?;
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
