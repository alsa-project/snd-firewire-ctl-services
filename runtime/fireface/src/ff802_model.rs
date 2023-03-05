// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::{latter_ctls::*, *},
    protocols::{
        latter::{ff802::*, *},
        *,
    },
};

#[derive(Default, Debug)]
pub struct Ff802Model {
    req: FwReq,
    meter_ctl: LatterMeterCtl<Ff802Protocol>,
    dsp_ctl: LatterDspCtl<Ff802Protocol>,
    cfg_ctl: CfgCtl,
    status_ctl: StatusCtl,
}

const TIMEOUT_MS: u32 = 100;

impl CtlModel<(SndUnit, FwNode)> for Ff802Model {
    fn load(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.meter_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.dsp_ctl.cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;

        self.meter_ctl.load(card_cntr)?;
        self.dsp_ctl.load(card_cntr)?;
        self.cfg_ctl
            .load(unit, &mut self.req, TIMEOUT_MS, card_cntr)?;
        self.status_ctl
            .load(unit, &mut self.req, TIMEOUT_MS, card_cntr)?;
        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.dsp_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.cfg_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .dsp_ctl
            .write(&mut self.req, &mut unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .cfg_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndUnit, FwNode)> for Ff802Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
        elem_id_list.extend_from_slice(&self.status_ctl.measured_elem_list);
    }

    fn measure_states(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.meter_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.status_ctl
            .measure_states(unit, &mut self.req, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &(SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.status_ctl.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

fn clk_src_to_string(src: &Ff802ClkSrc) -> String {
    match src {
        Ff802ClkSrc::Internal => "Internal",
        Ff802ClkSrc::AdatA => "ADAT-A",
        Ff802ClkSrc::AdatB => "ADAT-B",
        Ff802ClkSrc::AesEbu => "AES/EBU",
        Ff802ClkSrc::WordClk => "Word-clock",
    }
    .to_string()
}

fn ff802_spdif_iface_to_string(iface: &Ff802SpdifIface) -> String {
    match iface {
        Ff802SpdifIface::Xlr => "XLR",
        Ff802SpdifIface::Optical => "Optical",
    }
    .to_string()
}

fn update_cfg<F>(
    unit: &mut (SndUnit, FwNode),
    req: &mut FwReq,
    cfg: &mut Ff802Config,
    timeout_ms: u32,
    cb: F,
) -> Result<(), Error>
where
    F: Fn(&mut Ff802Config) -> Result<(), Error>,
{
    let mut cache = cfg.clone();
    cb(&mut cache)?;
    Ff802Protocol::write_cfg(req, &mut unit.1, &cache, timeout_ms).map(|_| *cfg = cache)
}

#[derive(Default, Debug)]
struct CfgCtl(Ff802Config);

const PRIMARY_CLK_SRC_NAME: &str = "primary-clock-source";
const SPDIF_INPUT_IFACE_NAME: &str = "spdif-input-interface";
const OPT_OUTPUT_SIGNAL_NAME: &str = "optical-output-signal";
const EFFECT_ON_INPUT_NAME: &str = "effect-on-input";
const SPDIF_OUTPUT_FMT_NAME: &str = "spdif-output-format";
const WORD_CLOCK_SINGLE_SPPED_NAME: &str = "word-clock-single-speed";

impl CfgCtl {
    const SPDIF_IFACES: [Ff802SpdifIface; 2] = [Ff802SpdifIface::Xlr, Ff802SpdifIface::Optical];

    const CLK_SRCS: [Ff802ClkSrc; 5] = [
        Ff802ClkSrc::Internal,
        Ff802ClkSrc::AdatA,
        Ff802ClkSrc::AdatB,
        Ff802ClkSrc::AesEbu,
        Ff802ClkSrc::WordClk,
    ];

    const CLK_RATES: [ClkNominalRate; 9] = [
        ClkNominalRate::R32000,
        ClkNominalRate::R44100,
        ClkNominalRate::R48000,
        ClkNominalRate::R64000,
        ClkNominalRate::R88200,
        ClkNominalRate::R96000,
        ClkNominalRate::R128000,
        ClkNominalRate::R176400,
        ClkNominalRate::R192000,
    ];

    const OPT_OUT_SIGNALS: [OpticalOutputSignal; 2] =
        [OpticalOutputSignal::Adat, OpticalOutputSignal::Spdif];

    const SPDIF_FMTS: [SpdifFormat; 2] = [SpdifFormat::Consumer, SpdifFormat::Professional];

    fn load(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        Ff802Protocol::write_cfg(req, &mut unit.1, &self.0, timeout_ms)?;

        let labels: Vec<String> = Self::CLK_SRCS
            .iter()
            .map(|s| clk_src_to_string(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, PRIMARY_CLK_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<String> = Self::SPDIF_IFACES
            .iter()
            .map(|i| ff802_spdif_iface_to_string(i))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SPDIF_INPUT_IFACE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<String> = Self::OPT_OUT_SIGNALS
            .iter()
            .map(|f| optical_output_signal_to_string(f))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OPT_OUTPUT_SIGNAL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, EFFECT_ON_INPUT_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::SPDIF_FMTS
            .iter()
            .map(|f| spdif_format_to_string(f))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SPDIF_OUTPUT_FMT_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, WORD_CLOCK_SINGLE_SPPED_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PRIMARY_CLK_SRC_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let pos = Self::CLK_SRCS
                    .iter()
                    .position(|s| s.eq(&self.0.clk_src))
                    .unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            SPDIF_INPUT_IFACE_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let pos = Self::SPDIF_IFACES
                    .iter()
                    .position(|i| i.eq(&self.0.spdif_in_iface))
                    .unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            OPT_OUTPUT_SIGNAL_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let pos = Self::OPT_OUT_SIGNALS
                    .iter()
                    .position(|f| f.eq(&self.0.opt_out_signal))
                    .unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            EFFECT_ON_INPUT_NAME => {
                elem_value.set_bool(&[self.0.effect_on_inputs]);
                Ok(true)
            }
            SPDIF_OUTPUT_FMT_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let pos = Self::SPDIF_FMTS
                    .iter()
                    .position(|f| f.eq(&self.0.spdif_out_format))
                    .unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            WORD_CLOCK_SINGLE_SPPED_NAME => {
                elem_value.set_bool(&[self.0.word_out_single]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PRIMARY_CLK_SRC_NAME => update_cfg(unit, req, &mut self.0, timeout_ms, |cfg| {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    let src = Self::CLK_SRCS.iter().nth(val as usize).ok_or_else(|| {
                        let msg = format!("Invalid value for index of clock source: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })?;
                    cfg.clk_src = *src;
                    Ok(())
                })
            })
            .map(|_| true),
            SPDIF_INPUT_IFACE_NAME => update_cfg(unit, req, &mut self.0, timeout_ms, |cfg| {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::SPDIF_IFACES
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of S/PDIF iface: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&i| cfg.spdif_in_iface = i)
                })
            })
            .map(|_| true),
            OPT_OUTPUT_SIGNAL_NAME => update_cfg(unit, req, &mut self.0, timeout_ms, |cfg| {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::OPT_OUT_SIGNALS
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!(
                                "Invalid value for index of optical output signal: {}",
                                val
                            );
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| cfg.opt_out_signal = s)
                })
            })
            .map(|_| true),
            EFFECT_ON_INPUT_NAME => update_cfg(unit, req, &mut self.0, timeout_ms, |cfg| {
                cfg.effect_on_inputs = elem_value.boolean()[0];
                Ok(())
            })
            .map(|_| true),
            SPDIF_OUTPUT_FMT_NAME => update_cfg(unit, req, &mut self.0, timeout_ms, |cfg| {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::SPDIF_FMTS
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of S/PDIF format: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&f| cfg.spdif_out_format = f)
                })
            })
            .map(|_| true),
            WORD_CLOCK_SINGLE_SPPED_NAME => update_cfg(unit, req, &mut self.0, timeout_ms, |cfg| {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    cfg.word_out_single = val;
                    Ok(())
                })
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct StatusCtl {
    status: Ff802Status,
    measured_elem_list: Vec<ElemId>,
}

const EXT_SRC_LOCK_NAME: &str = "external-source-lock";
const EXT_SRC_SYNC_NAME: &str = "external-source-sync";
const EXT_SRC_RATE_NAME: &str = "external-source-rate";
const ACTIVE_CLK_RATE_NAME: &str = "active-clock-rate";
const ACTIVE_CLK_SRC_NAME: &str = "active-clock-source";

impl StatusCtl {
    const EXT_CLK_SRCS: [Ff802ClkSrc; 4] = [
        Ff802ClkSrc::AdatA,
        Ff802ClkSrc::AdatB,
        Ff802ClkSrc::AesEbu,
        Ff802ClkSrc::WordClk,
    ];

    const EXT_CLK_RATES: [Option<ClkNominalRate>; 10] = [
        None,
        Some(ClkNominalRate::R32000),
        Some(ClkNominalRate::R44100),
        Some(ClkNominalRate::R48000),
        Some(ClkNominalRate::R64000),
        Some(ClkNominalRate::R88200),
        Some(ClkNominalRate::R96000),
        Some(ClkNominalRate::R128000),
        Some(ClkNominalRate::R176400),
        Some(ClkNominalRate::R192000),
    ];

    fn load(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        Ff802Protocol::read_status(req, &mut unit.1, &mut self.status, timeout_ms)?;

        [EXT_SRC_LOCK_NAME, EXT_SRC_SYNC_NAME]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, name, 0);
                card_cntr
                    .add_bool_elems(&elem_id, 1, Self::EXT_CLK_SRCS.len(), false)
                    .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))
            })?;

        let labels: Vec<String> = Self::EXT_CLK_RATES
            .iter()
            .map(|r| optional_clk_nominal_rate_to_string(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, EXT_SRC_RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, Self::EXT_CLK_SRCS.len(), &labels, None, false)
            .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))?;

        let labels: Vec<String> = CfgCtl::CLK_SRCS
            .iter()
            .map(|r| clk_src_to_string(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, ACTIVE_CLK_SRC_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))?;

        let labels: Vec<String> = CfgCtl::CLK_RATES
            .iter()
            .map(|r| clk_nominal_rate_to_string(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, ACTIVE_CLK_RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))?;

        Ok(())
    }

    fn measure_states(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Ff802Protocol::read_status(req, &mut unit.1, &mut self.status, timeout_ms)
    }

    fn read_measured_elem(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            EXT_SRC_LOCK_NAME => {
                let vals = [
                    self.status.ext_lock.adat_a,
                    self.status.ext_lock.adat_b,
                    self.status.ext_lock.spdif,
                    self.status.ext_lock.word_clk,
                ];
                elem_value.set_bool(&vals);
                Ok(true)
            }
            EXT_SRC_SYNC_NAME => {
                let vals = [
                    self.status.ext_sync.adat_a,
                    self.status.ext_sync.adat_b,
                    self.status.ext_sync.spdif,
                    self.status.ext_sync.word_clk,
                ];
                elem_value.set_bool(&vals);
                Ok(true)
            }
            EXT_SRC_RATE_NAME => {
                let vals: Vec<u32> = [
                    self.status.ext_rate.adat_a,
                    self.status.ext_rate.adat_b,
                    self.status.ext_rate.spdif,
                    self.status.ext_rate.word_clk,
                ]
                .iter()
                .map(|rate| {
                    Self::EXT_CLK_RATES
                        .iter()
                        .position(|r| r.eq(rate))
                        .map(|pos| pos as u32)
                        .unwrap()
                })
                .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            ACTIVE_CLK_SRC_NAME => {
                let pos = CfgCtl::CLK_SRCS
                    .iter()
                    .position(|r| r.eq(&self.status.active_clk_src))
                    .map(|p| p as u32)
                    .unwrap();
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            ACTIVE_CLK_RATE_NAME => {
                let pos = CfgCtl::CLK_RATES
                    .iter()
                    .position(|r| r.eq(&self.status.active_clk_rate))
                    .map(|p| p as u32)
                    .unwrap();
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
