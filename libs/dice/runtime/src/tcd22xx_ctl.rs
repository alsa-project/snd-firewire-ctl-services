// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use hinawa::{FwNode, FwReq};
use hinawa::{SndDice, SndUnitExt};

use core::card_cntr::*;
use core::elem_value_accessor::*;

use alsa_ctl_tlv_codec::items::DbInterval;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::tcat::extension::{*, caps_section::*, cmd_section::*, mixer_section::*};
use dice_protocols::tcat::extension::peak_section::*;
use dice_protocols::tcat::extension::{current_config_section::*, standalone_section::*};
use dice_protocols::tcat::tcd22xx_spec::*;

#[derive(Default, Debug)]
pub struct Tcd22xxCtl<S>
    where for<'a> S: Tcd22xxSpec<'a> + AsRef<Tcd22xxState> + AsMut<Tcd22xxState>,
{
    pub state: S,
    caps: ExtensionCaps,
    meter_ctl: MeterCtl,
    router_ctl: RouterCtl,
    mixer_ctl: MixerCtl,
    standalone_ctl: StandaloneCtl,
}

impl<S> Tcd22xxCtl<S>
    where for<'a> S: Tcd22xxSpec<'a> + AsRef<Tcd22xxState> + AsMut<Tcd22xxState>,
{
    pub fn load(&mut self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections,
                caps: &ClockCaps, src_labels: &ClockSourceLabels, timeout_ms: u32, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        let node = unit.get_node();

        self.caps = proto.read_caps(&node, sections, timeout_ms)?;

        self.meter_ctl.load(&node, proto, sections, &self.caps, &self.state, timeout_ms, card_cntr)?;
        self.router_ctl.load(&node, proto, sections, &self.caps, &self.state, caps, timeout_ms, card_cntr)?;
        self.mixer_ctl.load(&self.caps, &self.state, card_cntr)?;
        self.standalone_ctl.load(caps, src_labels, card_cntr)?;

        Ok(())
    }

    pub fn cache(&mut self, unit: &SndDice, proto: &FwReq, sections: &GeneralSections,
                 extension_sections: &ExtensionSections, timeout_ms: u32)
        -> Result<(), Error>
    {
        let node = unit.get_node();
        let config = proto.read_clock_config(&node, &sections, timeout_ms)?;
        let rate_mode = RateMode::from(config.rate);

        self.state.cache(&node, proto, extension_sections, &self.caps, rate_mode, timeout_ms)
    }

    pub fn read(&self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections, elem_id: &ElemId,
                elem_value: &mut ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        if self.router_ctl.read(&self.state, elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(&self.state, elem_id, elem_value)? {
            Ok(true)
        } else if self.standalone_ctl.read(&unit.get_node(), proto, sections, elem_id, elem_value,
                                           timeout_ms)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn write(&mut self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections,
                 elem_id: &ElemId, old: &ElemValue, new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        let node = unit.get_node();

        if self.router_ctl.write(&node, proto, sections, &self.caps, &mut self.state, elem_id,
                              old, new, timeout_ms)? {
            Ok(true)
        } else if self.mixer_ctl.write(&node, proto, sections, &self.caps, &mut self.state, elem_id,
                                       old, new, timeout_ms)? {
            Ok(true)
        } else if self.standalone_ctl.write(&node, proto, sections, elem_id, new, timeout_ms)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_measured_elem_list(&self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.measured_elem_list);
    }

    pub fn measure_states(&mut self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections,
                          timeout_ms: u32)
        -> Result<(), Error>
    {
        self.meter_ctl.measure_states(&unit.get_node(), proto, sections, &self.caps, timeout_ms)
    }

    pub fn measure_elem(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        self.meter_ctl.measure_elem(elem_id, elem_value)
    }

    pub fn get_notified_elem_list(&self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.router_ctl.notified_elem_list);
    }

    pub fn parse_notification(&mut self, unit: &SndDice, proto: &FwReq, sections: &GeneralSections,
                              extension_sections: &ExtensionSections, timeout_ms: u32, msg: u32)
         -> Result<(), Error>
     {
        if msg.has_clock_accepted() {
            self.cache(unit, proto, sections, extension_sections, timeout_ms)?;
        }
        Ok(())
     }

    pub fn read_notified_elem(&self, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.router_ctl.read(&self.state, elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(&self.state, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Debug)]
pub struct MeterCtl {
    // Maximum number block at low rate mode.
    real_blk_dsts: Vec<u8>,
    stream_blk_dsts: Vec<u8>,
    mixer_blk_dsts: Vec<u8>,

    real_meter: Vec<i32>,
    stream_meter: Vec<i32>,
    mixer_meter: Vec<i32>,

    out_sat: Vec<bool>,

    measured_elem_list: Vec<alsactl::ElemId>,
}

impl<'a> MeterCtl {
    const OUT_METER_NAME: &'a str = "output-source-meter";
    const STREAM_TX_METER_NAME: &'a str = "stream-source-meter";
    const MIXER_INPUT_METER_NAME: &'a str = "mixer-source-meter";
    const INPUT_SATURATION_NAME: &'a str = "mixer-out-saturation";

    const COEF_MIN: i32 = 0;
    const COEF_MAX: i32 = 0x00000fffi32; // Upper 12 bits of each sample.
    const COEF_STEP: i32 = 1;

    pub fn load<T>(&mut self, node: &FwNode, proto: &FwReq, sections: &ExtensionSections,
                   caps: &ExtensionCaps, state: &T, timeout_ms: u32, card_cntr: &mut CardCntr)
        -> Result<(), Error>
        where for<'b> T: Tcd22xxSpec<'b>,
    {
        let (_, real_blk_dsts) = state.compute_avail_real_blk_pair(RateMode::Low);
        self.real_blk_dsts = real_blk_dsts;
        let mut elem_id_list = Self::add_an_elem_for_meter(card_cntr, Self::OUT_METER_NAME, &self.real_blk_dsts)?;
        self.measured_elem_list.append(&mut elem_id_list);
        self.real_meter = vec![0;self.real_blk_dsts.len()];

        let (tx_entries, rx_entries) =
            proto.read_current_stream_format_entries(&node, sections, caps, RateMode::Low, timeout_ms)?;
        let (_, stream_blk_dsts) = state.compute_avail_stream_blk_pair(&tx_entries, &rx_entries);
        self.stream_blk_dsts = stream_blk_dsts;
        let mut elem_id_list = Self::add_an_elem_for_meter(card_cntr, Self::STREAM_TX_METER_NAME,
                                                           &self.stream_blk_dsts)?;
        self.measured_elem_list.append(&mut elem_id_list);
        self.stream_meter = vec![0;self.stream_blk_dsts.len()];

        let (_, mixer_blk_dsts) = state.compute_avail_mixer_blk_pair(caps, RateMode::Low);
        self.mixer_blk_dsts = mixer_blk_dsts;
        let mut elem_id_list = Self::add_an_elem_for_meter(card_cntr, Self::MIXER_INPUT_METER_NAME,
                                                           &self.mixer_blk_dsts)?;
        self.measured_elem_list.append(&mut elem_id_list);
        self.mixer_meter = vec![0;self.mixer_blk_dsts.len()];

        self.out_sat = vec![false;self.mixer_blk_dsts.len()];
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::INPUT_SATURATION_NAME, 0);
        let mut elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, self.mixer_blk_dsts.len(), false)?;
        self.measured_elem_list.append(&mut elem_id_list);

        Ok(())
    }

    fn add_an_elem_for_meter(card_cntr: &mut CardCntr, label: &str, targets: &Vec<u8>)
        -> Result<Vec<ElemId>, Error>
    {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, label, 0);
        let elem_id_list = card_cntr.add_int_elems(&elem_id, 1,
                                                   Self::COEF_MIN, Self::COEF_MAX, Self::COEF_STEP,
                                                   targets.len(), None, false)?;
        Ok(elem_id_list)
    }

    pub fn measure_states(&mut self, node: &FwNode, proto: &FwReq, sections: &ExtensionSections,
                             caps: &ExtensionCaps, timeout_ms: u32)
        -> Result<(), Error>
    {
        let entries = proto.read_peak_entries(&node, sections, caps, timeout_ms)?;

        self.real_meter.iter_mut().chain(self.stream_meter.iter_mut()).chain(self.mixer_meter.iter_mut())
            .zip(self.real_blk_dsts.iter().chain(self.stream_blk_dsts.iter()).chain(self.mixer_blk_dsts.iter()))
            .for_each(|(val, &dst)| {
                *val = entries.iter().find(|data| data[RouterEntry::DST_OFFSET] == dst)
                    .map(|data| {
                        let entry = RouterEntry::from(data);
                        entry.peak as i32
                    })
                    .unwrap_or(0);
            });

        self.out_sat = proto.read_saturation(&node, sections, caps, timeout_ms)?;

        Ok(())
    }

    pub fn measure_elem(&self, elem_id: &ElemId, elem_value: &ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::OUT_METER_NAME => {
                elem_value.set_int(&self.real_meter);
                Ok(true)
            }
            Self::STREAM_TX_METER_NAME => {
                elem_value.set_int(&self.stream_meter);
                Ok(true)
            }
            Self::MIXER_INPUT_METER_NAME => {
                elem_value.set_int(&self.mixer_meter);
                Ok(true)
            }
            Self::INPUT_SATURATION_NAME => {
                elem_value.set_bool(&self.out_sat);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub struct RouterCtl {
    // Maximum number block in low rate mode.
    real_blk_pair: (Vec<u8>, Vec<u8>),
    stream_blk_pair: (Vec<u8>, Vec<u8>),
    mixer_blk_pair: (Vec<u8>, Vec<u8>),
    pub notified_elem_list: Vec<alsactl::ElemId>,
}

impl<'a> RouterCtl {
    const OUT_SRC_NAME: &'a str = "output-source";
    const CAP_SRC_NAME: &'a str = "stream-source";
    const MIXER_SRC_NAME: &'a str = "mixer-source";

    const NONE_SRC_LABEL: &'a str = "None";

    pub fn load<T>(&mut self, node: &FwNode, proto: &FwReq, sections: &ExtensionSections,
                   caps: &ExtensionCaps, state: &T, clk_caps: &ClockCaps, timeout_ms: u32,
                   card_cntr: &mut CardCntr)
        -> Result<(), Error>
        where for<'b> T: Tcd22xxSpec<'b>,
    {
        self.real_blk_pair = state.compute_avail_real_blk_pair(RateMode::Low);

        // Compute the pair of blocks for tx/rx streams at each of available mode of rate. It's for
        // such models that second rx or tx stream is not available at mode of low rate.
        let mut rate_modes: Vec<RateMode> = Vec::default();
        clk_caps.get_rate_entries().iter()
            .map(|&r| RateMode::from(r))
            .for_each(|m| {
                if rate_modes.iter().find(|&&mode| mode.eq(&m)).is_none() {
                    rate_modes.push(m);
                }
            });
        rate_modes.iter()
            .try_for_each(|&m| {
                proto.read_current_stream_format_entries(node, sections, caps, m, timeout_ms)
                    .map(|(tx, rx)| {
                        let (mut tx_blk, mut rx_blk) = state.compute_avail_stream_blk_pair(&tx, &rx);
                        self.stream_blk_pair.0.append(&mut tx_blk);
                        self.stream_blk_pair.1.append(&mut rx_blk);
                        ()
                    })
            })?;
        self.stream_blk_pair.0.sort();
        self.stream_blk_pair.0.dedup();
        self.stream_blk_pair.1.sort();
        self.stream_blk_pair.1.dedup();

        self.mixer_blk_pair = state.compute_avail_mixer_blk_pair(caps, RateMode::Low);

        let mut elem_id_list = Self::add_an_elem_for_src(card_cntr, Self::OUT_SRC_NAME, &self.real_blk_pair.1,
                                    &[&self.real_blk_pair.0, &self.stream_blk_pair.0, &self.mixer_blk_pair.0],
                                    state)?;
        self.notified_elem_list.append(&mut elem_id_list);

        let mut elem_id_list = Self::add_an_elem_for_src(card_cntr, Self::CAP_SRC_NAME, &self.stream_blk_pair.1,
                                                         &[&self.real_blk_pair.0, &self.mixer_blk_pair.0],
                                                         state)?;
        self.notified_elem_list.append(&mut elem_id_list);

        let mut elem_id_list = Self::add_an_elem_for_src(card_cntr, Self::MIXER_SRC_NAME, &self.mixer_blk_pair.1,
                                                         &[&self.real_blk_pair.0, &self.stream_blk_pair.0],
                                                         state)?;
        self.notified_elem_list.append(&mut elem_id_list);

        Ok(())
    }

    pub fn read<T>(&self, state: &T, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
        where T: AsRef<Tcd22xxState>,
    {
        match elem_id.get_name().as_str() {
            Self::OUT_SRC_NAME => {
                Self::read_elem_src(state, elem_value, &self.real_blk_pair.1,
                                    &[&self.real_blk_pair.0, &self.stream_blk_pair.0, &self.mixer_blk_pair.0]);
                Ok(true)
            }
            Self::CAP_SRC_NAME => {
                Self::read_elem_src(state, elem_value, &self.stream_blk_pair.1,
                                    &[&self.real_blk_pair.0, &self.mixer_blk_pair.0]);
                Ok(true)
            }
            Self::MIXER_SRC_NAME => {
                Self::read_elem_src(state, elem_value, &self.mixer_blk_pair.1,
                                    &[&self.real_blk_pair.0, &self.stream_blk_pair.0]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write<T>(&self, node: &FwNode, proto: &FwReq, sections: &ExtensionSections,
                    caps: &ExtensionCaps, state: &mut T, elem_id: &ElemId,
                    old: &ElemValue, new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where for<'b> T: Tcd22xxSpec<'b> + AsRef<Tcd22xxState> + AsMut<Tcd22xxState>,
    {
        match elem_id.get_name().as_str() {
            Self::OUT_SRC_NAME => {
                Self::write_elem_src(node, proto, sections, caps, state, old, new, &self.real_blk_pair.1,
                                     &[&self.real_blk_pair.0, &self.stream_blk_pair.0, &self.mixer_blk_pair.0],
                                     timeout_ms)
                .map(|_| true)
            }
            Self::CAP_SRC_NAME => {
                Self::write_elem_src(node, proto, sections, caps, state, old, new, &self.stream_blk_pair.1,
                                     &[&self.real_blk_pair.0, &self.mixer_blk_pair.0], timeout_ms)
                .map(|_| true)
            }
            Self::MIXER_SRC_NAME => {
                Self::write_elem_src(node, proto, sections, caps, state, old, new, &self.mixer_blk_pair.1,
                                     &[&self.real_blk_pair.0, &self.stream_blk_pair.0], timeout_ms)
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn add_an_elem_for_src<T>(card_cntr: &mut CardCntr, label: &'a str, dsts: &[u8], srcs: &[&[u8]], state: &T)
        -> Result<Vec<ElemId>, Error>
        where for<'b> T: Tcd22xxSpec<'b>,
    {
        let targets = dsts.iter().map(|&dst| state.get_dst_blk_label(dst)).collect::<Vec<String>>();
        let mut sources = srcs.iter()
            .flat_map(|srcs| srcs.iter())
            .map(|&src| state.get_src_blk_label(src))
            .collect::<Vec<String>>();
        sources.insert(0, Self::NONE_SRC_LABEL.to_string());

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, label, 0);
        let elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, targets.len(), &sources, None, true)?;
        Ok(elem_id_list)
    }

    fn read_elem_src<T>(state: &T, elem_value: &alsactl::ElemValue, dsts: &[u8], srcs: &[&[u8]])
        where T: AsRef<Tcd22xxState>,
    {
        let _ = ElemValueAccessor::<u32>::set_vals(elem_value, dsts.len(), |idx| {
            let dst = dsts[idx];

            let val = state.as_ref().router_entries.iter()
                .find(|data| data[RouterEntry::DST_OFFSET] == dst)
                .and_then(|data| {
                    srcs.iter()
                        .flat_map(|srcs| srcs.iter().map(|&s| u8::from(s)))
                        .position(|src| data[RouterEntry::SRC_OFFSET] == src)
                        .map(|pos| 1 + pos as u32)
                })
                .unwrap_or(0);
            Ok(val)
        });
    }

    fn write_elem_src<T>(node: &FwNode, proto: &FwReq, sections: &ExtensionSections,
                         caps: &ExtensionCaps, state: &mut T, old: &ElemValue, new: &ElemValue,
                         dsts: &[u8], srcs: &[&[u8]], timeout_ms: u32)
        -> Result<(), Error>
        where for<'b> T: Tcd22xxSpec<'b> + AsRef<Tcd22xxState> + AsMut<Tcd22xxState>,
    {
        let mut entries = state.as_ref().router_entries.clone();

        ElemValueAccessor::<u32>::get_vals(new, old, dsts.len(), |idx, val| {
            let dst = u8::from(dsts[idx]);

            let src = if val > 0 {
                let pos = (val as usize) - 1;
                srcs.iter()
                    .flat_map(|srcs| srcs.iter().map(|&s| u8::from(s)))
                    .nth(pos)
                    .unwrap_or_else(|| {
                        let entry = SrcBlk{id: SrcBlkId::Reserved(0xff), ch: 0xff};
                        u8::from(entry)
                    })
            } else {
                let entry = SrcBlk{id: SrcBlkId::Reserved(0xff), ch: 0xff};
                u8::from(entry)
            };

            entries.iter_mut()
                .find(|data| data[RouterEntry::DST_OFFSET] == dst)
                .map(|data| data[RouterEntry::SRC_OFFSET] = src)
                .unwrap_or_else(|| {
                    let mut entry = RouterEntryData::default();
                    entry[RouterEntry::DST_OFFSET] = dst;
                    entry[RouterEntry::SRC_OFFSET] = src;
                    entries.push(RouterEntryData::from(entry))
                });

            Ok(())
        })?;

        state.update_router_entries(node, proto, sections, caps, entries, timeout_ms)
    }
}

#[derive(Default, Debug)]
pub struct MixerCtl {
    // Maximum number block in low rate mode.
    mixer_blk_pair: (Vec<u8>, Vec<u8>),
    pub notified_elem_list: Vec<alsactl::ElemId>,
}

impl<'a> MixerCtl {
    const SRC_GAIN_NAME: &'a str = "mixer-source-gain";

    const COEF_MIN: i32 = 0;
    const COEF_MAX: i32 = 0x0000ffffi32; // 2:14 Fixed-point.
    const COEF_STEP: i32 = 1;
    const COEF_TLV: DbInterval = DbInterval{min: -6000, max: 4, linear: false, mute_avail: false};

    pub fn load<T>(&mut self, caps: &ExtensionCaps, state: &T, card_cntr: &mut CardCntr)
        -> Result<(), Error>
        where for<'b> T: Tcd22xxSpec<'b> + AsRef<Tcd22xxState> + AsMut<Tcd22xxState>,
    {
        self.mixer_blk_pair = state.compute_avail_mixer_blk_pair(caps, RateMode::Low);

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::SRC_GAIN_NAME, 0);
        let mut elem_id_list = card_cntr.add_int_elems(&elem_id, self.mixer_blk_pair.0.len(),
                                            Self::COEF_MIN, Self::COEF_MAX, Self::COEF_STEP,
                                            self.mixer_blk_pair.1.len(),
                                            Some(&Into::<Vec<u32>>::into(Self::COEF_TLV)), true)?;
        self.notified_elem_list.append(&mut elem_id_list);

        Ok(())
    }

    pub fn read<T>(&self, state: &T, elem_id: &alsactl::ElemId, elem_value: &alsactl::ElemValue)
        -> Result<bool, Error>
        where for<'b> T: Tcd22xxSpec<'b> + AsRef<Tcd22xxState> + AsMut<Tcd22xxState>,
    {
        match elem_id.get_name().as_str() {
            Self::SRC_GAIN_NAME => {
                let dst_ch = elem_id.get_index() as usize;
                let res = state.as_ref().mixer_cache.iter()
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

    pub fn write<T>(&mut self, node: &FwNode, proto: &FwReq, sections: &ExtensionSections,
                   caps: &ExtensionCaps, state: &mut T, elem_id: &ElemId,
                   old: &ElemValue, new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where for<'b> T: Tcd22xxSpec<'b> + AsRef<Tcd22xxState> + AsMut<Tcd22xxState>,
    {
        match elem_id.get_name().as_str() {
            Self::SRC_GAIN_NAME => {
                let dst_ch = elem_id.get_index() as usize;
                let mut cache = state.as_mut().mixer_cache.clone();
                let res = match cache.iter_mut().nth(dst_ch) {
                    Some(entries) => {
                        let _ = ElemValueAccessor::<i32>::get_vals(new, old, entries.len(), |src_ch, val| {
                            entries[src_ch] = val;
                            Ok(())
                        });
                        state.update_mixer_coef(node, proto, sections, caps, &cache, timeout_ms)?;
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

#[derive(Default, Debug)]
pub struct StandaloneCtl {
    rates: Vec<ClockRate>,
    srcs: Vec<ClockSource>,
}

impl<'a> StandaloneCtl {
    const CLK_SRC_NAME: &'a str = "standalone-clock-source";
    const SPDIF_HIGH_RATE_NAME: &'a str = "standalone-spdif-high-rate";
    const ADAT_MODE_NAME: &'a str = "standalone-adat-mode";
    const WC_MODE_NAME: &'a str = "standalone-word-clock-mode";
    const WC_RATE_NUMERATOR_NAME: &'a str = "standalone-word-clock-rate-numerator";
    const WC_RATE_DENOMINATOR_NAME: &'a str = "standalone-word-clock-rate-denominator";
    const INTERNAL_CLK_RATE_NAME: &'a str = "standalone-internal-clock-rate";

    const ADAT_MODE_LABELS: &'a [&'a str] = &["Normal", "S/MUX2", "S/MUX4", "Auto"];

    const WC_MODE_LABELS: &'a [&'a str] = &["Normal", "Low", "Middle", "High"];

    pub fn load(&mut self, caps: &ClockCaps, src_labels: &ClockSourceLabels, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        self.rates = caps.get_rate_entries();
        self.srcs = caps.get_src_entries(src_labels);

        let labels = self.srcs.iter()
            .map(|s| s.get_label(&src_labels, false).unwrap())
            .collect::<Vec<_>>();

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::CLK_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        if self.srcs.iter()
            .find(|&s| {
                *s == ClockSource::Aes1 || *s == ClockSource::Aes2 ||
                *s == ClockSource::Aes3 || *s == ClockSource::Aes4
            }).is_some() {
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                       0, 0, Self::SPDIF_HIGH_RATE_NAME, 0);
            let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;
        }

        if self.srcs.iter().find(|&s| *s == ClockSource::Adat).is_some() {
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                       0, 0, Self::ADAT_MODE_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &Self::ADAT_MODE_LABELS, None, true)?;
        }

        if self.srcs.iter().find(|&s| *s == ClockSource::WordClock).is_some() {
                let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                           0, 0, Self::WC_MODE_NAME, 0);
                let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &Self::WC_MODE_LABELS, None, true)?;

                let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                           0, 0, Self::WC_RATE_NUMERATOR_NAME, 0);
                let _ = card_cntr.add_int_elems(&elem_id, 1, 1, 4095, 1, 1, None, true)?;

                let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                           0, 0, Self::WC_RATE_DENOMINATOR_NAME, 0);
                let _ = card_cntr.add_int_elems(&elem_id, 1, 1, std::u16::MAX as i32, 1, 1, None, true)?;
        }

        let labels = self.rates.iter()
            .map(|r| r.to_string())
            .collect::<Vec<_>>();

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card,
                                                   0, 0, Self::INTERNAL_CLK_RATE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    pub fn read(&self, node: &FwNode, proto: &FwReq, sections: &ExtensionSections,
                elem_id: &ElemId, elem_value: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::CLK_SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto.read_standalone_clock_source(node, sections, timeout_ms)
                        .and_then(|src| {
                            self.srcs.iter()
                                .position(|&s| s == src)
                                .ok_or_else(|| {
                                    let msg = format!("Unexpected value for source: {}", src);
                                    Error::new(FileError::Nxio, &msg)
                                })
                        })
                        .map(|pos| pos as u32)
                })
                .map(|_| true)
            }
            Self::SPDIF_HIGH_RATE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    proto.read_standalone_aes_high_rate(node, sections, timeout_ms)
                })
                .map(|_| true)
            }
            Self::ADAT_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto.read_standalone_adat_mode(node, sections, timeout_ms)
                        .map(|mode| {
                            match mode {
                                AdatParam::Normal => 0,
                                AdatParam::SMUX2 => 1,
                                AdatParam::SMUX4 => 2,
                                AdatParam::Auto => 3,
                            }
                        })
                })
                .map(|_| true)
            }
            Self::WC_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto.read_standalone_word_clock_param(node, sections, timeout_ms)
                        .map(|param| {
                            match param.mode {
                                WordClockMode::Normal => 0,
                                WordClockMode::Low => 1,
                                WordClockMode::Middle => 2,
                                WordClockMode::High => 3,
                            }
                        })
                })
                .map(|_| true)
            }
            Self::WC_RATE_NUMERATOR_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    proto.read_standalone_word_clock_param(node, sections, timeout_ms)
                        .map(|param| param.rate.numerator as i32)
                })
                .map(|_| true)
            }
            Self::WC_RATE_DENOMINATOR_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    proto.read_standalone_word_clock_param(node, sections, timeout_ms)
                        .map(|param| param.rate.denominator as i32)
                })
                .map(|_| true)
            }
            Self::INTERNAL_CLK_RATE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    proto.read_standalone_internal_rate(node, sections, timeout_ms)
                        .and_then(|rate| {
                            self.rates.iter()
                                .position(|&r| r == rate)
                                .ok_or_else(|| {
                                    let msg = format!("Unexpected value for rate: {}", rate);
                                    Error::new(FileError::Nxio, &msg)
                                })
                                .map(|pos| pos as u32)
                        })
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, node: &FwNode, proto: &FwReq, sections: &ExtensionSections,
                 elem_id: &ElemId, new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::CLK_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    self.srcs.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .and_then(|&s| {
                            proto.write_standalone_clock_source(node, &sections, s, timeout_ms)
                        })
                })
                .map(|_| true)
            }
            Self::SPDIF_HIGH_RATE_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    proto.write_standalone_aes_high_rate(node, &sections, val, timeout_ms)
                })
                .map(|_| true)
            }
            Self::ADAT_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mode = match val {
                        1 => AdatParam::SMUX2,
                        2 => AdatParam::SMUX4,
                        3 => AdatParam::Auto,
                        _ => AdatParam::Normal,
                    };
                    proto.write_standalone_adat_mode(node, &sections, mode, timeout_ms)
                })
                .map(|_| true)
            }
            Self::WC_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mode = match val {
                        1 => WordClockMode::Low,
                        2 => WordClockMode::Middle,
                        3 => WordClockMode::High,
                        _ => WordClockMode::Normal,
                    };
                    proto.read_standalone_word_clock_param(node, &sections, timeout_ms)
                        .and_then(|mut param| {
                            param.mode = mode;
                            proto.write_standalone_word_clock_param(node, &sections, param, timeout_ms)
                        })
                })
                .map(|_| true)
            }
            Self::WC_RATE_NUMERATOR_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    proto.read_standalone_word_clock_param(node, &sections, timeout_ms)
                        .and_then(|mut param| {
                            param.rate.numerator = val as u16;
                            proto.write_standalone_word_clock_param(node, &sections, param, timeout_ms)
                        })
                })
                .map(|_| true)
            }
            Self::WC_RATE_DENOMINATOR_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    proto.read_standalone_word_clock_param(node, &sections, timeout_ms)
                        .and_then(|mut param| {
                            param.rate.denominator = val as u16;
                            proto.write_standalone_word_clock_param(node, &sections, param, timeout_ms)
                        })
                })
                .map(|_| true)
            }
            Self::INTERNAL_CLK_RATE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    self.rates.iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of rate: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .and_then(|&r| {
                            proto.write_standalone_internal_rate(node, &sections, r, timeout_ms)
                        })
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
