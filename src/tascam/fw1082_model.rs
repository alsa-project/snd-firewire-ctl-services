// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndTscmExtManual;

use crate::card_cntr;

use super::protocol::ClkSrc;
use super::common_ctl::CommonCtl;
use super::meter_ctl::MeterCtl;
use super::console_ctl::ConsoleCtl;

use super::isoc_console_unit::ConsoleData;

pub struct Fw1082Model<'a> {
    req: hinawa::FwReq,
    common: CommonCtl<'a>,
    meter: MeterCtl<'a>,
    console: ConsoleCtl,
}

impl<'a> Fw1082Model<'a> {
    const CLK_SRCS: &'a [ClkSrc] = &[
        ClkSrc::Internal,
        ClkSrc::Spdif,
    ];

    const CLK_SRC_LABELS: &'a [&'a str] = &[
        "Internal",
        "S/PDIF",
    ];

    pub fn new() -> Self {
        Fw1082Model{
            req: hinawa::FwReq::new(),
            common: CommonCtl::new(Self::CLK_SRCS,
                                   Self::CLK_SRC_LABELS),
            meter: MeterCtl::new(Self::CLK_SRC_LABELS, 2, false, false),
            console: ConsoleCtl::new(),
        }
    }
}

impl<'a> card_cntr::MonitorModel<hinawa::SndTscm> for Fw1082Model<'a> {
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

impl<'a> card_cntr::CtlModel<hinawa::SndTscm> for Fw1082Model<'a> {
    fn load(
        &mut self,
        unit: &hinawa::SndTscm,
        card_cntr: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        self.common.load(unit, &self.req, card_cntr)?;
        self.meter.load(card_cntr)?;
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
        } else if self.console.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> ConsoleData<'a> for Fw1082Model<'a> {
    const SIMPLE_LEDS: &'a [&'a [u16]] = &[
        &[3],               // ol-1
        &[22],              // ol-2
        &[35],              // ol-3
        &[54],              // ol-4
        &[67, 131],         // ol-5
        &[86, 150, 163],    // ol-6
        &[99, 182, 195],    // ol-7
        &[118, 214, 227],   // ol-8

        &[4],               // signal-1
        &[23],              // signal-2
        &[36],              // signal-3
        &[55],              // signal-4
        &[68, 132],         // signal-5
        &[87, 151, 164],    // signal-6
        &[100, 183, 196],   // signal-7
        &[119, 215, 228],   // signal-8

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
        ((9, 0x00000001), &[11]),       // f1/5
        ((9, 0x00000002), &[30, 43]),   // f2/6
        ((9, 0x00000004), &[62, 75]),   // f3/7
        ((9, 0x00000008), &[94, 107]),  // f4/8
    ];

    const TOGGLED_BUTTONS: &'a [((u32, u32), &'a [u16])] = &[
        ((6, 0x00010000), &[0]),            // select-1
        ((6, 0x00020000), &[19, 32]),       // select-2
        ((6, 0x00040000), &[51, 64]),       // select-3
        ((6, 0x00080000), &[83, 96]),       // select-4
        ((6, 0x00100000), &[115, 128]),     // select-5
        ((6, 0x00200000), &[147, 160]),     // select-6
        ((6, 0x00400000), &[179, 192]),     // select-7
        ((6, 0x00800000), &[211, 224]),     // select-8
        ((6, 0x01000000), &[1]),            // solo-1
        ((6, 0x02000000), &[20, 33]),       // solo-2
        ((6, 0x04000000), &[52, 65]),       // solo-3
        ((6, 0x08000000), &[84, 97]),       // solo-4
        ((6, 0x10000000), &[116, 129]),     // solo-5
        ((6, 0x20000000), &[148, 161]),     // solo-6
        ((6, 0x40000000), &[180, 193]),     // solo-7
        ((6, 0x80000000), &[212, 225]),     // solo-8

        ((7, 0x00000001), &[2]),            // mute-1
        ((7, 0x00000002), &[21, 34]),       // mute-2
        ((7, 0x00000004), &[53, 66]),       // mute-3
        ((7, 0x00000008), &[85, 98]),       // mute-4
        ((7, 0x00000010), &[117, 130]),     // mute-5
        ((7, 0x00000020), &[149, 162]),     // mute-6
        ((7, 0x00000040), &[181, 194]),     // mute-7
        ((7, 0x00000080), &[213, 226]),     // mute-8

        ((8, 0x20000000), &[157, 170]),     // eq, pan
        ((8, 0x40000000), &[189, 202]),     // aux-1/2/3/4
        ((8, 0x80000000), &[221, 234]),     // aux-5/6/7/8

        ((9, 0x00000100), &[12]),           // eq-hi, aux-1/5
        ((9, 0x00000200), &[31, 44]),       // eq-hi-mid, aux-2/6
        ((9, 0x00000400), &[63, 76]),       // eq-lo-mid, aux-3/7
        ((9, 0x00000800), &[95, 108]),      // eq-low, aux-4/8
        ((9, 0x00800000), &[77]),           // shuttle
    ];
}
