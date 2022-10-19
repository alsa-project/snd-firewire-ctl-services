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

    _phantom: PhantomData<T>,
}

impl<T> Tcd22xxCtls<T>
where
    T: Tcd22xxSpecOperation + Tcd22xxRouterOperation + Tcd22xxMixerOperation,
{
    pub fn cache_whole_params(
        &mut self,
        _: &mut FwReq,
        _: &mut FwNode,
        _: &ExtensionSections,
        _: &GlobalParameters,
        _: u32,
    ) -> Result<(), Error> {
        Ok(())
    }

    pub fn load(&mut self, _: &mut CardCntr) -> Result<(), Error> {
        Ok(())
    }

    pub fn read(&self, _: &ElemId, _: &ElemValue) -> Result<bool, Error> {
        Ok(false)
    }

    pub fn write(
        &mut self,
        _: &mut FwReq,
        _: &mut FwNode,
        _: &ExtensionSections,
        _: &ElemId,
        _: &ElemValue,
        _: u32,
    ) -> Result<bool, Error> {
        Ok(false)
    }

    pub fn cache_partial_params(
        &mut self,
        _: &mut FwReq,
        _: &mut FwNode,
        _: &ExtensionSections,
        _: u32,
    ) -> Result<(), Error> {
        Ok(())
    }

    pub fn parse_notification(
        &mut self,
        _: &mut FwReq,
        _: &mut FwNode,
        _: &ExtensionSections,
        _: &GlobalParameters,
        _: u32,
        _: u32,
    ) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct Tcd22xxCtl {
    state: Tcd22xxState,
    caps: ExtensionCaps,
    meter_ctl: MeterCtl,
    router_ctl: RouterCtl,
    mixer_ctl: MixerCtl,
    standalone_ctl: StandaloneCtl,
}

pub trait Tcd22xxCtlOperation<T>
where
    T: Tcd22xxSpecOperation + Tcd22xxRouterOperation + Tcd22xxMixerOperation,
{
    fn tcd22xx_ctl(&self) -> &Tcd22xxCtl;
    fn tcd22xx_ctl_mut(&mut self) -> &mut Tcd22xxCtl;
}

#[derive(Default, Debug)]
struct MeterCtl {
    // Maximum number block at low rate mode.
    real_blk_dsts: Vec<DstBlk>,
    stream_blk_dsts: Vec<DstBlk>,
    mixer_blk_dsts: Vec<DstBlk>,

    real_meter: Vec<i32>,
    stream_meter: Vec<i32>,
    mixer_meter: Vec<i32>,

    out_sat: Vec<bool>,

    measured_elem_list: Vec<ElemId>,
}

const OUT_METER_NAME: &str = "output-source-meter";
const STREAM_TX_METER_NAME: &str = "stream-source-meter";
const MIXER_INPUT_METER_NAME: &str = "mixer-source-meter";
const INPUT_SATURATION_NAME: &str = "mixer-out-saturation";

pub trait MeterCtlOperation<T>: Tcd22xxCtlOperation<T>
where
    T: Tcd22xxSpecOperation + Tcd22xxRouterOperation + Tcd22xxMixerOperation,
{
    const COEF_MIN: i32 = 0;
    const COEF_MAX: i32 = 0x00000fffi32; // Upper 12 bits of each sample.
    const COEF_STEP: i32 = 1;

    fn cache_meter(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let ctls = self.tcd22xx_ctl_mut();

        let (_, real_blk_dsts) = T::compute_avail_real_blk_pair(RateMode::Low);
        ctls.meter_ctl.real_meter = vec![0; real_blk_dsts.len()];
        ctls.meter_ctl.real_blk_dsts = real_blk_dsts;

        let (tx_entries, rx_entries) =
            CurrentConfigSectionProtocol::read_current_stream_format_entries(
                req,
                node,
                sections,
                &ctls.caps,
                RateMode::Low,
                timeout_ms,
            )?;
        let (_, stream_blk_dsts) = T::compute_avail_stream_blk_pair(&tx_entries, &rx_entries);
        ctls.meter_ctl.stream_meter = vec![0; stream_blk_dsts.len()];
        ctls.meter_ctl.stream_blk_dsts = stream_blk_dsts;

        let (_, mixer_blk_dsts) = T::compute_avail_mixer_blk_pair(&ctls.caps, RateMode::Low);
        ctls.meter_ctl.mixer_meter = vec![0; mixer_blk_dsts.len()];
        ctls.meter_ctl.out_sat = vec![false; mixer_blk_dsts.len()];
        ctls.meter_ctl.mixer_blk_dsts = mixer_blk_dsts;

        Ok(())
    }

    fn load_meter(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let ctl = &mut self.tcd22xx_ctl_mut().meter_ctl;

        Self::add_an_elem_for_meter(card_cntr, OUT_METER_NAME, &ctl.real_blk_dsts)
            .map(|mut elem_id_list| ctl.measured_elem_list.append(&mut elem_id_list))?;

        Self::add_an_elem_for_meter(card_cntr, STREAM_TX_METER_NAME, &ctl.stream_blk_dsts)
            .map(|mut elem_id_list| ctl.measured_elem_list.append(&mut elem_id_list))?;

        Self::add_an_elem_for_meter(card_cntr, MIXER_INPUT_METER_NAME, &ctl.mixer_blk_dsts)
            .map(|mut elem_id_list| ctl.measured_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_SATURATION_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, ctl.mixer_blk_dsts.len(), false)
            .map(|mut elem_id_list| ctl.measured_elem_list.append(&mut elem_id_list))?;

        Ok(())
    }

    fn add_an_elem_for_meter(
        card_cntr: &mut CardCntr,
        label: &str,
        targets: &Vec<DstBlk>,
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

    fn measure_states_meter(
        &mut self,
        node: &mut FwNode,
        req: &mut FwReq,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let ctls = &mut self.tcd22xx_ctl_mut();

        let entries =
            PeakSectionProtocol::read_peak_entries(req, node, sections, &ctls.caps, timeout_ms)?;

        ctls.meter_ctl
            .real_meter
            .iter_mut()
            .chain(&mut ctls.meter_ctl.stream_meter)
            .chain(&mut ctls.meter_ctl.mixer_meter)
            .zip(
                ctls.meter_ctl
                    .real_blk_dsts
                    .iter()
                    .chain(&ctls.meter_ctl.stream_blk_dsts)
                    .chain(&ctls.meter_ctl.mixer_blk_dsts),
            )
            .for_each(|(val, dst)| {
                *val = entries
                    .iter()
                    .find(|entry| entry.dst.eq(dst))
                    .map(|entry| entry.peak as i32)
                    .unwrap_or(0);
            });

        ctls.meter_ctl.out_sat =
            MixerSectionProtocol::read_saturation(req, node, sections, &ctls.caps, timeout_ms)?;

        Ok(())
    }

    fn read_meter(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUT_METER_NAME => {
                elem_value.set_int(&self.tcd22xx_ctl().meter_ctl.real_meter);
                Ok(true)
            }
            STREAM_TX_METER_NAME => {
                elem_value.set_int(&self.tcd22xx_ctl().meter_ctl.stream_meter);
                Ok(true)
            }
            MIXER_INPUT_METER_NAME => {
                elem_value.set_int(&self.tcd22xx_ctl().meter_ctl.mixer_meter);
                Ok(true)
            }
            INPUT_SATURATION_NAME => {
                elem_value.set_bool(&self.tcd22xx_ctl().meter_ctl.out_sat);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

impl<O, T> MeterCtlOperation<T> for O
where
    O: Tcd22xxCtlOperation<T>,
    T: Tcd22xxSpecOperation + Tcd22xxRouterOperation + Tcd22xxMixerOperation,
{
}

#[derive(Default, Debug)]
struct RouterCtl {
    // Maximum number block in low rate mode.
    real_blk_pair: (Vec<SrcBlk>, Vec<DstBlk>),
    stream_blk_pair: (Vec<SrcBlk>, Vec<DstBlk>),
    mixer_blk_pair: (Vec<SrcBlk>, Vec<DstBlk>),
    notified_elem_list: Vec<ElemId>,
}

const ROUTER_OUT_SRC_NAME: &str = "output-source";
const ROUTER_CAP_SRC_NAME: &str = "stream-source";
const ROUTER_MIXER_SRC_NAME: &str = "mixer-source";

pub trait RouterCtlOperation<T: Tcd22xxRouterOperation>: Tcd22xxCtlOperation<T>
where
    T: Tcd22xxSpecOperation + Tcd22xxRouterOperation + Tcd22xxMixerOperation,
{
    const NONE_SRC_LABEL: &'static str = "None";

    fn cache_router(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        global_params: &GlobalParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let ctls = &mut self.tcd22xx_ctl_mut();

        ctls.router_ctl.real_blk_pair = T::compute_avail_real_blk_pair(RateMode::Low);

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
                req, node, sections, &ctls.caps, m, timeout_ms,
            )
            .map(|(tx, rx)| {
                let (tx_blk, rx_blk) = T::compute_avail_stream_blk_pair(&tx, &rx);
                tx_blk.iter().for_each(|src| {
                    if ctls
                        .router_ctl
                        .stream_blk_pair
                        .0
                        .iter()
                        .find(|s| s.eq(&src))
                        .is_none()
                    {
                        ctls.router_ctl.stream_blk_pair.0.push(*src);
                    }
                });
                rx_blk.iter().for_each(|dst| {
                    if ctls
                        .router_ctl
                        .stream_blk_pair
                        .1
                        .iter()
                        .find(|d| d.eq(&dst))
                        .is_none()
                    {
                        ctls.router_ctl.stream_blk_pair.1.push(*dst);
                    }
                });
            })
        })?;

        ctls.router_ctl.stream_blk_pair.0.sort();
        ctls.router_ctl.stream_blk_pair.1.sort();

        ctls.router_ctl.mixer_blk_pair = T::compute_avail_mixer_blk_pair(&ctls.caps, RateMode::Low);

        Ok(())
    }

    fn load_router(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let ctl = &mut self.tcd22xx_ctl_mut().router_ctl;

        Self::add_an_elem_for_src(
            card_cntr,
            ROUTER_OUT_SRC_NAME,
            &ctl.real_blk_pair.1,
            &[
                &ctl.real_blk_pair.0,
                &ctl.stream_blk_pair.0,
                &ctl.mixer_blk_pair.0,
            ],
        )
        .map(|mut elem_id_list| ctl.notified_elem_list.append(&mut elem_id_list))?;

        Self::add_an_elem_for_src(
            card_cntr,
            ROUTER_CAP_SRC_NAME,
            &ctl.stream_blk_pair.1,
            &[&ctl.real_blk_pair.0, &ctl.mixer_blk_pair.0],
        )
        .map(|mut elem_id_list| ctl.notified_elem_list.append(&mut elem_id_list))?;

        Self::add_an_elem_for_src(
            card_cntr,
            ROUTER_MIXER_SRC_NAME,
            &ctl.mixer_blk_pair.1,
            &[&ctl.real_blk_pair.0, &ctl.stream_blk_pair.0],
        )
        .map(|mut elem_id_list| ctl.notified_elem_list.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read_router(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ROUTER_OUT_SRC_NAME => {
                let ctls = self.tcd22xx_ctl();
                Self::read_elem_src(
                    &ctls.state,
                    elem_value,
                    &ctls.router_ctl.real_blk_pair.1,
                    &[
                        &ctls.router_ctl.real_blk_pair.0,
                        &ctls.router_ctl.stream_blk_pair.0,
                        &ctls.router_ctl.mixer_blk_pair.0,
                    ],
                );
                Ok(true)
            }
            ROUTER_CAP_SRC_NAME => {
                let ctls = self.tcd22xx_ctl();
                Self::read_elem_src(
                    &ctls.state,
                    elem_value,
                    &ctls.router_ctl.stream_blk_pair.1,
                    &[
                        &ctls.router_ctl.real_blk_pair.0,
                        &ctls.router_ctl.mixer_blk_pair.0,
                    ],
                );
                Ok(true)
            }
            ROUTER_MIXER_SRC_NAME => {
                let ctls = self.tcd22xx_ctl();
                Self::read_elem_src(
                    &ctls.state,
                    elem_value,
                    &ctls.router_ctl.mixer_blk_pair.1,
                    &[
                        &ctls.router_ctl.real_blk_pair.0,
                        &ctls.router_ctl.stream_blk_pair.0,
                    ],
                );
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_router(
        &mut self,
        node: &mut FwNode,
        req: &mut FwReq,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ROUTER_OUT_SRC_NAME => {
                let ctls = self.tcd22xx_ctl_mut();
                Self::write_elem_src(
                    &mut ctls.state,
                    node,
                    req,
                    sections,
                    &ctls.caps,
                    old,
                    new,
                    &ctls.router_ctl.real_blk_pair.1,
                    &[
                        &ctls.router_ctl.real_blk_pair.0,
                        &ctls.router_ctl.stream_blk_pair.0,
                        &ctls.router_ctl.mixer_blk_pair.0,
                    ],
                    timeout_ms,
                )
                .map(|_| true)
            }
            ROUTER_CAP_SRC_NAME => {
                let ctls = self.tcd22xx_ctl_mut();
                Self::write_elem_src(
                    &mut ctls.state,
                    node,
                    req,
                    sections,
                    &ctls.caps,
                    old,
                    new,
                    &ctls.router_ctl.stream_blk_pair.1,
                    &[
                        &ctls.router_ctl.real_blk_pair.0,
                        &ctls.router_ctl.mixer_blk_pair.0,
                    ],
                    timeout_ms,
                )
                .map(|_| true)
            }
            ROUTER_MIXER_SRC_NAME => {
                let ctls = self.tcd22xx_ctl_mut();
                Self::write_elem_src(
                    &mut ctls.state,
                    node,
                    req,
                    sections,
                    &ctls.caps,
                    old,
                    new,
                    &ctls.router_ctl.mixer_blk_pair.1,
                    &[
                        &ctls.router_ctl.real_blk_pair.0,
                        &ctls.router_ctl.stream_blk_pair.0,
                    ],
                    timeout_ms,
                )
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn add_an_elem_for_src(
        card_cntr: &mut CardCntr,
        label: &str,
        dsts: &[DstBlk],
        srcs: &[&[SrcBlk]],
    ) -> Result<Vec<ElemId>, Error> {
        let targets = dsts
            .iter()
            .map(|&dst| T::dst_blk_label(dst))
            .collect::<Vec<String>>();
        let mut sources = srcs
            .iter()
            .flat_map(|srcs| *srcs)
            .map(|src| T::src_blk_label(src))
            .collect::<Vec<String>>();
        sources.insert(0, Self::NONE_SRC_LABEL.to_string());

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, label, 0);
        card_cntr.add_enum_elems(&elem_id, 1, targets.len(), &sources, None, true)
    }

    fn read_elem_src(
        state: &Tcd22xxState,
        elem_value: &ElemValue,
        dsts: &[DstBlk],
        srcs: &[&[SrcBlk]],
    ) {
        let _ = ElemValueAccessor::<u32>::set_vals(elem_value, dsts.len(), |idx| {
            let dst = dsts[idx];

            let val = state
                .router_entries
                .iter()
                .find(|entry| entry.dst.eq(&dst))
                .and_then(|entry| {
                    srcs.iter()
                        .flat_map(|srcs| *srcs)
                        .position(|src| entry.src.eq(src))
                        .map(|pos| 1 + pos as u32)
                })
                .unwrap_or(0);
            Ok(val)
        });
    }

    fn write_elem_src(
        state: &mut Tcd22xxState,
        node: &mut FwNode,
        req: &mut FwReq,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        old: &ElemValue,
        new: &ElemValue,
        dsts: &[DstBlk],
        srcs: &[&[SrcBlk]],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut entries = state.router_entries.clone();

        ElemValueAccessor::<u32>::get_vals(new, old, dsts.len(), |idx, val| {
            let dst = dsts[idx];

            let src = if val > 0 {
                let pos = (val as usize) - 1;
                srcs.iter()
                    .flat_map(|srcs| *srcs)
                    .nth(pos)
                    .cloned()
                    .unwrap_or_else(|| SrcBlk {
                        id: SrcBlkId::Reserved(0xff),
                        ch: 0xff,
                    })
            } else {
                SrcBlk {
                    id: SrcBlkId::Reserved(0xff),
                    ch: 0xff,
                }
            };

            match entries.iter_mut().find(|entry| entry.dst.eq(&dst)) {
                Some(entry) => entry.src = src,
                None => entries.push(RouterEntry {
                    dst,
                    src,
                    ..Default::default()
                }),
            }

            Ok(())
        })?;

        T::update_router_entries(node, req, sections, caps, state, entries, timeout_ms)
    }
}

impl<O, T> RouterCtlOperation<T> for O
where
    O: Tcd22xxCtlOperation<T>,
    T: Tcd22xxSpecOperation + Tcd22xxRouterOperation + Tcd22xxMixerOperation,
{
}

#[derive(Default, Debug)]
struct MixerCtl {
    // Maximum number block in low rate mode.
    mixer_blk_pair: (Vec<SrcBlk>, Vec<DstBlk>),
    notified_elem_list: Vec<ElemId>,
}

const MIXER_SRC_GAIN_NAME: &str = "mixer-source-gain";

pub trait MixerCtlOperation<T>: Tcd22xxCtlOperation<T>
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

    fn cache_mixer(&mut self) -> Result<(), Error> {
        let ctls = self.tcd22xx_ctl_mut();
        ctls.mixer_ctl.mixer_blk_pair = T::compute_avail_mixer_blk_pair(&ctls.caps, RateMode::Low);

        Ok(())
    }

    fn load_mixer(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let ctl = &mut self.tcd22xx_ctl_mut().mixer_ctl;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SRC_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                ctl.mixer_blk_pair.0.len(),
                Self::COEF_MIN,
                Self::COEF_MAX,
                Self::COEF_STEP,
                ctl.mixer_blk_pair.1.len(),
                Some(&Into::<Vec<u32>>::into(Self::COEF_TLV)),
                true,
            )
            .map(|mut elem_id_list| ctl.notified_elem_list.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read_mixer(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_SRC_GAIN_NAME => {
                let dst_ch = elem_id.index() as usize;
                let res = self
                    .tcd22xx_ctl()
                    .state
                    .mixer_cache
                    .iter()
                    .nth(dst_ch)
                    .map(|entries| {
                        elem_value.set_int(entries);
                        true
                    })
                    .unwrap_or(false);
                Ok(res)
            }
            _ => Ok(false),
        }
    }

    fn write_mixer(
        &mut self,
        node: &mut FwNode,
        req: &mut FwReq,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_SRC_GAIN_NAME => {
                let ctls = &mut self.tcd22xx_ctl_mut();
                let dst_ch = elem_id.index() as usize;
                let mut cache = ctls.state.mixer_cache.clone();
                let res = match cache.iter_mut().nth(dst_ch) {
                    Some(entries) => {
                        let _ = ElemValueAccessor::<i32>::get_vals(
                            new,
                            old,
                            entries.len(),
                            |src_ch, val| {
                                entries[src_ch] = val;
                                Ok(())
                            },
                        );
                        T::update_mixer_coef(
                            node,
                            req,
                            sections,
                            &ctls.caps,
                            &mut ctls.state,
                            &cache,
                            timeout_ms,
                        )?;
                        true
                    }
                    None => false,
                };
                Ok(res)
            }
            _ => Ok(false),
        }
    }
}

impl<O, T> MixerCtlOperation<T> for O
where
    O: Tcd22xxCtlOperation<T>,
    T: Tcd22xxSpecOperation + Tcd22xxRouterOperation + Tcd22xxMixerOperation,
{
}

#[derive(Default, Debug)]
struct StandaloneCtl {
    params: StandaloneParameters,
    rates: Vec<ClockRate>,
    srcs: Vec<ClockSource>,
}

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

pub trait StandaloneCtlOperation<T>: Tcd22xxCtlOperation<T>
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

    fn cache_standalone(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let params = &mut self.tcd22xx_ctl_mut().standalone_ctl.params;
        StandaloneSectionProtocol::cache_standalone_params(req, node, sections, params, timeout_ms)
    }

    fn load_standalone(
        &mut self,
        global_params: &GlobalParameters,
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        let labels: Vec<&str> = global_params
            .avail_sources
            .iter()
            .filter_map(|src| {
                global_params
                    .clock_source_labels
                    .iter()
                    .find(|(s, _)| src.eq(s))
                    .map(|(_, l)| l.as_str())
            })
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, STANDALONE_CLK_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        if global_params
            .avail_sources
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

        if global_params
            .avail_sources
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

        if global_params
            .avail_sources
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

        let labels: Vec<String> = global_params
            .avail_rates
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

        self.tcd22xx_ctl_mut().standalone_ctl.rates = global_params.avail_rates.to_vec();
        self.tcd22xx_ctl_mut().standalone_ctl.srcs = global_params.avail_sources.to_vec();

        Ok(())
    }

    fn read_standalone(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            STANDALONE_CLK_SRC_NAME => {
                let ctl = &self.tcd22xx_ctl().standalone_ctl;
                let pos = ctl
                    .srcs
                    .iter()
                    .position(|src| ctl.params.clock_source.eq(src))
                    .ok_or_else(|| {
                        let msg = format!(
                            "Unexpected value for source: {}",
                            clock_source_to_string(&ctl.params.clock_source)
                        );
                        Error::new(FileError::Nxio, &msg)
                    })?;
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            STANDALONE_SPDIF_HIGH_RATE_NAME => {
                let ctl = &self.tcd22xx_ctl().standalone_ctl;
                elem_value.set_bool(&[ctl.params.aes_high_rate]);
                Ok(true)
            }
            STANDALONE_ADAT_MODE_NAME => {
                let ctl = &self.tcd22xx_ctl().standalone_ctl;
                let pos = Self::ADAT_MODES
                    .iter()
                    .position(|src| ctl.params.adat_mode.eq(src))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            STANDALONE_WC_MODE_NAME => {
                let ctl = &self.tcd22xx_ctl().standalone_ctl;
                let pos = Self::WC_MODES
                    .iter()
                    .position(|mode| ctl.params.word_clock_param.mode.eq(mode))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            STANDALONE_WC_RATE_NUMERATOR_NAME => {
                let ctl = &self.tcd22xx_ctl().standalone_ctl;
                elem_value.set_int(&[ctl.params.word_clock_param.rate.numerator as i32]);
                Ok(true)
            }
            STANDALONE_WC_RATE_DENOMINATOR_NAME => {
                let ctl = &self.tcd22xx_ctl().standalone_ctl;
                elem_value.set_int(&[ctl.params.word_clock_param.rate.denominator as i32]);
                Ok(true)
            }
            STANDALONE_INTERNAL_CLK_RATE_NAME => {
                let ctl = &self.tcd22xx_ctl().standalone_ctl;
                let pos = ctl
                    .rates
                    .iter()
                    .position(|rate| ctl.params.internal_rate.eq(rate))
                    .ok_or_else(|| {
                        let msg = format!(
                            "Unexpected value for rate: {}",
                            clock_rate_to_string(&ctl.params.internal_rate)
                        );
                        Error::new(FileError::Nxio, &msg)
                    })?;
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_standalone(
        &mut self,
        node: &mut FwNode,
        req: &mut FwReq,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            STANDALONE_CLK_SRC_NAME => {
                let ctl = &mut self.tcd22xx_ctl_mut().standalone_ctl;
                let mut params = ctl.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.clock_source = ctl
                    .srcs
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of source: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                StandaloneSectionProtocol::update_standalone_params(
                    req,
                    node,
                    &sections,
                    &params,
                    &mut ctl.params,
                    timeout_ms,
                )
                .map(|_| true)
            }
            STANDALONE_SPDIF_HIGH_RATE_NAME => {
                let ctl = &mut self.tcd22xx_ctl_mut().standalone_ctl;
                let mut params = ctl.params.clone();
                params.aes_high_rate = elem_value.boolean()[0];
                StandaloneSectionProtocol::update_standalone_params(
                    req,
                    node,
                    &sections,
                    &params,
                    &mut ctl.params,
                    timeout_ms,
                )
                .map(|_| true)
            }
            STANDALONE_ADAT_MODE_NAME => {
                let ctl = &mut self.tcd22xx_ctl_mut().standalone_ctl;
                let mut params = ctl.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.adat_mode = Self::ADAT_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Standalone ADAT mode not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                StandaloneSectionProtocol::update_standalone_params(
                    req,
                    node,
                    &sections,
                    &params,
                    &mut ctl.params,
                    timeout_ms,
                )
                .map(|_| true)
            }
            STANDALONE_WC_MODE_NAME => {
                let ctl = &mut self.tcd22xx_ctl_mut().standalone_ctl;
                let mut params = ctl.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.word_clock_param.mode = Self::WC_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg =
                            format!("Standalone Word Clock mode not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                StandaloneSectionProtocol::update_standalone_params(
                    req,
                    node,
                    &sections,
                    &params,
                    &mut ctl.params,
                    timeout_ms,
                )
                .map(|_| true)
            }
            STANDALONE_WC_RATE_NUMERATOR_NAME => {
                let ctl = &mut self.tcd22xx_ctl_mut().standalone_ctl;
                let mut params = ctl.params.clone();
                params.word_clock_param.rate.numerator = elem_value.int()[0] as u16;
                StandaloneSectionProtocol::update_standalone_params(
                    req,
                    node,
                    &sections,
                    &params,
                    &mut ctl.params,
                    timeout_ms,
                )
                .map(|_| true)
            }
            STANDALONE_WC_RATE_DENOMINATOR_NAME => {
                let ctl = &mut self.tcd22xx_ctl_mut().standalone_ctl;
                let mut params = ctl.params.clone();
                params.word_clock_param.rate.denominator = elem_value.int()[0] as u16;
                StandaloneSectionProtocol::update_standalone_params(
                    req,
                    node,
                    &sections,
                    &params,
                    &mut ctl.params,
                    timeout_ms,
                )
                .map(|_| true)
            }
            STANDALONE_INTERNAL_CLK_RATE_NAME => {
                let ctl = &mut self.tcd22xx_ctl_mut().standalone_ctl;
                let mut params = ctl.params.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.internal_rate = ctl
                    .rates
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of rate: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                StandaloneSectionProtocol::update_standalone_params(
                    req,
                    node,
                    &sections,
                    &params,
                    &mut ctl.params,
                    timeout_ms,
                )
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

impl<O, T> StandaloneCtlOperation<T> for O
where
    O: Tcd22xxCtlOperation<T>,
    T: Tcd22xxSpecOperation + Tcd22xxRouterOperation + Tcd22xxMixerOperation,
{
}

pub trait Tcd22xxCtlExt<T>:
    Tcd22xxCtlOperation<T>
    + MeterCtlOperation<T>
    + RouterCtlOperation<T>
    + MixerCtlOperation<T>
    + StandaloneCtlOperation<T>
where
    T: Tcd22xxSpecOperation + Tcd22xxRouterOperation + Tcd22xxMixerOperation,
{
    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        global_params: &GlobalParameters,
    ) -> Result<(), Error> {
        self.load_meter(card_cntr)?;
        self.load_router(card_cntr)?;
        self.load_mixer(card_cntr)?;
        self.load_standalone(global_params, card_cntr)?;

        Ok(())
    }

    fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        global_params: &GlobalParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let ctls = self.tcd22xx_ctl_mut();

        ctls.caps = CapsSectionProtocol::read_caps(req, node, sections, timeout_ms)?;
        T::cache(
            node,
            req,
            sections,
            &ctls.caps,
            &mut ctls.state,
            RateMode::from(global_params.clock_config.rate),
            timeout_ms,
        )?;

        self.cache_meter(req, node, sections, timeout_ms)?;
        self.cache_router(req, node, sections, global_params, timeout_ms)?;
        self.cache_mixer()?;
        self.cache_standalone(req, node, sections, timeout_ms)?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_meter(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_router(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_mixer(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_standalone(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        sections: &ExtensionSections,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if self.write_router(&mut unit.1, req, sections, elem_id, old, new, timeout_ms)? {
            Ok(true)
        } else if self.write_mixer(&mut unit.1, req, sections, elem_id, old, new, timeout_ms)? {
            Ok(true)
        } else if self.write_standalone(&mut unit.1, req, sections, elem_id, new, timeout_ms)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_measured_elem_list(&self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.tcd22xx_ctl().meter_ctl.measured_elem_list);
    }

    fn measure_states(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        sections: &ExtensionSections,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.measure_states_meter(&mut unit.1, req, sections, timeout_ms)
    }

    fn get_notified_elem_list(&self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.tcd22xx_ctl().router_ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.tcd22xx_ctl().mixer_ctl.notified_elem_list);
    }

    fn parse_notification(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        global_params: &GlobalParameters,
        timeout_ms: u32,
        _: u32,
    ) -> Result<(), Error> {
        let ctls = self.tcd22xx_ctl_mut();
        T::cache(
            node,
            req,
            sections,
            &ctls.caps,
            &mut ctls.state,
            RateMode::from(global_params.clock_config.rate),
            timeout_ms,
        )
    }
}

impl<O, T> Tcd22xxCtlExt<T> for O
where
    O: Tcd22xxCtlOperation<T>
        + MeterCtlOperation<T>
        + RouterCtlOperation<T>
        + MixerCtlOperation<T>
        + StandaloneCtlOperation<T>,
    T: Tcd22xxSpecOperation + Tcd22xxRouterOperation + Tcd22xxMixerOperation,
{
}
