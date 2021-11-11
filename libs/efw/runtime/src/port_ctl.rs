// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    glib::{Error, FileError},
    hinawa::SndEfw,
    alsactl::{ElemId, ElemIfaceType, ElemValue},
    core::{card_cntr::*, elem_value_accessor::*},
    efw_protocols::{hw_info::*, port_conf::*},
};

fn phys_group_type_to_str(phys_group_type: &PhysGroupType) -> &'static str {
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
    }
}

fn digital_mode_to_str(mode: &DigitalMode) -> &'static str {
    match mode {
        DigitalMode::SpdifCoax  => "S/PDIF-Coaxial",
        DigitalMode::AesebuXlr  => "AES/EBU-XLR",
        DigitalMode::SpdifOpt   => "S/PDIF-Optical",
        DigitalMode::AdatOpt    => "ADAT-Optical",
        DigitalMode::Unknown(_) => "Unknown",
    }
}

#[derive(Default)]
pub struct PortCtl {
    dig_modes: Vec<DigitalMode>,
    pub notified_elem_id_list: Vec<ElemId>,
    phys_in_pairs: usize,
    phys_out_pairs: usize,
    tx_stream_pair_counts: [usize; 3],
    rx_stream_pair_counts: [usize; 3],
    tx_stream_map: Vec<usize>,
    rx_stream_map: Vec<usize>,
}

const MIRROR_OUTPUT_NAME: &str = "mirror-output";
const DIG_MODE_NAME: &str = "digital-mode";
const PHANTOM_NAME: &str = "phantom-powering";
const RX_MAP_NAME: &str = "stream-playback-routing";
const TX_MAP_NAME: &str = "stream-capture-routing";

impl PortCtl {
    const DIG_MODES: [(HwCap, DigitalMode);4] = [
        (HwCap::SpdifCoax, DigitalMode::SpdifCoax),
        (HwCap::AesebuXlr, DigitalMode::AesebuXlr),
        (HwCap::SpdifOpt, DigitalMode::SpdifOpt),
        (HwCap::AdatOpt, DigitalMode::AdatOpt),
    ];

    fn add_mapping_ctl(
        &mut self,
        card_cntr: &mut CardCntr,
        name: &str,
        phys_pairs: usize,
        stream_pairs: usize,
    ) -> Result<(), Error> {
        let labels: Vec<String> = (0..stream_pairs)
            .map(|pair| format!("Stream-{}/{}", pair * 2 + 1, pair * 2 + 2))
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr.add_enum_elems(&elem_id, 1, phys_pairs, &labels, None, true)
            .map(|mut elem_id_list| self.notified_elem_id_list.append(&mut elem_id_list))
    }

    pub fn load(
        &mut self,
        hwinfo: &HwInfo,
        card_cntr: &mut CardCntr,
        unit: &mut SndEfw,
        timeout_ms: u32
    ) -> Result<(), Error> {
        if hwinfo.caps.iter().find(|&cap| *cap == HwCap::MirrorOutput).is_some() {
            let labels = hwinfo.phys_outputs.iter()
                .filter(|entry| entry.group_type != PhysGroupType::AnalogMirror)
                .map(|entry| {
                    (0..(entry.group_count / 2))
                        .map(move |i| {
                            format!("{}-{}/{}",
                                    phys_group_type_to_str(&entry.group_type), i * 2 + 1, i * 2 + 2)
                        })
                })
                .flatten()
                .collect::<Vec<String>>();

            let elem_id = ElemId::new_by_name(
                ElemIfaceType::Mixer, 0, 0, MIRROR_OUTPUT_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        }

        Self::DIG_MODES.iter().for_each(|(cap, mode)| {
            if hwinfo.caps.iter().find(|&c| *c == *cap).is_some() {
                self.dig_modes.push(*mode);
            }
        });
        if self.dig_modes.len() > 1 {
            let labels: Vec<&str> = self.dig_modes.iter()
                .map(|mode| digital_mode_to_str(mode))
                .collect();

            let elem_id = ElemId::new_by_name(
                ElemIfaceType::Mixer, 0, 0, DIG_MODE_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        }

        if hwinfo.caps.iter().position(|cap| *cap == HwCap::PhantomPowering).is_some() {
            let elem_id = ElemId::new_by_name(
                ElemIfaceType::Mixer, 0, 0, PHANTOM_NAME, 0);
            let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;
        }

        let has_rx_mapping = hwinfo.caps.iter().find(|cap| HwCap::OutputMapping.eq(cap)).is_some();
        let has_tx_mapping = hwinfo.caps.iter().find(|cap| HwCap::InputMapping.eq(cap)).is_some();

        if has_rx_mapping || has_tx_mapping {
            self.phys_in_pairs = hwinfo.phys_inputs
                .iter()
                .fold(0, |accm, entry| accm + entry.group_count) / 2;
            self.phys_out_pairs = hwinfo.phys_outputs
                .iter()
                .fold(0, |accm, entry| accm + entry.group_count) / 2;

            hwinfo.tx_channels
                .iter()
                .enumerate()
                .for_each(|(i, count)| self.tx_stream_pair_counts[i] = count / 2);
            hwinfo.rx_channels
                .iter()
                .enumerate()
                .for_each(|(i, count)| self.rx_stream_pair_counts[i] = count / 2);

            self.tx_stream_map = vec![0; self.tx_stream_pair_counts[0]];
            self.rx_stream_map = vec![0; self.rx_stream_pair_counts[0]];

            self.cache(unit, timeout_ms)?;

            if has_tx_mapping {
                self.add_mapping_ctl(
                    card_cntr,
                    TX_MAP_NAME,
                    self.phys_in_pairs,
                    self.tx_stream_map.len(),
                )?;
            }

            if has_rx_mapping {
                self.add_mapping_ctl(
                    card_cntr,
                    RX_MAP_NAME,
                    self.phys_out_pairs,
                    self.rx_stream_map.len(),
                )?;
            }
        }

        Ok(())
    }

    pub fn cache(
        &mut self,
        unit: &mut SndEfw,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let (rx_entries, tx_entries) = unit.get_stream_map(timeout_ms)?;
        self.tx_stream_map.iter_mut()
            .enumerate()
            .for_each(|(i, map)| {
                *map = if i < tx_entries.len() {
                    tx_entries[i]
                } else {
                    0
                }
            });
        self.rx_stream_map.iter_mut()
            .enumerate()
            .for_each(|(i, map)| {
                *map = if i < rx_entries.len() {
                    rx_entries[i]
                } else {
                    0
                }
            });
        Ok(())
    }

    pub fn read(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIRROR_OUTPUT_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pair = unit.get_output_mirror(timeout_ms)?;
                    Ok(pair as u32)
                })?;
                Ok(true)
            }
            DIG_MODE_NAME => {
                ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let mode = unit.get_digital_mode(timeout_ms)?;
                    if let Some(pos) = self.dig_modes.iter().position(|&m| m == mode) {
                        Ok(pos as u32)
                    } else {
                        unreachable!();
                    }
                })?;
                Ok(true)
            }
            PHANTOM_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    unit.get_phantom_powering(timeout_ms)
                })?;
                Ok(true)
            }
            _ => self.read_notified_elem(elem_id, elem_value),
        }
    }

    pub fn read_notified_elem(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            RX_MAP_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, self.rx_stream_map.len(), |idx| {
                    Ok(self.rx_stream_map[idx] as u32)
                })?;
                Ok(true)
            }
            TX_MAP_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, self.tx_stream_map.len(), |idx| {
                    Ok(self.tx_stream_map[idx] as u32)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIRROR_OUTPUT_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    unit.set_output_mirror(val as usize, timeout_ms)
                })?;
                Ok(true)
            }
            DIG_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    if self.dig_modes.len() > val as usize {
                        unit.set_digital_mode(self.dig_modes[val as usize], timeout_ms)
                    } else {
                        let label = "Invalid value for digital mode";
                        Err(Error::new(FileError::Inval, &label))
                    }
                })?;
                Ok(true)
            }
            PHANTOM_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    unit.set_phantom_powering(val, timeout_ms)
                })?;
                Ok(true)
            }
            RX_MAP_NAME => {
                let (mut rx_entries, _) = unit.get_stream_map(timeout_ms)?;
                ElemValueAccessor::<u32>::get_vals(new, old, self.rx_stream_map.len(), |idx, val| {
                    rx_entries[idx] = val as usize;
                    Ok(())
                })?;
                unit.set_stream_map(Some(rx_entries), None, timeout_ms)?;
                Ok(true)
            }
            TX_MAP_NAME => {
                let (_, mut tx_entries) = unit.get_stream_map(timeout_ms)?;
                ElemValueAccessor::<u32>::get_vals(new, old, self.tx_stream_map.len(), |idx, val| {
                    tx_entries[idx] = val as usize;
                    Ok(())
                })?;
                unit.set_stream_map(None, Some(tx_entries), timeout_ms)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
