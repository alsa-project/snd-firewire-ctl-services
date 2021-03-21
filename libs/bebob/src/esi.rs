// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::Error;

use hinawa::{SndUnit, SndUnitExt, FwFcpExt};
use alsactl::{ElemId, ElemIfaceType, ElemValue};

use alsa_ctl_tlv_codec::items::{DbInterval, CTL_VALUE_MUTE};

use core::card_cntr::*;
use core::elem_value_accessor::ElemValueAccessor;

use ta1394::{*, ccm::*, audio::*};

use super::{*, common_ctls::ClkCtl};

pub struct QuatafireModel<'a>{
    avc: BebobAvc,
    clk_ctl: ClkCtl<'a>,
    input_ctl: InputCtl,
}

impl<'a> QuatafireModel<'a> {
    const FCP_TIMEOUT_MS: u32 = 100;

    const CLK_DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr{
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x01,
    });
    const CLK_SRCS: [SignalAddr;1] = [
        SignalAddr::Subunit(SignalSubunitAddr{
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x01,
        }),
    ];

    const CLK_LABELS: [&'a str;1] = [
        "Internal",
    ];
}

impl<'a> Default for QuatafireModel<'a> {
    fn default() -> Self {
        Self{
            avc: Default::default(),
            clk_ctl: ClkCtl::new(&Self::CLK_DST, &Self::CLK_SRCS, &Self::CLK_LABELS),
            input_ctl: Default::default(),
        }
    }
}

impl<'a> CtlModel<SndUnit> for QuatafireModel<'a> {
    fn load(&mut self, unit: &SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.fcp.bind(&unit.get_node())?;
        self.clk_ctl.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;
        self.input_ctl.load(card_cntr)?;
        Ok(())
    }

    fn read(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.input_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &SndUnit, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.write(unit, &self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.input_ctl.write(&self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> NotifyModel<SndUnit, bool> for QuatafireModel<'a> {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, _: &SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.clk_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)
    }
}

#[derive(Default, Debug)]
struct InputCtl;

const INPUT_GAIN_NAME: &str = "input-gain";
const INPUT_BALANCE_NAME: &str = "input-pan";

const GAIN_MIN: i32 = FeatureCtl::NEG_INFINITY as i32;
const GAIN_MAX: i32 = 0;
const GAIN_STEP: i32 = 1;
const GAIN_TLV: DbInterval = DbInterval{min: -12800, max: 0, linear: false, mute_avail: false};

const BALANCE_MIN: i32 = FeatureCtl::NEG_INFINITY as i32;
const BALANCE_MAX: i32 = FeatureCtl::INFINITY as i32;
const BALANCE_STEP: i32 = 1;

const INPUT_LABELS: [&str;6] = [
    "mic-input-1", "mic-input-2",
    "line-input-1", "line-input-2",
    "S/PDIF-input-1", "S/PDIF-input-2",
];

const INPUT_FB_IDS: [u8;3] = [1, 2, 3];

impl InputCtl {
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, GAIN_MIN, GAIN_MAX, GAIN_STEP, INPUT_LABELS.len(),
                                        Some(&Into::<Vec<u32>>::into(GAIN_TLV)), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_BALANCE_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, BALANCE_MIN, BALANCE_MAX, BALANCE_STEP, 2,
                                        None, true)?;

        Ok(())
    }

    fn read(&mut self, avc: &BebobAvc, elem_id: &ElemId, elem_value: &mut ElemValue,
            timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            INPUT_GAIN_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, INPUT_LABELS.len(), |idx| {
                    let func_blk_id = INPUT_FB_IDS[idx / 2];
                    let audio_ch_num = AudioCh::Each((idx % 2) as u8);
                    let mut op = AudioFeature::new(func_blk_id, CtlAttr::Current, audio_ch_num,
                                                   FeatureCtl::Volume(vec![-1]));
                    avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)?;
                    if let FeatureCtl::Volume(data) = op.ctl {
                        let val = if data[0] == FeatureCtl::NEG_INFINITY { CTL_VALUE_MUTE } else { data[0] as i32 };
                        Ok(val)
                    } else {
                        unreachable!();
                    }
                })
                .map(|_| true)
            }
            INPUT_BALANCE_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| {
                    let func_blk_id = INPUT_FB_IDS[idx / 2];
                    let audio_ch_num = AudioCh::Each((idx % 2) as u8);
                    let mut op = AudioFeature::new(func_blk_id, CtlAttr::Current, audio_ch_num,
                                                   FeatureCtl::LrBalance(-1));
                    avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)?;
                    if let FeatureCtl::LrBalance(val) = op.ctl {
                        Ok(val as i32)
                    } else {
                        unreachable!();
                    }
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, avc: &BebobAvc, elem_id: &ElemId, old: &ElemValue, new: &ElemValue,
             timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            INPUT_GAIN_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, INPUT_LABELS.len(), |idx, val| {
                    let func_blk_id = INPUT_FB_IDS[idx / 2];
                    let audio_ch_num = AudioCh::Each((idx % 2) as u8);
                    let v = if val == CTL_VALUE_MUTE { FeatureCtl::NEG_INFINITY } else { val as i16 };
                    let mut op = AudioFeature::new(func_blk_id, CtlAttr::Current, audio_ch_num,
                                                   FeatureCtl::Volume(vec![v]));
                    avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
                })
                .map(|_| true)
            }
            INPUT_BALANCE_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    let func_blk_id = INPUT_FB_IDS[idx / 2];
                    let audio_ch_num = AudioCh::Each((idx % 2) as u8);
                    let mut op = AudioFeature::new(func_blk_id, CtlAttr::Current, audio_ch_num,
                                                   FeatureCtl::LrBalance(val as i16));
                    avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
