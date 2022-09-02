// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use super::register_dsp_runtime::*;

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

const RATE_NAME: &str = "sampling- rate";
const SRC_NAME: &str = "clock-source";

pub trait V2ClkCtlOperation<T: V2ClkOperation> {
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = T::CLK_RATES.iter().map(|e| clk_rate_to_str(&e.0)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, RATE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = T::CLK_SRCS.iter().map(|e| clk_src_to_str(&e.0)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SRC_NAME, 0);
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
            RATE_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                T::get_clk_rate(req, &mut unit.1, timeout_ms).map(|idx| idx as u32)
            })
            .map(|_| true),
            SRC_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                T::get_clk_src(req, &mut unit.1, timeout_ms).and_then(|idx| {
                    if T::HAS_LCD {
                        let label = clk_src_to_str(&T::CLK_SRCS[idx].0);
                        T::update_clk_display(req, &mut unit.1, &label, timeout_ms)?;
                    }
                    Ok(idx as u32)
                })
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
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            RATE_NAME => ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                unit.0.lock()?;
                let res = T::set_clk_rate(req, &mut unit.1, val as usize, timeout_ms);
                let _ = unit.0.unlock();
                res
            })
            .map(|_| true),
            SRC_NAME => ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                let prev_src = T::get_clk_src(req, &mut unit.1, timeout_ms)?;
                unit.0.lock()?;
                let mut res = T::set_clk_src(req, &mut unit.1, val as usize, timeout_ms);
                if res.is_ok() && T::HAS_LCD {
                    let label = clk_src_to_str(&T::CLK_SRCS[val as usize].0);
                    res = T::update_clk_display(req, &mut unit.1, &label, timeout_ms);
                    if res.is_err() {
                        let _ = T::set_clk_src(req, &mut unit.1, prev_src, timeout_ms);
                    }
                }
                let _ = unit.0.unlock();
                res
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }
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

pub trait V2OptIfaceCtlOperation<T: V2OptIfaceOperation> {
    fn state(&self) -> &(usize, usize);
    fn state_mut(&mut self) -> &mut (usize, usize);

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        self.cache(unit, req, timeout_ms)?;

        let mut notified_elem_id_list = Vec::new();

        let labels: Vec<&str> = T::OPT_IFACE_MODES
            .iter()
            .map(|e| opt_iface_mode_to_str(&e.0))
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OPT_IN_IFACE_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OPT_OUT_IFACE_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn cache(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::get_opt_in_iface_mode(req, &mut unit.1, timeout_ms)
            .map(|val| self.state_mut().0 = val)?;
        T::get_opt_out_iface_mode(req, &mut unit.1, timeout_ms).map(|val| self.state_mut().1 = val)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OPT_IN_IFACE_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || Ok(self.state().0 as u32))
                    .map(|_| true)
            }
            OPT_OUT_IFACE_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || Ok(self.state().1 as u32))
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndMotu, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OPT_IN_IFACE_MODE_NAME => ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                unit.0.lock()?;
                let res = T::set_opt_in_iface_mode(req, &mut unit.1, val as usize, timeout_ms);
                if res.is_ok() {
                    self.state_mut().0 = val as usize;
                }
                unit.0.unlock()?;
                res
            })
            .map(|_| true),
            OPT_OUT_IFACE_MODE_NAME => ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                unit.0.lock()?;
                let res = T::set_opt_out_iface_mode(req, &mut unit.1, val as usize, timeout_ms);
                if res.is_ok() {
                    self.state_mut().1 = val as usize;
                }
                unit.0.unlock()?;
                res
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }
}
