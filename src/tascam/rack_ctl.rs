// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndUnitExt;

use alsactl::{ElemValueExt, ElemValueExtManual};

use crate::card_cntr;

use super::protocol::RackProtocol;

pub struct RackCtl {
    cache: [u8; 72],
}

impl<'a> RackCtl {
    const GAIN_NAME: &'a str = "input-gain";
    const BALANCE_NAME: &'a str = "input-balance";
    const MUTE_NAME: &'a str = "input-mute";

    const INPUT_LABELS: &'a [&'a str] = &[
        "Analog-1", "Analog-2", "Analog-3", "Analog-4", "Analog-5", "Analog-6", "Analog-7",
        "Analog-8", "ADAT-1", "ADAT-2", "ADAT-3", "ADAT-4", "ADAT-5", "ADAT-6", "ADAT-7", "ADAT-8",
        "S/PDIF-1", "S/PDIF-2",
    ];

    const GAIN_MIN: i32 = 0;
    const GAIN_MAX: i32 = 32767;
    const GAIN_STEP: i32 = 256;

    const BALANCE_MIN: i32 = 0;
    const BALANCE_MAX: i32 = 255;
    const BALANCE_STEP: i32 = 1;

    pub fn new() -> Self {
        RackCtl { cache: [0; 72] }
    }

    pub fn load(
        &mut self,
        unit: &hinawa::SndTscm,
        req: &hinawa::FwReq,
        card_cntr: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        let node = unit.get_node();

        req.init_states(&node, &mut self.cache)?;

        // For gain of input.
        let elem_id =
            alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, Self::GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::GAIN_MIN,
            Self::GAIN_MAX,
            Self::GAIN_STEP,
            Self::INPUT_LABELS.len(),
            None,
            true,
        )?;

        // For l/r balance.
        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Mixer,
            0,
            0,
            Self::BALANCE_NAME,
            0,
        );
        let _ =
            card_cntr.add_int_elems(&elem_id, 1,
                                    Self::BALANCE_MIN, Self::BALANCE_MAX, Self::BALANCE_STEP,
                                    Self::INPUT_LABELS.len(), None, true)?;

        // For mute of input.
        let elem_id =
            alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, Self::MUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::INPUT_LABELS.len(), true)?;

        Ok(())
    }

    pub fn read(
        &mut self,
        _: &hinawa::SndTscm,
        req: &hinawa::FwReq,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::GAIN_NAME => {
                let mut vals = vec![0;Self::INPUT_LABELS.len()];
                vals.iter_mut().enumerate().try_for_each(|(i, val)| {
                    *val = req.get_gain(&self.cache, i)? as i32;
                    Ok(())
                })?;
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::BALANCE_NAME => {
                let mut vals = vec![0;Self::INPUT_LABELS.len()];
                vals.iter_mut().enumerate().try_for_each(|(i, val)| {
                    *val = req.get_balance(&self.cache, i)? as i32;
                    Ok(())
                })?;
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::MUTE_NAME => {
                let mut vals = vec![false;Self::INPUT_LABELS.len()];
                vals.iter_mut().enumerate().try_for_each(|(i, val)| {
                    *val = req.get_mute(&self.cache, i)?;
                    Ok(())
                })?;
                elem_value.set_bool(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &hinawa::SndTscm,
        req: &hinawa::FwReq,
        elem_id: &alsactl::ElemId,
        old: &alsactl::ElemValue,
        new: &alsactl::ElemValue,
    ) -> Result<bool, Error> {
        let node = unit.get_node();

        match elem_id.get_name().as_str() {
            Self::GAIN_NAME => {
                let len = Self::INPUT_LABELS.len();
                let mut vals = vec![0; len * 2];
                old.get_int(&mut vals[0..len]);
                new.get_int(&mut vals[len..]);

                (0..len).try_for_each(|i| {
                    if vals[i] != vals[len + i] {
                        req.set_gain(&node, &mut self.cache, i, vals[len + i] as i16)?;
                    }
                    Ok(())
                })?;

                Ok(true)
            }
            Self::BALANCE_NAME => {
                let len = Self::INPUT_LABELS.len();
                let mut vals = vec![0; len * 2];
                old.get_int(&mut vals[0..len]);
                new.get_int(&mut vals[len..]);

                (0..len).try_for_each(|i| {
                    if vals[i] != vals[len + i] {
                        req.set_balance(&node, &mut self.cache, i, vals[len + i] as u8)?;
                    }
                    Ok(())
                })?;

                Ok(true)
            }
            Self::MUTE_NAME => {
                let len = Self::INPUT_LABELS.len();
                let mut vals = vec![false; len * 2];
                old.get_bool(&mut vals[0..len]);
                new.get_bool(&mut vals[len..]);

                (0..len).try_for_each(|i| {
                    if vals[i] != vals[len + i] {
                        req.set_mute(&node, &mut self.cache, i, vals[len + i])?;
                    }
                    Ok(())
                })?;

                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
