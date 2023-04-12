// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub(crate) use super::{command_dsp_runtime::*, protocols::version_3::*};

#[derive(Default, Debug)]
pub(crate) struct V3ClkCtl<T>
where
    T: MotuVersion3ClockSpecification
        + MotuWhollyCacheableParamsOperation<Version3ClockParameters>
        + MotuWhollyUpdatableParamsOperation<Version3ClockParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: Version3ClockParameters,
    _phantom: PhantomData<T>,
}

const RATE_NAME: &str = "sampling-rate";
const SRC_NAME: &str = "clock-source";

impl<T> V3ClkCtl<T>
where
    T: MotuVersion3ClockSpecification
        + MotuWhollyCacheableParamsOperation<Version3ClockParameters>
        + MotuWhollyUpdatableParamsOperation<Version3ClockParameters>,
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
        let labels: Vec<&str> = T::CLOCK_RATES.iter().map(|r| clk_rate_to_str(r)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = T::CLOCK_SRCS.iter().map(|s| clk_src_to_str(s)).collect();
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
                let pos = T::CLOCK_RATES
                    .iter()
                    .position(|r| self.params.rate.eq(r))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            SRC_NAME => {
                let pos = T::CLOCK_SRCS
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
                T::CLOCK_RATES
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
                T::CLOCK_SRCS
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
pub(crate) struct V3LcdClkCtl<T>
where
    T: MotuVersion3ClockSpecification
        + MotuWhollyCacheableParamsOperation<Version3ClockParameters>
        + MotuWhollyUpdatableParamsOperation<Version3ClockParameters>
        + MotuWhollyUpdatableParamsOperation<ClockNameDisplayParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: Version3ClockParameters,
    _phantom: PhantomData<T>,
}

fn clk_src_to_str(src: &V3ClkSrc) -> &'static str {
    match src {
        V3ClkSrc::Internal => "Internal",
        V3ClkSrc::SpdifCoax => "S/PDIF-on-coax",
        V3ClkSrc::WordClk => "Word-clk-on-BNC",
        V3ClkSrc::AesEbuXlr => "AES/EBU-on-XLR",
        V3ClkSrc::SignalOptA => "Signal-on-opt-A",
        V3ClkSrc::SignalOptB => "Signal-on-opt-B",
    }
}

impl<T> V3LcdClkCtl<T>
where
    T: MotuVersion3ClockSpecification
        + MotuWhollyCacheableParamsOperation<Version3ClockParameters>
        + MotuWhollyUpdatableParamsOperation<Version3ClockParameters>
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
        let labels: Vec<&str> = T::CLOCK_RATES.iter().map(|r| clk_rate_to_str(r)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = T::CLOCK_SRCS.iter().map(|s| clk_src_to_str(s)).collect();
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
                let pos = T::CLOCK_RATES
                    .iter()
                    .position(|r| self.params.rate.eq(r))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            SRC_NAME => {
                let pos = T::CLOCK_SRCS
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
                T::CLOCK_RATES
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
                T::CLOCK_SRCS
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
pub(crate) struct V3PortAssignCtl<T>
where
    T: MotuPortAssignSpecification
        + MotuWhollyCacheableParamsOperation<V3PortAssignParameters>
        + MotuWhollyUpdatableParamsOperation<V3PortAssignParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: V3PortAssignParameters,
    _phantom: PhantomData<T>,
}

const MAIN_ASSIGN_NAME: &str = "main-assign";
const RETURN_ASSIGN_NAME: &str = "return-assign";

impl<T> V3PortAssignCtl<T>
where
    T: MotuPortAssignSpecification
        + MotuWhollyCacheableParamsOperation<V3PortAssignParameters>
        + MotuWhollyUpdatableParamsOperation<V3PortAssignParameters>,
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
        let labels: Vec<String> = T::ASSIGN_PORT_TARGETS
            .iter()
            .map(|p| target_port_to_string(p))
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MAIN_ASSIGN_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, RETURN_ASSIGN_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.extend_from_slice(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MAIN_ASSIGN_NAME => {
                let pos = T::ASSIGN_PORT_TARGETS
                    .iter()
                    .position(|p| self.params.main.eq(p))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            RETURN_ASSIGN_NAME => {
                let pos = T::ASSIGN_PORT_TARGETS
                    .iter()
                    .position(|p| self.params.mixer_return.eq(p))
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
            MAIN_ASSIGN_NAME => {
                let mut params = self.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                T::ASSIGN_PORT_TARGETS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid argument for main assignment: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&p| params.main = p)?;
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            RETURN_ASSIGN_NAME => {
                let mut params = self.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                T::ASSIGN_PORT_TARGETS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid argument for main assignment: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&p| params.mixer_return = p)?;
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Debug)]
pub(crate) struct V3OptIfaceCtl<T>
where
    T: MotuVersion3OpticalIfaceSpecification
        + MotuWhollyCacheableParamsOperation<V3OpticalIfaceParameters>
        + MotuWhollyUpdatableParamsOperation<V3OpticalIfaceParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: V3OpticalIfaceParameters,
    _phantom: PhantomData<T>,
}

fn opt_iface_mode_to_str(mode: &V3OptIfaceMode) -> &'static str {
    match mode {
        V3OptIfaceMode::Disabled => "Disabled",
        V3OptIfaceMode::Adat => "ADAT",
        V3OptIfaceMode::Spdif => "S/PDIF",
    }
}

const OPT_IFACE_IN_MODE_NAME: &str = "optical-iface-in-mode";
const OPT_IFACE_OUT_MODE_NAME: &str = "optical-iface-out-mode";

const OPT_IFACE_MODES: &[V3OptIfaceMode; 3] = &[
    V3OptIfaceMode::Disabled,
    V3OptIfaceMode::Adat,
    V3OptIfaceMode::Spdif,
];

impl<T> Default for V3OptIfaceCtl<T>
where
    T: MotuVersion3OpticalIfaceSpecification
        + MotuWhollyCacheableParamsOperation<V3OpticalIfaceParameters>
        + MotuWhollyUpdatableParamsOperation<V3OpticalIfaceParameters>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_optical_iface_parameters(),
            _phantom: Default::default(),
        }
    }
}

impl<T> V3OptIfaceCtl<T>
where
    T: MotuVersion3OpticalIfaceSpecification
        + MotuWhollyCacheableParamsOperation<V3OpticalIfaceParameters>
        + MotuWhollyUpdatableParamsOperation<V3OpticalIfaceParameters>,
{
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::cache_wholly(req, node, &mut self.params, timeout_ms);
        debug!(params = ?self.params, ?res);
        Ok(())
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = OPT_IFACE_MODES
            .iter()
            .map(|m| opt_iface_mode_to_str(m))
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OPT_IFACE_IN_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, T::OPT_IFACE_COUNT, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OPT_IFACE_OUT_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, T::OPT_IFACE_COUNT, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OPT_IFACE_IN_MODE_NAME => {
                let vals: Vec<u32> = self
                    .params
                    .input_modes
                    .iter()
                    .map(|mode| {
                        let pos = OPT_IFACE_MODES.iter().position(|m| mode.eq(m)).unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            OPT_IFACE_OUT_MODE_NAME => {
                let vals: Vec<u32> = self
                    .params
                    .output_modes
                    .iter()
                    .map(|mode| {
                        let pos = OPT_IFACE_MODES.iter().position(|m| mode.eq(m)).unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
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
            OPT_IFACE_IN_MODE_NAME => {
                let mut params = self.params.clone();
                params
                    .input_modes
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(mode, &val)| {
                        let pos = val as usize;
                        OPT_IFACE_MODES
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg =
                                    format!("Invalid index for mode of opt interface: {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&m| *mode = m)
                    })?;
                unit.lock()?;
                let res =
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.params = params);
                let _ = unit.unlock();
                debug!(params = ?self.params, ?res);
                res.map(|_| true)
            }
            OPT_IFACE_OUT_MODE_NAME => {
                let mut params = self.params.clone();
                params
                    .output_modes
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(mode, &val)| {
                        let pos = val as usize;
                        OPT_IFACE_MODES
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg =
                                    format!("Invalid index for mode of opt interface: {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&m| *mode = m)
                    })?;
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
