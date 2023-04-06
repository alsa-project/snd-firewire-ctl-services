// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub(crate) use super::{command_dsp_runtime::*, protocols::version_3::*};

#[derive(Default, Debug)]
pub(crate) struct V3ClkCtl<T: V3ClkOperation> {
    pub elem_id_list: Vec<ElemId>,
    rate: usize,
    source: usize,
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

const RATE_NAME: &str = "sampling-rate";
const SRC_NAME: &str = "clock-source";

impl<T: V3ClkOperation> V3ClkCtl<T> {
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::get_clk_rate(req, node, timeout_ms).map(|idx| self.rate = idx)?;
        T::get_clk_src(req, node, timeout_ms).and_then(|idx| {
            if T::HAS_LCD {
                let label = clk_src_to_str(&T::CLK_SRCS[idx].0);
                T::update_clk_display(req, node, &label, timeout_ms)?;
            }
            self.source = idx;
            Ok(())
        })?;
        Ok(())
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = T::CLK_RATES.iter().map(|e| clk_rate_to_str(&e.0)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = T::CLK_SRCS.iter().map(|e| clk_src_to_str(&e.0)).collect();
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
                let _ = unit.unlock();
                res.map(|_| true)
            }
            SRC_NAME => {
                let val = elem_value.enumerated()[0] as usize;
                unit.lock()?;
                let res = T::set_clk_src(req, node, val, timeout_ms).and_then(|_| {
                    if T::HAS_LCD {
                        let label = clk_src_to_str(&T::CLK_SRCS[val].0);
                        if T::update_clk_display(req, node, &label, timeout_ms).is_err() {
                            let _ = T::set_clk_src(req, node, self.source, timeout_ms);
                        }
                    }
                    self.source = val;
                    Ok(())
                });
                let _ = unit.unlock();
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct V3PortAssignCtl<T: V3PortAssignOperation> {
    pub elem_id_list: Vec<ElemId>,
    main: usize,
    mixer_return: usize,
    _phantom: PhantomData<T>,
}

const MAIN_ASSIGN_NAME: &str = "main-assign";
const RETURN_ASSIGN_NAME: &str = "return-assign";

impl<T: V3PortAssignOperation> V3PortAssignCtl<T> {
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::get_main_assign(req, node, timeout_ms).map(|idx| self.main = idx)?;
        T::get_return_assign(req, node, timeout_ms).map(|idx| self.mixer_return = idx)?;
        Ok(())
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = T::ASSIGN_PORTS
            .iter()
            .map(|p| target_port_to_string(&p.0))
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
                elem_value.set_enum(&[self.main as u32]);
                Ok(true)
            }
            RETURN_ASSIGN_NAME => {
                elem_value.set_enum(&[self.mixer_return as u32]);
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
                let val = elem_value.enumerated()[0] as usize;
                T::set_main_assign(req, node, val, timeout_ms).map(|_| self.main = val)?;
                Ok(true)
            }
            RETURN_ASSIGN_NAME => {
                let val = elem_value.enumerated()[0] as usize;
                T::set_return_assign(req, node, val, timeout_ms)
                    .map(|_| self.mixer_return = val)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Debug)]
pub(crate) struct V3OptIfaceCtl<T: V3OptIfaceOperation> {
    pub elem_id_list: Vec<ElemId>,
    inputs: Vec<V3OptIfaceMode>,
    outputs: Vec<V3OptIfaceMode>,
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

impl<T: V3OptIfaceOperation> Default for V3OptIfaceCtl<T> {
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            inputs: vec![Default::default(); T::TARGETS.len()],
            outputs: vec![Default::default(); T::TARGETS.len()],
            _phantom: Default::default(),
        }
    }
}

impl<T: V3OptIfaceOperation> V3OptIfaceCtl<T> {
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.inputs
            .iter_mut()
            .zip(T::TARGETS)
            .try_for_each(|(input, &target)| {
                T::get_opt_input_iface_mode(req, node, target, timeout_ms).map(|mode| {
                    *input = mode;
                })
            })?;

        self.outputs
            .iter_mut()
            .zip(T::TARGETS)
            .try_for_each(|(output, &target)| {
                T::get_opt_output_iface_mode(req, node, target, timeout_ms).map(|mode| {
                    *output = mode;
                })
            })?;

        Ok(())
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = T::MODES.iter().map(|m| opt_iface_mode_to_str(m)).collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OPT_IFACE_IN_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, T::TARGETS.len(), &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OPT_IFACE_OUT_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, T::TARGETS.len(), &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OPT_IFACE_IN_MODE_NAME => {
                let vals: Vec<u32> = self
                    .inputs
                    .iter()
                    .zip(T::TARGETS)
                    .map(|(input, _)| T::MODES.iter().position(|t| input.eq(t)).unwrap() as u32)
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            OPT_IFACE_OUT_MODE_NAME => {
                let vals: Vec<u32> = self
                    .outputs
                    .iter()
                    .zip(T::TARGETS)
                    .map(|(output, _)| T::MODES.iter().position(|t| output.eq(t)).unwrap() as u32)
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
                unit.lock()?;
                let res = self
                    .inputs
                    .iter_mut()
                    .zip(T::TARGETS)
                    .zip(elem_value.enumerated())
                    .try_for_each(|((input, &target), &val)| {
                        let pos = val as usize;
                        let &mode = T::MODES.iter().nth(pos).ok_or_else(|| {
                            let msg = format!("Invalid index for mode of opt interface: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })?;
                        T::set_opt_input_iface_mode(req, node, target, mode, timeout_ms)
                            .map(|_| *input = mode)
                    });
                let _ = unit.unlock();
                res.map(|_| true)
            }
            OPT_IFACE_OUT_MODE_NAME => {
                unit.lock()?;
                let res = self
                    .outputs
                    .iter_mut()
                    .zip(T::TARGETS)
                    .zip(elem_value.enumerated())
                    .try_for_each(|((output, &target), &val)| {
                        let pos = val as usize;
                        let &mode = T::MODES.iter().nth(pos).ok_or_else(|| {
                            let msg = format!("Invalid index for mode of opt interface: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })?;
                        T::set_opt_output_iface_mode(req, node, target, mode, timeout_ms)
                            .map(|_| *output = mode)
                    });
                let _ = unit.unlock();
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
