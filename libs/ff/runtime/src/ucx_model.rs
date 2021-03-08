// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExtManual};

use hinawa::{SndUnit, SndUnitExt};

use core::card_cntr::*;
use core::elem_value_accessor::*;

use ff_protocols::{*, latter::{*, ucx::*}};

use super::model::*;

#[derive(Default, Debug)]
pub struct UcxModel{
    proto: FfUcxProtocol,
    cfg_ctl: CfgCtl,
}

const TIMEOUT_MS: u32 = 100;

impl CtlModel<SndUnit> for UcxModel {
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

impl MeasureModel<SndUnit> for UcxModel {
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

fn clk_src_to_string(src: &FfUcxClkSrc) -> String {
    match src {
        FfUcxClkSrc::Internal => "Internal",
        FfUcxClkSrc::Coax => "Coaxial",
        FfUcxClkSrc::Opt => "Optical",
        FfUcxClkSrc::WordClk => "Word-clock",
    }.to_string()
}

fn update_cfg<F>(unit: &SndUnit, proto: &FfUcxProtocol, cfg: &mut FfUcxConfig, timeout_ms: u32, cb: F)
    -> Result<(), Error>
    where F: Fn(&mut FfUcxConfig) -> Result<(), Error>,
{
    let mut cache = cfg.clone();
    cb(&mut cache)?;
    proto.write_cfg(&unit.get_node(), &cache, timeout_ms)
        .map(|_| *cfg = cache)
}

#[derive(Default, Debug)]
struct CfgCtl(FfUcxConfig);

impl<'a> CfgCtl {
    const PRIMARY_CLK_SRC_NAME: &'a str = "primary-clock-source";
    const OPT_OUTPUT_SIGNAL_NAME: &'a str = "optical-output-signal";
    const EFFECT_ON_INPUT_NAME: &'a str = "effect-on-input";
    const SPDIF_OUTPUT_FMT_NAME: &'a str = "spdif-output-format";
    const WORD_CLOCK_SINGLE_SPPED_NAME: &'a str = "word-clock-single-speed";
    const WORD_CLOCK_IN_TERMINATE_NAME: &'a str = "word-clock-input-terminate";

    const CLK_SRCS: [FfUcxClkSrc;4] = [
        FfUcxClkSrc::Internal,
        FfUcxClkSrc::Coax,
        FfUcxClkSrc::Opt,
        FfUcxClkSrc::WordClk,
    ];

    const OPT_OUT_SIGNALS: [OpticalOutputSignal;2] = [
        OpticalOutputSignal::Adat,
        OpticalOutputSignal::Spdif,
    ];

    const SPDIF_FMTS: [SpdifFormat;2] = [
        SpdifFormat::Consumer,
        SpdifFormat::Professional,
    ];

    fn load(&mut self, unit: &SndUnit, proto: &FfUcxProtocol, timeout_ms: u32, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        proto.write_cfg(&unit.get_node(), &self.0, timeout_ms)?;

        let labels: Vec<String> = Self::CLK_SRCS.iter()
            .map(|s| clk_src_to_string(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::PRIMARY_CLK_SRC_NAME, 0);
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

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::WORD_CLOCK_IN_TERMINATE_NAME, 0);
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
            Self::WORD_CLOCK_IN_TERMINATE_NAME => {
                elem_value.set_bool(&[self.0.word_in_terminate]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &SndUnit, proto: &FfUcxProtocol, elem_id: &ElemId,
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
            Self::WORD_CLOCK_IN_TERMINATE_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                        cfg.word_in_terminate = val;
                        Ok(())
                    })
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
