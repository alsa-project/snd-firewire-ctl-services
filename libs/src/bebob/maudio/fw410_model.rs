// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{FwFcpExt, SndUnitExt};

use alsactl::{ElemValueExt, ElemValueExtManual};

use crate::card_cntr;
use card_cntr::{CtlModel, MeasureModel};

use crate::ta1394::{AvcAddr, MUSIC_SUBUNIT_0, Ta1394Avc};
use crate::ta1394::general::UnitInfo;
use crate::ta1394::ccm::{SignalAddr, SignalSubunitAddr, SignalUnitAddr};
use crate::ta1394::audio::{AUDIO_SUBUNIT_0_ADDR, CtlAttr, AudioCh, ProcessingCtl, AudioProcessing, AudioSelector};

use crate::bebob::common_ctls::ClkCtl;

use crate::bebob::BebobAvc;

use super::common_proto::FCP_TIMEOUT_MS;
use super::normal_ctls::{MeterCtl, MixerCtl, InputCtl, AuxCtl, OutputCtl, HpCtl};

pub struct Fw410Model<'a>{
    avc: BebobAvc,
    req: hinawa::FwReq,
    clk_ctl: ClkCtl<'a>,
    meter_ctl: MeterCtl<'a>,
    mixer_ctl: MixerCtl<'a>,
    input_ctl: InputCtl<'a>,
    aux_ctl: AuxCtl<'a>,
    output_ctl: OutputCtl<'a>,
    hp_ctl: HpCtl<'a>,
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
    const HP_SRC_LABELS: &'a [&'a str] = &["mixer", "aux-1/2"];

    const PHYS_IN_FB_IDS: &'a [u8] = &[0x03, 0x04];
    const STREAM_IN_FB_IDS: &'a [u8] = &[0x01, 0x01, 0x01, 0x01, 0x02];

    const AUX_OUT_FB_ID: u8 = 0x09;
    const AUX_PHYS_SRC_FB_IDS: &'a [u8] = &[0x07, 0x08];
    const AUX_STREAM_SRC_FB_IDS: &'a [u8] = &[0x05, 0x05, 0x05, 0x05, 0x06];

    const PHYS_OUT_LABELS: &'a [&'a str] = &[
        "analog-1/2", "analog-3/4", "analog-5/6", "analog-7/8",
        "digital-1/2",
    ];
    const PHYS_OUT_FB_IDS: &'a [u8] = &[0x0a, 0x0b, 0x0c, 0x0d, 0x0e];
    const PHYS_OUT_SRC_FB_IDS: &'a [u8] = &[0x02, 0x03, 0x04, 0x05, 0x06];

    const HP_SRC_FB_ID: u8 = 0x07;
    const HP_OUT_FB_ID: u8 = 0x0f;

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
            output_ctl: OutputCtl::new(
                Self::PHYS_OUT_LABELS,
                Self::PHYS_OUT_FB_IDS,
                Self::PHYS_OUT_SRC_FB_IDS,
            ),
            hp_ctl: HpCtl::new(Self::HP_OUT_FB_ID, Self::HP_SRC_FB_ID, Self::HP_SRC_LABELS),
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
        self.output_ctl.load(&self.avc, card_cntr)?;
        self.hp_ctl.load(&self.avc, card_cntr)?;
        HpMixerCtl::load(&self.avc, card_cntr)?;

        SpdifSrcCtl::load(&self.avc, card_cntr)?;

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
        } else if self.output_ctl.read(&self.avc, elem_id, elem_value)? {
            Ok(true)
        } else if self.hp_ctl.read(&self.avc, elem_id, elem_value)? {
            Ok(true)
        } else if HpMixerCtl::read(&self.avc, elem_id, elem_value)? {
            Ok(true)
        } else if SpdifSrcCtl::read(&self.avc, elem_id, elem_value)? {
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
        } else if self.output_ctl.write(&self.avc, elem_id, old, new)? {
            Ok(true)
        } else if self.hp_ctl.write(&self.avc, elem_id, old, new)? {
            Ok(true)
        } else if HpMixerCtl::write(&self.avc, elem_id, old, new)? {
            Ok(true)
        } else if SpdifSrcCtl::write(&self.avc, elem_id, old, new)? {
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

const HP_MIXER_SRC_NAME: &str = "headphone-mixer-source";
const HP_MIXER_DST_FB_ID: u8 = 0x07;
const HP_MIXER_SRC_FB_ID: u8 = 0x00;

const HP_MIXER_ON: i16 = 0x0000;
const HP_MIXER_OFF: i16 = (0x8000 as u16) as i16;

trait HpMixerCtl : Ta1394Avc {
    fn load(&self, card_cntr: &mut card_cntr::CardCntr,) -> Result<(), Error> {
        // For physical/stream inputs to headphone mixer.
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, HP_MIXER_SRC_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Fw410Model::PHYS_OUT_LABELS.len(), true)?;

        Ok(())
    }

    fn read(&self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            HP_MIXER_SRC_NAME => {
                let len = Fw410Model::PHYS_OUT_SRC_FB_IDS.len();
                let mut vals = vec![false;len];
                vals.iter_mut().enumerate()
                    .try_for_each(|(i, v)| {
                        // NOTE: The value of 0/1 for out_ch has the same effect.
                        let mut op = AudioProcessing::new(HP_MIXER_DST_FB_ID, CtlAttr::Current, HP_MIXER_SRC_FB_ID,
                                                          AudioCh::Each(Fw410Model::PHYS_OUT_SRC_FB_IDS[i]),
                                                          AudioCh::Each(0), ProcessingCtl::Mixer(vec![-1]));
                        self.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
                        if let ProcessingCtl::Mixer(data) = op.ctl {
                            *v = data[0] == HP_MIXER_ON;
                            Ok(())
                        } else {
                            unreachable!();
                        }
                    })?;
                elem_value.set_bool(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(&self, elem_id: &alsactl::ElemId, old: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            HP_MIXER_SRC_NAME => {
                let len = Fw410Model::PHYS_OUT_SRC_FB_IDS.len();
                let mut vals = vec![false;len * 2];
                new.get_bool(&mut vals[0..len]);
                old.get_bool(&mut vals[len..]);
                vals[..len].iter().zip(vals[len..].iter()).enumerate()
                    .filter(|(_, (n, o))| *n != *o)
                    .try_for_each(|(i, (v, _))| {
                        let ctl = ProcessingCtl::Mixer(vec![if *v { HP_MIXER_ON } else { HP_MIXER_OFF }]);
                        // NOTE: The value of 0/1 for out_ch has the same effect.
                        let mut op = AudioProcessing::new(HP_MIXER_DST_FB_ID, CtlAttr::Current, HP_MIXER_SRC_FB_ID,
                                            AudioCh::Each(Fw410Model::PHYS_OUT_SRC_FB_IDS[i]), AudioCh::Each(0), ctl);
                        self.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)
                    })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

impl HpMixerCtl for BebobAvc {}

const SPDIF_SRC_NAME: &str = "S/PDIF-input-source";
const SPDIF_SRC_LABELS: &[&str] = &["coaxial", "optical"];
const SPDIF_SRC_FB_ID: u8 = 0x01;

trait SpdifSrcCtl : Ta1394Avc {
    fn load(&self, card_cntr: &mut card_cntr::CardCntr) -> Result<(), Error> {
        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, SPDIF_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, SPDIF_SRC_LABELS, None, true)?;
        Ok(())
    }

    fn read(&self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            SPDIF_SRC_NAME => {
                let mut op = AudioSelector::new(SPDIF_SRC_FB_ID, CtlAttr::Current, 0xff);
                self.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
                elem_value.set_enum(&[op.input_plug_id as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(&self, elem_id: &alsactl::ElemId, _: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            SPDIF_SRC_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                let mut op = AudioSelector::new(SPDIF_SRC_FB_ID, CtlAttr::Current, vals[0] as u8);
                self.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, FCP_TIMEOUT_MS)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

impl SpdifSrcCtl for BebobAvc {}
