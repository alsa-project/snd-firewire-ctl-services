// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    protocols::{hw_info::*, port_conf::*},
};

fn phys_group_type_to_str(phys_group_type: &PhysGroupType) -> &'static str {
    match phys_group_type {
        PhysGroupType::Analog => "Analog",
        PhysGroupType::Spdif => "S/PDIF",
        PhysGroupType::Adat => "ADAT",
        PhysGroupType::SpdifOrAdat => "S/PDIForADAT",
        PhysGroupType::AnalogMirror => "AnalogMirror",
        PhysGroupType::Headphones => "HeadPhones",
        PhysGroupType::I2s => "I2S",
        PhysGroupType::Guitar => "Guitar",
        PhysGroupType::PiezoGuitar => "PiezoGuitar",
        PhysGroupType::GuitarString => "GuitarString",
        PhysGroupType::Unknown(_) => "Unknown",
    }
}

fn digital_mode_to_str(mode: &EfwDigitalMode) -> &'static str {
    match mode {
        EfwDigitalMode::SpdifCoax => "S/PDIF-Coaxial",
        EfwDigitalMode::AesebuXlr => "AES/EBU-XLR",
        EfwDigitalMode::SpdifOpt => "S/PDIF-Optical",
        EfwDigitalMode::AdatOpt => "ADAT-Optical",
        EfwDigitalMode::Unknown(_) => "Unknown",
    }
}

#[derive(Default)]
pub struct PortCtl {
    control_room_source: EfwControlRoomSource,
    digital_mode: EfwDigitalMode,
    dig_modes: Vec<EfwDigitalMode>,
    pub notified_elem_id_list: Vec<ElemId>,
    curr_rate: u32,
    phys_in_pairs: usize,
    phys_out_pairs: usize,
    tx_stream_pair_counts: [usize; 3],
    rx_stream_pair_counts: [usize; 3],
    tx_stream_map: Vec<Option<usize>>,
    rx_stream_map: Vec<Option<usize>>,
}

const CONTROL_ROOM_SOURCE_NAME: &str = "control-room-source";
const DIG_MODE_NAME: &str = "digital-mode";
const PHANTOM_NAME: &str = "phantom-powering";
const RX_MAP_NAME: &str = "stream-playback-routing";
const TX_MAP_NAME: &str = "stream-capture-routing";

fn create_stream_map_labels(phys_entries: &[PhysGroupEntry]) -> Vec<String> {
    let mut labels = vec!["Disable".to_string()];
    phys_entries.iter().for_each(|entry| {
        let name = phys_group_type_to_str(&entry.group_type);
        let pair_count = entry.group_count / 2;
        (0..pair_count).for_each(|i| {
            let label = format!("{}-{}/{}", name, i * 2 + 1, i * 2 + 2);
            labels.push(label);
        });
    });
    labels
}

fn enum_values_from_entries(elem_value: &mut ElemValue, entries: &[Option<usize>]) {
    let vals: Vec<u32> = entries
        .iter()
        .map(|entry| {
            if let Some(pos) = entry {
                1 + *pos as u32
            } else {
                0
            }
        })
        .collect();
    elem_value.set_enum(&vals);
}

fn enum_values_to_entries(elem_value: &ElemValue, entries: &mut [Option<usize>]) {
    let vals = &elem_value.enumerated()[..entries.len()];
    entries.iter_mut().zip(vals).for_each(|(entry, &pos)| {
        *entry = if pos == 0 {
            None
        } else {
            Some((pos as usize) - 1)
        }
    });
}

const DIG_MODES: [(HwCap, EfwDigitalMode); 4] = [
    (HwCap::OptionalSpdifCoax, EfwDigitalMode::SpdifCoax),
    (HwCap::OptionalAesebuXlr, EfwDigitalMode::AesebuXlr),
    (HwCap::OptionalSpdifOpt, EfwDigitalMode::SpdifOpt),
    (HwCap::OptionalAdatOpt, EfwDigitalMode::AdatOpt),
];

impl PortCtl {
    pub fn load(
        &mut self,
        hwinfo: &HwInfo,
        card_cntr: &mut CardCntr,
        unit: &mut SndEfw,
        curr_rate: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.cache(hwinfo, unit, curr_rate, timeout_ms)?;

        if hwinfo
            .caps
            .iter()
            .find(|cap| HwCap::ControlRoom.eq(cap))
            .is_some()
        {
            let labels = hwinfo
                .phys_outputs
                .iter()
                .filter(|entry| entry.group_type != PhysGroupType::AnalogMirror)
                .map(|entry| {
                    (0..(entry.group_count / 2)).map(move |i| {
                        format!(
                            "{}-{}/{}",
                            phys_group_type_to_str(&entry.group_type),
                            i * 2 + 1,
                            i * 2 + 2
                        )
                    })
                })
                .flatten()
                .collect::<Vec<String>>();

            let elem_id =
                ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, CONTROL_ROOM_SOURCE_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        }

        DIG_MODES.iter().for_each(|(cap, mode)| {
            if hwinfo.caps.iter().find(|&c| *c == *cap).is_some() {
                self.dig_modes.push(*mode);
            }
        });
        if self.dig_modes.len() > 1 {
            let labels: Vec<&str> = self
                .dig_modes
                .iter()
                .map(|mode| digital_mode_to_str(mode))
                .collect();

            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, DIG_MODE_NAME, 0);
            let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        }

        if hwinfo
            .caps
            .iter()
            .position(|cap| *cap == HwCap::PhantomPowering)
            .is_some()
        {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PHANTOM_NAME, 0);
            let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;
        }

        let has_rx_mapping = hwinfo
            .caps
            .iter()
            .find(|cap| HwCap::OutputMapping.eq(cap))
            .is_some();
        let has_tx_mapping = hwinfo
            .caps
            .iter()
            .find(|cap| HwCap::InputMapping.eq(cap))
            .is_some();

        if has_rx_mapping || has_tx_mapping {
            let phys_input_pair_labels = create_stream_map_labels(&hwinfo.phys_inputs);
            let phys_output_pair_labels = create_stream_map_labels(&hwinfo.phys_outputs);

            self.phys_in_pairs = phys_input_pair_labels.len();
            self.phys_out_pairs = phys_output_pair_labels.len();

            hwinfo
                .tx_channels
                .iter()
                .enumerate()
                .for_each(|(i, count)| self.tx_stream_pair_counts[i] = count / 2);
            hwinfo
                .rx_channels
                .iter()
                .enumerate()
                .for_each(|(i, count)| self.rx_stream_pair_counts[i] = count / 2);

            self.tx_stream_map = vec![Default::default(); self.tx_stream_pair_counts[0]];
            self.rx_stream_map = vec![Default::default(); self.rx_stream_pair_counts[0]];

            if has_tx_mapping {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, TX_MAP_NAME, 0);
                card_cntr
                    .add_enum_elems(
                        &elem_id,
                        1,
                        self.tx_stream_map.len(),
                        &phys_input_pair_labels,
                        None,
                        true,
                    )
                    .map(|mut elem_id_list| {
                        self.notified_elem_id_list.append(&mut elem_id_list);
                    })?;
            }

            if has_rx_mapping {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, RX_MAP_NAME, 0);
                card_cntr
                    .add_enum_elems(
                        &elem_id,
                        1,
                        self.rx_stream_map.len(),
                        &phys_output_pair_labels,
                        None,
                        true,
                    )
                    .map(|mut elem_id_list| {
                        self.notified_elem_id_list.append(&mut elem_id_list);
                    })?;
            }
        }

        Ok(())
    }

    pub fn cache(
        &mut self,
        hw_info: &HwInfo,
        unit: &mut SndEfw,
        curr_rate: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if hw_info
            .caps
            .iter()
            .find(|cap| HwCap::ControlRoom.eq(cap))
            .is_some()
        {
            unit.get_control_room_source(timeout_ms).map(|src| {
                self.control_room_source.0 = src;
            })?;
        }

        if DIG_MODES
            .iter()
            .find(|(cap, _)| hw_info.caps.iter().find(|c| cap.eq(c)).is_some())
            .is_some()
        {
            unit.get_digital_mode(timeout_ms).map(|mode| {
                self.digital_mode = mode;
            })?;
        }

        if hw_info
            .caps
            .iter()
            .find(|cap| HwCap::OutputMapping.eq(cap) || HwCap::InputMapping.eq(cap))
            .is_some()
        {
            unit.get_stream_map(
                curr_rate,
                self.phys_out_pairs,
                self.phys_in_pairs,
                &mut self.rx_stream_map,
                &mut self.tx_stream_map,
                timeout_ms,
            )
            .map(|_| self.curr_rate = curr_rate)?;
        }

        Ok(())
    }

    pub fn read(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CONTROL_ROOM_SOURCE_NAME => {
                elem_value.set_enum(&[self.control_room_source.0 as u32]);
                Ok(true)
            }
            DIG_MODE_NAME => {
                let pos = self
                    .dig_modes
                    .iter()
                    .position(|m| self.digital_mode.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
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
        match elem_id.name().as_str() {
            RX_MAP_NAME => {
                enum_values_from_entries(elem_value, &self.rx_stream_map);
                Ok(true)
            }
            TX_MAP_NAME => {
                enum_values_from_entries(elem_value, &self.tx_stream_map);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CONTROL_ROOM_SOURCE_NAME => {
                let pos = new.enumerated()[0] as usize;
                unit.set_control_room_source(pos, timeout_ms)
                    .map(|_| self.control_room_source.0 = pos)?;
                Ok(true)
            }
            DIG_MODE_NAME => {
                let pos = new.enumerated()[0] as usize;
                let mode = self.dig_modes.iter().nth(pos).copied().ok_or_else(|| {
                    let label = "Invalid value for digital mode";
                    Error::new(FileError::Inval, &label)
                })?;
                unit.set_digital_mode(mode, timeout_ms)
                    .map(|_| self.digital_mode = mode)?;
                Ok(true)
            }
            PHANTOM_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    unit.set_phantom_powering(val, timeout_ms)
                })?;
                Ok(true)
            }
            RX_MAP_NAME => {
                let mut rx_stream_map = vec![Default::default(); self.rx_stream_map.len()];
                enum_values_to_entries(new, &mut rx_stream_map);
                unit.set_stream_map(
                    self.curr_rate,
                    self.phys_out_pairs,
                    self.phys_in_pairs,
                    &rx_stream_map,
                    &self.tx_stream_map,
                    timeout_ms,
                )
                .map(|_| {
                    self.rx_stream_map.copy_from_slice(&rx_stream_map);
                    true
                })
            }
            TX_MAP_NAME => {
                let mut tx_stream_map = vec![Default::default(); self.tx_stream_map.len()];
                enum_values_to_entries(new, &mut tx_stream_map);
                unit.set_stream_map(
                    self.curr_rate,
                    self.phys_out_pairs,
                    self.phys_in_pairs,
                    &self.rx_stream_map,
                    &tx_stream_map,
                    timeout_ms,
                )
                .map(|_| {
                    self.tx_stream_map.copy_from_slice(&tx_stream_map);
                    true
                })
            }
            _ => Ok(false),
        }
    }
}
