// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::FwReq;
use hinawa::{SndTscm, SndTscmExtManual};

use alsactl::{ElemId, ElemValue};

use core::card_cntr::*;

use tascam_protocols::isoch::{fw1884::*, *};

use super::isoch_ctls::*;

use super::isoc_console_runtime::ConsoleData;

#[derive(Default)]
pub struct Fw1884Model {
    req: FwReq,
    meter_ctl: MeterCtl,
    common_ctl: CommonCtl,
    optical_ctl: OpticalCtl,
    console_ctl: ConsoleCtl,
}

const TIMEOUT_MS: u32 = 50;

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

impl IsochMeterCtl<Fw1884Protocol> for MeterCtl {
    const INPUT_LABELS: &'static [&'static str] = &[
        "analog-input-1", "analog-input-2", "analog-input-3", "analog-input-4",
        "analog-input-5", "analog-input-6", "analog-input-7", "analog-input-8",
        "adat-input-1", "adat-input-2", "adat-input-3", "adat-input-4",
        "adat-input-5", "adat-input-6", "adat-input-7", "adat-input-8",
        "spdif-input-1", "spdif-input-2",
    ];
    const OUTPUT_LABELS: &'static [&'static str] = &[
        "analog-output-1", "analog-output-2", "analog-output-3", "analog-output-4",
        "analog-output-5", "analog-output-6", "analog-output-7", "analog-output-8",
        "adat-output-1", "adat-output-2", "adat-output-3", "adat-output-4",
        "adat-output-5", "adat-output-6", "adat-output-7", "adat-output-8",
        "spdif-input-1", "spdif-input-2",
    ];
}

#[derive(Default)]
struct CommonCtl;

impl IsochCommonCtl<Fw1884Protocol> for CommonCtl {}

#[derive(Default)]
struct OpticalCtl;

impl IsochOpticalCtl<Fw1884Protocol> for OpticalCtl {
    const OPTICAL_OUTPUT_SOURCES: &'static [OpticalOutputSource] = &[
        OpticalOutputSource::StreamInputPairs,
        OpticalOutputSource::AnalogOutputPairs,
        OpticalOutputSource::CoaxialOutputPair0,
        OpticalOutputSource::AnalogInputPair0,
    ];
}

#[derive(Default)]
struct ConsoleCtl(IsochConsoleState, Vec<ElemId>);

impl AsRef<IsochConsoleState> for ConsoleCtl {
    fn as_ref(&self) -> &IsochConsoleState {
        &self.0
    }
}

impl AsMut<IsochConsoleState> for ConsoleCtl {
    fn as_mut(&mut self) -> &mut IsochConsoleState {
        &mut self.0
    }
}

impl IsochConsoleCtl<Fw1884Protocol> for ConsoleCtl {}

impl MeasureModel<SndTscm> for Fw1884Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
        elem_id_list.extend_from_slice(&self.console_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut hinawa::SndTscm) -> Result<(), Error> {
        let image = unit.get_state()?;
        self.meter_ctl.parse_state(image)?;
        self.console_ctl.parse_states(image)?;
        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &SndTscm,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.console_ctl.read_states(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl CtlModel<SndTscm> for Fw1884Model {
    fn load(
        &mut self,
        unit: &mut SndTscm,
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        let image = unit.get_state()?;
        self.meter_ctl.load_state(card_cntr, image)
            .map(|mut elem_id_list| self.meter_ctl.1.append(&mut elem_id_list))?;

        self.common_ctl.load_params(card_cntr)?;
        self.optical_ctl.load_params(card_cntr)?;

        self.console_ctl.load_params(card_cntr, image)
            .map(|mut elem_id_list| self.console_ctl.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut SndTscm,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.common_ctl.read_params(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.optical_ctl.read_params(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.console_ctl.read_params(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut SndTscm,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.write_params(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.optical_ctl.write_params(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.console_ctl.write_params(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a> ConsoleData<'a> for Fw1884Model {
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
