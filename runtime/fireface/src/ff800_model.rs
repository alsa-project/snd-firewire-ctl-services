// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::{former_ctls::*, *},
    protocols::former::ff800::*,
};

#[derive(Default, Debug)]
pub struct Ff800Model {
    req: FwReq,
    meter_ctl: FormerMeterCtl<Ff800Protocol>,
    out_ctl: FormerOutputCtl<Ff800Protocol>,
    mixer_ctl: FormerMixerCtl<Ff800Protocol>,
    status_ctl: StatusCtl,
    cfg_ctl: CfgCtl,
}

const TIMEOUT_MS: u32 = 100;

impl CtlModel<(SndFireface, FwNode)> for Ff800Model {
    fn cache(&mut self, unit: &mut (SndFireface, FwNode)) -> Result<(), Error> {
        self.meter_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.out_ctl.cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.mixer_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.status_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.cfg_ctl
            .cache(&mut self.req, &mut unit.1, &self.status_ctl.1, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.meter_ctl.load(card_cntr)?;
        self.out_ctl.load(card_cntr)?;
        self.mixer_ctl.load(card_cntr)?;
        self.status_ctl.load(card_cntr)?;
        self.cfg_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.status_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.cfg_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndFireface, FwNode),
        elem_id: &ElemId,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .out_ctl
            .write(&mut self.req, &mut unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_ctl
            .write(&mut self.req, &mut unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .cfg_ctl
            .write(&mut self.req, &mut unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndFireface, FwNode)> for Ff800Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
        elem_id_list.extend_from_slice(&self.status_ctl.0);
    }

    fn measure_states(&mut self, unit: &mut (SndFireface, FwNode)) -> Result<(), Error> {
        self.meter_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.status_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        Ok(())
    }
}

fn clk_src_to_string(src: &Ff800ClkSrc) -> String {
    match src {
        Ff800ClkSrc::Internal => "Internal",
        Ff800ClkSrc::WordClock => "Word-clock",
        Ff800ClkSrc::AdatA => "ADAT-A",
        Ff800ClkSrc::AdatB => "ADAT-B",
        Ff800ClkSrc::Spdif => "S/PDIF",
        Ff800ClkSrc::Tco => "TCO",
    }
    .to_string()
}

fn line_in_jack_to_string(jack: &Ff800AnalogInputJack) -> String {
    match jack {
        Ff800AnalogInputJack::Front => "Front",
        Ff800AnalogInputJack::Rear => "Rear",
        Ff800AnalogInputJack::FrontRear => "Front+Rear",
    }
    .to_string()
}

#[derive(Default, Debug)]
struct StatusCtl(Vec<ElemId>, Ff800Status);

const EXT_SRC_LOCK_NAME: &str = "external-source-lock";
const EXT_SRC_SYNC_NAME: &str = "external-source-sync";
const SPDIF_SRC_RATE_NAME: &str = "spdif-source-rate";
const EXT_SRC_RATE_NAME: &str = "external-source-rate";
const ACTIVE_CLK_SRC_NAME: &str = "active-clock-source";

impl StatusCtl {
    const EXT_SRCS: [Ff800ClkSrc; 5] = [
        Ff800ClkSrc::Spdif,
        Ff800ClkSrc::AdatA,
        Ff800ClkSrc::AdatB,
        Ff800ClkSrc::WordClock,
        Ff800ClkSrc::Tco,
    ];

    const EXT_SRC_RATES: [Option<ClkNominalRate>; 10] = [
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

    fn cache(&mut self, req: &mut FwReq, node: &mut FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = Ff800Protocol::cache_wholly(req, node, &mut self.1, timeout_ms);
        debug!(params = ?self.1, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = CfgCtl::CLK_SRCS
            .iter()
            .map(|s| clk_src_to_string(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, ACTIVE_CLK_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, false)?;

        [EXT_SRC_LOCK_NAME, EXT_SRC_SYNC_NAME]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, name, 0);
                card_cntr
                    .add_bool_elems(&elem_id, 1, Self::EXT_SRCS.len(), false)
                    .map(|mut elem_id_list| self.0.append(&mut elem_id_list))
            })?;

        let labels: Vec<String> = Self::EXT_SRC_RATES
            .iter()
            .map(|r| optional_clk_nominal_rate_to_string(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SPDIF_SRC_RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, EXT_SRC_RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            EXT_SRC_LOCK_NAME => {
                let vals = [
                    self.1.lock.spdif,
                    self.1.lock.adat_a,
                    self.1.lock.adat_b,
                    self.1.lock.word_clock,
                    self.1.lock.tco,
                ];
                elem_value.set_bool(&vals);
                Ok(true)
            }
            EXT_SRC_SYNC_NAME => {
                let vals = [
                    self.1.sync.spdif,
                    self.1.sync.adat_a,
                    self.1.sync.adat_b,
                    self.1.sync.word_clock,
                    self.1.sync.tco,
                ];
                elem_value.set_bool(&vals);
                Ok(true)
            }
            SPDIF_SRC_RATE_NAME => {
                let pos = Self::EXT_SRC_RATES
                    .iter()
                    .position(|r| r.eq(&self.1.spdif_rate))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            EXT_SRC_RATE_NAME => {
                let pos = Self::EXT_SRC_RATES
                    .iter()
                    .position(|r| r.eq(&self.1.external_clk_rate))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            ACTIVE_CLK_SRC_NAME => {
                let pos = CfgCtl::CLK_SRCS
                    .iter()
                    .position(|s| s.eq(&self.1.active_clk_src))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
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
    const CLK_SRCS: [Ff800ClkSrc; 6] = [
        Ff800ClkSrc::Internal,
        Ff800ClkSrc::WordClock,
        Ff800ClkSrc::AdatA,
        Ff800ClkSrc::AdatB,
        Ff800ClkSrc::Spdif,
        Ff800ClkSrc::Tco,
    ];

    const INPUT_INPUT_JACK_TARGETS: [&'static str; 3] = ["analog-1", "analog-7", "analog-8"];

    const INPUT_POWER_TARGETS: [&'static str; 4] =
        ["analog-7", "analog-8", "analog-9", "analog-10"];

    const INPUT_JACKS: [Ff800AnalogInputJack; 3] = [
        Ff800AnalogInputJack::Front,
        Ff800AnalogInputJack::Rear,
        Ff800AnalogInputJack::FrontRear,
    ];

    const INPUT_LINE_LEVELS: [FormerLineInNominalLevel; 3] = [
        FormerLineInNominalLevel::Low,
        FormerLineInNominalLevel::Consumer,
        FormerLineInNominalLevel::Professional,
    ];

    const OUTPUT_LINE_LEVELS: [LineOutNominalLevel; 3] = [
        LineOutNominalLevel::High,
        LineOutNominalLevel::Consumer,
        LineOutNominalLevel::Professional,
    ];

    const SPDIF_IFACES: [SpdifIface; 2] = [SpdifIface::Coaxial, SpdifIface::Optical];

    const SPDIF_FMTS: [SpdifFormat; 2] = [SpdifFormat::Consumer, SpdifFormat::Professional];

    const OPT_OUT_SIGNALS: [OpticalOutputSignal; 2] =
        [OpticalOutputSignal::Adat, OpticalOutputSignal::Spdif];

    fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        status: &Ff800Status,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.0.init(&status);
        let res = Ff800Protocol::update_wholly(req, node, &self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = Self::CLK_SRCS
            .iter()
            .map(|s| clk_src_to_string(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, PRIMARY_CLK_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<String> = Self::INPUT_JACKS
            .iter()
            .map(|l| line_in_jack_to_string(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, INPUT_JACK_NAME, 0);
        let _ = card_cntr.add_enum_elems(
            &elem_id,
            1,
            Self::INPUT_INPUT_JACK_TARGETS.len(),
            &labels,
            None,
            true,
        )?;

        let labels: Vec<String> = Self::INPUT_LINE_LEVELS
            .iter()
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

        let labels: Vec<String> = Self::OUTPUT_LINE_LEVELS
            .iter()
            .map(|l| line_out_nominal_level_to_string(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUTPUT_LINE_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<String> = Self::SPDIF_IFACES
            .iter()
            .map(|i| spdif_iface_to_string(i))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SPDIF_INPUT_IFACE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SPDIF_INPUT_USE_PREEMBLE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::SPDIF_FMTS
            .iter()
            .map(|f| spdif_format_to_string(f))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SPDIF_OUTPUT_FMT_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SPDIF_OUTPUT_EMPHASIS_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SPDIF_OUTPUT_NON_AUDIO_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let labels: Vec<String> = Self::OPT_OUT_SIGNALS
            .iter()
            .map(|f| optical_output_signal_to_string(f))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OPT_OUTPUT_SIGNAL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, WORD_CLOCK_SINGLE_SPPED_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PRIMARY_CLK_SRC_NAME => {
                let pos = Self::CLK_SRCS
                    .iter()
                    .position(|s| s.eq(&self.0.clk.primary_src))
                    .unwrap() as u32;
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            INPUT_JACK_NAME => {
                let vals: Vec<u32> = self
                    .0
                    .analog_in
                    .jacks
                    .iter()
                    .map(|jack| Self::INPUT_JACKS.iter().position(|j| j.eq(jack)).unwrap() as u32)
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            INPUT_LINE_LEVEL_NAME => {
                let pos = Self::INPUT_LINE_LEVELS
                    .iter()
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
                let pos = Self::OUTPUT_LINE_LEVELS
                    .iter()
                    .position(|l| l.eq(&self.0.line_out_level))
                    .unwrap() as u32;
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            SPDIF_INPUT_IFACE_NAME => {
                let pos = Self::SPDIF_IFACES
                    .iter()
                    .position(|i| i.eq(&self.0.spdif_in.iface))
                    .unwrap() as u32;
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            SPDIF_INPUT_USE_PREEMBLE_NAME => {
                elem_value.set_bool(&[self.0.spdif_in.use_preemble]);
                Ok(true)
            }
            SPDIF_OUTPUT_FMT_NAME => {
                let pos = Self::SPDIF_FMTS
                    .iter()
                    .position(|f| f.eq(&self.0.spdif_out.format))
                    .unwrap() as u32;
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
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
                let pos = Self::OPT_OUT_SIGNALS
                    .iter()
                    .position(|f| f.eq(&self.0.opt_out_signal))
                    .unwrap() as u32;
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            WORD_CLOCK_SINGLE_SPPED_NAME => {
                elem_value.set_bool(&[self.0.word_out_single]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PRIMARY_CLK_SRC_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.clk.primary_src = Self::CLK_SRCS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of clock source: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = Ff800Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            INPUT_JACK_NAME => {
                let mut params = self.0.clone();
                params
                    .analog_in
                    .jacks
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(jack, &val)| {
                        let pos = val as usize;
                        Self::INPUT_JACKS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg = format!("Invalid value for index of jack: {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|j| *jack = *j)
                    })?;
                let res = Ff800Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            INPUT_LINE_LEVEL_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.analog_in.line_level = Self::INPUT_LINE_LEVELS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of line input level: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = Ff800Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            INPUT_POWER_NAME => {
                let mut params = self.0.clone();
                params
                    .analog_in
                    .phantom_powering
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = Ff800Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            INPUT_INST_DRIVE_NAME => {
                let mut params = self.0.clone();
                params.analog_in.inst.drive = elem_value.boolean()[0];
                let res = Ff800Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            INPUT_INST_LIMITTER_NAME => {
                let mut params = self.0.clone();
                params.analog_in.inst.limitter = elem_value.boolean()[0];
                let res = Ff800Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            INPUT_INST_SPKR_EMU_NAME => {
                let mut params = self.0.clone();
                params.analog_in.inst.speaker_emulation = elem_value.boolean()[0];
                let res = Ff800Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            OUTPUT_LINE_LEVEL_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.line_out_level = Self::OUTPUT_LINE_LEVELS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of line output level: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = Ff800Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            SPDIF_INPUT_IFACE_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.spdif_in.iface = Self::SPDIF_IFACES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of S/PDIF iface: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = Ff800Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            SPDIF_INPUT_USE_PREEMBLE_NAME => {
                let mut params = self.0.clone();
                params.spdif_in.use_preemble = elem_value.boolean()[0];
                let res = Ff800Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            SPDIF_OUTPUT_FMT_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.spdif_out.format = Self::SPDIF_FMTS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of S/PDIF format: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = Ff800Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            SPDIF_OUTPUT_EMPHASIS_NAME => {
                let mut params = self.0.clone();
                params.spdif_out.emphasis = elem_value.boolean()[0];
                let res = Ff800Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            SPDIF_OUTPUT_NON_AUDIO_NAME => {
                let mut params = self.0.clone();
                params.spdif_out.non_audio = elem_value.boolean()[0];
                let res = Ff800Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            OPT_OUTPUT_SIGNAL_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.opt_out_signal = Self::OPT_OUT_SIGNALS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg =
                            format!("Invalid value for index of optical output signal: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = Ff800Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            WORD_CLOCK_SINGLE_SPPED_NAME => {
                let mut params = self.0.clone();
                params.word_out_single = elem_value.boolean()[0];
                let res = Ff800Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
