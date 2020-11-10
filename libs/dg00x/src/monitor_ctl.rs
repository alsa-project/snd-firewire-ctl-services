// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::SndUnitExt;

use core::card_cntr;
use core::elem_value_accessor::ElemValueAccessor;

use super::protocol::{CommonProtocol, MonitorProtocol};

pub struct MonitorCtl {
    pub notified_elems: Vec<alsactl::ElemId>,
}

impl<'a> MonitorCtl {
    const ENABLE_NAME: &'a str = "monitor-enable";
    const SRC_GAIN_NAME: &'a str = "monitor-source-gain";

    const MIXER_LABELS: &'a [&'a str] = &["monitor-1", "monitor-2"];
    const IN_LABELS: &'a [&'a str] = &[
        "Analog-1", "Analog-2", "Analog-3", "Analog-4", "Analog-5", "Analog-6", "Analog-7",
        "Analog-8", "S/PDIF-1", "S/PDIF-2", "ADAT-1", "ADAT-2", "ADAT-3", "ADAT-4", "ADAT-5",
        "ADAT-6", "ADAT-7", "ADAT-8",
    ];

    const GAIN_MIN: i32 = 0;
    const GAIN_MAX: i32 = 0x80;
    const GAIN_STEP: i32 = 1;
    const GAIN_TLV: &'a [u32; 4] = &[5, 8, -4800i32 as u32, 0];

    const ENABLE_OFFSET: u64 = 0x0124;

    pub fn new() -> Self {
        MonitorCtl {
            notified_elems: Vec::new(),
        }
    }

    pub fn load(
        &mut self,
        _: &hinawa::SndDg00x,
        _: &hinawa::FwReq,
        card_cntr: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        let elem_id =
            alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, Self::ENABLE_NAME, 0);
        let mut elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;
        self.notified_elems.push(elem_id_list.remove(0));

        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::SRC_GAIN_NAME,
            0,
        );
        let mut elem_id_list = card_cntr.add_int_elems(
            &elem_id,
            Self::MIXER_LABELS.len(),
            Self::GAIN_MIN,
            Self::GAIN_MAX,
            Self::GAIN_STEP,
            Self::IN_LABELS.len(),
            Some(Self::GAIN_TLV),
            true,
        )?;
        self.notified_elems.append(&mut elem_id_list);

        Ok(())
    }

    pub fn read_notified_elems(&mut self, unit: &hinawa::SndDg00x, req: &hinawa::FwReq,
                               elem_id: &alsactl::ElemId, elem_value: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        // Any write transaction to register has effective to configure internal multiplexer just
        // during packet streaming. Without packet streaming, the transaction can change register
        // to keep its value, however has no effects to the internal multiplexer. Here, attempt to
        // update the registers with cached value at the beginning of packet streaming.
        if unit.get_property_streaming() {
            let dummy = alsactl::ElemValue::new();
            self.write(unit, req, elem_id, &dummy, elem_value)
        } else {
            Ok(false)
        }
    }

    pub fn read(
        &mut self,
        unit: &hinawa::SndDg00x,
        req: &hinawa::FwReq,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        let node = unit.get_node();

        match elem_id.get_name().as_str() {
            Self::ENABLE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    let val = req.read_quadlet(&node, Self::ENABLE_OFFSET)?;
                    Ok(val > 0)
                })?;
                Ok(true)
            }
            Self::SRC_GAIN_NAME => {
                let monitor = elem_id.get_index() as usize;
                ElemValueAccessor::<i32>::set_vals(elem_value, Self::IN_LABELS.len(), |idx| {
                    let val = req.read_gain(&node, monitor, idx)?;
                    Ok((val >> 24) as i32)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &hinawa::SndDg00x,
        req: &hinawa::FwReq,
        elem_id: &alsactl::ElemId,
        old: &alsactl::ElemValue,
        new: &alsactl::ElemValue,
    ) -> Result<bool, Error> {
        if !unit.get_property_streaming() {
            let label = "elements are not available without packet streaming";
            Err(Error::new(FileError::Again, &label))
        } else {
            let node = unit.get_node();

            match elem_id.get_name().as_str() {
                Self::ENABLE_NAME => {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        req.write_quadlet(&node, Self::ENABLE_OFFSET, val as u32)
                    })?;
                    Ok(true)
                }
                Self::SRC_GAIN_NAME => {
                    let monitor = elem_id.get_index() as usize;
                    ElemValueAccessor::<i32>::get_vals(new, old, Self::IN_LABELS.len(), |idx, val| {
                        let val = (val << 24) as u32;
                        req.write_gain(&node, monitor, idx, val)
                    })?;
                    Ok(true)
                }
                _ => Ok(false),
            }
        }
    }
}
