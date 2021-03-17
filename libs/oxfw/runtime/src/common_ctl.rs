// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::SndUnitExt;

use core::card_cntr;
use core::elem_value_accessor::ElemValueAccessor;

use ta1394::*;
use ta1394::general::*;
use ta1394::stream_format::*;
use ta1394::amdtp::*;

pub struct CommonCtl{
    output_fmt_entries: Vec<CompoundAm824Stream>,
    input_fmt_entries: Vec<CompoundAm824Stream>,
    supported_rates: Vec<u32>,
    assumed: bool,
    pub notified_elem_list: Vec<alsactl::ElemId>,
}

impl<'a> CommonCtl {
    const CLK_RATE_NAME: &'a str = "sampling-rate";

    const SUPPORTED_RATES: &'a [u32] = &[32000, 44100, 48000, 88200, 96000, 176400, 192000];

    pub fn new() -> Self {
        CommonCtl{
            output_fmt_entries: Vec::new(),
            input_fmt_entries: Vec::new(),
            supported_rates: Vec::new(),
            assumed: false,
            notified_elem_list: Vec::new(),
        }
    }

    pub fn load<O>(&mut self, avc: &O, card_cntr: &mut card_cntr::CardCntr, timeout_ms: u32)
        -> Result<(), Error>
        where O: Ta1394Avc
    {
        let mut op = PlugInfo::new_for_unit_isoc_ext_plugs();
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let (isoc_input_plugs, isoc_output_plugs) = match op {
            PlugInfo::Unit(u) => match u {
                PlugInfoUnitData::IsocExt(d) => (d.isoc_input_plugs, d.isoc_output_plugs),
                _ => unreachable!(),
            }
            _ => unreachable!(),
        };

        if isoc_output_plugs > 0 {
            self.output_fmt_entries = self.detect_stream_formats(avc, PlugDirection::Output,
                                                                 timeout_ms)?;
        }

        if isoc_input_plugs > 0 {
            self.input_fmt_entries = self.detect_stream_formats(avc, PlugDirection::Input,
                                                                timeout_ms)?;
        }

        let mut rates = Vec::new();
        self.output_fmt_entries.iter().for_each(|entry| {
            if rates.iter().find(|&rate| *rate == entry.freq).is_none() {
                rates.push(entry.freq);
            }
        });
        self.input_fmt_entries.iter().for_each(|entry| {
            if rates.iter().find(|&rate| *rate == entry.freq).is_none() {
                rates.push(entry.freq);
            }
        });
        rates.sort();
        self.supported_rates = rates;

        let labels = self.supported_rates.iter().map(|rate| rate.to_string()).collect::<Vec<String>>();

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Card, 0, 0, Self::CLK_RATE_NAME, 0);
        let mut elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        self.notified_elem_list.append(&mut elem_id_list);

        Ok(())
    }

    fn read_freq<O>(&self, avc: &O, timeout_ms: u32) -> Result<usize, Error>
        where O: Ta1394Avc
    {
        // For playback direction.
        let mut op = InputPlugSignalFormat::new(0);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
        let fdf = AmdtpFdf::from(op.fdf.as_ref());

        if let Some(pos) = self.supported_rates.iter().position(|rate| *rate == fdf.freq) {
            Ok(pos)
        } else {
            let label = format!("Unsupported sampling rate: {}", fdf.freq);
            Err(Error::new(FileError::Io, &label))
        }
    }

    pub fn read<O>(&mut self, avc: &O, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue,
                   timeout_ms: u32)
        -> Result<bool, Error>
        where O: Ta1394Avc
    {
        match elem_id.get_name().as_str() {
            Self::CLK_RATE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let idx = self.read_freq(avc, timeout_ms)?;
                    Ok(idx as u32)
                })?;
                Ok(true)
            },
            _ => Ok(false),
        }
    }

    fn write_freq_for_fallback_mode<O>(&self, avc: &O, freq: u32, direction: PlugDirection,
                                       timeout_ms: u32)
        -> Result<(), Error>
        where O: Ta1394Avc
    {
        let fdf: [u8;3] = AmdtpFdf::new(AmdtpEventType::Am824, false, freq).into();

        if direction == PlugDirection::Input {
            let mut op = InputPlugSignalFormat{
                plug_id: 0,
                fmt: FMT_IS_AMDTP,
                fdf: fdf.clone(),
            };
            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
        } else {
            let mut op = OutputPlugSignalFormat{
                plug_id: 0,
                fmt: FMT_IS_AMDTP,
                fdf: fdf.clone(),
            };
            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
        }
    }

    fn write_freq_for_enhanced_mode<O>(&self, avc: &O, freq: u32, direction: PlugDirection,
                                       timeout_ms: u32)
        -> Result<(), Error>
        where O: Ta1394Avc
    {
        let entries = match direction {
            PlugDirection::Input => &self.input_fmt_entries,
            _ => &self.output_fmt_entries,
        };

        let plug_addr = PlugAddr{
            direction,
            mode: PlugAddrMode::Unit(UnitPlugData{
                unit_type: UnitPlugType::Pcr,
                plug_id: 0,
            }),
        };
        let mut op = ExtendedStreamFormatSingle::new(&plug_addr);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let stream_format = op.stream_format.as_compound_am824_stream()?;
        let pos = entries.iter().position(|entry|{
            entry.freq == freq && entry.entries == stream_format.entries
        }).ok_or_else(||{
            let label = "Stream format entry is not found";
            Error::new(FileError::Nxio, &label)
        })?;

        op.stream_format= StreamFormat::Am(AmStream::CompoundAm824(entries[pos].clone()));
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }

    fn write_freq<O>(&self, avc: &O, idx: usize, timeout_ms: u32) -> Result<(), Error>
        where O: Ta1394Avc
    {
        if idx >= self.supported_rates.len() {
            let label = format!("Invalid value for index of sampling rate: {}", idx);
            return Err(Error::new(FileError::Io, &label));
        }
        let freq = self.supported_rates[idx];

        // For fallback mode.
        if self.assumed {
            if self.output_fmt_entries.len() > 0 {
                self.write_freq_for_fallback_mode(avc, freq, PlugDirection::Output, timeout_ms)?;
            }
            if self.input_fmt_entries.len() > 0 {
                self.write_freq_for_fallback_mode(avc, freq, PlugDirection::Input, timeout_ms)?;
            }
        } else {
            if self.output_fmt_entries.len() > 0 {
                self.write_freq_for_enhanced_mode(avc, freq, PlugDirection::Output, timeout_ms)?;
            }
            if self.input_fmt_entries.len() > 0 {
                self.write_freq_for_enhanced_mode(avc, freq, PlugDirection::Input, timeout_ms)?;
            }
        }

        Ok(())
    }

    pub fn write<O>(&mut self, unit: &hinawa::SndUnit, avc: &O, elem_id: &alsactl::ElemId,
                 elem_value: &alsactl::ElemValue, timeout_ms: u32)
        -> Result<bool, Error>
        where O: Ta1394Avc
    {
        match elem_id.get_name().as_str() {
            Self::CLK_RATE_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    unit.lock()?;
                    let res = self.write_freq(avc, val as usize, timeout_ms);
                    let _ = unit.unlock();
                    res
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn detect_stream_formats<O>(&mut self, avc: &O, direction: PlugDirection, timeout_ms: u32)
        -> Result<Vec<CompoundAm824Stream>, Error>
        where O: Ta1394Avc
    {
        let mut entries = Vec::new();

        let plug_addr = PlugAddr{
            direction,
            mode: PlugAddrMode::Unit(UnitPlugData{
                unit_type: UnitPlugType::Pcr,
                plug_id: 0,
            }),
        };
        let mut op = ExtendedStreamFormatList::new(&plug_addr, 0);

        if avc.status(&AvcAddr::Unit, &mut op, timeout_ms).is_ok() {
            let stream_format = op.stream_format.as_compound_am824_stream()?;
            entries.push(stream_format.clone());

            let _ = (1..10).try_for_each(|i| {
                op.index = i as u8;
                avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;
                let stream_format = op.stream_format.as_compound_am824_stream()?;
                entries.push(stream_format.clone());
                Ok::<(), Error>(())
            });
        } else {
            // Fallback. At first, retrieve current format information.
            let mut op = ExtendedStreamFormatSingle::new(&plug_addr);
            avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;

            let stream_format = op.stream_format.as_compound_am824_stream()?;

            // Next, inquire supported sampling rates and make entries.
            Self::SUPPORTED_RATES.iter().for_each(|&freq| {
                let fdf: [u8;3] = AmdtpFdf::new(AmdtpEventType::Am824, false, freq).into();

                if direction == PlugDirection::Input {
                    let mut op = InputPlugSignalFormat{
                        plug_id: 0,
                        fmt: FMT_IS_AMDTP,
                        fdf,
                    };
                    if avc.specific_inquiry(&AvcAddr::Unit, &mut op, timeout_ms).is_err() {
                        return;
                    }
                } else {
                    let mut op = OutputPlugSignalFormat{
                        plug_id: 0,
                        fmt: FMT_IS_AMDTP,
                        fdf,
                    };
                    if avc.specific_inquiry(&AvcAddr::Unit, &mut op, timeout_ms).is_err() {
                        return;
                    }
                }

                let mut entry = stream_format.clone();
                entry.freq = freq;
                entries.push(entry);
            });

            self.assumed = true;
        }

        Ok(entries)
    }
}
