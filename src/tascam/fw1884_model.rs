// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndTscmExtManual;

use crate::card_cntr;

use super::protocol::ClkSrc;
use super::common_ctl::CommonCtl;
use super::meter_ctl::MeterCtl;
use super::optical_ctl::OpticalCtl;
use super::console_ctl::ConsoleCtl;

use super::isoc_console_unit::ConsoleData;

pub struct Fw1884Model<'a> {
    req: hinawa::FwReq,
    common: CommonCtl<'a>,
    meter: MeterCtl<'a>,
    optical: OpticalCtl<'a>,
    console: ConsoleCtl,
}

impl<'a> Fw1884Model<'a> {
    const CLK_SRCS: &'a [ClkSrc] = &[
        ClkSrc::Internal,
        ClkSrc::Wordclock,
        ClkSrc::Spdif,
        ClkSrc::Adat,
    ];

    const CLK_SRC_LABELS: &'a [&'a str] = &[
        "Internal",
        "Word-clock",
        "S/PDIF",
        "ADAT",
    ];

    const OPT_OUT_SRC_LABELS: &'a [&'a str] = &[
        "ADAT-1/2/3/4/5/6/7/8",
        "S/PDIF-1/2",
        "Analog-1/2/3/4/5/6/7/8",
    ];

    pub fn new() -> Self {
        Fw1884Model{
            req: hinawa::FwReq::new(),
            common: CommonCtl::new(Self::CLK_SRCS,
                                   Self::CLK_SRC_LABELS),
            meter: MeterCtl::new(Self::CLK_SRC_LABELS, 8, true, true),
            optical: OpticalCtl::new(Self::OPT_OUT_SRC_LABELS),
            console: ConsoleCtl::new(),
        }
    }
}

impl<'a> card_cntr::MonitorModel<hinawa::SndTscm> for Fw1884Model<'a> {
    fn get_monitored_elems(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.meter.get_monitored_elems());
        elem_id_list.extend_from_slice(&self.console.get_monitored_elems());
    }

    fn monitor_unit(&mut self, unit: &hinawa::SndTscm) -> Result<(), Error> {
        let states = unit.get_state()?;
        self.meter.parse_states(states);
        self.console.parse_states(states);
        Ok(())
    }

    fn monitor_elems(
        &mut self,
        unit: &hinawa::SndTscm,
        elem_id: &alsactl::ElemId,
        _: &alsactl::ElemValue,
        new: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        if self.meter.read(elem_id, new)? {
            Ok(true)
        } else if self.console.read(unit, &self.req, elem_id, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> card_cntr::CtlModel<hinawa::SndTscm> for Fw1884Model<'a> {
    fn load(
        &mut self,
        unit: &hinawa::SndTscm,
        card_cntr: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        self.common.load(unit, &self.req, card_cntr)?;
        self.meter.load(card_cntr)?;
        self.optical.load(unit, &self.req, card_cntr)?;
        self.console.load(unit, &self.req, card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        unit: &hinawa::SndTscm,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        if self.common.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else if self.optical.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else if self.console.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &hinawa::SndTscm,
        elem_id: &alsactl::ElemId,
        old: &alsactl::ElemValue,
        new: &alsactl::ElemValue,
    ) -> Result<bool, Error> {
        if self.common.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if self.optical.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if self.console.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> ConsoleData<'a> for Fw1884Model<'a> {
    const SIMPLE_LEDS: &'a [&'a [u16]] = &[
        &[3],               // ol-1
        &[22],              // ol-2
        &[35],              // ol-3
        &[54],              // ol-4
        &[67, 131],         // ol-5
        &[86, 150, 163],    // ol-6
        &[99, 182, 195],    // ol-7
        &[118, 214, 227],   // ol-8

        &[5],               // rec-1
        &[24],              // rec-2
        &[37],              // rec-3
        &[56],              // rec-4
        &[69, 133],         // rec-5
        &[88, 152, 165],    // rec-6
        &[101, 184, 197],   // rec-7
        &[120, 216, 229],   // rec-8
    ];

    const STATELESS_BUTTONS: &'a [((u32, u32), &'a [u16])] = &[
        ((7, 0x02000000), &[122, 135]),   // func-1
        ((7, 0x04000000), &[123, 136]),   // func-3
        ((7, 0x08000000), &[124, 137]),   // func-5
        ((7, 0x10000000), &[57, 70]),     // cut
        ((7, 0x20000000), &[58, 71]),     // copy
        ((7, 0x40000000), &[59, 72]),     // alt/cmd
        ((7, 0x80000000), &[60, 73]),     // shift

        ((8, 0x00000001), &[7]),          // func-2
        ((8, 0x00000002), &[8]),          // func-4
        ((8, 0x00000004), &[9]),          // func-6
        ((8, 0x00000008), &[25, 38]),     // del
        ((8, 0x00000010), &[26, 39]),     // paste
        ((8, 0x00000020), &[27, 40]),     // undo
        ((8, 0x00000040), &[28, 41]),     // ctrl

        ((9, 0x00000001), &[11]),         // func-7
        ((9, 0x00000002), &[30, 43]),     // func-8
        ((9, 0x00000004), &[62, 75]),     // func-9
        ((9, 0x00000008), &[94, 107]),    // func-10
        ((9, 0x00000010), &[126, 139]),   // read
        ((9, 0x00000020), &[158, 171]),   // wrt
        ((9, 0x00000040), &[190, 203]),   // tch
        ((9, 0x00000080), &[222, 235]),   // latch
    ];
}
