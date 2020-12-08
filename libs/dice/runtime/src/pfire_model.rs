// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExtManual};

use hinawa::FwReq;
use hinawa::{SndDice, SndUnitExt};

use core::card_cntr::*;
use core::elem_value_accessor::*;

use dice_protocols::tcat::{*, global_section::*};
use dice_protocols::tcat::extension::*;
use dice_protocols::maudio::*;

use super::common_ctl::*;
use super::tcd22xx_spec::*;
use super::tcd22xx_ctl::*;

#[derive(Default)]
pub struct PfireModel<S>
    where for<'a> S: AsRef<Tcd22xxState> + AsMut<Tcd22xxState> + Tcd22xxSpec<'a> + PfireClkSpec<'a>,
{
    proto: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    ctl: CommonCtl,
    tcd22xx_ctl: Tcd22xxCtl<S>,
    specific_ctl: SpecificCtl,
}

pub type Pfire2626Model = PfireModel<Pfire2626State>;
pub type Pfire610Model = PfireModel<Pfire610State>;

const TIMEOUT_MS: u32 = 20;

impl<S> CtlModel<SndDice> for PfireModel<S>
    where for<'a> S: AsRef<Tcd22xxState> + AsMut<Tcd22xxState> + Tcd22xxSpec<'a> + PfireClkSpec<'a>,
{
    fn load(&mut self, unit: &SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let node = unit.get_node();

        self.sections = self.proto.read_general_sections(&node, TIMEOUT_MS)?;
        let caps = ClockCaps::new(S::AVAIL_CLK_RATES, S::AVAIL_CLK_SRCS);
        let src_labels = self.proto.read_clock_source_labels(&node, &self.sections, TIMEOUT_MS)?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.extension_sections = self.proto.read_extension_sections(&node, TIMEOUT_MS)?;
        self.tcd22xx_ctl.load(unit, &self.proto, &self.extension_sections, &caps, &src_labels,
                          TIMEOUT_MS, card_cntr)?;
        self.specific_ctl.load(unit, &self.proto, &self.extension_sections, TIMEOUT_MS, card_cntr)?;

        self.tcd22xx_ctl.cache(unit, &self.proto, &self.sections, &self.extension_sections, TIMEOUT_MS)?;

        Ok(())
    }

    fn read(&mut self, unit: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read(unit, &self.proto, &self.sections, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read(unit, &self.proto, &self.extension_sections, elem_id,
                                    elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.specific_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &SndDice, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.write(unit, &self.proto, &self.sections, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.tcd22xx_ctl.write(unit, &self.proto, &self.extension_sections, elem_id,
                                     old, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.specific_ctl.write(unit, &self.proto, &self.extension_sections, elem_id,
                                          old, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<S> NotifyModel<SndDice, u32> for PfireModel<S>
    where for<'a> S: AsRef<Tcd22xxState> + AsMut<Tcd22xxState> + Tcd22xxSpec<'a> + PfireClkSpec<'a>,
{
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        self.tcd22xx_ctl.get_notified_elem_list(elem_id_list);
    }

    fn parse_notification(&mut self, unit: &SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl.parse_notification(unit, &self.proto, &self.sections, *msg, TIMEOUT_MS)?;
        self.tcd22xx_ctl.parse_notification(unit, &self.proto, &self.sections,
                                        &self.extension_sections, TIMEOUT_MS, *msg)?;
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<S> MeasureModel<hinawa::SndDice> for PfireModel<S>
    where for<'a> S: AsRef<Tcd22xxState> + AsMut<Tcd22xxState> + Tcd22xxSpec<'a> + PfireClkSpec<'a>,
{
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        self.tcd22xx_ctl.get_measured_elem_list(elem_id_list);
    }

    fn measure_states(&mut self, unit: &SndDice) -> Result<(), Error> {
        self.ctl.measure_states(unit, &self.proto, &self.sections, TIMEOUT_MS)?;
        self.tcd22xx_ctl.measure_states(unit, &self.proto, &self.extension_sections, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(&mut self, _: &SndDice, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

pub trait PfireClkSpec<'a> {
    const AVAIL_CLK_RATES: &'a [ClockRate] = &[
        ClockRate::R32000, ClockRate::R44100, ClockRate::R48000,
        ClockRate::R88200, ClockRate::R96000,
        ClockRate::R176400, ClockRate::R192000,
    ];

    const AVAIL_CLK_SRCS: &'a [ClockSource];
}

#[derive(Default, Debug)]
pub struct Pfire2626State(Tcd22xxState);

impl<'a> Tcd22xxSpec<'a> for Pfire2626State {
    const INPUTS: &'a [Input<'a>] = &[
        Input{id: SrcBlkId::Ins0, offset: 0, count: 8, label: None},
        Input{id: SrcBlkId::Adat, offset: 0, count: 8, label: None},
        Input{id: SrcBlkId::Adat, offset: 0, count: 8, label: None},
        Input{id: SrcBlkId::Aes, offset: 0, count: 2, label: None},
    ];
    const OUTPUTS: &'a [Output<'a>] = &[
        Output{id: DstBlkId::Ins0, offset: 0, count: 8, label: None},
        Output{id: DstBlkId::Adat, offset: 0, count: 8, label: None},
        Output{id: DstBlkId::Adat, offset: 0, count: 8, label: None},
        Output{id: DstBlkId::Aes, offset: 0, count: 2, label: None},
    ];
    const FIXED: &'a [SrcBlk] = &[
        SrcBlk{id: SrcBlkId::Ins0, ch: 0},
        SrcBlk{id: SrcBlkId::Ins0, ch: 1},
        SrcBlk{id: SrcBlkId::Ins0, ch: 2},
        SrcBlk{id: SrcBlkId::Ins0, ch: 3},
        SrcBlk{id: SrcBlkId::Ins0, ch: 4},
        SrcBlk{id: SrcBlkId::Ins0, ch: 5},
        SrcBlk{id: SrcBlkId::Ins0, ch: 6},
        SrcBlk{id: SrcBlkId::Ins0, ch: 7},
    ];
}

impl AsMut<Tcd22xxState> for Pfire2626State {
    fn as_mut(&mut self) -> &mut Tcd22xxState {
        &mut self.0
    }
}

impl AsRef<Tcd22xxState> for Pfire2626State {
    fn as_ref(&self) -> &Tcd22xxState {
        &self.0
    }
}

impl<'a> PfireClkSpec<'a> for Pfire2626State {
    const AVAIL_CLK_SRCS: &'a [ClockSource] = &[
            ClockSource::Aes1,
            ClockSource::Aes4,
            ClockSource::Adat,
            ClockSource::Tdif,
            ClockSource::WordClock,
            ClockSource::Internal,
    ];
}

#[derive(Default, Debug)]
pub struct Pfire610State(Tcd22xxState);

impl<'a> Tcd22xxSpec<'a> for Pfire610State {
    const INPUTS: &'a [Input<'a>] = &[
        Input{id: SrcBlkId::Ins0, offset: 0, count: 4, label: None},
        Input{id: SrcBlkId::Aes,  offset: 0, count: 2, label: None},
    ];
    const OUTPUTS: &'a [Output<'a>] = &[
        Output{id: DstBlkId::Ins0, offset: 0, count: 8, label: None},
        Output{id: DstBlkId::Aes,  offset: 0, count: 2, label: None},
    ];
    const FIXED: &'a [SrcBlk] = &[
        SrcBlk{id: SrcBlkId::Ins0, ch: 0},
        SrcBlk{id: SrcBlkId::Ins0, ch: 1},
    ];
}

impl AsRef<Tcd22xxState> for Pfire610State {
    fn as_ref(&self) -> &Tcd22xxState {
        &self.0
    }
}

impl AsMut<Tcd22xxState> for Pfire610State {
    fn as_mut(&mut self) -> &mut Tcd22xxState {
        &mut self.0
    }
}

impl<'a> PfireClkSpec<'a> for Pfire610State {
    const AVAIL_CLK_SRCS: &'a [ClockSource] = &[
            ClockSource::Aes1,
            ClockSource::Internal,
    ];
}

#[derive(Default, Debug)]
struct SpecificCtl{
    targets: [bool;KNOB_COUNT],
}

impl<'a> SpecificCtl {
    const MASTER_KNOB_NAME: &'a str = "master-knob-target";

    // MEMO: Both models support 'Output{id: DstBlkId::Ins0, count: 8}'.
    const MASTER_KNOB_TARGET_LABELS: &'a [&'a str] = &[
        "analog-out-1/2",
        "analog-out-3/4",
        "analog-out-5/6",
        "analog-out-7/8",
    ];

    fn load(&self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections, timeout_ms: u32,
            card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        let node = unit.get_node();
        proto.write_knob_assign(&node, sections, &self.targets, timeout_ms)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MASTER_KNOB_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::MASTER_KNOB_TARGET_LABELS.len(), true)?;
        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::MASTER_KNOB_NAME => {
                elem_value.set_bool(&self.targets);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &SndDice, proto: &FwReq, sections: &ExtensionSections,
             elem_id: &ElemId, old: &ElemValue, new: &ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MASTER_KNOB_NAME => {
                ElemValueAccessor::<bool>::get_vals(new, old, self.targets.len(), |idx, val| {
                    self.targets[idx] = val;
                    Ok(())
                })?;
                let node = unit.get_node();
                proto.write_knob_assign(&node, sections, &self.targets, timeout_ms)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
