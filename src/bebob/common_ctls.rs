// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::SndUnitExt;

use alsactl::{ElemValueExt, ElemValueExtManual};

use crate::card_cntr::CardCntr;

use crate::ta1394::{AvcAddr, Ta1394Avc};
use crate::ta1394::general::{InputPlugSignalFormat, OutputPlugSignalFormat};
use crate::ta1394::amdtp::{AmdtpEventType, AmdtpFdf, FMT_IS_AMDTP};
use crate::ta1394::ccm::{SignalSource, SignalAddr};

use super::extensions::{BcoPlugDirection, BcoPlugAddr, BcoPlugAddrUnitType};
use super::extensions::{ExtendedStreamFormatSingle, ExtendedStreamFormatList};

use super::model::{CLK_RATE_NAME, CLK_SRC_NAME};

pub struct ClkCtl<'a> {
    supported_clk_rates: Vec<u32>,
    clk_dst: &'a SignalAddr,
    clk_srcs: &'a [SignalAddr],
    clk_src_labels: &'a [&'a str],
}

impl<'a> ClkCtl<'a> {
    pub fn new(clk_dst: &'a SignalAddr, clk_srcs: &'a [SignalAddr], clk_src_labels: &'a [&'a str]) -> Self {
        ClkCtl{
            supported_clk_rates: Vec::new(),
            clk_dst,
            clk_srcs,
            clk_src_labels,
        }
    }

    pub fn load<O>(&mut self, avc: &O, card_cntr: &mut CardCntr, timeout_ms: u32) -> Result<(), Error>
        where O: Ta1394Avc
    {
        let mut input_formats = Vec::new();
        let mut output_formats = Vec::new();

        // Detect stream formats for input/output direction.
        [BcoPlugDirection::Input, BcoPlugDirection::Output].iter().try_for_each(|direction| {
            let entries = match *direction {
                BcoPlugDirection::Input => &mut input_formats,
                BcoPlugDirection::Output => &mut output_formats,
                BcoPlugDirection::Reserved(_) => unreachable!(),
            };
            let plug_addr = BcoPlugAddr::new_for_unit(*direction, BcoPlugAddrUnitType::Isoc, 0);
            let _ = (0..0xff).try_for_each(|index| {
                let mut op = ExtendedStreamFormatList::new(&plug_addr, index);
                avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
                let format = op.stream_format.as_bco_compound_am824_stream()?;
                entries.push(format.clone());
                Ok::<(), Error>(())
            });
            Ok(())
        })?;

        // Scan available sampling frequencies.
        input_formats.iter().for_each(|entry| {
            if self.supported_clk_rates.iter().position(|r| entry.freq == *r).is_none() {
                self.supported_clk_rates.push(entry.freq);
            }
        });
        output_formats.iter().for_each(|entry| {
            if self.supported_clk_rates.iter().position(|r| entry.freq == *r).is_none() {
                self.supported_clk_rates.push(entry.freq);
            }
        });

        // Create labels.
        let labels = self.supported_clk_rates.iter().map(|r| r.to_string()).collect::<Vec<String>>();

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card, 0, 0, CLK_RATE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        if self.clk_srcs.len() > 1 {
            // NOTE: all of bebob models support "SignalAddr::Unit(SignalUnitAddr::Isoc(0x00))"
            // named as "PCR Compound Input" and "SignalAddr::Unit(SignalUnitAddr::Isoc(0x01))"
            // named as "PCR Sync Input" for source of sampling clock. They are available to be
            // synchronized to the series of syt field in incoming packets from the other unit on
            // IEEE 1394 bus. However, the most of models doesn't work with it actually even if
            // configured, therefore useless.
            let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card, 0, 0, CLK_SRC_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &self.clk_src_labels, None, true)?;
        }

        Ok(())
    }

    pub fn read<O>(&mut self, avc: &O, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue,
                   timeout_ms: u32)
        -> Result<bool, Error>
        where O: Ta1394Avc
    {
        match elem_id.get_name().as_str() {
            CLK_RATE_NAME => {
                let plug_addr = BcoPlugAddr::new_for_unit(BcoPlugDirection::Output, BcoPlugAddrUnitType::Isoc, 0);
                let mut op = ExtendedStreamFormatSingle::new(&plug_addr);
                avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
                let format = op.stream_format.as_bco_compound_am824_stream()?;
                match self.supported_clk_rates.iter().position(|r| *r == format.freq) {
                    Some(idx) => {
                        elem_value.set_enum(&[idx as u32]);
                        Ok(true)
                    }
                    None => Ok(false),
                }
            }
            CLK_SRC_NAME => {
                let mut op = SignalSource::new(self.clk_dst);
                avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
                match self.clk_srcs.iter().position(|s| *s == op.src) {
                    Some(idx) => {
                        elem_value.set_enum(&[idx as u32]);
                        Ok(true)
                    }
                    None => Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

    pub fn write<O>(&mut self, unit: &hinawa::SndUnit, avc: &O, elem_id: &alsactl::ElemId,
                 _: &alsactl::ElemValue, new: &alsactl::ElemValue, mut timeout_ms: u32)
        -> Result<bool, Error>
        where O: Ta1394Avc
    {
        // NOTE: Interim at first, then Accepted or Implemented/Stable.
        timeout_ms *= 2;
        match elem_id.get_name().as_str() {
            CLK_RATE_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                let freq = self.supported_clk_rates[vals[0] as usize];
                let fdf = AmdtpFdf::new(AmdtpEventType::Am824, false, freq);
                unit.lock()?;
                let mut op = InputPlugSignalFormat{
                    plug_id: 0,
                    fmt: FMT_IS_AMDTP,
                    fdf: fdf.into(),
                };
                let mut res = avc.control(&AvcAddr::Unit, &mut op, timeout_ms);
                if res.is_ok() {
                    let mut op = OutputPlugSignalFormat{
                        plug_id: 0,
                        fmt: FMT_IS_AMDTP,
                        fdf: fdf.into(),
                    };
                    res = avc.control(&AvcAddr::Unit, &mut op, timeout_ms);
                }
                let _ = unit.unlock();
                res.and(Ok(true))
            }
            CLK_SRC_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                match self.clk_srcs.iter().nth(vals[0] as usize) {
                    Some(src) => {
                        let mut op = SignalSource::new(self.clk_dst);
                        op.src = *src;
                        unit.lock()?;
                        let res = avc.control(&AvcAddr::Unit, &mut op, timeout_ms);
                        let _ = unit.unlock();
                        res.and(Ok(true))
                    }
                    None => Ok(false),
                }
            }
            _ => Ok(false),
        }
    }
}
