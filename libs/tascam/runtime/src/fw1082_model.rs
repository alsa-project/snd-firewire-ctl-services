// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndTscmExtManual;

use alsactl::ElemId;

use core::card_cntr;

use tascam_protocols::isoch::{fw1082::*, *};

use super::isoch_ctls::*;

use super::protocol::ClkSrc;
use super::common_ctl::CommonCtl;
use super::console_ctl::ConsoleCtl;

use super::isoc_console_runtime::ConsoleData;

pub struct Fw1082Model<'a> {
    req: hinawa::FwReq,
    meter_ctl: MeterCtl,
    common: CommonCtl<'a>,
    console: ConsoleCtl,
}

#[derive(Default)]
struct MeterCtl(IsochMeterState, Vec<ElemId>);

impl AsRef<IsochMeterState> for MeterCtl {
    fn as_ref(&self) -> &IsochMeterState {
        &self.0
    }
}

impl AsMut<IsochMeterState> for MeterCtl {
    fn as_mut(&mut self) -> &mut IsochMeterState {
        &mut self.0
    }
}

impl IsochMeterCtl<Fw1082Protocol> for MeterCtl {
    const INPUT_LABELS: &'static [&'static str] = &[
        "analog-input-1", "analog-input-2", "analog-input-3", "analog-input-4",
        "analog-input-5", "analog-input-6", "analog-input-7", "analog-input-8",
        "spdif-input-1", "spdif-input-2",
    ];
    const OUTPUT_LABELS: &'static [&'static str] = &[
        "analog-output-1", "analog-output-2", "spdif-output-1", "spdif-output-2",
    ];
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
            meter_ctl: Default::default(),
            common: CommonCtl::new(Self::CLK_SRCS,
                                   Self::CLK_SRC_LABELS),
            console: ConsoleCtl::new(),
        }
    }
}

impl<'a> card_cntr::MeasureModel<hinawa::SndTscm> for Fw1082Model<'a> {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
        elem_id_list.extend_from_slice(&self.console.get_monitored_elems());
    }

    fn measure_states(&mut self, unit: &mut hinawa::SndTscm) -> Result<(), Error> {
        let image = unit.get_state()?;
        self.meter_ctl.parse_state(image)?;
        self.console.parse_states(image);
        Ok(())
    }

    fn measure_elem(&mut self, unit: &hinawa::SndTscm, elem_id: &alsactl::ElemId,
                    elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.console.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> card_cntr::CtlModel<hinawa::SndTscm> for Fw1082Model<'a> {
    fn load(
        &mut self,
        unit: &mut hinawa::SndTscm,
        card_cntr: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        let image = unit.get_state()?;
        self.meter_ctl.load_state(card_cntr, image)
            .map(|mut elem_id_list| self.meter_ctl.1.append(&mut elem_id_list))?;

        self.common.load(unit, &self.req, card_cntr)?;
        self.console.load(unit, &self.req, card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut hinawa::SndTscm,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.common.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else if self.console.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut hinawa::SndTscm,
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

    const SIMPLE_BUTTONS: &'a [(u32, u32)] = &[
        (8, 0x01000000),    // shift
        (8, 0x10000000),    // panel
        (9, 0x00001000),    // up
        (9, 0x00002000),    // left
        (9, 0x00004000),    // down
        (9, 0x00008000),    // right
        (9, 0x00010000),    // rec
        (9, 0x00200000),    // locate-l
        (9, 0x00400000),    // locate-r
        (9, 0x01000000),    // set
        (9, 0x02000000),    // in
        (9, 0x04000000),    // out
    ];

    const DIALS: &'a [((u32, u32), u8)] = &[
        ((14, 0x0000ffff), 0),    // eq gain, aux-1/5
        ((14, 0xffff0000), 16),   // eq freq, aux-2/6
        ((15, 0x0000ffff), 0),    // eq q, aux-3/7
        ((10, 0x0000ffff), 16),   // pan, aux-4/8

        ((15, 0xffff0000), 16),   // dial for transport.
    ];
}
