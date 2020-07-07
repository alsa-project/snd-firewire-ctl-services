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
        ((7, 0x00000100), &[188, 200]),     // aux-5
        ((7, 0x00000200), &[187, 201]),     // aux-7
        ((7, 0x00000400), &[92, 105]),      // aux-6
        ((7, 0x00000800), &[6]),            // aux-8
        ((7, 0x00010000), &[154, 167]),     // flip
        ((7, 0x00020000), &[155, 168]),     // aux-1
        ((7, 0x00040000), &[156, 169]),     // aux-3
        ((7, 0x00080000), &[90, 103]),      // pan
        ((7, 0x00100000), &[89, 102]),      // aux-2
        ((7, 0x00200000), &[91, 104]),      // aux-4

        ((9, 0x00000100), &[12]),           // high
        ((9, 0x00000200), &[31, 44]),       // hi-mid
        ((9, 0x00000400), &[63, 76]),       // low-mid
        ((9, 0x00000800), &[95, 108]),      // low
        ((9, 0x00800000), &[77]),           // shuttle
    ];

    const SIMPLE_BUTTONS: &'a [(u32, u32)] = &[
        (7, 0x01000000),    // panel
        (8, 0x01000000),    // pfl
        (8, 0x02000000),    // computer
        (8, 0x10000000),    // clock
        (9, 0x00001000),    // up
        (9, 0x00002000),    // left
        (9, 0x00004000),    // down
        (9, 0x00008000),    // right
        (9, 0x00010000),    // eq-rec
        (9, 0x00020000),    // nudge-l
        (9, 0x00040000),    // nudge-r
        (9, 0x00200000),    // locate-l
        (9, 0x00400000),    // locate-r
        (9, 0x01000000),    // set
        (9, 0x02000000),    // in
        (9, 0x04000000),    // out
    ];

    const DIALS: &'a [((u32, u32), u8)] = &[
        ((10, 0x0000ffff), 0),    // rotary-1
        ((10, 0xffff0000), 16),   // rotary-2
        ((11, 0x0000ffff), 0),    // rotary-3
        ((11, 0xffff0000), 16),   // rotary-4
        ((12, 0x0000ffff), 0),    // rotary-5
        ((12, 0xffff0000), 16),   // rotary-6
        ((13, 0x0000ffff), 0),    // rotary-7
        ((13, 0xffff0000), 16),   // rotary-8

        ((14, 0x0000ffff), 0),    // gain
        ((14, 0xffff0000), 16),   // freq
        ((15, 0x0000ffff), 0),    // q

        ((15, 0xffff0000), 16),   // dial for transport.
    ];
}
