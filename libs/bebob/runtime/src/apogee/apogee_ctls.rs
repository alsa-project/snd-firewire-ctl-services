// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::SndUnitExt;

use alsactl::{ElemValueExt, ElemValueExtManual};

use core::card_cntr;

use ta1394::{AvcAddr, Ta1394Avc};

use bebob_protocols::{apogee::ensemble::*, *};
use bebob_protocols::bridgeco::{BcoPlugAddr, BcoPlugDirection, BcoPlugAddrUnitType};
use bebob_protocols::bridgeco::BcoCompoundAm824StreamFormat;
use bebob_protocols::bridgeco::ExtendedStreamFormatSingle;

use bebob_protocols::apogee::ensemble::{EnsembleOperation, EnsembleCmd, HwCmd};

pub struct HwCtl{
    stream: StreamMode,
}

fn stream_mode_to_str(mode: &StreamMode) -> &str {
    match mode {
        StreamMode::Format18x18 => "18x18",
        StreamMode::Format10x10 => "10x10",
        StreamMode::Format8x8 => "8x8",
    }
}

impl<'a> HwCtl {
    const STREAM_MODE_NAME: &'a str = "stream-mode";

    const STREAM_MODES: [StreamMode; 3] = [
        StreamMode::Format18x18,
        StreamMode::Format10x10,
        StreamMode::Format8x8,
    ];

    pub fn new() -> Self {
        HwCtl {
            stream: Default::default(),
        }
    }

    pub fn load(&mut self, avc: &BebobAvc, card_cntr: &mut card_cntr::CardCntr, timeout_ms: u32)
        -> Result<(), Error>
    {
        let plug_addr = BcoPlugAddr::new_for_unit(BcoPlugDirection::Output, BcoPlugAddrUnitType::Isoc,
                                                  0);
        let mut op = ExtendedStreamFormatSingle::new(&plug_addr);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
        let info = op.stream_format.as_bco_compound_am824_stream()?;
        let count = info.entries.iter()
            .filter(|entry| entry.format == BcoCompoundAm824StreamFormat::MultiBitLinearAudioRaw)
            .fold(0, |count, entry| count + entry.count as usize);
        self.stream = match count {
            18 => StreamMode::Format18x18,
            10 => StreamMode::Format10x10,
            _ => StreamMode::Format8x8,
        };

        let labels: Vec<&str> = Self::STREAM_MODES.iter()
            .map(|m| stream_mode_to_str(m))
            .collect();
        let elem_id = alsactl::ElemId::new_by_name(
            alsactl::ElemIfaceType::Card,
            0,
            0,
            Self::STREAM_MODE_NAME,
            0,
        );
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    pub fn read(&mut self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::STREAM_MODE_NAME => {
                let pos = Self::STREAM_MODES.iter()
                    .position(|m| m.eq(&self.stream))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, unit: &hinawa::SndUnit, avc: &BebobAvc, elem_id: &alsactl::ElemId,
                 _: &alsactl::ElemValue, new: &alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::STREAM_MODE_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                let &mode = Self::STREAM_MODES.iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of mode of stream: {}", vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })?;
                unit.lock()?;
                let cmd = EnsembleCmd::Hw(HwCmd::StreamMode(mode));
                let mut op = EnsembleOperation::new(cmd);
                let res = avc.control(&AvcAddr::Unit, &mut op, timeout_ms);
                let _ = unit.unlock();
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
