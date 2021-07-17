// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::Error;

use alsactl::{ElemId, ElemIfaceType, ElemValue};

use core::card_cntr::*;
use core::elem_value_accessor::ElemValueAccessor;

use ta1394::Ta1394Avc;
use ta1394::audio::{AUDIO_SUBUNIT_0_ADDR, CtlAttr, AudioCh, AudioProcessing, ProcessingCtl};

use bebob_protocols::*;

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
