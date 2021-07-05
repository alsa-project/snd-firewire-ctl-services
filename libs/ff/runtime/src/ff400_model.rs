// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use hinawa::{SndUnit, SndUnitExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;
use core::elem_value_accessor::*;

use ff_protocols::{*, former::{*, ff400::*}};

use super::former_ctls::*;

use super::model::*;

#[derive(Default, Debug)]
pub struct Ff400Model{
    proto: Ff400Protocol,
    meter_ctl: FormerMeterCtl<Ff400MeterState>,
    out_ctl: FormerOutCtl<Ff400OutputVolumeState>,
    input_gain_ctl: InputGainCtl,
    mixer_ctl: FormerMixerCtl<Ff400MixerState>,
    status_ctl: StatusCtl,
    cfg_ctl: CfgCtl,
}

const TIMEOUT_MS: u32 = 100;

impl CtlModel<SndUnit> for Ff400Model {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.meter_ctl.load(unit, &self.proto, card_cntr, TIMEOUT_MS)?;
        self.out_ctl.load(unit, &self.proto, card_cntr, TIMEOUT_MS)?;
        self.mixer_ctl.load(unit, &self.proto, card_cntr, TIMEOUT_MS)?;
        self.input_gain_ctl.load(unit, &self.proto, card_cntr, TIMEOUT_MS)?;
        self.status_ctl.load(unit, &self.proto, card_cntr, TIMEOUT_MS)?;
        self.cfg_ctl.load(unit, &self.proto, &self.status_ctl.status, card_cntr, TIMEOUT_MS)?;
        Ok(())
    }

    fn read(&mut self, _: &mut SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.out_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_gain_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.cfg_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndUnit, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.out_ctl.write(unit, &self.proto, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctl.write(unit, &self.proto, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.input_gain_ctl.write(unit, &self.proto, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.cfg_ctl.write(unit, &self.proto, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<SndUnit> for Ff400Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        self.meter_ctl.get_measured_elem_list(elem_id_list);
        elem_id_list.extend_from_slice(&self.status_ctl.measured_elem_list);
    }

    fn measure_states(&mut self, unit: &SndUnit) -> Result<(), Error> {
        self.meter_ctl.measure_states(unit, &self.proto, TIMEOUT_MS)?;
        self.status_ctl.measure_states(unit, &self.proto, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.meter_ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.status_ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Debug)]
struct InputGainCtl{
    status: Ff400InputGainStatus,
}

impl<'a> InputGainCtl {
    const MIC_GAIN_NAME: &'a str = "mic-input-gain";
    const LINE_GAIN_NAME: &'a str = "line-input-gain";

    const MIC_GAIN_MIN: i32 = 0;
    const MIC_GAIN_MAX: i32 = 65;
    const MIC_GAIN_STEP: i32 = 1;
    const MIC_GAIN_TLV: DbInterval = DbInterval{min: 0, max: 6500, linear: false, mute_avail: false};

    const LINE_GAIN_MIN: i32 = 0;
    const LINE_GAIN_MAX: i32 = 36;
    const LINE_GAIN_STEP: i32 = 1;
    const LINE_GAIN_TLV: DbInterval = DbInterval{min: 0, max: 18000, linear: false, mute_avail: false};

    fn load(&mut self, unit: &SndUnit, proto: &Ff400Protocol, card_cntr: &mut CardCntr, timeout_ms: u32)
        -> Result<(), Error>
    {
        proto.init_input_gains(&unit.get_node(), &mut self.status, timeout_ms)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MIC_GAIN_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::MIC_GAIN_MIN, Self::MIC_GAIN_MAX, Self::MIC_GAIN_STEP,
                                2, Some(&Vec::<u32>::from(&Self::MIC_GAIN_TLV)), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::LINE_GAIN_NAME, 0);
        card_cntr.add_int_elems(&elem_id, 1, Self::LINE_GAIN_MIN, Self::LINE_GAIN_MAX, Self::LINE_GAIN_STEP,
                                2, Some(&Vec::<u32>::from(&Self::LINE_GAIN_TLV)), true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::MIC_GAIN_NAME => {
                let vals: Vec<i32> = self.status.mic.iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::LINE_GAIN_NAME => {
                let vals: Vec<i32> = self.status.line.iter()
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &SndUnit, proto: &Ff400Protocol, elem_id: &ElemId,
             elem_value: &alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MIC_GAIN_NAME => {
                let mut vals = [0;2];
                elem_value.get_int(&mut vals);
                let gains: Vec<i8> = vals.iter()
                    .map(|&val| val as i8)
                    .collect();
                proto.write_input_mic_gains(&unit.get_node(), &mut self.status, &gains, timeout_ms)
                    .map(|_| true)
            }
            Self::LINE_GAIN_NAME => {
                let mut vals = [0;2];
                elem_value.get_int(&mut vals);
                let gains: Vec<i8> = vals.iter()
                    .map(|&val| val as i8)
                    .collect();
                proto.write_input_line_gains(&unit.get_node(), &mut self.status, &gains, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

fn clk_src_to_string(src: &Ff400ClkSrc) -> String {
    match src {
        Ff400ClkSrc::Internal => "Internal",
        Ff400ClkSrc::WordClock => "Word-clock",
        Ff400ClkSrc::Adat => "ADAT",
        Ff400ClkSrc::Spdif => "S/PDIF",
        Ff400ClkSrc::Ltc => "LTC",
    }.to_string()
}

fn update_cfg<F>(unit: &SndUnit, proto: &Ff400Protocol, cfg: &mut Ff400Config, timeout_ms: u32, cb: F)
    -> Result<(), Error>
    where F: Fn(&mut Ff400Config) -> Result<(), Error>,
{
    let mut cache = cfg.clone();
    cb(&mut cache)?;
    proto.write_cfg(&unit.get_node(), &cache, timeout_ms)
        .map(|_| *cfg = cache)
}

#[derive(Default, Debug)]
struct StatusCtl{
    status: Ff400Status,
    measured_elem_list: Vec<ElemId>,
}

impl<'a> StatusCtl {
    const EXT_SRC_LOCK_NAME: &'a str = "external-source-lock";
    const EXT_SRC_SYNC_NAME: &'a str = "external-source-sync";
    const SPDIF_SRC_RATE_NAME: &'a str = "spdif-source-rate";
    const EXT_SRC_RATE_NAME: &'a str = "external-source-rate";
    const ACTIVE_CLK_SRC_NAME: &'a str = "active-clock-source";

    const EXT_SRCS: [Ff400ClkSrc;4] = [
        Ff400ClkSrc::Spdif,
        Ff400ClkSrc::Adat,
        Ff400ClkSrc::WordClock,
        Ff400ClkSrc::Ltc,
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

    fn load(&mut self, unit: &SndUnit, proto: &Ff400Protocol, card_cntr: &mut CardCntr, timeout_ms: u32)
        -> Result<(), Error>
    {
        proto.read_status(&unit.get_node(), &mut self.status, timeout_ms)?;

        let labels: Vec<String> = CfgCtl::CLK_SRCS.iter()
            .map(|s| clk_src_to_string(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::ACTIVE_CLK_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, false)?;

        [Self::EXT_SRC_LOCK_NAME, Self::EXT_SRC_SYNC_NAME].iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, name, 0);
                card_cntr.add_bool_elems(&elem_id, 1, Self::EXT_SRCS.len(), false)
                    .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))
            })?;

        let labels: Vec<String> = Self::EXT_SRC_RATES.iter()
            .map(|r| optional_clk_nominal_rate_to_string(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SPDIF_SRC_RATE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::EXT_SRC_RATE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| self.measured_elem_list.append(&mut elem_id_list))?;

        Ok(())
    }

    fn measure_states(&mut self, unit: &SndUnit, proto: &Ff400Protocol, timeout_ms: u32)
        -> Result<(), Error>
    {
        proto.read_status(&unit.get_node(), &mut self.status, timeout_ms)
    }

    fn measure_elem(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::EXT_SRC_LOCK_NAME => {
                let vals = [
                    self.status.lock.spdif,
                    self.status.lock.adat,
                    self.status.lock.word_clock,
                ];
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::EXT_SRC_SYNC_NAME => {
                let vals = [
                    self.status.sync.spdif,
                    self.status.sync.adat,
                    self.status.sync.word_clock,
                ];
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::SPDIF_SRC_RATE_NAME => {
                let pos = Self::EXT_SRC_RATES.iter()
                    .position(|r| r.eq(&self.status.spdif_rate))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            Self::EXT_SRC_RATE_NAME => {
                let pos = Self::EXT_SRC_RATES.iter()
                    .position(|r| r.eq(&self.status.external_clk_rate))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            Self::ACTIVE_CLK_SRC_NAME => {
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
struct CfgCtl(Ff400Config);

impl<'a> CfgCtl {
    const PRIMARY_CLK_SRC_NAME: &'a str = "primary-clock-source";
    const LINE_INPUT_LEVEL_NAME: &'a str = "line-input-level";
    const MIC_POWER_NAME: &'a str = "mic-1/2-powering";
    const LINE_INST_NAME: &'a str = "line-3/4-inst";
    const LINE_PAD_NAME: &'a str = "line-3/4-pad";
    const LINE_OUTPUT_LEVEL_NAME: &'a str = "line-output-level";
    const HP_OUTPUT_LEVEL_NAME: &'a str = "headphone-output-level";
    const SPDIF_INPUT_IFACE_NAME: &'a str = "spdif-input-interface";
    const SPDIF_INPUT_USE_PREEMBLE_NAME: &'a str = "spdif-input-use-preemble";
    const SPDIF_OUTPUT_FMT_NAME: &'a str = "spdif-output-format";
    const SPDIF_OUTPUT_EMPHASIS_NAME: &'a str = "spdif-output-emphasis";
    const SPDIF_OUTPUT_NON_AUDIO_NAME: &'a str = "spdif-output-non-audio";
    const OPT_OUTPUT_SIGNAL_NAME: &'a str = "optical-output-signal";
    const WORD_CLOCK_SINGLE_SPPED_NAME: &'a str = "word-clock-single-speed";

    const CLK_SRCS: [Ff400ClkSrc;5] = [
        Ff400ClkSrc::Internal,
        Ff400ClkSrc::WordClock,
        Ff400ClkSrc::Adat,
        Ff400ClkSrc::Spdif,
        Ff400ClkSrc::Ltc,
    ];

    const LINE_INPUT_LEVELS: [FormerLineInNominalLevel;3] = [
        FormerLineInNominalLevel::Low,
        FormerLineInNominalLevel::Consumer,
        FormerLineInNominalLevel::Professional,
    ];

    const LINE_OUTPUT_LEVELS: [LineOutNominalLevel;3] = [
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

    fn load(&mut self, unit: &SndUnit, proto: &Ff400Protocol, status: &Ff400Status, card_cntr: &mut CardCntr,
            timeout_ms: u32)
        -> Result<(), Error>
    {
        self.0.init(&status);
        proto.write_cfg(&unit.get_node(), &self.0, timeout_ms)?;

        let labels: Vec<String> = Self::CLK_SRCS.iter()
            .map(|s| clk_src_to_string(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::PRIMARY_CLK_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<String> = Self::LINE_INPUT_LEVELS.iter()
            .map(|l| former_line_in_nominal_level_to_string(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::LINE_INPUT_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MIC_POWER_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 2, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::LINE_INST_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 2, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::LINE_PAD_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 2, true)?;

        let labels: Vec<String> = Self::LINE_OUTPUT_LEVELS.iter()
            .map(|l| line_out_nominal_level_to_string(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::LINE_OUTPUT_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::HP_OUTPUT_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<String> = Self::SPDIF_IFACES.iter()
            .map(|i| spdif_iface_to_string(i))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SPDIF_INPUT_IFACE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SPDIF_INPUT_USE_PREEMBLE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::SPDIF_FMTS.iter()
            .map(|f| spdif_format_to_string(f))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SPDIF_OUTPUT_FMT_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SPDIF_OUTPUT_EMPHASIS_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SPDIF_OUTPUT_NON_AUDIO_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::OPT_OUT_SIGNALS.iter()
            .map(|f| optical_output_signal_to_string(f))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OPT_OUTPUT_SIGNAL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::WORD_CLOCK_SINGLE_SPPED_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::SPDIF_IFACES.iter()
            .map(|i| spdif_iface_to_string(i))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SPDIF_INPUT_IFACE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SPDIF_INPUT_USE_PREEMBLE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::SPDIF_FMTS.iter()
            .map(|f| spdif_format_to_string(f))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SPDIF_OUTPUT_FMT_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SPDIF_OUTPUT_EMPHASIS_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SPDIF_OUTPUT_NON_AUDIO_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::OPT_OUT_SIGNALS.iter()
            .map(|f| optical_output_signal_to_string(f))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::OPT_OUTPUT_SIGNAL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::WORD_CLOCK_SINGLE_SPPED_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::PRIMARY_CLK_SRC_NAME => {
                let pos = Self::CLK_SRCS.iter()
                    .position(|s| s.eq(&self.0.clk.primary_src))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            Self::LINE_INPUT_LEVEL_NAME => {
                let pos = Self::LINE_INPUT_LEVELS.iter()
                    .position(|l| l.eq(&self.0.analog_in.line_level))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            Self::MIC_POWER_NAME => {
                elem_value.set_bool(&self.0.analog_in.phantom_powering);
                Ok(true)
            }
            Self::LINE_INST_NAME => {
                elem_value.set_bool(&self.0.analog_in.insts);
                Ok(true)
            }
            Self::LINE_PAD_NAME => {
                elem_value.set_bool(&self.0.analog_in.pad);
                Ok(true)
            }
            Self::LINE_OUTPUT_LEVEL_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::LINE_OUTPUT_LEVELS.iter()
                        .position(|l| l.eq(&self.0.line_out_level))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::HP_OUTPUT_LEVEL_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::LINE_OUTPUT_LEVELS.iter()
                        .position(|l| l.eq(&self.0.line_out_level))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::SPDIF_INPUT_IFACE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::SPDIF_IFACES.iter()
                        .position(|i| i.eq(&self.0.spdif_in.iface))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::SPDIF_INPUT_USE_PREEMBLE_NAME => {
                elem_value.set_bool(&[self.0.spdif_in.use_preemble]);
                Ok(true)
            }
            Self::SPDIF_OUTPUT_FMT_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::SPDIF_FMTS.iter()
                        .position(|f| f.eq(&self.0.spdif_out.format))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            Self::SPDIF_OUTPUT_EMPHASIS_NAME => {
                elem_value.set_bool(&[self.0.spdif_out.emphasis]);
                Ok(true)
            }
            Self::SPDIF_OUTPUT_NON_AUDIO_NAME => {
                elem_value.set_bool(&[self.0.spdif_out.non_audio]);
                Ok(true)
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
            Self::WORD_CLOCK_SINGLE_SPPED_NAME => {
                elem_value.set_bool(&[self.0.word_out_single]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &SndUnit, proto: &Ff400Protocol, elem_id: &ElemId,
             _: &alsactl::ElemValue, new: &alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::PRIMARY_CLK_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    Self::CLK_SRCS.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of clock source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .and_then(|&src| {
                            update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| Ok(cfg.clk.primary_src = src))
                        })
                })
                .map(|_| true)
            }
            Self::LINE_INPUT_LEVEL_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<u32>::get_val(new, |val| {
                        Self::LINE_INPUT_LEVELS.iter()
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
            Self::MIC_POWER_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    Ok(new.get_bool(&mut cfg.analog_in.phantom_powering))
                })
                .map(|_| true)
            }
            Self::LINE_INST_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    Ok(new.get_bool(&mut cfg.analog_in.insts))
                })
                .map(|_| true)
            }
            Self::LINE_PAD_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    Ok(new.get_bool(&mut cfg.analog_in.pad))
                })
                .map(|_| true)
            }
            Self::LINE_OUTPUT_LEVEL_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<u32>::get_val(new, |val| {
                        Self::LINE_OUTPUT_LEVELS.iter()
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
            Self::HP_OUTPUT_LEVEL_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<u32>::get_val(new, |val| {
                        Self::LINE_OUTPUT_LEVELS.iter()
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
            Self::SPDIF_INPUT_IFACE_NAME => {
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
            Self::SPDIF_INPUT_USE_PREEMBLE_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        cfg.spdif_in.use_preemble = val;
                        Ok(())
                    })
                })
                .map(|_| true)
            }
            Self::SPDIF_OUTPUT_FMT_NAME => {
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
            Self::SPDIF_OUTPUT_EMPHASIS_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        cfg.spdif_out.emphasis = val;
                        Ok(())
                    })
                })
                .map(|_| true)
            }
            Self::SPDIF_OUTPUT_NON_AUDIO_NAME => {
                update_cfg(unit, proto, &mut self.0, timeout_ms, |cfg| {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        cfg.spdif_out.non_audio = val;
                        Ok(())
                    })
                })
                .map(|_| true)
            }
            Self::OPT_OUTPUT_SIGNAL_NAME => {
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
            Self::WORD_CLOCK_SINGLE_SPPED_NAME => {
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
