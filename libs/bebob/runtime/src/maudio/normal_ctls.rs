// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::Error;

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use alsa_ctl_tlv_codec::items::{DbInterval, CTL_VALUE_MUTE};

use core::card_cntr::*;
use core::elem_value_accessor::ElemValueAccessor;

use ta1394::Ta1394Avc;
use ta1394::audio::{AUDIO_SUBUNIT_0_ADDR, CtlAttr, AudioCh, AudioProcessing, ProcessingCtl, AudioFeature, FeatureCtl, AudioSelector};

use bebob_protocols::*;

use super::super::model::{OUT_SRC_NAME, OUT_VOL_NAME, HP_SRC_NAME};

pub const FCP_TIMEOUT_MS: u32 = 100;

fn get_fb_id_and_ch(fb_ids: &[u8], target: usize) -> (u8, u8) {
    let pos = target / 2;
    let mut val = u8::MAX;
    let mut left = 0;
    for i in 0..(pos + 1) {
        if val == fb_ids[i] {
            left += 2;
        } else {
            val = fb_ids[i];
            left = 0;
        }
    }
    (val, left + (target % 2) as u8)
}

pub struct MixerCtl<'a> {
    dst_fb_ids: &'a [u8],
    dst_labels: &'a [&'a str],

    phys_src_fb_ids: &'a [u8],
    phys_src_labels: &'a [&'a str],

    stream_src_fb_ids: &'a [u8],
    stream_src_labels: &'a [&'a str],
}

const ON: i16 = 0x0000;
const OFF: i16 = (0x8000 as u16) as i16;

impl<'a> MixerCtl<'a> {
    const MIXER_SRC_NAME: &'a str = "mixer-source";

    pub fn new(dst_fb_ids: &'a [u8], dst_labels: &'a [&'a str],
               phys_src_fb_ids: &'a [u8], phys_src_labels: &'a [&'a str],
               stream_src_fb_ids: &'a [u8], stream_src_labels: &'a [&'a str])
        -> Self
    {
        assert_eq!(dst_fb_ids.len(), dst_labels.len());
        assert_eq!(phys_src_fb_ids.len(), phys_src_labels.len());
        assert_eq!(stream_src_fb_ids.len(), stream_src_labels.len());

        MixerCtl {dst_fb_ids, dst_labels, phys_src_fb_ids, phys_src_labels, stream_src_fb_ids, stream_src_labels}
    }

    pub fn load(&mut self, avc: &BebobAvc, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        (0..self.dst_fb_ids.len()).take(self.stream_src_fb_ids.len()).try_for_each(|i| {
            let (dst_fb, dst_ch) = get_fb_id_and_ch(self.dst_fb_ids, i * 2);
            let (src_fb, src_ch) = get_fb_id_and_ch(self.stream_src_fb_ids, i * 2);
            let mut op = AudioProcessing::new(dst_fb, CtlAttr::Current, src_fb,
                                    AudioCh::Each(src_ch), AudioCh::Each(dst_ch),
                                    ProcessingCtl::Mixer(vec![ON]));
            avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
        })?;

        let src_count = self.stream_src_labels.len() + self.phys_src_labels.len();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer,
                                                   0, 0, Self::MIXER_SRC_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, self.dst_labels.len(), src_count, true)?;

        Ok(())
    }

    pub fn read(&mut self, avc: &BebobAvc, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MIXER_SRC_NAME => {
                let dst = elem_id.get_index() as usize;
                let (dst_fb, dst_ch) = get_fb_id_and_ch(self.dst_fb_ids, dst * 2);

                let mut src_fb_ids = Vec::new();
                src_fb_ids.extend_from_slice(&self.stream_src_fb_ids);
                src_fb_ids.extend_from_slice(&self.phys_src_fb_ids);

                ElemValueAccessor::<bool>::set_vals(elem_value, src_fb_ids.len(), |idx| {
                    let (src_fb, src_ch) = get_fb_id_and_ch(&src_fb_ids, idx * 2);
                    let mut op = AudioProcessing::new(dst_fb, CtlAttr::Current, src_fb,
                                                      AudioCh::Each(src_ch), AudioCh::Each(dst_ch),
                                                      ProcessingCtl::Mixer(vec![-1]));
                    avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
                    if let ProcessingCtl::Mixer(data) = op.ctl {
                        Ok(data[0] == ON)
                    } else {
                        unreachable!();
                    }
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &ElemId, old: &ElemValue,
                 new: &ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MIXER_SRC_NAME => {
                let dst = elem_id.get_index() as usize;
                let (dst_fb, dst_ch) = get_fb_id_and_ch(self.dst_fb_ids, dst * 2);

                let mut src_fb_ids = Vec::new();
                src_fb_ids.extend_from_slice(&self.stream_src_fb_ids);
                src_fb_ids.extend_from_slice(&self.phys_src_fb_ids);

                ElemValueAccessor::<bool>::get_vals(new, old, src_fb_ids.len(), |idx, val| {
                    // Left channel is a representative.
                    let (src_fb, src_ch) = get_fb_id_and_ch(&src_fb_ids, idx * 2);
                    let mut op = AudioProcessing::new(dst_fb, CtlAttr::Current, src_fb,
                                            AudioCh::Each(src_ch), AudioCh::Each(dst_ch),
                                            ProcessingCtl::Mixer(vec![if val { ON } else { OFF }]));
                    avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub struct InputCtl<'a> {
    phys_fb_ids: &'a [u8],
    phys_labels: &'a [&'a str],
    stream_fb_ids: &'a [u8],
    stream_labels: &'a [&'a str],
}

const GAIN_MIN: i32 = i16::MIN as i32;
const GAIN_MAX: i32 = 0;
const GAIN_STEP: i32 = 256;
const GAIN_TLV: DbInterval = DbInterval{min: -12800, max: 0, linear: false, mute_avail: true};

const PAN_MIN: i32 = i16::MIN as i32;
const PAN_MAX: i32 = i16::MAX as i32;
const PAN_STEP: i32 = 256;
const PAN_TLV: DbInterval = DbInterval{min: -12800, max: 12800, linear: false, mute_avail: false};

impl<'a> InputCtl<'a> {
    const PHYS_GAIN_NAME: &'a str = "phys-in-gain";
    const PHYS_BALANCE_NAME: &'a str = "phys-in-balance";
    const STREAM_GAIN_NAME: &'a str = "stream-in-gain";

    pub fn new(phys_fb_ids: &'a [u8], phys_labels: &'a [&'a str],
               stream_fb_ids: &'a [u8], stream_labels: &'a [&'a str])
        -> Self
    {
        assert_eq!(phys_fb_ids.len(), phys_labels.len());
        assert_eq!(stream_fb_ids.len(), stream_labels.len());

        InputCtl {
            phys_fb_ids,
            phys_labels,
            stream_fb_ids,
            stream_labels,
        }
    }

    pub fn load(&mut self, _: &BebobAvc, card_cntr: &mut CardCntr) -> Result<(), Error> {
        // For gain of physical inputs.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer,
                                                   0, 0, Self::PHYS_GAIN_NAME, 0);
        let len = 2 * self.phys_labels.len();
        let _ = card_cntr.add_int_elems(&elem_id, 1, GAIN_MIN, GAIN_MAX, GAIN_STEP, len,
                                        Some(&Into::<Vec<u32>>::into(GAIN_TLV)), true)?;

        // For balance of physical inputs.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer,
                                                   0, 0, Self::PHYS_BALANCE_NAME, 0);
        let len = 2 * self.phys_labels.len();
        let _ = card_cntr.add_int_elems(&elem_id, 1, PAN_MIN, PAN_MAX, PAN_STEP, len,
                                        Some(&Into::<Vec<u32>>::into(PAN_TLV)), true)?;

        // For gain of stream inputs.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer,
                                                   0, 0, Self::STREAM_GAIN_NAME, 0);
        let len = 2 * self.stream_labels.len();
        let _ = card_cntr.add_int_elems(&elem_id, 1, GAIN_MIN, GAIN_MAX, GAIN_STEP, len,
                                        Some(&Into::<Vec<u32>>::into(GAIN_TLV)), true)?;

        // Balance of stream inputs is not available.

        Ok(())
    }

    pub fn read(&mut self, avc: &BebobAvc, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::PHYS_GAIN_NAME => self.read_gain(avc, elem_value, self.phys_fb_ids),
            Self::PHYS_BALANCE_NAME => self.read_balance(avc, elem_value, self.phys_fb_ids),
            Self::STREAM_GAIN_NAME => self.read_gain(avc, elem_value, self.stream_fb_ids),
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &ElemId, old: &ElemValue,
                 new: &ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::PHYS_GAIN_NAME => self.write_gain(avc, old, new, self.phys_fb_ids),
            Self::PHYS_BALANCE_NAME => self.write_balance(avc, old, new, self.phys_fb_ids),
            Self::STREAM_GAIN_NAME => self.write_gain(avc, old, new, self.stream_fb_ids),
            _ => Ok(false),
        }
    }

    pub fn read_gain(&mut self, avc: &BebobAvc, elem_value: &mut ElemValue, fb_ids: &[u8])
        -> Result<bool, Error>
    {
        ElemValueAccessor::<i32>::set_vals(elem_value, fb_ids.len(), |idx| {
            let (fb_id, ch) = get_fb_id_and_ch(fb_ids, idx);
            let mut op = AudioFeature::new(fb_id, CtlAttr::Current, AudioCh::Each(ch),
                                           FeatureCtl::Volume(vec![-1]));
            avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
            if let FeatureCtl::Volume(data) = op.ctl {
                let val = if data[0] == FeatureCtl::NEG_INFINITY { CTL_VALUE_MUTE } else { data[0] as i32 };
                Ok(val)
            } else {
                unreachable!();
            }
        })?;
        Ok(true)
    }

    pub fn write_gain(&mut self, avc: &BebobAvc, old: &ElemValue, new: &ElemValue,
                      fb_ids: &[u8])
        -> Result<bool, Error>
    {
        ElemValueAccessor::<i32>::get_vals(new, old, fb_ids.len(), |idx, val| {
            let (fb_id, ch) = get_fb_id_and_ch(fb_ids, idx);
            let v = if val == CTL_VALUE_MUTE { FeatureCtl::NEG_INFINITY } else { val as i16 };
            let mut op = AudioFeature::new(fb_id, CtlAttr::Current, AudioCh::Each(ch),
                                           FeatureCtl::Volume(vec![v]));
            avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
        })?;
        Ok(true)
    }

    fn read_balance(&mut self, avc: &BebobAvc, elem_value: &mut ElemValue, fb_ids: &[u8])
        -> Result<bool, Error>
    {
        ElemValueAccessor::<i32>::set_vals(elem_value, fb_ids.len(), |idx| {
            let (fb_id, ch) = get_fb_id_and_ch(fb_ids, idx);
            let mut op = AudioFeature::new(fb_id, CtlAttr::Current, AudioCh::Each(ch),
                                           FeatureCtl::LrBalance(-1));
            avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
            if let FeatureCtl::LrBalance(val) = op.ctl {
                Ok(val as i32)
            } else {
                unreachable!();
            }
        })?;
        Ok(true)
    }

    fn write_balance(&mut self, avc: &BebobAvc, old: &ElemValue, new: &ElemValue,
                     fb_ids: &[u8])
        -> Result<bool, Error>
    {
        ElemValueAccessor::<i32>::get_vals(new, old, fb_ids.len(), |idx, val| {
            let (fb_id, ch) = get_fb_id_and_ch(fb_ids, idx);
            let mut op = AudioFeature::new(fb_id, CtlAttr::Current, AudioCh::Each(ch),
                                           FeatureCtl::LrBalance(val as i16));
            avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
        })?;
        Ok(true)
    }
}

const VOL_MIN: i32 = i16::MIN as i32;
const VOL_MAX: i32 = 0;
const VOL_STEP: i32 = 256;
const VOL_TLV: DbInterval = DbInterval{min: -12800, max: 0, linear: false, mute_avail: true};

pub struct AuxCtl<'a> {
    out_fb_id: u8,
    src_fb_ids: Vec<u8>,
    src_labels: Vec<&'a str>,
}

impl<'a> AuxCtl<'a> {
    const AUX_SRC_GAIN_NAME: &'a str = "aux-source-gain";
    const AUX_OUT_VOLUME_NAME: &'a str = "aux-output-volume";

    pub fn new(out_fb_id: u8, phys_src_fb_ids: &'a [u8], phys_src_labels: &'a [&'a str],
               stream_src_fb_ids: &'a [u8], stream_src_labels: &'a [&'a str])
        -> Self
    {
        assert_eq!(phys_src_fb_ids.len(), phys_src_labels.len());
        assert_eq!(stream_src_fb_ids.len(), stream_src_labels.len());

        let mut src_fb_ids = phys_src_fb_ids.to_vec();
        src_fb_ids.extend(stream_src_fb_ids);
        let mut src_labels = phys_src_labels.to_vec();
        src_labels.extend(stream_src_labels);

        AuxCtl {
            out_fb_id: out_fb_id,
            src_fb_ids: src_fb_ids,
            src_labels: src_labels,
        }
    }

    pub fn load(&mut self, _: &BebobAvc, card_cntr: &mut CardCntr) -> Result<(), Error> {
        // For gain of sources to aux.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer,
                                                   0, 0, Self::AUX_SRC_GAIN_NAME, 0);
        let len = 2 * self.src_labels.len();
        let _ = card_cntr.add_int_elems(&elem_id, 1, GAIN_MIN, GAIN_MAX, GAIN_STEP, len,
                                        Some(&Into::<Vec<u32>>::into(GAIN_TLV)), true)?;

        // For volume of output from aux.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer,
                                                   0, 0, Self::AUX_OUT_VOLUME_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, VOL_MIN, VOL_MAX, VOL_STEP, 2,
                                        Some(&Into::<Vec<u32>>::into(VOL_TLV)), true)?;

        Ok(())
    }

    pub fn read(&mut self, avc: &BebobAvc, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::AUX_SRC_GAIN_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, self.src_labels.len(), |idx| {
                    let (fb_id, ch) = get_fb_id_and_ch(&self.src_fb_ids, idx);
                    let mut op = AudioFeature::new(fb_id, CtlAttr::Current, AudioCh::Each(ch as u8),
                                                   FeatureCtl::Volume(vec![-1]));
                    avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
                    if let FeatureCtl::Volume(data) = op.ctl {
                        let val = if data[0] == FeatureCtl::NEG_INFINITY { CTL_VALUE_MUTE } else { data[0] as i32 };
                        Ok(val)
                    } else {
                        unreachable!();
                    }
                })?;
                Ok(true)
            }
            Self::AUX_OUT_VOLUME_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| {
                    let mut op = AudioFeature::new(self.out_fb_id, CtlAttr::Current, AudioCh::Each(idx as u8),
                                                   FeatureCtl::Volume(vec![-1]));
                    avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
                    if let FeatureCtl::Volume(data) = op.ctl {
                        let val = if data[0] == FeatureCtl::NEG_INFINITY { CTL_VALUE_MUTE } else { data[0] as i32 };
                        Ok(val)
                    } else {
                        unreachable!();
                    }
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &ElemId,
                 old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::AUX_SRC_GAIN_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, self.src_labels.len(), |idx, val| {
                    let (fb_id, ch) = get_fb_id_and_ch(&self.src_fb_ids, idx);
                    let v = if val == CTL_VALUE_MUTE { FeatureCtl::NEG_INFINITY } else { val as i16 };
                    let mut op = AudioFeature::new(fb_id, CtlAttr::Current, AudioCh::Each(ch as u8),
                                                   FeatureCtl::Volume(vec![v]));
                    avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
                })?;
                Ok(true)
            }
            Self::AUX_OUT_VOLUME_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    let v = if val == CTL_VALUE_MUTE { FeatureCtl::NEG_INFINITY } else { val as i16 };
                    let mut op = AudioFeature::new(self.out_fb_id, CtlAttr::Current, AudioCh::Each(idx as u8),
                                                   FeatureCtl::Volume(vec![v]));
                    avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub struct OutputCtl<'a> {
    labels: &'a [&'a str],
    vol_fb_ids: &'a [u8],
    src_fb_ids: &'a [u8],
}

impl<'a> OutputCtl<'a> {
    const OUT_SRC_LABELS: &'a [&'a str] = &["mixer", "aux-1/2"];

    pub fn new(labels: &'a [&'a str], vol_fb_ids: &'a [u8], src_fb_ids: &'a [u8]) -> Self {
        assert_eq!(labels.len(), vol_fb_ids.len());
        assert_eq!(labels.len(), src_fb_ids.len());
        OutputCtl {
            labels,
            vol_fb_ids,
            src_fb_ids,
        }
    }

    pub fn load(&mut self, _: &BebobAvc, card_cntr: &mut CardCntr) -> Result<(), Error> {
        // For source of output.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, self.labels.len(), Self::OUT_SRC_LABELS, None, true)?;

        // For volume of output.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_VOL_NAME, 0);
        let len = 2 * self.labels.len();
        let _ = card_cntr.add_int_elems(&elem_id, 1, VOL_MIN, VOL_MAX, VOL_STEP, len,
                                        Some(&Into::<Vec<u32>>::into(VOL_TLV)), true)?;

        Ok(())
    }

    pub fn read(&mut self, avc: &BebobAvc, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            OUT_SRC_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, self.labels.len(), |idx| {
                    let mut op = AudioSelector::new(self.src_fb_ids[idx], CtlAttr::Current, 0xff);
                    avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
                    Ok(op.input_plug_id as u32)
                })?;
                Ok(true)
            }
            OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, self.labels.len(), |idx| {
                    let ch = (idx % 2) as u8;
                    let mut op = AudioFeature::new(self.vol_fb_ids[idx / 2], CtlAttr::Current, AudioCh::Each(ch),
                                                   FeatureCtl::Volume(vec![-1]));
                    avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
                    if let FeatureCtl::Volume(data) = op.ctl {
                        let val = if data[0] == FeatureCtl::NEG_INFINITY { CTL_VALUE_MUTE } else { data[0] as i32 };
                        Ok(val)
                    } else {
                        unreachable!();
                    }
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &ElemId,
                 old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            OUT_SRC_NAME => {
                ElemValueAccessor::<u32>::get_vals(new, old, self.labels.len(), |idx, val| {
                    let mut op = AudioSelector::new(self.src_fb_ids[idx], CtlAttr::Current, val as u8);
                    avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
                })?;
                Ok(true)
            }
            OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, self.labels.len(), |idx, val| {
                    let ch = (idx % 2) as u8;
                    let v = if val == CTL_VALUE_MUTE { FeatureCtl::NEG_INFINITY } else { val as i16 };
                    let mut op = AudioFeature::new(self.vol_fb_ids[idx / 2], CtlAttr::Current, AudioCh::Each(ch),
                                                   FeatureCtl::Volume(vec![v]));
                    avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub struct HpCtl<'a> {
    vol_fb_id: u8,
    src_fb_id: u8,
    src_labels: &'a [&'a str],
    pub measure_elems: Vec<ElemId>,
}

impl<'a> HpCtl<'a> {
    const HP_VOL_NAME: &'a str = "headphone-volume";

    pub fn new(vol_fb_id: u8, src_fb_id: u8, src_labels: &'a [&'a str]) -> Self {
        HpCtl {
            vol_fb_id,
            src_fb_id,
            src_labels,
            measure_elems: Vec::new(),
        }
    }

    pub fn load(&mut self, _: &BebobAvc, card_cntr: &mut CardCntr) -> Result<(), Error> {
        // For source of headphone.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, HP_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &self.src_labels, None, true)?;

        // For volume of headphone.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::HP_VOL_NAME, 0);
        let elem_id_list = card_cntr.add_int_elems(&elem_id, 1, VOL_MIN, VOL_MAX, VOL_STEP, 2,
                                                   Some(&Into::<Vec<u32>>::into(VOL_TLV)), true)?;
        self.measure_elems.push(elem_id_list[0].clone());

        Ok(())
    }

    pub fn read(&mut self, avc: &BebobAvc, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            HP_SRC_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let mut op = AudioSelector::new(self.src_fb_id, CtlAttr::Current, 0xff);
                    avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
                    Ok(op.input_plug_id as u32)
                })?;
                Ok(true)
            }
            Self::HP_VOL_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| {
                    let mut op = AudioFeature::new(self.vol_fb_id, CtlAttr::Current, AudioCh::Each(idx as u8),
                                                   FeatureCtl::Volume(vec![-1]));
                    avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
                    if let FeatureCtl::Volume(data) = op.ctl {
                        let val = if data[0] == FeatureCtl::NEG_INFINITY { CTL_VALUE_MUTE } else { data[0] as i32 };
                        Ok(val)
                    } else {
                        unreachable!();
                    }
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &ElemId,
                 old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            HP_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    let mut op = AudioSelector::new(self.src_fb_id, CtlAttr::Current, val as u8);
                    avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
                })?;
                Ok(true)
            }
            Self::HP_VOL_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    let v = if val == CTL_VALUE_MUTE { FeatureCtl::NEG_INFINITY } else { val as i16 };
                    let mut op = AudioFeature::new(self.vol_fb_id, CtlAttr::Current, AudioCh::Each(idx as u8),
                                                   FeatureCtl::Volume(vec![v]));
                    avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
