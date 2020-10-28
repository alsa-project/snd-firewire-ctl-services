// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{SndUnitExt, FwFcpExt};
use alsactl::{ElemValueExt, ElemValueExtManual};

use core::card_cntr;
use card_cntr::CtlModel;

use crate::ta1394::{MUSIC_SUBUNIT_0, Ta1394Avc};
use crate::ta1394::ccm::{SignalAddr, SignalSubunitAddr};
use crate::ta1394::audio::{AUDIO_SUBUNIT_0_ADDR, AudioFeature, FeatureCtl, CtlAttr, AudioCh};

use crate::bebob::BebobAvc;
use crate::bebob::common_ctls::ClkCtl;
use crate::bebob::model::OUT_VOL_NAME;

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

    pub fn new() -> Self {
        ScratchampModel{
            avc: BebobAvc::new(),
            clk_ctl: ClkCtl::new(&Self::CLK_DST, Self::CLK_SRCS, Self::CLK_LABELS),
        }
    }
}

impl<'a> CtlModel<hinawa::SndUnit> for ScratchampModel<'a> {
    fn load(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        self.avc.fcp.bind(&unit.get_node())?;

        self.clk_ctl.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;
        InputCtl::load(&self.avc, card_cntr)?;

        Ok(())
    }

    fn read(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
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

    fn write(&mut self, unit: &hinawa::SndUnit, elem_id: &alsactl::ElemId, old: &alsactl::ElemValue,
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

    fn parse_notification(&mut self, _: &hinawa::SndUnit, _: &bool) -> Result<(), Error> {
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
const VOL_TLV: &[i32;4] = &[4, 8, -12800, 0];

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
                                        OUTPUT_LABELS.len(), Some(VOL_TLV), true)?;

        Ok(())
    }

    fn read(&self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            OUT_VOL_NAME => {
                let mut vals = vec![0;OUTPUT_LABELS.len()];
                (0..OUTPUT_LABELS.len()).try_for_each(|i| {
                    let func_blk_id = FB_IDS[i / 2];
                    let audio_ch_num = AudioCh::Each((i % 2) as u8);
                    let mut op = AudioFeature::new(func_blk_id, CtlAttr::Current, audio_ch_num,
                                                   FeatureCtl::Volume(vec![-1]));
                    self.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)?;
                    if let FeatureCtl::Volume(data) = op.ctl {
                        vals[i] = data[0] as i32;
                        Ok(())
                    } else {
                        unreachable!();
                    }
                })?;
                elem_value.set_int(&vals);
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
                let mut vals = vec![0;OUTPUT_LABELS.len() * 2];
                new.get_int(&mut vals[..OUTPUT_LABELS.len()]);
                old.get_int(&mut vals[OUTPUT_LABELS.len()..]);
                vals[..OUTPUT_LABELS.len()].iter().zip(vals[OUTPUT_LABELS.len()..].iter()).enumerate()
                    .filter(|(_, (n, o))| *n != *o)
                    .try_for_each(|(i, (v, _))| {
                        let func_blk_id = FB_IDS[i / 2];
                        let audio_ch_num = AudioCh::Each((i % 2) as u8);
                        let mut op = AudioFeature::new(func_blk_id, CtlAttr::Current, audio_ch_num,
                                                       FeatureCtl::Volume(vec![*v as i16]));
                        self.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
                    })?;
                Ok(true)
            },
            _ => Ok(false),
        }
    }
}

impl InputCtl for BebobAvc {}
