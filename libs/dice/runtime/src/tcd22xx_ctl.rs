// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use hinawa::{FwNode, FwReq};
use hinawa::{SndDice, SndUnitExt};

use core::card_cntr::*;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::tcat::extension::{*, caps_section::*, cmd_section::*, mixer_section::*};
use dice_protocols::tcat::extension::peak_section::*;
use dice_protocols::tcat::extension::current_config_section::*;

use super::tcd22xx_spec::{Tcd22xxState, Tcd22xxSpec, Tcd22xxStateOperation};

#[derive(Default, Debug)]
pub struct Tcd22xxCtl<S>
    where for<'a> S: Tcd22xxSpec<'a> + AsRef<Tcd22xxState> + AsMut<Tcd22xxState>,
{
    state: S,
    caps: ExtensionCaps,
    meter_ctl: MeterCtl,
}

impl<S> Tcd22xxCtl<S>
    where for<'a> S: Tcd22xxSpec<'a> + AsRef<Tcd22xxState> + AsMut<Tcd22xxState>,
{
    pub fn load(&mut self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections,
                _: &ClockCaps, _: &ClockSourceLabels, timeout_ms: u32, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        let node = unit.get_node();

        self.caps = proto.read_caps(&node, sections, timeout_ms)?;

        self.meter_ctl.load(&node, proto, sections, &self.caps, &self.state, timeout_ms, card_cntr)?;

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

    pub fn get_measured_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.measured_elem_list);
    }

    pub fn measure_states(&mut self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections,
                          timeout_ms: u32)
        -> Result<(), Error>
    {
        self.meter_ctl.measure_states(&unit.get_node(), proto, sections, &self.caps, timeout_ms)
    }

    pub fn measure_elem(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        self.meter_ctl.measure_elem(elem_id, elem_value)
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
    const COEF_MAX: i32 = 0x00000fffi32; // 2:14 Fixed-point.
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
