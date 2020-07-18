// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use crate::card_cntr;

use alsactl::{ElemValueExt, ElemValueExtManual};

use super::transactions::{HwCap, DigitalMode, EfwPortConf, PhysGroupType, HwInfo};

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
        let list: Vec<String> = (0..stream_pairs)
            .map(|pair| format!("Stream-{}/{}", pair * 2 + 1, pair * 2 + 2))
            .collect();
        let labels: Vec<&str> = list.iter().map(|entry| entry.as_str()).collect();

        let elem_id = alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, name, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, phys_pairs, &labels, None, true)?;

        Ok(())
    }

    pub fn load(&mut self, hwinfo: &HwInfo, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        if hwinfo.caps.iter().find(|&cap| *cap == HwCap::MirrorOutput).is_some() {
            let mut list = Vec::new();
            hwinfo.phys_outputs.iter().for_each(|entry| {
                if entry.group_type != PhysGroupType::AnalogMirror {
                    (0..(entry.group_count / 2)).for_each(|i| {
                        let name = String::from(&entry.group_type);
                        list.push(format!("{}-{}/{}", name, i * 2 + 1, i * 2 + 2))
                    })
                }
            });
            let labels: Vec<&str> = list.iter().map(|entry| entry.as_str()).collect();

            let elem_id = alsactl::ElemId::new_by_name(
                alsactl::ElemIfaceType::Mixer, 0, 0, Self::MIRROR_OUTPUT_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        }

        Self::DIG_MODES.iter().for_each(|(cap, mode)| {
            if hwinfo.caps.iter().find(|&c| *c == *cap).is_some() {
                self.dig_modes.push(*mode);
            }
        });
        if self.dig_modes.len() > 0 {
            let modes: Vec<String> = self.dig_modes.iter()
                .map(|mode| String::from(mode))
                .collect();
            let labels: Vec<&str> = modes.iter()
                .map(|label| label.as_str())
                .collect();

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
                let pair = EfwPortConf::get_output_mirror(unit)?;
                elem_value.set_enum(&[pair as u32]);
                Ok(true)
            }
            Self::DIG_MODE_NAME => {
                let mode = EfwPortConf::get_digital_mode(unit)?;
                if let Some(pos) = self.dig_modes.iter().position(|&m| m == mode) {
                    elem_value.set_enum(&[pos as u32]);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::PHANTOM_NAME => {
                let state = EfwPortConf::get_phantom_powering(unit)?;
                elem_value.set_bool(&[state]);
                Ok(true)
            }
            Self::RX_MAP_NAME => {
                let (rx_entries, _) = EfwPortConf::get_stream_map(unit)?;
                let vals: Vec<u32> = rx_entries.iter().map(|entry| *entry as u32).collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            Self::TX_MAP_NAME => {
                let (_, tx_entries) = EfwPortConf::get_stream_map(unit)?;
                let vals: Vec<u32> = tx_entries.iter().map(|entry| *entry as u32).collect();
                elem_value.set_enum(&vals);
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
                let mut vals = [0];
                new.get_enum(&mut vals);
                EfwPortConf::set_output_mirror(unit, vals[0] as usize)?;
                Ok(true)
            }
            Self::DIG_MODE_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                if self.dig_modes.len() > vals[0] as usize {
                    EfwPortConf::set_digital_mode(unit, self.dig_modes[vals[0] as usize])?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::PHANTOM_NAME => {
                let mut vals = [false];
                new.get_bool(&mut vals);
                EfwPortConf::set_phantom_powering(unit, vals[0])?;
                Ok(true)
            }
            Self::RX_MAP_NAME => {
                let mut vals = vec![0; self.rx_pairs * 2];
                new.get_enum(&mut vals[..self.rx_pairs]);
                old.get_enum(&mut vals[self.rx_pairs..]);
                if vals[..self.rx_pairs] != vals[self.rx_pairs..] {
                    let entries: Vec<usize> = vals[..self.rx_pairs]
                        .iter()
                        .map(|entry| *entry as usize)
                        .collect();
                    EfwPortConf::set_stream_map(unit, Some(entries), None)?;
                }
                Ok(true)
            }
            Self::TX_MAP_NAME => {
                let mut vals = vec![0; self.tx_pairs * 2];
                new.get_enum(&mut vals[..self.tx_pairs]);
                old.get_enum(&mut vals[self.tx_pairs..]);
                if vals[..self.tx_pairs] != vals[self.tx_pairs..] {
                    let entries: Vec<usize> = vals[..self.tx_pairs]
                        .iter()
                        .map(|entry| *entry as usize)
                        .collect();
                    EfwPortConf::set_stream_map(unit, None, Some(entries))?;
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

impl From<&PhysGroupType> for String {
    fn from(group_type: &PhysGroupType) -> String {
        match group_type {
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
}

impl From<&DigitalMode> for String {
    fn from(mode: &DigitalMode) -> String {
        match mode {
            DigitalMode::SpdifCoax  => "S/PDIF-Coaxial",
            DigitalMode::AesebuXlr  => "AES/EBU-XLR",
            DigitalMode::SpdifOpt   => "S/PDIF-Optical",
            DigitalMode::AdatOpt    => "ADAT-Optical",
            DigitalMode::Unknown(_) => "Unknown",
        }.to_string()
    }
}
