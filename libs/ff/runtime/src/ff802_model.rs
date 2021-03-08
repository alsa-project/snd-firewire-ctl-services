// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExtManual};

use hinawa::{SndUnit, SndUnitExt};

use core::card_cntr::*;
use core::elem_value_accessor::*;

use ff_protocols::{*, latter::{*, ff802::*}};

use super::model::*;

#[derive(Default, Debug)]
pub struct Ff802Model{
    proto: Ff802Protocol,
    cfg_ctl: CfgCtl,
}

const TIMEOUT_MS: u32 = 100;

impl CtlModel<SndUnit> for Ff802Model {
    fn load(&mut self, unit: &SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.cfg_ctl.load(unit, &self.proto, TIMEOUT_MS, card_cntr)?;
        Ok(())
    }

    fn read(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.cfg_ctl.read(elem_id, elem_value)
    }

    fn write(&mut self, unit: &SndUnit, elem_id: &ElemId, _: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        self.cfg_ctl.write(unit, &self.proto, elem_id, new, TIMEOUT_MS)
    }
}

impl MeasureModel<SndUnit> for Ff802Model {
    fn get_measure_elem_list(&mut self, _: &mut Vec<ElemId>) {
    }

    fn measure_states(&mut self, _: &SndUnit) -> Result<(), Error> {
        Ok(())
    }

    fn measure_elem(&mut self, _: &SndUnit, _: &ElemId, _: &mut ElemValue)
        -> Result<bool, Error>
    {
        Ok(false)
    }
}

fn clk_src_to_string(src: &Ff802ClkSrc) -> String {
    match src {
        Ff802ClkSrc::Internal => "Internal",
        Ff802ClkSrc::AdatA => "ADAT-A",
        Ff802ClkSrc::AdatB => "ADAT-B",
        Ff802ClkSrc::AesEbu => "AES/EBU",
        Ff802ClkSrc::WordClk => "Word-clock",
    }.to_string()
}

fn ff802_spdif_iface_to_string(iface: &Ff802SpdifIface) -> String {
    match iface {
        Ff802SpdifIface::Xlr => "XLR",
        Ff802SpdifIface::Optical => "Optical",
    }.to_string()
}

fn update_cfg<F>(unit: &SndUnit, proto: &Ff802Protocol, cfg: &mut Ff802Config, timeout_ms: u32, cb: F)
    -> Result<(), Error>
    where F: Fn(&mut Ff802Config) -> Result<(), Error>,
{
    let mut cache = cfg.clone();
    cb(&mut cache)?;
    proto.write_cfg(&unit.get_node(), &cache, timeout_ms)
        .map(|_| *cfg = cache)
}

#[derive(Default, Debug)]
struct CfgCtl(Ff802Config);

impl<'a> CfgCtl {
    const PRIMARY_CLK_SRC_NAME: &'a str = "primary-clock-source";
    const SPDIF_INPUT_IFACE_NAME: &'a str = "spdif-input-interface";
    const OPT_OUTPUT_SIGNAL_NAME: &'a str = "optical-output-signal";
    const EFFECT_ON_INPUT_NAME: &'a str = "effect-on-input";
    const SPDIF_OUTPUT_FMT_NAME: &'a str = "spdif-output-format";
    const WORD_CLOCK_SINGLE_SPPED_NAME: &'a str = "word-clock-single-speed";

    const SPDIF_IFACES: [Ff802SpdifIface;2] = [
        Ff802SpdifIface::Xlr,
        Ff802SpdifIface::Optical,
    ];

    const CLK_SRCS: [Ff802ClkSrc;5] = [
        Ff802ClkSrc::Internal,
        Ff802ClkSrc::AdatA,
        Ff802ClkSrc::AdatB,
        Ff802ClkSrc::AesEbu,
        Ff802ClkSrc::WordClk,
    ];

    const OPT_OUT_SIGNALS: [OpticalOutputSignal;2] = [
        OpticalOutputSignal::Adat,
        OpticalOutputSignal::Spdif,
    ];

    const SPDIF_FMTS: [SpdifFormat;2] = [
        SpdifFormat::Consumer,
        SpdifFormat::Professional,
    ];

    fn load(&mut self, unit: &SndUnit, proto: &Ff802Protocol, timeout_ms: u32, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        proto.write_cfg(&unit.get_node(), &self.0, timeout_ms)?;

        let labels: Vec<String> = Self::CLK_SRCS.iter()
            .map(|s| clk_src_to_string(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::PRIMARY_CLK_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<String> = Self::SPDIF_IFACES.iter()
            .map(|i| ff802_spdif_iface_to_string(i))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SPDIF_INPUT_IFACE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<String> = Self::OPT_OUT_SIGNALS.iter()
            .map(|f| optical_output_signal_to_string(f))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OPT_OUTPUT_SIGNAL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EFFECT_ON_INPUT_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::SPDIF_FMTS.iter()
            .map(|f| spdif_format_to_string(f))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SPDIF_OUTPUT_FMT_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::WORD_CLOCK_SINGLE_SPPED_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::PRIMARY_CLK_SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::CLK_SRCS.iter()
                        .position(|s| s.eq(&self.0.clk_src))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::SPDIF_INPUT_IFACE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::SPDIF_IFACES.iter()
                        .position(|i| i.eq(&self.0.spdif_in_iface))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::OPT_OUTPUT_SIGNAL_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::OPT_OUT_SIGNALS.iter()
                        .position(|f| f.eq(&self.0.opt_out_signal))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::EFFECT_ON_INPUT_NAME => {
                elem_value.set_bool(&[self.0.effect_on_inputs]);
                Ok(true)
            }
            Self::SPDIF_OUTPUT_FMT_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::SPDIF_FMTS.iter()
                        .position(|f| f.eq(&self.0.spdif_out_format))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::WORD_CLOCK_SINGLE_SPPED_NAME => {
                elem_value.set_bool(&[self.0.word_out_single]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &SndUnit, proto: &Ff802Protocol, elem_id: &ElemId,
             elem_value: &alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::PRIMARY_CLK_SRC_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                        let src = Self::CLK_SRCS.iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid value for index of clock source: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })?;
                        cfg.clk_src = *src;
                        Ok(())
                    })
                })
                .map(|_| true)
            }
            Self::SPDIF_INPUT_IFACE_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                        Self::SPDIF_IFACES.iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid value for index of S/PDIF iface: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&i| cfg.spdif_in_iface = i)
                    })
                })
                .map(|_| true)
            }
            Self::OPT_OUTPUT_SIGNAL_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                        Self::OPT_OUT_SIGNALS.iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid value for index of optical output signal: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&s| cfg.opt_out_signal = s)
                    })
                })
                .map(|_| true)
            }
            Self::EFFECT_ON_INPUT_NAME => {
                let mut vals = [false];
                elem_value.get_bool(&mut vals);
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    cfg.effect_on_inputs = vals[0];
                    Ok(())
                })
                .map(|_| true)
            }
            Self::SPDIF_OUTPUT_FMT_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                        Self::SPDIF_FMTS.iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid value for index of S/PDIF format: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&f| cfg.spdif_out_format = f)
                    })
                })
                .map(|_| true)
            }
            Self::WORD_CLOCK_SINGLE_SPPED_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                        cfg.word_out_single = val;
                        Ok(())
                    })
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
