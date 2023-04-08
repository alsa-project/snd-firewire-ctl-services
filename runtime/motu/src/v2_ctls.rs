// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub(crate) use super::{protocols::version_2::*, register_dsp_runtime::*};

#[derive(Default, Debug)]
pub(crate) struct V2ClkCtl<T: V2ClkOperation> {
    pub elem_id_list: Vec<ElemId>,
    rate: usize,
    source: usize,
    _phantom: PhantomData<T>,
}

const RATE_NAME: &str = "sampling- rate";
const SRC_NAME: &str = "clock-source";

impl<T: V2ClkOperation> V2ClkCtl<T> {
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::get_clk_rate(req, node, timeout_ms).map(|pos| self.rate = pos)?;
        T::get_clk_src(req, node, timeout_ms).map(|pos| self.source = pos)?;
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
                let pos = elem_value.enumerated()[0] as usize;
                unit.lock()?;
                let res = T::set_clk_rate(req, node, pos, timeout_ms).map(|_| self.rate = pos);
                let _ = unit.unlock();
                res.map(|_| true)
            }
            SRC_NAME => {
                let pos = elem_value.enumerated()[0] as usize;
                unit.lock()?;
                let res = T::set_clk_src(req, node, pos, timeout_ms).map(|_| self.source = pos);
                let _ = unit.unlock();
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct V2LcdClkCtl<T: V2ClkOperation> {
    pub elem_id_list: Vec<ElemId>,
    rate: usize,
    source: usize,
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

impl<T: V2ClkOperation> V2LcdClkCtl<T> {
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::get_clk_rate(req, node, timeout_ms).map(|pos| self.rate = pos)?;
        T::get_clk_src(req, node, timeout_ms).and_then(|pos| {
            let label = clk_src_to_str(&T::CLK_SRCS[pos].0);
            T::update_clk_display(req, node, &label, timeout_ms).map(|_| {
                self.source = pos;
            })
        })
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
                let idx = elem_value.enumerated()[0] as usize;
                unit.lock()?;
                let res = T::set_clk_rate(req, node, idx, timeout_ms).map(|_| self.rate = idx);
                let _ = unit.unlock();
                res.map(|_| true)
            }
            SRC_NAME => {
                let idx = elem_value.enumerated()[0] as usize;
                unit.lock()?;
                let res = T::set_clk_src(req, node, idx, timeout_ms).and_then(|_| {
                    let label = clk_src_to_str(&T::CLK_SRCS[idx].0);
                    T::update_clk_display(req, node, &label, timeout_ms)
                        .or_else(|_| T::set_clk_src(req, node, self.source, timeout_ms))
                        .map(|_| self.source = idx)
                });
                let _ = unit.unlock();
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct V2OptIfaceCtl<T: V2OptIfaceOperation> {
    pub elem_id_list: Vec<ElemId>,
    input_mode: usize,
    output_mode: usize,
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

impl<T: V2OptIfaceOperation> V2OptIfaceCtl<T> {
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::get_opt_in_iface_mode(req, node, timeout_ms).map(|val| self.input_mode = val)?;
        T::get_opt_out_iface_mode(req, node, timeout_ms).map(|val| self.output_mode = val)?;
        Ok(())
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = T::OPT_IFACE_MODES
            .iter()
            .map(|e| opt_iface_mode_to_str(&e.0))
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
                elem_value.set_enum(&[self.input_mode as u32]);
                Ok(true)
            }
            OPT_OUT_IFACE_MODE_NAME => {
                elem_value.set_enum(&[self.output_mode as u32]);
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
                let val = elem_value.enumerated()[0] as usize;
                unit.lock()?;
                let res = T::set_opt_in_iface_mode(req, node, val, timeout_ms)
                    .map(|_| self.input_mode = val);
                let _ = unit.unlock();
                res.map(|_| true)
            }
            OPT_OUT_IFACE_MODE_NAME => {
                let val = elem_value.enumerated()[0] as usize;
                unit.lock()?;
                let res = T::set_opt_out_iface_mode(req, node, val, timeout_ms)
                    .map(|_| self.output_mode = val);
                let _ = unit.unlock();
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
