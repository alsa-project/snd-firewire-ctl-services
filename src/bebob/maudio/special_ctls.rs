// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndUnitExt;

use alsactl::{ElemValueExt, ElemValueExtManual};

use crate::card_cntr;

use crate::ta1394::{AvcAddr, Ta1394Avc};
use crate::ta1394::general::{InputPlugSignalFormat, OutputPlugSignalFormat};
use crate::ta1394::amdtp::{AmdtpEventType, AmdtpFdf, FMT_IS_AMDTP};

use crate::bebob::BebobAvc;
use crate::bebob::model::CLK_RATE_NAME;

use super::common_proto::FCP_TIMEOUT_MS;

pub struct ClkCtl{
    supported_clk_rates: Vec<u32>,
    pub notified_elem_list: Vec<alsactl::ElemId>,
}

impl<'a> ClkCtl {
    pub fn new(is_fw1814: bool) -> Self {
        let mut supported_clk_rates = Vec::new();
        supported_clk_rates.extend_from_slice(&[32000, 44100, 48000, 88200, 96000]);
        if is_fw1814 {
            supported_clk_rates.extend_from_slice(&[176400, 192000]);
        }
        ClkCtl{
            supported_clk_rates,
            notified_elem_list: Vec::new(),
        }
    }

    pub fn load(&mut self, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        let labels = self.supported_clk_rates.iter().map(|l| l.to_string()).collect::<Vec<String>>();

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card, 0, 0, CLK_RATE_NAME, 0);
        let mut elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        self.notified_elem_list.append(&mut elem_id_list);

        Ok(())
    }

    pub fn read(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            CLK_RATE_NAME => {
                let mut op = InputPlugSignalFormat::new(0);
                avc.status(&AvcAddr::Unit, &mut op, FCP_TIMEOUT_MS)?;

                let fdf = AmdtpFdf::from(op.fdf.as_ref());
                match self.supported_clk_rates.iter().position(|r| *r == fdf.freq) {
                    Some(p) => {
                        elem_value.set_enum(&[p as u32]);
                        Ok(true)
                    }
                    None => Ok(false),
                }
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, unit: &hinawa::SndUnit, avc: &BebobAvc, elem_id: &alsactl::ElemId,
                 _: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            CLK_RATE_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);

                let freq = self.supported_clk_rates[vals[0] as usize];
                let fdf = AmdtpFdf::new(AmdtpEventType::Am824, false, freq);

                unit.lock()?;
                let mut op = OutputPlugSignalFormat{
                    plug_id: 0,
                    fmt: FMT_IS_AMDTP,
                    fdf: fdf.into(),
                };
                let mut res = avc.control(&AvcAddr::Unit, &mut op, FCP_TIMEOUT_MS * 2);
                if res.is_ok() {
                    let mut op = InputPlugSignalFormat{
                        plug_id: 0,
                        fmt: FMT_IS_AMDTP,
                        fdf: fdf.into(),
                    };
                    res = avc.control(&AvcAddr::Unit, &mut op, FCP_TIMEOUT_MS * 2)
                }
                unit.unlock()?;
                res.and(Ok(true))
            }
            _ => Ok(false),
        }
    }
}
