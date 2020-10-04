// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{FwFcpExt, SndUnitExt};

use crate::card_cntr;
use card_cntr::{CtlModel, MeasureModel};

use crate::ta1394::{AvcAddr, MUSIC_SUBUNIT_0, Ta1394Avc};
use crate::ta1394::general::UnitInfo;
use crate::ta1394::ccm::{SignalAddr, SignalSubunitAddr, SignalUnitAddr};

use crate::bebob::common_ctls::ClkCtl;

use crate::bebob::BebobAvc;

use super::common_proto::FCP_TIMEOUT_MS;
use super::normal_ctls::{MeterCtl, MixerCtl, InputCtl, AuxCtl};

pub struct Fw410Model<'a>{
    avc: BebobAvc,
    req: hinawa::FwReq,
    clk_ctl: ClkCtl<'a>,
    meter_ctl: MeterCtl<'a>,
    mixer_ctl: MixerCtl<'a>,
    input_ctl: InputCtl<'a>,
    aux_ctl: AuxCtl<'a>,
}

impl<'a> Fw410Model<'a> {
    const CLK_DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr{
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x01,
    });
    const CLK_SRCS: &'a [SignalAddr] = &[
        SignalAddr::Subunit(SignalSubunitAddr{
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x01,
        }),
        SignalAddr::Unit(SignalUnitAddr::Ext(0x02)),
    ];
    const CLK_LABELS: &'a [&'a str] = &["Internal", "S/PDIF"];

    const IN_METER_LABELS: &'a [&'a str] = &[
        "analog-in-1", "analog-in-2", "digital-in-1", "digital-in-2",
    ];

    const OUT_METER_LABELS: &'a [&'a str] = &[
        "analog-out-1", "analog-out-2", "analog-out-3", "analog-out-4",
        "analog-out-5", "analog-out-6", "analog-out-7", "analog-out-8",
        "digital-out-1", "digital-out-2",
    ];

    const MIXER_DST_FB_IDS: &'a [u8] = &[0x01, 0x01, 0x01, 0x01, 0x01];
    const MIXER_LABELS: &'a [&'a str] = &[
        "mixer-1/2", "mixer-3/4", "mixer-5/6", "mixer-7/8",
        "mixer-9/10",
    ];
    const MIXER_PHYS_SRC_FB_IDS: &'a [u8] = &[0x02, 0x03];
    const PHYS_IN_LABELS: &'a [&'a str] = &["analog-1/2", "digital-1/2"];
    const MIXER_STREAM_SRC_FB_IDS: &'a [u8] = &[0x01, 0x00, 0x00, 0x00, 0x00];
    const STREAM_IN_LABELS: &'a [&'a str] = &[
        "stream-1/2", "stream-3/4", "stream-5/6", "stream-7/8",
        "stream-9/10",
    ];

    const PHYS_IN_FB_IDS: &'a [u8] = &[0x03, 0x04];
    const STREAM_IN_FB_IDS: &'a [u8] = &[0x01, 0x01, 0x01, 0x01, 0x02];

    const AUX_OUT_FB_ID: u8 = 0x09;
    const AUX_PHYS_SRC_FB_IDS: &'a [u8] = &[0x07, 0x08];
    const AUX_STREAM_SRC_FB_IDS: &'a [u8] = &[0x05, 0x05, 0x05, 0x05, 0x06];

    pub fn new() -> Self {
        Fw410Model{
            avc: BebobAvc::new(),
            req: hinawa::FwReq::new(),
            clk_ctl: ClkCtl::new(&Self::CLK_DST, Self::CLK_SRCS, Self::CLK_LABELS),
            meter_ctl: MeterCtl::new(Self::IN_METER_LABELS, &[], Self::OUT_METER_LABELS, false, 1, true),
            mixer_ctl: MixerCtl::new(
                Self::MIXER_DST_FB_IDS, Self::MIXER_LABELS,
                Self::MIXER_PHYS_SRC_FB_IDS, Self::PHYS_IN_LABELS,
                Self::MIXER_STREAM_SRC_FB_IDS, Self::STREAM_IN_LABELS,
            ),
            input_ctl: InputCtl::new(
                Self::PHYS_IN_FB_IDS, Self::PHYS_IN_LABELS,
                Self::STREAM_IN_FB_IDS, Self::STREAM_IN_LABELS,
            ),
            aux_ctl: AuxCtl::new(Self::AUX_OUT_FB_ID,
                Self::AUX_PHYS_SRC_FB_IDS, Self::PHYS_IN_LABELS,
                Self::AUX_STREAM_SRC_FB_IDS, Self::STREAM_IN_LABELS,
            ),
        }
    }
}

impl<'a> CtlModel<hinawa::SndUnit> for Fw410Model<'a> {
    fn load(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        self.avc.fcp.bind(&unit.get_node())?;

        let mut op = UnitInfo::new();
        self.avc.status(&AvcAddr::Unit, &mut op, FCP_TIMEOUT_MS)?;
        self.avc.company_id = op.company_id;

        self.clk_ctl.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;
        self.meter_ctl.load(unit, &self.avc, &self.req, card_cntr)?;
        self.mixer_ctl.load(&self.avc, card_cntr)?;
        self.input_ctl.load(&self.avc, card_cntr)?;
        self.aux_ctl.load(&self.avc, card_cntr)?;

        Ok(())
    }

    fn read(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(&self.avc, elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read(&self.avc, elem_id, elem_value)? {
            Ok(true)
        } else if self.aux_ctl.read(&self.avc, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &hinawa::SndUnit, elem_id: &alsactl::ElemId,
             old: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.write(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.meter_ctl.write(&self.avc, elem_id, old, new)? {
            Ok(true)
        } else if self.mixer_ctl.write(&self.avc, elem_id, old, new)? {
            Ok(true)
        } else if self.input_ctl.write(&self.avc, elem_id, old, new)? {
            Ok(true)
        } else if self.aux_ctl.write(&self.avc, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> MeasureModel<hinawa::SndUnit> for Fw410Model<'a> {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.measure_elems);
    }

    fn measure_states(&mut self, unit: &hinawa::SndUnit) -> Result<(), Error> {
        self.meter_ctl.measure_states(unit, &self.avc, &self.req)
    }

    fn measure_elem(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        self.meter_ctl.measure_elem(elem_id, elem_value)
    }
}

impl<'a> card_cntr::NotifyModel<hinawa::SndUnit, bool> for Fw410Model<'a> {
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
        self.clk_ctl.read(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)
    }
}
