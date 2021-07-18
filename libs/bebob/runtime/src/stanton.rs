// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{SndUnitExt, FwFcpExt};

use alsa_ctl_tlv_codec::items::{DbInterval, CTL_VALUE_MUTE};

use core::card_cntr;
use card_cntr::CtlModel;
use core::elem_value_accessor::ElemValueAccessor;

use ta1394::{MUSIC_SUBUNIT_0, Ta1394Avc};
use ta1394::ccm::{SignalAddr, SignalSubunitAddr};
use ta1394::audio::{AUDIO_SUBUNIT_0_ADDR, AudioFeature, FeatureCtl, CtlAttr, AudioCh};

use bebob_protocols::*;

use super::common_ctls::ClkCtl;
use super::model::OUT_VOL_NAME;

pub struct ScratchampModel<'a>{
    avc: BebobAvc,
    clk_ctl: ClkCtl<'a>,
}

impl<'a> ScratchampModel<'a> {
    const FCP_TIMEOUT_MS: u32 = 100;

    const CLK_DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr{
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x05,
    });
    const CLK_SRCS: &'a [SignalAddr] = &[
        SignalAddr::Subunit(SignalSubunitAddr{
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x05,
        }),
    ];

    const CLK_LABELS: &'a [&'a str] = &[
        "Internal",
    ];
}

impl<'a> Default for ScratchampModel<'a> {
    fn default() -> Self {
        Self{
            avc: Default::default(),
            clk_ctl: ClkCtl::new(&Self::CLK_DST, Self::CLK_SRCS, Self::CLK_LABELS),
        }
    }
}

impl<'a> CtlModel<hinawa::SndUnit> for ScratchampModel<'a> {
    fn load(&mut self, unit: &mut hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.get_node())?;

        self.clk_ctl.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;
        InputCtl::load(&self.avc, card_cntr)?;

        Ok(())
    }

    fn read(&mut self, _: &mut hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if InputCtl::read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut hinawa::SndUnit, elem_id: &alsactl::ElemId, old: &alsactl::ElemValue,
             new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.write(unit, &self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if InputCtl::write(&self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> card_cntr::NotifyModel<hinawa::SndUnit, bool> for ScratchampModel<'a> {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, _: &mut hinawa::SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId,
                          elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        self.clk_ctl.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)
    }
}

const VOL_MIN: i32 = i16::MIN as i32;
const VOL_MAX: i32 = 0x0000;
const VOL_STEP: i32 = 0x0080;
const VOL_TLV: DbInterval = DbInterval{min: -12800, max: 0, linear: false, mute_avail: true};

const OUTPUT_LABELS: &[&str] = &[
    "analog-1", "analog-2", "analog-3", "analog-4",
    "headphone-1", "headphone-2",
];

const FB_IDS: [u8;3] = [1, 2, 3];

trait InputCtl : Ta1394Avc {
    fn load(&self, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        // For volume of outputs.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, OUT_VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, VOL_MIN, VOL_MAX, VOL_STEP,
                                        OUTPUT_LABELS.len(),
                                        Some(&Into::<Vec<u32>>::into(VOL_TLV)), true)?;

        Ok(())
    }

    fn read(&self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, OUTPUT_LABELS.len(), |idx| {
                    let func_blk_id = FB_IDS[idx / 2];
                    let audio_ch_num = AudioCh::Each((idx % 2) as u8);
                    let mut op = AudioFeature::new(func_blk_id, CtlAttr::Current, audio_ch_num,
                                                   FeatureCtl::Volume(vec![-1]));
                    self.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)?;
                    if let FeatureCtl::Volume(data) = op.ctl {
                        let val = if data[0] == FeatureCtl::NEG_INFINITY { CTL_VALUE_MUTE } else { data[0] as i32 };
                        Ok(val)
                    } else {
                        unreachable!();
                    }
                })?;
                Ok(true)
            },
            _ => Ok(false),
        }
    }

    fn write(&self, elem_id: &alsactl::ElemId, old: &alsactl::ElemValue, new: &alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, OUTPUT_LABELS.len(), |idx, val| {
                    let func_blk_id = FB_IDS[idx / 2];
                    let audio_ch_num = AudioCh::Each((idx % 2) as u8);
                    let v = if val == CTL_VALUE_MUTE { FeatureCtl::NEG_INFINITY } else { val as i16 };
                    let mut op = AudioFeature::new(func_blk_id, CtlAttr::Current, audio_ch_num,
                                                   FeatureCtl::Volume(vec![v]));
                    self.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
                })?;
                Ok(true)
            },
            _ => Ok(false),
        }
    }
}

impl InputCtl for BebobAvc {}
