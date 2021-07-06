// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use core::card_cntr;
use core::elem_value_accessor::ElemValueAccessor;

use efw_protocols::transactions::{HwCap, DigitalMode, EfwPortConf, PhysGroupType, HwInfo};

fn phys_group_type_to_string(phys_group_type: &PhysGroupType) -> String {
    match phys_group_type {
        PhysGroupType::Analog       => "Analog",
        PhysGroupType::Spdif        => "S/PDIF",
        PhysGroupType::Adat         => "ADAT",
        PhysGroupType::SpdifOrAdat  => "S/PDIForADAT",
        PhysGroupType::AnalogMirror => "AnalogMirror",
        PhysGroupType::Headphones   => "HeadPhones",
        PhysGroupType::I2s          => "I2S",
        PhysGroupType::Guitar       => "Guitar",
        PhysGroupType::PiezoGuitar  => "PiezoGuitar",
        PhysGroupType::GuitarString => "GuitarString",
        PhysGroupType::Unknown(_)   => "Unknown",
    }.to_string()
}

fn digital_mode_to_string(mode: &DigitalMode) -> String {
    match mode {
        DigitalMode::SpdifCoax  => "S/PDIF-Coaxial",
        DigitalMode::AesebuXlr  => "AES/EBU-XLR",
        DigitalMode::SpdifOpt   => "S/PDIF-Optical",
        DigitalMode::AdatOpt    => "ADAT-Optical",
        DigitalMode::Unknown(_) => "Unknown",
    }.to_string()
}

pub struct PortCtl {
    dig_modes: Vec<DigitalMode>,
    phys_in_pairs: usize,
    phys_out_pairs: usize,
    rx_pairs: usize,
    tx_pairs: usize,
}

impl<'a> PortCtl {
    const MIRROR_OUTPUT_NAME: &'a str = "mirror-output";
    const DIG_MODE_NAME: &'a str = "digital-mode";
    const PHANTOM_NAME: &'a str = "phantom-powering";
    const RX_MAP_NAME: &'a str = "stream-playback-routing";
    const TX_MAP_NAME: &'a str = "stream-capture-routing";

    const DIG_MODES: &'a [(HwCap, DigitalMode)] = &[
        (HwCap::SpdifCoax, DigitalMode::SpdifCoax),
        (HwCap::AesebuXlr, DigitalMode::AesebuXlr),
        (HwCap::SpdifOpt, DigitalMode::SpdifOpt),
        (HwCap::AdatOpt, DigitalMode::AdatOpt),
    ];

    pub fn new() -> Self {
        PortCtl {
            dig_modes: Vec::new(),
            phys_in_pairs: 0,
            phys_out_pairs: 0,
            tx_pairs: 0,
            rx_pairs: 0,
        }
    }

    fn add_mapping_ctl(
        &self,
        card_cntr: &mut card_cntr::CardCntr,
        name: &'a str,
        phys_pairs: usize,
        stream_pairs: usize,
    ) -> Result<(), Error> {
        let labels = (0..stream_pairs)
            .map(|pair| format!("Stream-{}/{}", pair * 2 + 1, pair * 2 + 2))
            .collect::<Vec<String>>();

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, name, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, phys_pairs, &labels, None, true)?;

        Ok(())
    }

    pub fn load(&mut self, hwinfo: &HwInfo, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        if hwinfo.caps.iter().find(|&cap| *cap == HwCap::MirrorOutput).is_some() {
            let labels = hwinfo.phys_outputs.iter()
                .filter(|entry| entry.group_type != PhysGroupType::AnalogMirror)
                .map(|entry| {
                    (0..(entry.group_count / 2))
                        .map(move |i| {
                            format!("{}-{}/{}",
                                    phys_group_type_to_string(&entry.group_type), i * 2 + 1, i * 2 + 2)
                        })
                })
                .flatten()
                .collect::<Vec<String>>();

            let elem_id = alsactl::ElemId::new_by_name(
                alsactl::ElemIfaceType::Mixer, 0, 0, Self::MIRROR_OUTPUT_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        }

        Self::DIG_MODES.iter().for_each(|(cap, mode)| {
            if hwinfo.caps.iter().find(|&c| *c == *cap).is_some() {
                self.dig_modes.push(*mode);
            }
        });
        if self.dig_modes.len() > 1 {
            let labels = self.dig_modes.iter()
                .map(|mode| digital_mode_to_string(mode))
                .collect::<Vec<String>>();

            let elem_id = alsactl::ElemId::new_by_name(
                alsactl::ElemIfaceType::Mixer, 0, 0, Self::DIG_MODE_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        }

        if hwinfo.caps.iter().position(|cap| *cap == HwCap::PhantomPowering).is_some() {
            let elem_id = alsactl::ElemId::new_by_name(
                alsactl::ElemIfaceType::Mixer, 0, 0, Self::PHANTOM_NAME, 0);
            let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;
        }

        if hwinfo.caps.iter().position(|cap| *cap == HwCap::OutputMapping).is_some() {
            self.phys_out_pairs = hwinfo.phys_outputs.iter()
                .fold(0, |accm, entry| accm + entry.group_count)
                / 2;
            self.rx_pairs = hwinfo.rx_channels[0] / 2;

            self.add_mapping_ctl(card_cntr, Self::RX_MAP_NAME, self.phys_out_pairs, self.rx_pairs)?;
        }

        if hwinfo.caps.iter().position(|cap| *cap == HwCap::InputMapping).is_some() {
            self.phys_in_pairs = hwinfo.phys_inputs.iter()
                .fold(0, |accm, entry| accm + entry.group_count) / 2;
            self.tx_pairs = hwinfo.tx_channels[0] / 2;

            self.add_mapping_ctl(
                card_cntr,
                Self::TX_MAP_NAME,
                self.phys_in_pairs,
                self.tx_pairs,
            )?;
        }

        Ok(())
    }

    pub fn read(
        &mut self,
        unit: &hinawa::SndEfw,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::MIRROR_OUTPUT_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pair = EfwPortConf::get_output_mirror(unit)?;
                    Ok(pair as u32)
                })?;
                Ok(true)
            }
            Self::DIG_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let mode = EfwPortConf::get_digital_mode(unit)?;
                    if let Some(pos) = self.dig_modes.iter().position(|&m| m == mode) {
                        Ok(pos as u32)
                    } else {
                        unreachable!();
                    }
                })?;
                Ok(true)
            }
            Self::PHANTOM_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    let state = EfwPortConf::get_phantom_powering(unit)?;
                    Ok(state)
                })?;
                Ok(true)
            }
            Self::RX_MAP_NAME => {
                let (rx_entries, _) = EfwPortConf::get_stream_map(unit)?;
                ElemValueAccessor::<u32>::set_vals(elem_value, rx_entries.len(), |idx| {
                    Ok(rx_entries[idx] as u32)
                })?;
                Ok(true)
            }
            Self::TX_MAP_NAME => {
                let (_, tx_entries) = EfwPortConf::get_stream_map(unit)?;
                ElemValueAccessor::<u32>::set_vals(elem_value, tx_entries.len(), |idx| {
                    Ok(tx_entries[idx] as u32)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &hinawa::SndEfw,
        elem_id: &alsactl::ElemId,
        old: &alsactl::ElemValue,
        new: &alsactl::ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::MIRROR_OUTPUT_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    EfwPortConf::set_output_mirror(unit, val as usize)
                })?;
                Ok(true)
            }
            Self::DIG_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    if self.dig_modes.len() > val as usize {
                        EfwPortConf::set_digital_mode(unit, self.dig_modes[val as usize])
                    } else {
                        let label = "Invalid value for digital mode";
                        Err(Error::new(FileError::Inval, &label))
                    }
                })?;
                Ok(true)
            }
            Self::PHANTOM_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    EfwPortConf::set_phantom_powering(unit, val)
                })?;
                Ok(true)
            }
            Self::RX_MAP_NAME => {
                let (mut rx_entries, _) = EfwPortConf::get_stream_map(unit)?;
                ElemValueAccessor::<u32>::get_vals(new, old, rx_entries.len(), |idx, val| {
                    rx_entries[idx] = val as usize;
                    Ok(())
                })?;
                EfwPortConf::set_stream_map(unit, Some(rx_entries), None)?;
                Ok(true)
            }
            Self::TX_MAP_NAME => {
                let (_, mut tx_entries) = EfwPortConf::get_stream_map(unit)?;
                ElemValueAccessor::<u32>::get_vals(new, old, tx_entries.len(), |idx, val| {
                    tx_entries[idx] = val as usize;
                    Ok(())
                })?;
                EfwPortConf::set_stream_map(unit, None, Some(tx_entries))?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
