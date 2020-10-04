// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{FwFcpExt, SndUnitExt};

use alsactl::{ElemValueExt, ElemValueExtManual};

use crate::card_cntr;
use card_cntr::{CtlModel, MeasureModel};

use crate::ta1394::{MUSIC_SUBUNIT_0};
use crate::ta1394::ccm::{SignalAddr, SignalUnitAddr, SignalSubunitAddr};

use crate::bebob::BebobAvc;
use crate::bebob::common_ctls::ClkCtl;
use crate::bebob::model::OUT_METER_NAME;

use super::common_proto::{FCP_TIMEOUT_MS, CommonProto};

pub struct ProfirelightbridgeModel<'a> {
    avc: BebobAvc,
    req: hinawa::FwReq,
    clk_ctl: ClkCtl<'a>,
    meter_ctl: MeterCtl,
}

impl<'a> ProfirelightbridgeModel<'a> {
    const CLK_DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr{
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x07,
    });
    const CLK_SRCS: &'a [SignalAddr] = &[
        SignalAddr::Subunit(SignalSubunitAddr{
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x08,
        }),
        SignalAddr::Unit(SignalUnitAddr::Ext(0x01)),
        SignalAddr::Unit(SignalUnitAddr::Ext(0x02)),
        SignalAddr::Unit(SignalUnitAddr::Ext(0x03)),
        SignalAddr::Unit(SignalUnitAddr::Ext(0x04)),
        SignalAddr::Unit(SignalUnitAddr::Ext(0x05)),
        SignalAddr::Unit(SignalUnitAddr::Ext(0x06)),
    ];
    const CLK_LABELS: &'a [&'a str] = &[
        "Internal",
        "S/PDIF",
        "ADAT-1",
        "ADAT-2",
        "ADAT-3",
        "ADAT-4",
        "Word-clock",
    ];

    pub fn new() -> Self {
        ProfirelightbridgeModel {
            avc: BebobAvc::new(),
            req: hinawa::FwReq::new(),
            clk_ctl: ClkCtl::new(&Self::CLK_DST, Self::CLK_SRCS, Self::CLK_LABELS),
            meter_ctl: MeterCtl::new(),
        }
    }
}

impl<'a> CtlModel<hinawa::SndUnit> for ProfirelightbridgeModel<'a> {
    fn load(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        self.avc.fcp.bind(&unit.get_node())?;

        self.clk_ctl.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;
        self.meter_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
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
        } else {
            Ok(false)
        }
    }
}

impl<'a> MeasureModel<hinawa::SndUnit> for ProfirelightbridgeModel<'a> {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.measure_elems);
    }

    fn measure_states(&mut self, unit: &hinawa::SndUnit) -> Result<(), Error> {
        self.meter_ctl.measure_states(unit, &self.req)
    }

    fn measure_elem(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        self.meter_ctl.measure_elem(elem_id, elem_value)
    }
}

struct MeterCtl {
    measure_elems: Vec<alsactl::ElemId>,
    cache: [u8;Self::FRAME_COUNT],
}

impl<'a> MeterCtl {
    const FRAME_COUNT: usize = 56;

    const DETECTED_RATE_NAME: &'a str = "Detected rate";
    const SYNC_STATUS_NAME: &'a str = "Sync status";

    const METER_LABELS: &'a [&'a str] = &["analog-out-1", "analog-out-2"];
    const RATE_LABELS: &'a [&'a str] = &["44100", "48000", "88200", "96000"];

    const METER_MIN: i32 = 0;
    const METER_MAX: i32 = 0x007fffff;
    const METER_STEP: i32 = 256;
    const METER_TLV: &'a [i32] = &[5, 8, -14400, 0];

    fn new() -> Self {
        MeterCtl {
            measure_elems: Vec::new(),
            cache: [0;Self::FRAME_COUNT],
        }
    }

    fn load(&mut self, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        // For metering.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, OUT_METER_NAME, 0);
        let elem_id_list = card_cntr.add_int_elems(&elem_id, 1,
                                                   Self::METER_MIN, Self::METER_MAX, Self::METER_STEP,
                                                   Self::METER_LABELS.len(), Some(Self::METER_TLV), false)?;
        self.measure_elems.push(elem_id_list[0].clone());

        // For detection of sampling clock frequency.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::DETECTED_RATE_NAME, 0);
        let elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, Self::RATE_LABELS, None, false)?;
        self.measure_elems.push(elem_id_list[0].clone());

        // For sync status.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::SYNC_STATUS_NAME, 0);
        let elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, 1, false)?;
        self.measure_elems.push(elem_id_list[0].clone());

        Ok(())
    }

    pub fn measure_states(&mut self, unit: &hinawa::SndUnit, req: &hinawa::FwReq) -> Result<(), Error> {
        req.read_meters(unit, &mut self.cache)
    }

    fn measure_elem(&mut self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            OUT_METER_NAME => {
                let mut vals = [0;2];
                vals.iter_mut().enumerate().for_each(|(i, v)| {
                    let mut quadlet = [0;4];
                    let pos = 4 + i * 4;
                    quadlet.copy_from_slice(&self.cache[pos..(pos + 4)]);
                    *v = i32::from_be_bytes(quadlet);
                });
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::DETECTED_RATE_NAME => {
                let mut quadlet = [0;4];
                quadlet.copy_from_slice(&self.cache[0..4]);
                let val = u32::from_be_bytes(quadlet);
                if val > 0 && val <= Self::RATE_LABELS.len() as u32 {
                    elem_value.set_enum(&[val - 1]);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::SYNC_STATUS_NAME => {
                let mut quadlet = [0;4];
                quadlet.copy_from_slice(&self.cache[20..24]);
                elem_value.set_bool(&[u32::from_be_bytes(quadlet) != 2]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

impl<'a> card_cntr::NotifyModel<hinawa::SndUnit, bool> for ProfirelightbridgeModel<'a> {
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
