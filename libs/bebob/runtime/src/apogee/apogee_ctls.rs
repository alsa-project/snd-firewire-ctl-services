// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::SndUnitExt;

use alsactl::{ElemValueExt, ElemValueExtManual};

use alsa_ctl_tlv_codec::items::DbInterval;

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

pub struct MixerCtl {
    mixers: [[i32; 36]; 4],
}

impl<'a> MixerCtl {
    const MIXER_LABELS: &'a [&'a str] = &["mixer-1", "mixer-2", "mixer-3", "mixer-4"];

    const MIXER_SRC_LABELS: &'a [&'a str] = &[
        // = EnsembleCmd::MixerSrc0.
        "analog-1", "analog-2", "analog-3", "analog-4",
        "analog-5", "analog-6", "analog-7", "analog-8",
        "stream-1",
        // = EnsembleCmd::MixerSrc1.
        "stream-2", "stream-3", "stream-4",
        "stream-5", "stream-6", "stream-7", "stream-8",
        "stream-9", "stream-10",
        // = EnsembleCmd::MixerSrc2.
        "stream-11", "stream-12",
        "stream-13", "stream-14", "stream-15", "stream-16",
        "stream-17", "stream-18",
        "adat-1",
        // = EnsembleCmd::MixerSrc3.
        "adat-2", "adat-3", "adat-4",
        "adat-5", "adat-6", "adat-7", "adat-8",
        "spdif-1", "spdif-2",
    ];

    const MIXER_SRC_GAIN_NAME: &'a str = "mixer-source-gain";

    const GAIN_MIN: i32 = 0;
    const GAIN_MAX: i32 = 0x7fff;
    const GAIN_STEP: i32 = 0xff;
    const GAIN_TLV: DbInterval = DbInterval{min: -4800, max: 0, linear: false, mute_avail: true};

    pub fn new() -> Self {
        let mut mixers = [[0; 36]; 4];

        mixers.iter_mut()
            .enumerate()
            .for_each(|(i, mixer)| {
                mixer.iter_mut()
                    .enumerate()
                    .filter(|(j, _)| i % 2 == j % 2)
                    .for_each(|(_, v)| {
                        *v = Self::GAIN_MAX;
                    });
            });

        MixerCtl{mixers}
    }

    fn write_pair(&mut self, avc: &BebobAvc, index: usize, vals: &[i32], pos: usize, timeout_ms: u32)
        -> Result<(), Error>
    {
        let mut args = Vec::new();
        args.push((index / 2) as u8);

        let mut idx = 0;
        let params = (pos..(pos + 9)).fold([0; 18], |mut params, i| {
            let (l, r) = match index % 2 {
                0 => (vals[i] as i16, self.mixers[index + 1][i] as i16),
                _ => (self.mixers[index - 1][i] as i16, vals[i] as i16),
            };
            params[idx] = l;
            params[idx + 1] = r;
            idx += 2;
            params
        });


        let p = index / 2;
        let cmd = match pos / 9 {
            3 => EnsembleCmd::MixerSrc3(p, params),
            2 => EnsembleCmd::MixerSrc2(p, params),
            1 => EnsembleCmd::MixerSrc1(p, params),
            _ => EnsembleCmd::MixerSrc0(p, params),
        };

        let mut op = EnsembleOperation::new(cmd);
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        self.mixers[index].copy_from_slice(&vals[0..Self::MIXER_SRC_LABELS.len()]);

        Ok(())
    }

    pub fn load(&mut self, avc: &BebobAvc, card_cntr: &mut card_cntr::CardCntr, timeout_ms: u32)
        -> Result<(), Error>
    {
        // Transfer initialized data.
        let mixers = self.mixers;
        (0..4).try_for_each(|i| {
            mixers.iter().enumerate().try_for_each(|(j, vals)| {
                self.write_pair(avc, j, vals, i * 9, timeout_ms)
            })
        })?;

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer,
                                                   0, 0, Self::MIXER_SRC_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, Self::MIXER_LABELS.len(),
                                        Self::GAIN_MIN, Self::GAIN_MAX, Self::GAIN_STEP,
                                        Self::MIXER_SRC_LABELS.len(),
                                        Some(&Into::<Vec<u32>>::into(Self::GAIN_TLV)), true)?;

        Ok(())
    }

    pub fn read(&mut self, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MIXER_SRC_GAIN_NAME => {
                let vals = &self.mixers[elem_id.get_index() as usize];
                elem_value.set_int(vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(&mut self, avc: &BebobAvc, elem_id: &alsactl::ElemId,
                 old: &alsactl::ElemValue, new: &alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MIXER_SRC_GAIN_NAME => {
                let len = Self::MIXER_SRC_LABELS.len();
                let mut vals = vec![0;len * 2];
                new.get_int(&mut vals[0..len]);
                old.get_int(&mut vals[len..]);
                let index = elem_id.get_index() as usize;
                for i in 0..4 {
                    let p = i * 9;
                    if vals[p..(p + 9)] != vals[(len + p)..(len + p + 9)] {
                        self.write_pair(avc, index, &vals, p, timeout_ms)?;
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
