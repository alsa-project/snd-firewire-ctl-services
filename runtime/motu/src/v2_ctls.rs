// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub(crate) use super::{protocols::version_2::*, register_dsp_runtime::*};

#[derive(Default, Debug)]
pub(crate) struct V2ClkCtl<T>
where
    T: MotuVersion2ClockSpecification
        + MotuWhollyCacheableParamsOperation<Version2ClockParameters>
        + MotuWhollyUpdatableParamsOperation<Version2ClockParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: Version2ClockParameters,
    _phantom: PhantomData<T>,
}

const RATE_NAME: &str = "sampling- rate";
const SRC_NAME: &str = "clock-source";

impl<T> V2ClkCtl<T>
where
    T: MotuVersion2ClockSpecification
        + MotuWhollyCacheableParamsOperation<Version2ClockParameters>
        + MotuWhollyUpdatableParamsOperation<Version2ClockParameters>,
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
        let labels: Vec<&str> = T::CLK_RATES.iter().map(|r| clk_rate_to_str(r)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = T::CLK_SRCS.iter().map(|s| clk_src_to_str(s)).collect();
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
                        let msg = format!("Invalid argument for rate of media clock: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&r| params.rate = r)?;
                unit.lock()?;
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                let _ = unit.unlock();
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
                        let msg = format!("Invalid argument for source of sampling clock: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&s| params.source = s)?;
                unit.lock()?;
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                let _ = unit.unlock();
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct V2LcdClkCtl<T>
where
    T: MotuVersion2ClockSpecification
        + MotuWhollyCacheableParamsOperation<Version2ClockParameters>
        + MotuWhollyUpdatableParamsOperation<Version2ClockParameters>
        + MotuWhollyUpdatableParamsOperation<ClockNameDisplayParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: Version2ClockParameters,
    _phantom: PhantomData<T>,
}

fn clk_src_to_str(src: &V2ClkSrc) -> &'static str {
    match src {
        V2ClkSrc::Internal => "Internal",
        V2ClkSrc::SpdifCoax => "S/PDIF-on-coax",
        V2ClkSrc::WordClk => "Word-on-BNC",
        V2ClkSrc::SignalOpt => "Signal-on-opt",
        V2ClkSrc::AdatOpt => "Adat-on-opt",
        V2ClkSrc::AdatDsub => "Adat-on-Dsub",
        V2ClkSrc::AesebuXlr => "AES/EBU-on-XLR",
    }
}

impl<T> V2LcdClkCtl<T>
where
    T: MotuVersion2ClockSpecification
        + MotuWhollyCacheableParamsOperation<Version2ClockParameters>
        + MotuWhollyUpdatableParamsOperation<Version2ClockParameters>
        + MotuWhollyUpdatableParamsOperation<ClockNameDisplayParameters>,
{
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::cache_wholly(req, node, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        res?;

        let label = clk_src_to_str(&self.params.source);
        let params = ClockNameDisplayParameters(label.to_string());
        let res = T::update_wholly(req, node, &params, timeout_ms);
        debug!(?params, ?res);
        res
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = T::CLK_RATES.iter().map(|r| clk_rate_to_str(r)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = T::CLK_SRCS.iter().map(|s| clk_src_to_str(s)).collect();
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
                        let msg = format!("Invalid argument for rate of media clock: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&r| params.rate = r)?;
                unit.lock()?;
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                let _ = unit.unlock();
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
                        let msg = format!("Invalid argument for source of sampling clock: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&s| params.source = s)?;
                unit.lock()?;
                let res = T::update_wholly(req, node, &params, timeout_ms).and_then(|_| {
                    let label = clk_src_to_str(&params.source);
                    let p = ClockNameDisplayParameters(label.to_string());
                    T::update_wholly(req, node, &p, timeout_ms)
                        .map(|_| self.params = params)
                        .or_else(|_| T::update_wholly(req, node, &self.params, timeout_ms))
                });
                let _ = unit.unlock();
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct V2OptIfaceCtl<T>
where
    T: MotuVersion2OpticalIfaceSpecification
        + MotuWhollyCacheableParamsOperation<Version2OpticalIfaceParameters>
        + MotuWhollyUpdatableParamsOperation<Version2OpticalIfaceParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: Version2OpticalIfaceParameters,
    _phantom: PhantomData<T>,
}

fn opt_iface_mode_to_str(mode: &V2OptIfaceMode) -> &'static str {
    match mode {
        V2OptIfaceMode::None => "None",
        V2OptIfaceMode::Adat => "ADAT",
        V2OptIfaceMode::Spdif => "S/PDIF",
    }
}

const OPT_IN_IFACE_MODE_NAME: &str = "optical-iface-in-mode";
const OPT_OUT_IFACE_MODE_NAME: &str = "optical-iface-out-mode";

impl<T> V2OptIfaceCtl<T>
where
    T: MotuVersion2OpticalIfaceSpecification
        + MotuWhollyCacheableParamsOperation<Version2OpticalIfaceParameters>
        + MotuWhollyUpdatableParamsOperation<Version2OpticalIfaceParameters>,
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
        let labels: Vec<&str> = T::OPT_IFACE_MODES
            .iter()
            .map(|m| opt_iface_mode_to_str(m))
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OPT_IN_IFACE_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OPT_OUT_IFACE_MODE_NAME, 0);
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
            OPT_IN_IFACE_MODE_NAME => {
                let pos = T::OPT_IFACE_MODES
                    .iter()
                    .position(|m| self.params.input_mode.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            OPT_OUT_IFACE_MODE_NAME => {
                let pos = T::OPT_IFACE_MODES
                    .iter()
                    .position(|m| self.params.output_mode.eq(m))
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
            OPT_IN_IFACE_MODE_NAME => {
                let mut params = self.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                T::OPT_IFACE_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg =
                            format!("Invalid argument for optical input interface mode: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&mode| params.input_mode = mode)?;
                unit.lock()?;
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                let _ = unit.unlock();
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            OPT_OUT_IFACE_MODE_NAME => {
                let mut params = self.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                T::OPT_IFACE_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg =
                            format!("Invalid argument for optical input interface mode: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&mode| params.input_mode = mode)?;
                unit.lock()?;
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                let _ = unit.unlock();
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
