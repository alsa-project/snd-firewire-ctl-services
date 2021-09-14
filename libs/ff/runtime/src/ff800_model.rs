// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use hinawa::{SndUnit, SndUnitExt};

use core::card_cntr::*;
use core::elem_value_accessor::*;

use super::model::*;

use ff_protocols::{*, former::{*, ff800::*}};

use super::former_ctls::*;

#[derive(Default, Debug)]
pub struct Ff800Model{
    proto: Ff800Protocol,
    cfg_ctl: CfgCtl,
    status_ctl: StatusCtl,
    out_ctl: FormerOutCtl<Ff800OutputVolumeState>,
    mixer_ctl: FormerMixerCtl<Ff800MixerState>,
    meter_ctl: FormerMeterCtl<Ff800MeterState>,
}

const TIMEOUT_MS: u32 = 100;

impl CtlModel<SndUnit> for Ff800Model {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.status_ctl.load(unit, &self.proto, TIMEOUT_MS, card_cntr)?;
        self.cfg_ctl.load(unit, &self.proto, &self.status_ctl.status, card_cntr, TIMEOUT_MS)?;
        self.out_ctl.load(unit, &self.proto, card_cntr, TIMEOUT_MS)?;
        self.mixer_ctl.load(unit, &self.proto, card_cntr, TIMEOUT_MS)?;
        self.meter_ctl.load(unit, &self.proto, card_cntr, TIMEOUT_MS)?;

        Ok(())
    }

    fn read(&mut self, _: &mut SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.cfg_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndUnit, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.cfg_ctl.write(unit, &self.proto, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.out_ctl.write(unit, &self.proto, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.write(unit, &self.proto, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<SndUnit> for Ff800Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.status_ctl.measured_elem_list);
        self.meter_ctl.get_measured_elem_list(elem_id_list);
    }

    fn measure_states(&mut self, unit: &mut SndUnit) -> Result<(), Error> {
        self.status_ctl.measure_states(unit, &self.proto, TIMEOUT_MS)?;
        self.meter_ctl.measure_states(unit, &self.proto, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.status_ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

fn update_cfg<F>(unit: &SndUnit, proto: &Ff800Protocol, cfg: &mut Ff800Config, timeout_ms: u32, cb: F)
    -> Result<(), Error>
    where F: Fn(&mut Ff800Config) -> Result<(), Error>,
{
    let mut cache = cfg.clone();
    cb(&mut cache)?;
    Ff800Protocol::write_cfg(proto, &mut unit.get_node(), &cache, timeout_ms)
        .map(|_| *cfg = cache)
}

fn clk_src_to_string(src: &Ff800ClkSrc) -> String {
    match src {
        Ff800ClkSrc::Internal => "Internal",
        Ff800ClkSrc::WordClock => "Word-clock",
        Ff800ClkSrc::AdatA => "ADAT-A",
        Ff800ClkSrc::AdatB => "ADAT-B",
        Ff800ClkSrc::Spdif => "S/PDIF",
        Ff800ClkSrc::Tco => "TCO",
    }.to_string()
}

fn line_in_jack_to_string(jack: &Ff800AnalogInputJack) -> String {
    match jack {
        Ff800AnalogInputJack::Front => "Front",
        Ff800AnalogInputJack::Rear => "Rear",
        Ff800AnalogInputJack::FrontRear => "Front+Rear",
    }.to_string()
}

#[derive(Default, Debug)]
struct StatusCtl{
    status: Ff800Status,
    measured_elem_list: Vec<ElemId>,
}

const EXT_SRC_LOCK_NAME: &str = "external-source-lock";
const EXT_SRC_SYNC_NAME: &str = "external-source-sync";
const SPDIF_SRC_RATE_NAME: &str = "spdif-source-rate";
const EXT_SRC_RATE_NAME: &str = "external-source-rate";
const ACTIVE_CLK_SRC_NAME: &str = "active-clock-source";

impl StatusCtl {
    const EXT_SRCS: [Ff800ClkSrc;5] = [
        Ff800ClkSrc::Spdif,
        Ff800ClkSrc::AdatA,
        Ff800ClkSrc::AdatB,
        Ff800ClkSrc::WordClock,
        Ff800ClkSrc::Tco,
    ];

    const EXT_SRC_RATES: [Option<ClkNominalRate>;10] = [
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

    fn load(&mut self, unit: &SndUnit, proto: &Ff800Protocol, timeout_ms: u32, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        Ff800Protocol::read_status(proto, &mut unit.get_node(), &mut self.status, timeout_ms)?;

        let labels: Vec<String> = CfgCtl::CLK_SRCS.iter()
            .map(|s| clk_src_to_string(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, ACTIVE_CLK_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, false)?;

        [EXT_SRC_LOCK_NAME, EXT_SRC_SYNC_NAME].iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, name, 0);
                card_cntr.add_bool_elems(&elem_id, 1, Self::EXT_SRCS.len(), false)
                    .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))
            })?;

        let labels: Vec<String> = Self::EXT_SRC_RATES.iter()
            .map(|r| optional_clk_nominal_rate_to_string(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SPDIF_SRC_RATE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, EXT_SRC_RATE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))?;

        Ok(())
    }

    fn measure_states(&mut self, unit: &SndUnit, proto: &Ff800Protocol, timeout_ms: u32)
        -> Result<(), Error>
    {
        Ff800Protocol::read_status(proto, &mut unit.get_node(), &mut self.status, timeout_ms)
    }

    fn measure_elem(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            EXT_SRC_LOCK_NAME => {
                let vals = [
                    self.status.lock.spdif,
                    self.status.lock.adat_a,
                    self.status.lock.adat_b,
                    self.status.lock.word_clock,
                    self.status.lock.tco,
                ];
                elem_value.set_bool(&vals);
                Ok(true)
            }
            EXT_SRC_SYNC_NAME => {
                let vals = [
                    self.status.sync.spdif,
                    self.status.sync.adat_a,
                    self.status.sync.adat_b,
                    self.status.sync.word_clock,
                    self.status.sync.tco,
                ];
                elem_value.set_bool(&vals);
                Ok(true)
            }
            SPDIF_SRC_RATE_NAME => {
                let pos = Self::EXT_SRC_RATES.iter()
                    .position(|r| r.eq(&self.status.spdif_rate))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            EXT_SRC_RATE_NAME => {
                let pos = Self::EXT_SRC_RATES.iter()
                    .position(|r| r.eq(&self.status.external_clk_rate))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            ACTIVE_CLK_SRC_NAME => {
                let pos = CfgCtl::CLK_SRCS.iter()
                    .position(|s| s.eq(&self.status.active_clk_src))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false)
        }
    }
}

#[derive(Default, Debug)]
struct CfgCtl(Ff800Config);

const PRIMARY_CLK_SRC_NAME: &str = "primary-clock-source";
const INPUT_JACK_NAME: &str = "input-1/7/8-jack";
const INPUT_LINE_LEVEL_NAME: &str = "input-line-level";
const INPUT_POWER_NAME: &str = "input-7/8/9/10-powering";
const INPUT_INST_DRIVE_NAME: &str = "input-1-inst-drive";
const INPUT_INST_LIMITTER_NAME: &str = "input-1-inst-limitter";
const INPUT_INST_SPKR_EMU_NAME: &str = "input-1-inst-speaker-emu";
const OUTPUT_LINE_LEVEL_NAME: &str = "output-line-level";
const SPDIF_INPUT_IFACE_NAME: &str = "spdif-input-interface";
const SPDIF_INPUT_USE_PREEMBLE_NAME: &str = "spdif-input-use-preemble";
const SPDIF_OUTPUT_FMT_NAME: &str = "spdif-output-format";
const SPDIF_OUTPUT_EMPHASIS_NAME: &str = "spdif-output-emphasis";
const SPDIF_OUTPUT_NON_AUDIO_NAME: &str = "spdif-output-non-audio";
const OPT_OUTPUT_SIGNAL_NAME: &str = "optical-output-signal";
const WORD_CLOCK_SINGLE_SPPED_NAME: &str = "word-clock-single-speed";

impl CfgCtl {
    const CLK_SRCS: [Ff800ClkSrc;6] = [
        Ff800ClkSrc::Internal,
        Ff800ClkSrc::WordClock,
        Ff800ClkSrc::AdatA,
        Ff800ClkSrc::AdatB,
        Ff800ClkSrc::Spdif,
        Ff800ClkSrc::Tco,
    ];

    const INPUT_INPUT_JACK_TARGETS: [&'static str; 3] = [
        "analog-1",
        "analog-7",
        "analog-8",
    ];

    const INPUT_POWER_TARGETS: [&'static str; 4] = [
        "analog-7",
        "analog-8",
        "analog-9",
        "analog-10",
    ];

    const INPUT_JACKS: [Ff800AnalogInputJack;3] = [
        Ff800AnalogInputJack::Front,
        Ff800AnalogInputJack::Rear,
        Ff800AnalogInputJack::FrontRear,
    ];

    const INPUT_LINE_LEVELS: [FormerLineInNominalLevel;3] = [
        FormerLineInNominalLevel::Low,
        FormerLineInNominalLevel::Consumer,
        FormerLineInNominalLevel::Professional,
    ];

    const OUTPUT_LINE_LEVELS: [LineOutNominalLevel;3] = [
        LineOutNominalLevel::High,
        LineOutNominalLevel::Consumer,
        LineOutNominalLevel::Professional,
    ];

    const SPDIF_IFACES: [SpdifIface;2] = [
        SpdifIface::Coaxial,
        SpdifIface::Optical,
    ];

    const SPDIF_FMTS: [SpdifFormat;2] = [
        SpdifFormat::Consumer,
        SpdifFormat::Professional,
    ];

    const OPT_OUT_SIGNALS: [OpticalOutputSignal;2] = [
        OpticalOutputSignal::Adat,
        OpticalOutputSignal::Spdif,
    ];

    fn load(&mut self, unit: &SndUnit, proto: &Ff800Protocol, status: &Ff800Status, card_cntr: &mut CardCntr,
            timeout_ms: u32)
        -> Result<(), Error>
    {
        self.0.init(&status);
        Ff800Protocol::write_cfg(proto, &mut unit.get_node(), &self.0, timeout_ms)?;

        let labels: Vec<String> = Self::CLK_SRCS.iter()
            .map(|s| clk_src_to_string(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, PRIMARY_CLK_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<String> = Self::INPUT_JACKS.iter()
            .map(|l| line_in_jack_to_string(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, INPUT_JACK_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, Self::INPUT_INPUT_JACK_TARGETS.len(), &labels,
                                         None, true)?;

        let labels: Vec<String> = Self::INPUT_LINE_LEVELS.iter()
            .map(|l| former_line_in_nominal_level_to_string(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, INPUT_LINE_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, INPUT_POWER_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::INPUT_POWER_TARGETS.len(), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, INPUT_INST_DRIVE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, INPUT_INST_LIMITTER_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, INPUT_INST_SPKR_EMU_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::OUTPUT_LINE_LEVELS.iter()
            .map(|l| line_out_nominal_level_to_string(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUTPUT_LINE_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<String> = Self::SPDIF_IFACES.iter()
            .map(|i| spdif_iface_to_string(i))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SPDIF_INPUT_IFACE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SPDIF_INPUT_USE_PREEMBLE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::SPDIF_FMTS.iter()
            .map(|f| spdif_format_to_string(f))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SPDIF_OUTPUT_FMT_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SPDIF_OUTPUT_EMPHASIS_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SPDIF_OUTPUT_NON_AUDIO_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::OPT_OUT_SIGNALS.iter()
            .map(|f| optical_output_signal_to_string(f))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OPT_OUTPUT_SIGNAL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, WORD_CLOCK_SINGLE_SPPED_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            PRIMARY_CLK_SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::CLK_SRCS.iter()
                        .position(|s| s.eq(&self.0.clk.primary_src))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            INPUT_JACK_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, 3, |idx| {
                    let pos = Self::INPUT_JACKS.iter()
                        .position(|j| j.eq(&self.0.analog_in.jacks[idx]))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            INPUT_LINE_LEVEL_NAME => {
                let pos = Self::INPUT_LINE_LEVELS.iter()
                    .position(|l| l.eq(&self.0.analog_in.line_level))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            INPUT_POWER_NAME => {
                elem_value.set_bool(&self.0.analog_in.phantom_powering);
                Ok(true)
            }
            INPUT_INST_DRIVE_NAME => {
                elem_value.set_bool(&[self.0.analog_in.inst.drive]);
                Ok(true)
            }
            INPUT_INST_LIMITTER_NAME => {
                elem_value.set_bool(&[self.0.analog_in.inst.limitter]);
                Ok(true)
            }
            INPUT_INST_SPKR_EMU_NAME => {
                elem_value.set_bool(&[self.0.analog_in.inst.speaker_emulation]);
                Ok(true)
            }
            OUTPUT_LINE_LEVEL_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::OUTPUT_LINE_LEVELS.iter()
                        .position(|l| l.eq(&self.0.line_out_level))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            SPDIF_INPUT_IFACE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::SPDIF_IFACES.iter()
                        .position(|i| i.eq(&self.0.spdif_in.iface))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            SPDIF_INPUT_USE_PREEMBLE_NAME => {
                elem_value.set_bool(&[self.0.spdif_in.use_preemble]);
                Ok(true)
            }
            SPDIF_OUTPUT_FMT_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::SPDIF_FMTS.iter()
                        .position(|f| f.eq(&self.0.spdif_out.format))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            SPDIF_OUTPUT_EMPHASIS_NAME => {
                elem_value.set_bool(&[self.0.spdif_out.emphasis]);
                Ok(true)
            }
            SPDIF_OUTPUT_NON_AUDIO_NAME => {
                elem_value.set_bool(&[self.0.spdif_out.non_audio]);
                Ok(true)
            }
            OPT_OUTPUT_SIGNAL_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::OPT_OUT_SIGNALS.iter()
                        .position(|f| f.eq(&self.0.opt_out_signal))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            WORD_CLOCK_SINGLE_SPPED_NAME => {
                elem_value.set_bool(&[self.0.word_out_single]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &SndUnit, proto: &Ff800Protocol, elem_id: &ElemId,
             old: &alsactl::ElemValue, new: &alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            PRIMARY_CLK_SRC_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<u32>::get_val(new, |val| {
                        let src = Self::CLK_SRCS.iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid value for index of clock source: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })?;
                        cfg.clk.primary_src = *src;
                        Ok(())
                    })
                })
                .map(|_| true)
            }
            INPUT_JACK_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<u32>::get_vals(new, old, 3, |idx, val| {
                        let jack = Self::INPUT_JACKS.iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid value for index of jack: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })?;
                        cfg.analog_in.jacks[idx] = *jack;
                        Ok(())
                    })
                })
                .map(|_| true)
            }
            INPUT_LINE_LEVEL_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<u32>::get_val(new, |val| {
                        Self::INPUT_LINE_LEVELS.iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid value for index of line input level: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&l| cfg.analog_in.line_level = l)
                    })
                })
                .map(|_| true)
            }
            INPUT_POWER_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    new.get_bool(&mut cfg.analog_in.phantom_powering);
                    Ok(())
                })
                .map(|_| true)
            }
            INPUT_INST_DRIVE_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        cfg.analog_in.inst.drive = val;
                        Ok(())
                    })
                })
                .map(|_| true)
            }
            INPUT_INST_LIMITTER_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        cfg.analog_in.inst.limitter = val;
                        Ok(())
                    })
                })
                .map(|_| true)
            }
            INPUT_INST_SPKR_EMU_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        cfg.analog_in.inst.speaker_emulation = val;
                        Ok(())
                    })
                })
                .map(|_| true)
            }
            OUTPUT_LINE_LEVEL_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<u32>::get_val(new, |val| {
                        Self::OUTPUT_LINE_LEVELS.iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid value for index of line output level: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&l| cfg.line_out_level = l)
                    })
                })
                .map(|_| true)
            }
            SPDIF_INPUT_IFACE_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<u32>::get_val(new, |val| {
                        Self::SPDIF_IFACES.iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid value for index of S/PDIF iface: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&i| cfg.spdif_in.iface = i)
                    })
                })
                .map(|_| true)
            }
            SPDIF_INPUT_USE_PREEMBLE_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        cfg.spdif_in.use_preemble = val;
                        Ok(())
                    })
                })
                .map(|_| true)
            }
            SPDIF_OUTPUT_FMT_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<u32>::get_val(new, |val| {
                        Self::SPDIF_FMTS.iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid value for index of S/PDIF format: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&f| cfg.spdif_out.format = f)
                    })
                })
                .map(|_| true)
            }
            SPDIF_OUTPUT_EMPHASIS_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        cfg.spdif_out.emphasis = val;
                        Ok(())
                    })
                })
                .map(|_| true)
            }
            SPDIF_OUTPUT_NON_AUDIO_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        cfg.spdif_out.non_audio = val;
                        Ok(())
                    })
                })
                .map(|_| true)
            }
            OPT_OUTPUT_SIGNAL_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<u32>::get_val(new, |val| {
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
            WORD_CLOCK_SINGLE_SPPED_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
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
