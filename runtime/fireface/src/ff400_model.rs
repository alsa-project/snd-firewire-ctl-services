// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::{former_ctls::*, *},
    alsa_ctl_tlv_codec::DbInterval,
    protocols::former::ff400::*,
};

#[derive(Default, Debug)]
pub struct Ff400Model {
    req: FwReq,
    meter_ctl: FormerMeterCtl<Ff400Protocol>,
    out_ctl: FormerOutputCtl<Ff400Protocol>,
    mixer_ctl: FormerMixerCtl<Ff400Protocol>,
    input_gain_ctl: InputGainCtl,
    status_ctl: StatusCtl,
    cfg_ctl: CfgCtl,
}

const TIMEOUT_MS: u32 = 100;

impl CtlModel<(SndFireface, FwNode)> for Ff400Model {
    fn cache(&mut self, unit: &mut (SndFireface, FwNode)) -> Result<(), Error> {
        self.meter_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.out_ctl.cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.mixer_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.input_gain_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.status_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.cfg_ctl
            .cache(&mut self.req, &mut unit.1, &self.status_ctl.1, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(
        &mut self,
        _: &mut (SndFireface, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.meter_ctl.load(card_cntr)?;
        self.out_ctl.load(card_cntr)?;
        self.mixer_ctl.load(card_cntr)?;
        self.input_gain_ctl.load(card_cntr)?;
        self.status_ctl.load(card_cntr)?;
        self.cfg_ctl.load(card_cntr)?;
        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndFireface, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_gain_ctl.read(elem_id, elem_value)? {
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
        _: &ElemValue,
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
            .input_gain_ctl
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

impl MeasureModel<(SndFireface, FwNode)> for Ff400Model {
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

    fn measure_elem(
        &mut self,
        _: &(SndFireface, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.status_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndFireface, FwNode), u32> for Ff400Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.input_gain_ctl.0);
        elem_id_list.extend_from_slice(&self.out_ctl.0);
    }

    fn parse_notification(
        &mut self,
        (_, node): &mut (SndFireface, FwNode),
        &msg: &u32,
    ) -> Result<(), Error> {
        let mut input_gains = self.input_gain_ctl.1.clone();
        let mut out_vols = self.out_ctl.1.clone();

        if Ff400Protocol::parse_message(&mut input_gains, msg) {
            if input_gains != self.input_gain_ctl.1 {
                Ff400Protocol::update_partially(
                    &mut self.req,
                    node,
                    &mut self.input_gain_ctl.1,
                    input_gains,
                    TIMEOUT_MS,
                )?;
            }
        } else if Ff400Protocol::parse_message(&mut out_vols, msg) {
            if out_vols != self.out_ctl.1 {
                Ff400Protocol::update_partially(
                    &mut self.req,
                    node,
                    &mut self.out_ctl.1,
                    out_vols,
                    TIMEOUT_MS,
                )?;
            }
        }

        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndFireface, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.input_gain_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Debug)]
struct InputGainCtl(Vec<ElemId>, Ff400InputGainStatus);

const MIC_GAIN_NAME: &str = "mic-input-gain";
const LINE_GAIN_NAME: &str = "line-input-gain";

impl InputGainCtl {
    const MIC_GAIN_MIN: i32 = Ff400Protocol::MIC_INPUT_GAIN_MIN as i32;
    const MIC_GAIN_MAX: i32 = Ff400Protocol::MIC_INPUT_GAIN_MAX as i32;
    const MIC_GAIN_STEP: i32 = Ff400Protocol::MIC_INPUT_GAIN_STEP as i32;
    const MIC_GAIN_TLV: DbInterval = DbInterval {
        min: 0,
        max: 6500,
        linear: false,
        mute_avail: false,
    };

    const LINE_GAIN_MIN: i32 = Ff400Protocol::LINE_INPUT_GAIN_MIN as i32;
    const LINE_GAIN_MAX: i32 = Ff400Protocol::LINE_INPUT_GAIN_MAX as i32;
    const LINE_GAIN_STEP: i32 = Ff400Protocol::LINE_INPUT_GAIN_STEP as i32;
    const LINE_GAIN_TLV: DbInterval = DbInterval {
        min: 0,
        max: 18000,
        linear: false,
        mute_avail: false,
    };

    fn cache(&mut self, req: &mut FwReq, node: &mut FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = Ff400Protocol::update_wholly(req, node, &mut self.1, timeout_ms);
        debug!(params = ?self.1, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MIC_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::MIC_GAIN_MIN,
                Self::MIC_GAIN_MAX,
                Self::MIC_GAIN_STEP,
                2,
                Some(&Vec::<u32>::from(&Self::MIC_GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, LINE_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::LINE_GAIN_MIN,
                Self::LINE_GAIN_MAX,
                Self::LINE_GAIN_STEP,
                2,
                Some(&Vec::<u32>::from(&Self::LINE_GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIC_GAIN_NAME => {
                let vals: Vec<i32> = self.1.mic.iter().map(|&gain| gain as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            LINE_GAIN_NAME => {
                let vals: Vec<i32> = self.1.line.iter().map(|&gain| gain as i32).collect();
                elem_value.set_int(&vals);
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
            MIC_GAIN_NAME => {
                let mut params = self.1.clone();
                params
                    .mic
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val as i8);
                let res =
                    Ff400Protocol::update_partially(req, node, &mut self.1, params, timeout_ms);
                debug!(params = ?self.1, ?res);
                res.map(|_| true)
            }
            LINE_GAIN_NAME => {
                let mut params = self.1.clone();
                params
                    .line
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val as i8);
                let res =
                    Ff400Protocol::update_partially(req, node, &mut self.1, params, timeout_ms);
                debug!(params = ?self.1, ?res);
                res.map(|_| true)
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
    }
    .to_string()
}

#[derive(Default, Debug)]
struct StatusCtl(Vec<ElemId>, Ff400Status);

const EXT_SRC_LOCK_NAME: &'static str = "external-source-lock";
const EXT_SRC_SYNC_NAME: &'static str = "external-source-sync";
const SPDIF_SRC_RATE_NAME: &'static str = "spdif-source-rate";
const EXT_SRC_RATE_NAME: &'static str = "external-source-rate";
const ACTIVE_CLK_SRC_NAME: &'static str = "active-clock-source";

impl StatusCtl {
    const EXT_SRCS: [Ff400ClkSrc; 4] = [
        Ff400ClkSrc::Spdif,
        Ff400ClkSrc::Adat,
        Ff400ClkSrc::WordClock,
        Ff400ClkSrc::Ltc,
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
        let res = Ff400Protocol::cache_wholly(req, node, &mut self.1, timeout_ms);
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
                let vals = [self.1.lock.spdif, self.1.lock.adat, self.1.lock.word_clock];
                elem_value.set_bool(&vals);
                Ok(true)
            }
            EXT_SRC_SYNC_NAME => {
                let vals = [self.1.sync.spdif, self.1.sync.adat, self.1.sync.word_clock];
                elem_value.set_bool(&vals);
                Ok(true)
            }
            SPDIF_SRC_RATE_NAME => {
                let pos = Self::EXT_SRC_RATES
                    .iter()
                    .position(|r| r.eq(&self.1.spdif_rate))
                    .unwrap() as u32;
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            EXT_SRC_RATE_NAME => {
                let pos = Self::EXT_SRC_RATES
                    .iter()
                    .position(|r| r.eq(&self.1.external_clk_rate))
                    .unwrap() as u32;
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            ACTIVE_CLK_SRC_NAME => {
                let pos = CfgCtl::CLK_SRCS
                    .iter()
                    .position(|s| s.eq(&self.1.active_clk_src))
                    .unwrap() as u32;
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct CfgCtl(Ff400Config);

const PRIMARY_CLK_SRC_NAME: &str = "primary-clock-source";
const LINE_INPUT_LEVEL_NAME: &str = "line-input-level";
const MIC_POWER_NAME: &str = "mic-1/2-powering";
const LINE_INST_NAME: &str = "line-3/4-inst";
const LINE_PAD_NAME: &str = "line-3/4-pad";
const LINE_OUTPUT_LEVEL_NAME: &str = "line-output-level";
const HP_OUTPUT_LEVEL_NAME: &str = "headphone-output-level";
const SPDIF_INPUT_IFACE_NAME: &str = "spdif-input-interface";
const SPDIF_INPUT_USE_PREEMBLE_NAME: &str = "spdif-input-use-preemble";
const SPDIF_OUTPUT_FMT_NAME: &str = "spdif-output-format";
const SPDIF_OUTPUT_EMPHASIS_NAME: &str = "spdif-output-emphasis";
const SPDIF_OUTPUT_NON_AUDIO_NAME: &str = "spdif-output-non-audio";
const OPT_OUTPUT_SIGNAL_NAME: &str = "optical-output-signal";
const WORD_CLOCK_SINGLE_SPPED_NAME: &str = "word-clock-single-speed";

impl CfgCtl {
    const CLK_SRCS: [Ff400ClkSrc; 5] = [
        Ff400ClkSrc::Internal,
        Ff400ClkSrc::WordClock,
        Ff400ClkSrc::Adat,
        Ff400ClkSrc::Spdif,
        Ff400ClkSrc::Ltc,
    ];

    const LINE_INPUT_LEVELS: [FormerLineInNominalLevel; 3] = [
        FormerLineInNominalLevel::Low,
        FormerLineInNominalLevel::Consumer,
        FormerLineInNominalLevel::Professional,
    ];

    const LINE_OUTPUT_LEVELS: [LineOutNominalLevel; 3] = [
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
        status: &Ff400Status,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.0.init(&status);
        let res = Ff400Protocol::update_wholly(req, node, &self.0, timeout_ms);
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

        let labels: Vec<String> = Self::LINE_INPUT_LEVELS
            .iter()
            .map(|l| former_line_in_nominal_level_to_string(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, LINE_INPUT_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MIC_POWER_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 2, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, LINE_INST_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 2, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, LINE_PAD_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 2, true)?;

        let labels: Vec<String> = Self::LINE_OUTPUT_LEVELS
            .iter()
            .map(|l| line_out_nominal_level_to_string(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, LINE_OUTPUT_LEVEL_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, HP_OUTPUT_LEVEL_NAME, 0);
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
            LINE_INPUT_LEVEL_NAME => {
                let pos = Self::LINE_INPUT_LEVELS
                    .iter()
                    .position(|l| l.eq(&self.0.analog_in.line_level))
                    .unwrap() as u32;
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            MIC_POWER_NAME => {
                elem_value.set_bool(&self.0.analog_in.phantom_powering);
                Ok(true)
            }
            LINE_INST_NAME => {
                elem_value.set_bool(&self.0.analog_in.insts);
                Ok(true)
            }
            LINE_PAD_NAME => {
                elem_value.set_bool(&self.0.analog_in.pad);
                Ok(true)
            }
            LINE_OUTPUT_LEVEL_NAME => {
                let pos = Self::LINE_OUTPUT_LEVELS
                    .iter()
                    .position(|l| l.eq(&self.0.line_out_level))
                    .unwrap() as u32;
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            HP_OUTPUT_LEVEL_NAME => {
                let pos = Self::LINE_OUTPUT_LEVELS
                    .iter()
                    .position(|l| l.eq(&self.0.hp_out_level))
                    .unwrap() as u32;
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            SPDIF_INPUT_IFACE_NAME => {
                let pos = Self::SPDIF_IFACES
                    .iter()
                    .position(|i| i.eq(&self.0.spdif_in.iface))
                    .unwrap() as u32;
                elem_value.set_enum(&[pos]);
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
                elem_value.set_enum(&[pos]);
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
                elem_value.set_enum(&[pos]);
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
                let res = Ff400Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            LINE_INPUT_LEVEL_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.analog_in.line_level = Self::LINE_INPUT_LEVELS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of line input level: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = Ff400Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            MIC_POWER_NAME => {
                let mut params = self.0.clone();
                params
                    .analog_in
                    .phantom_powering
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(enabled, val)| *enabled = val);
                let res = Ff400Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            LINE_INST_NAME => {
                let mut params = self.0.clone();
                params
                    .analog_in
                    .insts
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(enabled, val)| *enabled = val);
                let res = Ff400Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            LINE_PAD_NAME => {
                let mut params = self.0.clone();
                params
                    .analog_in
                    .pad
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = Ff400Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            LINE_OUTPUT_LEVEL_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.line_out_level = Self::LINE_OUTPUT_LEVELS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of line output level: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = Ff400Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            HP_OUTPUT_LEVEL_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.hp_out_level = Self::LINE_OUTPUT_LEVELS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg =
                            format!("Invalid value for index of headphone output level: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = Ff400Protocol::update_wholly(req, node, &params, timeout_ms);
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
                let res = Ff400Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            SPDIF_INPUT_USE_PREEMBLE_NAME => {
                let mut params = self.0.clone();
                params.spdif_in.use_preemble = elem_value.boolean()[0];
                let res = Ff400Protocol::update_wholly(req, node, &params, timeout_ms);
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
                let res = Ff400Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            SPDIF_OUTPUT_EMPHASIS_NAME => {
                let mut params = self.0.clone();
                params.spdif_out.emphasis = elem_value.boolean()[0];
                let res = Ff400Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            SPDIF_OUTPUT_NON_AUDIO_NAME => {
                let mut params = self.0.clone();
                params.spdif_out.non_audio = elem_value.boolean()[0];
                let res = Ff400Protocol::update_wholly(req, node, &params, timeout_ms);
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
                let res = Ff400Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            WORD_CLOCK_SINGLE_SPPED_NAME => {
                let mut params = self.0.clone();
                params.word_out_single = elem_value.boolean()[0];
                let res = Ff400Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
