// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    protocols::tcat::{
        extension::{
            peak_section::*,
            {caps_section::*, cmd_section::*, mixer_section::*, *},
            {current_config_section::*, standalone_section::*},
        },
        tcd22xx_spec::*,
    },
    std::marker::PhantomData,
};

#[derive(Default, Debug)]
pub struct Tcd22xxCtls<T>
where
    T: Tcd22xxSpecOperation + Tcd22xxRouterOperation + Tcd22xxMixerOperation,
{
    pub measured_elem_id_list: Vec<ElemId>,
    pub notified_elem_id_list: Vec<ElemId>,

    caps: ExtensionCaps,

    state: Tcd22xxState,

    supported_sources: Vec<ClockSource>,
    supported_source_labels: Vec<String>,
    supported_rates: Vec<ClockRate>,

    real_blk_pair: (Vec<SrcBlk>, Vec<DstBlk>),
    stream_blk_pair: (Vec<SrcBlk>, Vec<DstBlk>),
    mixer_blk_pair: (Vec<SrcBlk>, Vec<DstBlk>),

    current_rate: u32,

    standalone_ctls: StandaloneCtls<T>,
    mixer_ctls: MixerCtls<T>,
    router_ctls: RouterCtls<T>,
    meter_ctls: MeterCtls<T>,
}

impl<T> Tcd22xxCtls<T>
where
    T: Tcd22xxSpecOperation + Tcd22xxRouterOperation + Tcd22xxMixerOperation,
{
    pub fn cache_whole_params(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        global_params: &GlobalParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        CapsSectionProtocol::read_caps(req, node, sections, &mut self.caps, timeout_ms)?;

        self.supported_sources = global_params.avail_sources.to_vec();

        self.supported_source_labels = self
            .supported_sources
            .iter()
            .filter_map(|src| {
                global_params
                    .clock_source_labels
                    .iter()
                    .find(|(s, _)| src.eq(s))
                    .map(|(_, l)| l.to_string())
            })
            .collect();

        self.supported_rates = global_params.avail_rates.to_vec();

        self.real_blk_pair = T::compute_avail_real_blk_pair(RateMode::Low);

        // Compute the pair of blocks for tx/rx streams at each of available mode of rate. It's for
        // such models that second rx or tx stream is not available at mode of low rate.
        let mut rate_modes: Vec<RateMode> = Vec::default();
        global_params
            .avail_rates
            .iter()
            .map(|&r| RateMode::from(r))
            .for_each(|m| {
                if rate_modes.iter().find(|&&mode| mode.eq(&m)).is_none() {
                    rate_modes.push(m);
                }
            });
        rate_modes.iter().try_for_each(|&m| {
            CurrentConfigSectionProtocol::read_current_stream_format_entries(
                req, node, sections, &self.caps, m, timeout_ms,
            )
            .map(|(tx, rx)| {
                let (tx_blk, rx_blk) = T::compute_avail_stream_blk_pair(&tx, &rx);
                tx_blk.iter().for_each(|src| {
                    if self.stream_blk_pair.0.iter().find(|s| s.eq(&src)).is_none() {
                        self.stream_blk_pair.0.push(*src);
                    }
                });
                rx_blk.iter().for_each(|dst| {
                    if self.stream_blk_pair.1.iter().find(|d| d.eq(&dst)).is_none() {
                        self.stream_blk_pair.1.push(*dst);
                    }
                });
            })
        })?;
        self.stream_blk_pair.0.sort();
        self.stream_blk_pair.1.sort();

        self.mixer_blk_pair = T::compute_avail_mixer_blk_pair(&self.caps, RateMode::Low);

        T::cache(
            node,
            req,
            sections,
            &self.caps,
            &mut self.state,
            RateMode::from(global_params.clock_config.rate),
            timeout_ms,
        )?;
        self.current_rate = global_params.current_rate;

        self.standalone_ctls
            .cache(req, node, sections, timeout_ms)?;

        self.meter_ctls.cache(
            req,
            node,
            sections,
            &self.caps,
            &self.real_blk_pair.1,
            &self.stream_blk_pair.1,
            &self.mixer_blk_pair.1,
            timeout_ms,
        )?;

        Ok(())
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.standalone_ctls.load(
            card_cntr,
            &self.supported_sources,
            &self.supported_source_labels,
            &self.supported_rates,
        )?;

        self.mixer_ctls
            .load(card_cntr, &self.mixer_blk_pair)
            .map(|mut elem_id_list| self.notified_elem_id_list.append(&mut elem_id_list))?;

        self.router_ctls
            .load(
                card_cntr,
                &self.real_blk_pair,
                &self.stream_blk_pair,
                &self.mixer_blk_pair,
            )
            .map(|mut elem_id_list| self.notified_elem_id_list.append(&mut elem_id_list))?;

        self.meter_ctls
            .load(
                card_cntr,
                &self.real_blk_pair.1,
                &self.stream_blk_pair.1,
                &self.mixer_blk_pair.1,
            )
            .map(|mut elem_id_list| self.measured_elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub fn read(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        if self.standalone_ctls.read(
            elem_id,
            elem_value,
            &self.supported_sources,
            &self.supported_rates,
        )? {
            Ok(true)
        } else if self
            .mixer_ctls
            .read(elem_id, elem_value, &self.state.mixer_cache)?
        {
            Ok(true)
        } else if self.router_ctls.read(
            elem_id,
            elem_value,
            &self.state.router_entries,
            &self.real_blk_pair,
            &self.stream_blk_pair,
            &self.mixer_blk_pair,
        )? {
            Ok(true)
        } else if self.meter_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if self.standalone_ctls.write(
            req,
            node,
            sections,
            elem_id,
            elem_value,
            &self.supported_sources,
            &self.supported_rates,
            timeout_ms,
        )? {
            Ok(true)
        } else if self.mixer_ctls.write(
            req,
            node,
            sections,
            &self.caps,
            &mut self.state,
            elem_id,
            elem_value,
            timeout_ms,
        )? {
            Ok(true)
        } else if self.router_ctls.write(
            req,
            node,
            sections,
            &self.caps,
            elem_id,
            elem_value,
            &mut self.state,
            &self.real_blk_pair,
            &self.stream_blk_pair,
            &self.mixer_blk_pair,
            timeout_ms,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn cache_partial_params(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.meter_ctls.cache(
            req,
            node,
            sections,
            &self.caps,
            &self.real_blk_pair.1,
            &self.stream_blk_pair.1,
            &self.mixer_blk_pair.1,
            timeout_ms,
        )
    }

    pub fn parse_notification(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        global_params: &GlobalParameters,
        timeout_ms: u32,
        msg: u32,
    ) -> Result<(), Error> {
        if msg > 0 && global_params.current_rate != self.current_rate {
            T::cache(
                node,
                req,
                sections,
                &self.caps,
                &mut self.state,
                RateMode::from(global_params.clock_config.rate),
                timeout_ms,
            )?;
            self.current_rate = global_params.current_rate;
        }
        Ok(())
    }
}

#[derive(Default, Debug)]
struct StandaloneCtls<T>(StandaloneParameters, PhantomData<T>);

impl<T> StandaloneCtls<T>
where
    T: Tcd22xxSpecOperation + Tcd22xxRouterOperation + Tcd22xxMixerOperation,
{
    const ADAT_MODES: &'static [AdatParam] = &[
        AdatParam::Normal,
        AdatParam::SMUX2,
        AdatParam::SMUX4,
        AdatParam::Auto,
    ];

    const WC_MODES: &'static [WordClockMode] = &[
        WordClockMode::Normal,
        WordClockMode::Low,
        WordClockMode::Middle,
        WordClockMode::High,
    ];

    fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        StandaloneSectionProtocol::cache_standalone_params(
            req,
            node,
            sections,
            &mut self.0,
            timeout_ms,
        )
    }

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        supported_sources: &[ClockSource],
        supported_source_labels: &[String],
        supported_rates: &[ClockRate],
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, STANDALONE_CLK_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &supported_source_labels, None, true)?;

        if supported_sources
            .iter()
            .find(|&src| {
                src.eq(&ClockSource::Aes1)
                    || src.eq(&ClockSource::Aes2)
                    || src.eq(&ClockSource::Aes3)
                    || src.eq(&ClockSource::Aes4)
            })
            .is_some()
        {
            let elem_id = ElemId::new_by_name(
                ElemIfaceType::Card,
                0,
                0,
                STANDALONE_SPDIF_HIGH_RATE_NAME,
                0,
            );
            let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;
        }

        if supported_sources
            .iter()
            .find(|&src| src.eq(&ClockSource::Adat))
            .is_some()
        {
            let labels: Vec<&str> = Self::ADAT_MODES
                .iter()
                .map(|mode| adat_mode_to_str(mode))
                .collect();
            let elem_id =
                ElemId::new_by_name(ElemIfaceType::Card, 0, 0, STANDALONE_ADAT_MODE_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        }

        if supported_sources
            .iter()
            .find(|&src| src.eq(&ClockSource::WordClock))
            .is_some()
        {
            let labels: Vec<&str> = Self::WC_MODES
                .iter()
                .map(|mode| word_clock_mode_to_str(mode))
                .collect();
            let elem_id =
                ElemId::new_by_name(ElemIfaceType::Card, 0, 0, STANDALONE_WC_MODE_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

            let elem_id = ElemId::new_by_name(
                ElemIfaceType::Card,
                0,
                0,
                STANDALONE_WC_RATE_NUMERATOR_NAME,
                0,
            );
            let _ = card_cntr.add_int_elems(&elem_id, 1, 1, 4095, 1, 1, None, true)?;

            let elem_id = ElemId::new_by_name(
                ElemIfaceType::Card,
                0,
                0,
                STANDALONE_WC_RATE_DENOMINATOR_NAME,
                0,
            );
            let _ =
                card_cntr.add_int_elems(&elem_id, 1, 1, std::u16::MAX as i32, 1, 1, None, true)?;
        }

        let labels: Vec<String> = supported_rates
            .iter()
            .map(|r| clock_rate_to_string(r))
            .collect();

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Card,
            0,
            0,
            STANDALONE_INTERNAL_CLK_RATE_NAME,
            0,
        );
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read(
        &self,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        supported_sources: &[ClockSource],
        supported_rates: &[ClockRate],
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            STANDALONE_CLK_SRC_NAME => {
                let params = &self.0;
                let pos = supported_sources
                    .iter()
                    .position(|src| params.clock_source.eq(src))
                    .ok_or_else(|| {
                        let msg = format!(
                            "Unexpected value for source: {}",
                            clock_source_to_string(&params.clock_source)
                        );
                        Error::new(FileError::Nxio, &msg)
                    })?;
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            STANDALONE_SPDIF_HIGH_RATE_NAME => {
                let params = &self.0;
                elem_value.set_bool(&[params.aes_high_rate]);
                Ok(true)
            }
            STANDALONE_ADAT_MODE_NAME => {
                let params = &self.0;
                let pos = Self::ADAT_MODES
                    .iter()
                    .position(|src| params.adat_mode.eq(src))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            STANDALONE_WC_MODE_NAME => {
                let params = &self.0;
                let pos = Self::WC_MODES
                    .iter()
                    .position(|mode| params.word_clock_param.mode.eq(mode))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            STANDALONE_WC_RATE_NUMERATOR_NAME => {
                let params = &self.0;
                elem_value.set_int(&[params.word_clock_param.rate.numerator as i32]);
                Ok(true)
            }
            STANDALONE_WC_RATE_DENOMINATOR_NAME => {
                let params = &self.0;
                elem_value.set_int(&[params.word_clock_param.rate.denominator as i32]);
                Ok(true)
            }
            STANDALONE_INTERNAL_CLK_RATE_NAME => {
                let params = &self.0;
                let pos = supported_rates
                    .iter()
                    .position(|rate| params.internal_rate.eq(rate))
                    .ok_or_else(|| {
                        let msg = format!(
                            "Unexpected value for rate: {}",
                            clock_rate_to_string(&params.internal_rate)
                        );
                        Error::new(FileError::Nxio, &msg)
                    })?;
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        supported_sources: &[ClockSource],
        supported_rates: &[ClockRate],
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            STANDALONE_CLK_SRC_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                supported_sources
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of source: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&src| params.clock_source = src)?;
                StandaloneSectionProtocol::update_standalone_params(
                    req,
                    node,
                    &sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            STANDALONE_SPDIF_HIGH_RATE_NAME => {
                let mut params = self.0.clone();
                params.aes_high_rate = elem_value.boolean()[0];
                StandaloneSectionProtocol::update_standalone_params(
                    req,
                    node,
                    &sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            STANDALONE_ADAT_MODE_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::ADAT_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Standalone ADAT mode not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&mode| params.adat_mode = mode)?;
                StandaloneSectionProtocol::update_standalone_params(
                    req,
                    node,
                    &sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            STANDALONE_WC_MODE_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::WC_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg =
                            format!("Standalone Word Clock mode not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&mode| params.word_clock_param.mode = mode)?;
                StandaloneSectionProtocol::update_standalone_params(
                    req,
                    node,
                    &sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            STANDALONE_WC_RATE_NUMERATOR_NAME => {
                let mut params = self.0.clone();
                params.word_clock_param.rate.numerator = elem_value.int()[0] as u16;
                StandaloneSectionProtocol::update_standalone_params(
                    req,
                    node,
                    &sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            STANDALONE_WC_RATE_DENOMINATOR_NAME => {
                let mut params = self.0.clone();
                params.word_clock_param.rate.denominator = elem_value.int()[0] as u16;
                StandaloneSectionProtocol::update_standalone_params(
                    req,
                    node,
                    &sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            STANDALONE_INTERNAL_CLK_RATE_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                supported_rates
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of rate: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&rate| params.internal_rate = rate)?;
                StandaloneSectionProtocol::update_standalone_params(
                    req,
                    node,
                    &sections,
                    &params,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct MixerCtls<T>(PhantomData<T>);

impl<T> MixerCtls<T>
where
    T: Tcd22xxSpecOperation + Tcd22xxRouterOperation + Tcd22xxMixerOperation,
{
    const COEF_MIN: i32 = 0;
    const COEF_MAX: i32 = 0x0000ffffi32; // 2:14 Fixed-point.
    const COEF_STEP: i32 = 1;
    const COEF_TLV: DbInterval = DbInterval {
        min: -6000,
        max: 400,
        linear: false,
        mute_avail: false,
    };

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        (mixer_blk_srcs, mixer_blk_dsts): &(Vec<SrcBlk>, Vec<DstBlk>),
    ) -> Result<Vec<ElemId>, Error> {
        eprintln!("{:?} {:?}", mixer_blk_srcs, mixer_blk_dsts);
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SRC_GAIN_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            mixer_blk_srcs.len(),
            Self::COEF_MIN,
            Self::COEF_MAX,
            Self::COEF_STEP,
            mixer_blk_dsts.len(),
            Some(&Into::<Vec<u32>>::into(Self::COEF_TLV)),
            true,
        )
    }

    fn read(
        &self,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        mixer_cache: &[Vec<i32>],
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_SRC_GAIN_NAME => {
                let dst_ch = elem_id.index() as usize;
                mixer_cache
                    .iter()
                    .nth(dst_ch)
                    .map(|coefs| elem_value.set_int(coefs))
                    .unwrap();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        state: &mut Tcd22xxState,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_SRC_GAIN_NAME => {
                let dst_ch = elem_id.index() as usize;
                let mut params = state.mixer_cache.clone();
                params
                    .iter_mut()
                    .nth(dst_ch)
                    .ok_or_else(|| {
                        let msg = format!("Mixer destination not found for position {}", dst_ch);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|coefs| {
                        coefs
                            .iter_mut()
                            .zip(elem_value.int())
                            .for_each(|(coef, &val)| *coef = val);
                    })?;
                T::update_mixer_coef(node, req, sections, caps, state, &params, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct RouterCtls<T>(PhantomData<T>)
where
    T: Tcd22xxSpecOperation + Tcd22xxRouterOperation + Tcd22xxMixerOperation;

impl<T> RouterCtls<T>
where
    T: Tcd22xxSpecOperation + Tcd22xxRouterOperation + Tcd22xxMixerOperation,
{
    const NONE_SRC_LABEL: &'static str = "None";

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        real_blk_pair: &(Vec<SrcBlk>, Vec<DstBlk>),
        stream_blk_pair: &(Vec<SrcBlk>, Vec<DstBlk>),
        mixer_blk_pair: &(Vec<SrcBlk>, Vec<DstBlk>),
    ) -> Result<Vec<ElemId>, Error> {
        let mut notified_elem_id_list = Vec::new();

        Self::add_an_elem_for_src(
            card_cntr,
            ROUTER_OUT_SRC_NAME,
            &real_blk_pair.1,
            &[&real_blk_pair.0, &stream_blk_pair.0, &mixer_blk_pair.0],
        )
        .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Self::add_an_elem_for_src(
            card_cntr,
            ROUTER_CAP_SRC_NAME,
            &stream_blk_pair.1,
            &[&real_blk_pair.0, &mixer_blk_pair.0],
        )
        .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Self::add_an_elem_for_src(
            card_cntr,
            ROUTER_MIXER_SRC_NAME,
            &mixer_blk_pair.1,
            &[&real_blk_pair.0, &stream_blk_pair.0],
        )
        .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn add_an_elem_for_src(
        card_cntr: &mut CardCntr,
        label: &str,
        dsts: &[DstBlk],
        srcs: &[&[SrcBlk]],
    ) -> Result<Vec<ElemId>, Error> {
        let targets: Vec<String> = dsts.iter().map(|&dst| T::dst_blk_label(dst)).collect();

        let mut sources: Vec<String> = srcs
            .iter()
            .flat_map(|srcs| *srcs)
            .map(|src| T::src_blk_label(src))
            .collect();
        sources.insert(0, Self::NONE_SRC_LABEL.to_string());

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, label, 0);
        card_cntr.add_enum_elems(&elem_id, 1, targets.len(), &sources, None, true)
    }

    fn read(
        &self,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        router_entries: &[RouterEntry],
        real_blk_pair: &(Vec<SrcBlk>, Vec<DstBlk>),
        stream_blk_pair: &(Vec<SrcBlk>, Vec<DstBlk>),
        mixer_blk_pair: &(Vec<SrcBlk>, Vec<DstBlk>),
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ROUTER_OUT_SRC_NAME => Self::read_elem_src(
                elem_value,
                router_entries,
                &real_blk_pair.1,
                &[&real_blk_pair.0, &stream_blk_pair.0, &mixer_blk_pair.0],
            ),
            ROUTER_CAP_SRC_NAME => Self::read_elem_src(
                elem_value,
                router_entries,
                &stream_blk_pair.1,
                &[&real_blk_pair.0, &mixer_blk_pair.0],
            ),
            ROUTER_MIXER_SRC_NAME => Self::read_elem_src(
                elem_value,
                router_entries,
                &mixer_blk_pair.1,
                &[&real_blk_pair.0, &stream_blk_pair.0],
            ),
            _ => Ok(false),
        }
    }

    fn read_elem_src(
        elem_value: &ElemValue,
        router_entries: &[RouterEntry],
        dsts: &[DstBlk],
        srcs: &[&[SrcBlk]],
    ) -> Result<bool, Error> {
        let vals: Vec<u32> = dsts
            .iter()
            .map(|dst| {
                router_entries
                    .iter()
                    .find(|entry| dst.eq(&entry.dst))
                    .and_then(|entry| {
                        srcs.iter()
                            .flat_map(|&list| list)
                            .position(|src| entry.src.eq(src))
                            .map(|pos| 1 + pos as u32)
                    })
                    .unwrap_or(0)
            })
            .collect();
        elem_value.set_enum(&vals);
        Ok(true)
    }

    fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        state: &mut Tcd22xxState,
        real_blk_pair: &(Vec<SrcBlk>, Vec<DstBlk>),
        stream_blk_pair: &(Vec<SrcBlk>, Vec<DstBlk>),
        mixer_blk_pair: &(Vec<SrcBlk>, Vec<DstBlk>),
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ROUTER_OUT_SRC_NAME => Self::write_elem_src(
                req,
                node,
                sections,
                caps,
                elem_value,
                state,
                &real_blk_pair.1,
                &[&real_blk_pair.0, &stream_blk_pair.0, &mixer_blk_pair.0],
                timeout_ms,
            ),
            ROUTER_CAP_SRC_NAME => Self::write_elem_src(
                req,
                node,
                sections,
                caps,
                elem_value,
                state,
                &stream_blk_pair.1,
                &[&real_blk_pair.0, &mixer_blk_pair.0],
                timeout_ms,
            ),
            ROUTER_MIXER_SRC_NAME => Self::write_elem_src(
                req,
                node,
                sections,
                caps,
                elem_value,
                state,
                &mixer_blk_pair.1,
                &[&real_blk_pair.0, &stream_blk_pair.0],
                timeout_ms,
            ),
            _ => Ok(false),
        }
    }

    fn write_elem_src(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        elem_value: &ElemValue,
        state: &mut Tcd22xxState,
        dsts: &[DstBlk],
        srcs: &[&[SrcBlk]],
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        let mut entries = state.router_entries.clone();

        dsts.iter()
            .zip(elem_value.enumerated())
            .for_each(|(dst, &val)| {
                let src = if val == 0 {
                    SrcBlk {
                        id: SrcBlkId::Reserved(0xff),
                        ch: 0xff,
                    }
                } else {
                    let pos = (val - 1) as usize;
                    srcs.iter()
                        .flat_map(|&list| list)
                        .nth(pos)
                        .copied()
                        .unwrap_or_else(|| SrcBlk {
                            id: SrcBlkId::Reserved(0xff),
                            ch: 0xff,
                        })
                };

                match entries.iter_mut().find(|entry| entry.dst.eq(dst)) {
                    Some(entry) => entry.src = src,
                    None => entries.push(RouterEntry {
                        dst: *dst,
                        src,
                        ..Default::default()
                    }),
                }
            });

        T::update_router_entries(node, req, sections, caps, state, entries, timeout_ms)
            .map(|_| true)
    }
}

#[derive(Default, Debug)]
struct MeterCtls<T>
where
    T: Tcd22xxSpecOperation + Tcd22xxRouterOperation + Tcd22xxMixerOperation,
{
    real_meter: Vec<i32>,
    stream_meter: Vec<i32>,
    mixer_meter: Vec<i32>,

    out_sat: Vec<bool>,

    _phantom: PhantomData<T>,
}

impl<T> MeterCtls<T>
where
    T: Tcd22xxSpecOperation + Tcd22xxRouterOperation + Tcd22xxMixerOperation,
{
    const COEF_MIN: i32 = 0;
    const COEF_MAX: i32 = 0x00000fffi32; // Upper 12 bits of each sample.
    const COEF_STEP: i32 = 1;

    fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        real_blk_dsts: &[DstBlk],
        stream_blk_dsts: &[DstBlk],
        mixer_blk_dsts: &[DstBlk],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let entries =
            PeakSectionProtocol::read_peak_entries(req, node, sections, &caps, timeout_ms)?;

        self.real_meter
            .resize_with(real_blk_dsts.len(), Default::default);
        self.stream_meter
            .resize_with(real_blk_dsts.len(), Default::default);
        self.mixer_meter
            .resize_with(real_blk_dsts.len(), Default::default);

        self.real_meter
            .iter_mut()
            .chain(&mut self.stream_meter)
            .chain(&mut self.mixer_meter)
            .zip(
                real_blk_dsts
                    .iter()
                    .chain(stream_blk_dsts)
                    .chain(mixer_blk_dsts),
            )
            .for_each(|(meter, dst)| {
                *meter = entries
                    .iter()
                    .find(|entry| dst.eq(&entry.dst))
                    .map(|entry| entry.peak as i32)
                    .unwrap_or(0);
            });

        self.out_sat =
            MixerSectionProtocol::read_saturation(req, node, sections, &caps, timeout_ms)?;

        Ok(())
    }

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        real_blk_dsts: &[DstBlk],
        stream_blk_dsts: &[DstBlk],
        mixer_blk_dsts: &[DstBlk],
    ) -> Result<Vec<ElemId>, Error> {
        let mut measured_elem_id_list = Vec::new();

        Self::add_an_elem_for_meter(card_cntr, OUT_METER_NAME, real_blk_dsts)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        Self::add_an_elem_for_meter(card_cntr, STREAM_TX_METER_NAME, stream_blk_dsts)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        Self::add_an_elem_for_meter(card_cntr, MIXER_INPUT_METER_NAME, mixer_blk_dsts)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_SATURATION_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, mixer_blk_dsts.len(), false)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        Ok(measured_elem_id_list)
    }

    fn add_an_elem_for_meter(
        card_cntr: &mut CardCntr,
        label: &str,
        targets: &[DstBlk],
    ) -> Result<Vec<ElemId>, Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, label, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::COEF_MIN,
            Self::COEF_MAX,
            Self::COEF_STEP,
            targets.len(),
            None,
            false,
        )
    }

    fn read(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUT_METER_NAME => {
                elem_value.set_int(&self.real_meter);
                Ok(true)
            }
            STREAM_TX_METER_NAME => {
                elem_value.set_int(&self.stream_meter);
                Ok(true)
            }
            MIXER_INPUT_METER_NAME => {
                elem_value.set_int(&self.mixer_meter);
                Ok(true)
            }
            INPUT_SATURATION_NAME => {
                elem_value.set_bool(&self.out_sat);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

const OUT_METER_NAME: &str = "output-source-meter";
const STREAM_TX_METER_NAME: &str = "stream-source-meter";
const MIXER_INPUT_METER_NAME: &str = "mixer-source-meter";
const INPUT_SATURATION_NAME: &str = "mixer-out-saturation";

const ROUTER_OUT_SRC_NAME: &str = "output-source";
const ROUTER_CAP_SRC_NAME: &str = "stream-source";
const ROUTER_MIXER_SRC_NAME: &str = "mixer-source";

const MIXER_SRC_GAIN_NAME: &str = "mixer-source-gain";

const STANDALONE_CLK_SRC_NAME: &str = "standalone-clock-source";
const STANDALONE_SPDIF_HIGH_RATE_NAME: &str = "standalone-spdif-high-rate";
const STANDALONE_ADAT_MODE_NAME: &str = "standalone-adat-mode";
const STANDALONE_WC_MODE_NAME: &str = "standalone-word-clock-mode";
const STANDALONE_WC_RATE_NUMERATOR_NAME: &str = "standalone-word-clock-rate-numerator";
const STANDALONE_WC_RATE_DENOMINATOR_NAME: &str = "standalone-word-clock-rate-denominator";
const STANDALONE_INTERNAL_CLK_RATE_NAME: &str = "standalone-internal-clock-rate";

fn adat_mode_to_str(mode: &AdatParam) -> &str {
    match mode {
        AdatParam::Normal => "Normal",
        AdatParam::SMUX2 => "S/MUX2",
        AdatParam::SMUX4 => "S/MUX4",
        AdatParam::Auto => "Auto",
    }
}

fn word_clock_mode_to_str(mode: &WordClockMode) -> &str {
    match mode {
        WordClockMode::Normal => "Normal",
        WordClockMode::Low => "Low",
        WordClockMode::Middle => "Middle",
        WordClockMode::High => "High",
    }
}
