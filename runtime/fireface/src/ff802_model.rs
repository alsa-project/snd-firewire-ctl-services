// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::{latter_ctls::*, *},
    protocols::latter::ff802::*,
};

#[derive(Default, Debug)]
pub struct Ff802Model {
    req: FwReq,
    meter_ctl: LatterMeterCtl<Ff802Protocol>,
    cfg_ctl: CfgCtl,
    status_ctl: StatusCtl,
    input_ctl: LatterInputCtl<Ff802Protocol>,
    output_ctl: LatterOutputCtl<Ff802Protocol>,
    mixer_ctl: LatterMixerCtl<Ff802Protocol>,
    input_hpf_ctl: LatterInputHpfCtl<Ff802Protocol>,
    output_hpf_ctl: LatterOutputHpfCtl<Ff802Protocol>,
    input_eq_ctl: LatterInputEqualizerCtl<Ff802Protocol>,
    output_eq_ctl: LatterOutputEqualizerCtl<Ff802Protocol>,
    input_dyn_ctl: LatterInputDynamicsCtl<Ff802Protocol>,
    output_dyn_ctl: LatterOutputDynamicsCtl<Ff802Protocol>,
    input_al_ctl: LatterInputAutolevelCtl<Ff802Protocol>,
    output_al_ctl: LatterOutputAutolevelCtl<Ff802Protocol>,
    fx_source_ctl: LatterFxSourceCtl<Ff802Protocol>,
    fx_output_ctl: LatterFxOutputCtl<Ff802Protocol>,
    fx_reverb_ctl: LatterFxReverbCtl<Ff802Protocol>,
    fx_echo_ctl: LatterFxEchoCtl<Ff802Protocol>,
}

const TIMEOUT_MS: u32 = 100;

impl CtlModel<(SndFireface, FwNode)> for Ff802Model {
    fn cache(&mut self, (_, node): &mut (SndFireface, FwNode)) -> Result<(), Error> {
        self.meter_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.cfg_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.status_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.input_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.output_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.mixer_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.input_hpf_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.output_hpf_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.input_eq_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.output_eq_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.input_dyn_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.output_dyn_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.input_al_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.output_al_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.fx_source_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.fx_output_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.fx_reverb_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.fx_echo_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.meter_ctl.load(card_cntr)?;
        self.cfg_ctl.load(card_cntr)?;
        self.status_ctl.load(card_cntr)?;
        self.input_ctl.load(card_cntr)?;
        self.output_ctl.load(card_cntr)?;
        self.mixer_ctl.load(card_cntr)?;
        self.input_hpf_ctl.load(card_cntr)?;
        self.output_hpf_ctl.load(card_cntr)?;
        self.input_eq_ctl.load(card_cntr)?;
        self.output_eq_ctl.load(card_cntr)?;
        self.input_dyn_ctl.load(card_cntr)?;
        self.output_dyn_ctl.load(card_cntr)?;
        self.input_al_ctl.load(card_cntr)?;
        self.output_al_ctl.load(card_cntr)?;
        self.fx_source_ctl.load(card_cntr)?;
        self.fx_output_ctl.load(card_cntr)?;
        self.fx_reverb_ctl.load(card_cntr)?;
        self.fx_echo_ctl.load(card_cntr)?;
        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.cfg_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.status_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_hpf_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_hpf_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_eq_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_eq_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_dyn_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_dyn_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_al_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_al_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.fx_source_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.fx_output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.fx_reverb_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.fx_echo_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        (_, node): &mut (SndFireface, FwNode),
        elem_id: &ElemId,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .cfg_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .input_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .output_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .input_hpf_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .output_hpf_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .input_eq_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .output_eq_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .input_dyn_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .output_dyn_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .input_al_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .output_al_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .fx_source_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .fx_output_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .fx_reverb_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .fx_echo_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndFireface, FwNode)> for Ff802Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
        elem_id_list.extend_from_slice(&self.status_ctl.0);
    }

    fn measure_states(&mut self, (_, node): &mut (SndFireface, FwNode)) -> Result<(), Error> {
        self.meter_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.status_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        Ok(())
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

    fn cache(&mut self, req: &mut FwReq, node: &mut FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = Ff802Protocol::update_wholly(req, node, &self.0, timeout_ms);
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
            PRIMARY_CLK_SRC_NAME => {
                let pos = Self::CLK_SRCS
                    .iter()
                    .position(|src| self.0.clk_src.eq(src))
                    .unwrap() as u32;
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            SPDIF_INPUT_IFACE_NAME => {
                let pos = Self::SPDIF_IFACES
                    .iter()
                    .position(|iface| self.0.spdif_in_iface.eq(iface))
                    .unwrap() as u32;
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            OPT_OUTPUT_SIGNAL_NAME => {
                let pos = Self::OPT_OUT_SIGNALS
                    .iter()
                    .position(|fmt| self.0.opt_out_signal.eq(fmt))
                    .unwrap() as u32;
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            EFFECT_ON_INPUT_NAME => {
                elem_value.set_bool(&[self.0.effect_on_inputs]);
                Ok(true)
            }
            SPDIF_OUTPUT_FMT_NAME => {
                let pos = Self::SPDIF_FMTS
                    .iter()
                    .position(|fmt| self.0.spdif_out_format.eq(fmt))
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
                params.clk_src = Self::CLK_SRCS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of clock source: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = Ff802Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            SPDIF_INPUT_IFACE_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.spdif_in_iface = Self::SPDIF_IFACES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of S/PDIF iface: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = Ff802Protocol::update_wholly(req, node, &params, timeout_ms);
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
                let res = Ff802Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            EFFECT_ON_INPUT_NAME => {
                let mut params = self.0.clone();
                params.effect_on_inputs = elem_value.boolean()[0];
                let res = Ff802Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            SPDIF_OUTPUT_FMT_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.spdif_out_format = Self::SPDIF_FMTS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of S/PDIF format: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = Ff802Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            WORD_CLOCK_SINGLE_SPPED_NAME => {
                let mut params = self.0.clone();
                params.word_out_single = elem_value.boolean()[0];
                let res = Ff802Protocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct StatusCtl(Vec<ElemId>, Ff802Status);

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

    fn cache(&mut self, req: &mut FwReq, node: &mut FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = Ff802Protocol::cache_wholly(req, node, &mut self.1, timeout_ms);
        debug!(params = ?self.1, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        [EXT_SRC_LOCK_NAME, EXT_SRC_SYNC_NAME]
            .iter()
            .try_for_each(|name| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, name, 0);
                card_cntr
                    .add_bool_elems(&elem_id, 1, Self::EXT_CLK_SRCS.len(), false)
                    .map(|mut elem_id_list| self.0.append(&mut elem_id_list))
            })?;

        let labels: Vec<String> = Self::EXT_CLK_RATES
            .iter()
            .map(|r| optional_clk_nominal_rate_to_string(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, EXT_SRC_RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, Self::EXT_CLK_SRCS.len(), &labels, None, false)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let labels: Vec<String> = CfgCtl::CLK_SRCS
            .iter()
            .map(|r| clk_src_to_string(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, ACTIVE_CLK_SRC_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let labels: Vec<String> = CfgCtl::CLK_RATES
            .iter()
            .map(|r| clk_nominal_rate_to_string(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, ACTIVE_CLK_RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            EXT_SRC_LOCK_NAME => {
                let vals = [
                    self.1.ext_lock.adat_a,
                    self.1.ext_lock.adat_b,
                    self.1.ext_lock.spdif,
                    self.1.ext_lock.word_clk,
                ];
                elem_value.set_bool(&vals);
                Ok(true)
            }
            EXT_SRC_SYNC_NAME => {
                let vals = [
                    self.1.ext_sync.adat_a,
                    self.1.ext_sync.adat_b,
                    self.1.ext_sync.spdif,
                    self.1.ext_sync.word_clk,
                ];
                elem_value.set_bool(&vals);
                Ok(true)
            }
            EXT_SRC_RATE_NAME => {
                let vals: Vec<u32> = [
                    self.1.ext_rate.adat_a,
                    self.1.ext_rate.adat_b,
                    self.1.ext_rate.spdif,
                    self.1.ext_rate.word_clk,
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
                    .position(|r| r.eq(&self.1.active_clk_src))
                    .map(|p| p as u32)
                    .unwrap();
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            ACTIVE_CLK_RATE_NAME => {
                let pos = CfgCtl::CLK_RATES
                    .iter()
                    .position(|r| r.eq(&self.1.active_clk_rate))
                    .map(|p| p as u32)
                    .unwrap();
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
