// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::{latter_ctls::*, *},
    protocols::{latter::ucx::*, *},
};

#[derive(Default, Debug)]
pub struct UcxModel {
    req: FwReq,
    meter_ctl: LatterMeterCtl<FfUcxProtocol>,
    dsp_ctl: LatterDspCtl<FfUcxProtocol>,
    cfg_ctl: CfgCtl,
    status_ctl: StatusCtl,
}

const TIMEOUT_MS: u32 = 100;

impl UcxModel {
    pub fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.meter_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.dsp_ctl.cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.cfg_ctl.cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.status_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;

        Ok(())
    }
}

impl CtlModel<(SndUnit, FwNode)> for UcxModel {
    fn load(&mut self, _: &mut (SndUnit, FwNode), card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.meter_ctl.load(card_cntr)?;
        self.dsp_ctl.load(card_cntr)?;
        self.cfg_ctl.load(card_cntr)?;
        self.status_ctl.load(card_cntr)?;
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
        } else if self.status_ctl.read(elem_id, elem_value)? {
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
            .write(&mut self.req, &mut unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndUnit, FwNode)> for UcxModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
        elem_id_list.extend_from_slice(&self.status_ctl.0);
    }

    fn measure_states(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.meter_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.status_ctl
            .cache(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
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
        } else if self.status_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

fn clk_src_to_string(src: &FfUcxClkSrc) -> String {
    match src {
        FfUcxClkSrc::Internal => "Internal",
        FfUcxClkSrc::Coax => "Coaxial",
        FfUcxClkSrc::Opt => "Optical",
        FfUcxClkSrc::WordClk => "Word-clock",
    }
    .to_string()
}

#[derive(Default, Debug)]
struct CfgCtl(FfUcxConfig);

const PRIMARY_CLK_SRC_NAME: &str = "primary-clock-source";
const OPT_OUTPUT_SIGNAL_NAME: &str = "optical-output-signal";
const EFFECT_ON_INPUT_NAME: &str = "effect-on-input";
const SPDIF_OUTPUT_FMT_NAME: &str = "spdif-output-format";
const WORD_CLOCK_SINGLE_SPPED_NAME: &str = "word-clock-single-speed";
const WORD_CLOCK_IN_TERMINATE_NAME: &str = "word-clock-input-terminate";

impl CfgCtl {
    const CLK_SRCS: [FfUcxClkSrc; 4] = [
        FfUcxClkSrc::Internal,
        FfUcxClkSrc::Coax,
        FfUcxClkSrc::Opt,
        FfUcxClkSrc::WordClk,
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
        let res = FfUcxProtocol::update_wholly(req, node, &self.0, timeout_ms);
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

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, WORD_CLOCK_IN_TERMINATE_NAME, 0);
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
            WORD_CLOCK_IN_TERMINATE_NAME => {
                elem_value.set_bool(&[self.0.word_in_terminate]);
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
                let res = FfUcxProtocol::update_wholly(req, node, &params, timeout_ms);
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
                            format!("Invalid value for index of optical output signal: {}", pos,);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = FfUcxProtocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            EFFECT_ON_INPUT_NAME => {
                let mut params = self.0.clone();
                params.effect_on_inputs = elem_value.boolean()[0];
                let res = FfUcxProtocol::update_wholly(req, node, &params, timeout_ms);
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
                let res = FfUcxProtocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            WORD_CLOCK_SINGLE_SPPED_NAME => {
                let mut params = self.0.clone();
                params.word_out_single = elem_value.boolean()[0];
                let res = FfUcxProtocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            WORD_CLOCK_IN_TERMINATE_NAME => {
                let mut params = self.0.clone();
                params.word_in_terminate = elem_value.boolean()[0];
                let res = FfUcxProtocol::update_wholly(req, node, &params, timeout_ms);
                debug!(?params, ?res);
                self.0 = params;
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct StatusCtl(Vec<ElemId>, FfUcxStatus);

const EXT_SRC_LOCK_NAME: &str = "external-source-lock";
const EXT_SRC_SYNC_NAME: &str = "external-source-sync";
const EXT_SRC_RATE_NAME: &str = "external-source-rate";
const ACTIVE_CLK_RATE_NAME: &str = "active-clock-rate";
const ACTIVE_CLK_SRC_NAME: &str = "active-clock-source";

impl StatusCtl {
    const EXT_CLK_SRCS: [FfUcxClkSrc; 3] =
        [FfUcxClkSrc::Coax, FfUcxClkSrc::Opt, FfUcxClkSrc::WordClk];

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
        let res = FfUcxProtocol::cache_wholly(req, node, &mut self.1, timeout_ms);
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
                    self.1.ext_lock.coax_iface,
                    self.1.ext_lock.opt_iface,
                    self.1.ext_lock.word_clk,
                ];
                elem_value.set_bool(&vals);
                Ok(true)
            }
            EXT_SRC_SYNC_NAME => {
                let vals = [
                    self.1.ext_sync.coax_iface,
                    self.1.ext_sync.opt_iface,
                    self.1.ext_sync.word_clk,
                ];
                elem_value.set_bool(&vals);
                Ok(true)
            }
            EXT_SRC_RATE_NAME => {
                let vals: Vec<u32> = [
                    self.1.ext_rate.coax_iface,
                    self.1.ext_rate.opt_iface,
                    self.1.ext_rate.word_clk,
                ]
                .iter()
                .map(|rate| Self::EXT_CLK_RATES.iter().position(|r| r.eq(rate)).unwrap() as u32)
                .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            ACTIVE_CLK_SRC_NAME => {
                let pos = CfgCtl::CLK_SRCS
                    .iter()
                    .position(|r| r.eq(&self.1.active_clk_src))
                    .unwrap() as u32;
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            ACTIVE_CLK_RATE_NAME => {
                let pos = CfgCtl::CLK_RATES
                    .iter()
                    .position(|r| r.eq(&self.1.active_clk_rate))
                    .unwrap() as u32;
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
