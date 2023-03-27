// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, protocols::port_conf::*};

const CONTROL_ROOM_SOURCE_NAME: &str = "control-room-source";
const DIG_MODE_NAME: &str = "digital-mode";
const PHANTOM_NAME: &str = "phantom-powering";
const RX_MAP_NAME: &str = "stream-playback-routing";

#[derive(Default, Debug)]
pub(crate) struct ControlRoomSourceCtl<T>(pub Vec<ElemId>, EfwControlRoomSource, PhantomData<T>)
where
    T: EfwControlRoomSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwControlRoomSource>
        + EfwWhollyUpdatableParamsOperation<SndEfw, EfwControlRoomSource>;

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

impl<T> ControlRoomSourceCtl<T>
where
    T: EfwControlRoomSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwControlRoomSource>
        + EfwWhollyUpdatableParamsOperation<SndEfw, EfwControlRoomSource>,
{
    pub(crate) fn cache(&mut self, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        T::cache_wholly(unit, &mut self.1, timeout_ms)
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = T::control_room_source_pairs()
            .iter()
            .map(|(entry_type, pos)| {
                format!(
                    "{}-{}/{}",
                    phys_group_type_to_str(&entry_type),
                    pos + 1,
                    pos + 2
                )
            })
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, CONTROL_ROOM_SOURCE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))
    }

    pub(crate) fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CONTROL_ROOM_SOURCE_NAME => {
                elem_value.set_enum(&[self.1 .0 as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CONTROL_ROOM_SOURCE_NAME => {
                let mut params = self.1.clone();
                params.0 = elem_value.enumerated()[0] as usize;
                T::update_wholly(unit, &params, timeout_ms)?;
                self.1 = params;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Debug)]
pub(crate) struct DigitalModeCtl<T>(
    pub Vec<ElemId>,
    EfwDigitalMode,
    Vec<EfwDigitalMode>,
    PhantomData<T>,
)
where
    T: EfwDigitalModeSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwDigitalMode>
        + EfwWhollyUpdatableParamsOperation<SndEfw, EfwDigitalMode>;

impl<T: EfwDigitalModeSpecification> Default for DigitalModeCtl<T> {
    fn default() -> Self {
        Self(
            Default::default(),
            T::create_digital_mode(),
            T::create_digital_modes(),
            Default::default(),
        )
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

impl<T> DigitalModeCtl<T>
where
    T: EfwDigitalModeSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwDigitalMode>
        + EfwWhollyUpdatableParamsOperation<SndEfw, EfwDigitalMode>,
{
    pub(crate) fn cache(&mut self, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        T::cache_wholly(unit, &mut self.1, timeout_ms)
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.2 = DIG_MODES
            .iter()
            .filter(|(cap, _)| T::CAPABILITIES.iter().find(|c| cap.eq(c)).is_some())
            .map(|(_, mode)| *mode)
            .collect();

        let labels: Vec<&str> = self
            .2
            .iter()
            .map(|mode| digital_mode_to_str(mode))
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, DIG_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))
    }

    pub(crate) fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            DIG_MODE_NAME => {
                let pos = self.2.iter().position(|m| self.1.eq(m)).unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            DIG_MODE_NAME => {
                let pos = elem_value.enumerated()[0] as usize;
                let mode = self.2.iter().nth(pos).copied().ok_or_else(|| {
                    let label = format!("Invalid value {} for digital mode", pos);
                    Error::new(FileError::Inval, &label)
                })?;
                T::update_wholly(unit, &mode, timeout_ms).map(|_| self.1 = mode)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct PhantomPoweringCtl<T>(pub Vec<ElemId>, EfwPhantomPowering, PhantomData<T>)
where
    T: EfwPhantomPoweringSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwPhantomPowering>
        + EfwWhollyUpdatableParamsOperation<SndEfw, EfwPhantomPowering>;

impl<T> PhantomPoweringCtl<T>
where
    T: EfwPhantomPoweringSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwPhantomPowering>
        + EfwWhollyUpdatableParamsOperation<SndEfw, EfwPhantomPowering>,
{
    pub(crate) fn cache(&mut self, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        T::cache_wholly(unit, &mut self.1, timeout_ms)
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, PHANTOM_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))
    }

    pub(crate) fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PHANTOM_NAME => {
                elem_value.set_bool(&[self.1 .0]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            PHANTOM_NAME => {
                let mut params = self.1.clone();
                params.0 = elem_value.boolean()[0];
                T::update_wholly(unit, &params, timeout_ms).map(|_| self.1 = params)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Debug)]
pub(crate) struct RxStreamMapsCtl<T>(pub Vec<ElemId>, EfwRxStreamMaps, PhantomData<T>)
where
    T: EfwRxStreamMapsSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwRxStreamMaps>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwRxStreamMaps>;

impl<T> Default for RxStreamMapsCtl<T>
where
    T: EfwRxStreamMapsSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwRxStreamMaps>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwRxStreamMaps>,
{
    fn default() -> Self {
        RxStreamMapsCtl(
            Default::default(),
            T::create_rx_stream_maps(),
            Default::default(),
        )
    }
}

impl<T> RxStreamMapsCtl<T>
where
    T: EfwRxStreamMapsSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwRxStreamMaps>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwRxStreamMaps>,
{
    pub(crate) fn cache(&mut self, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        T::cache_wholly(unit, &mut self.1, timeout_ms)?;
        Ok(())
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let mut labels = vec!["Disable".to_string()];
        (0..T::RX_CHANNEL_COUNTS[0]).step_by(2).for_each(|pair| {
            let label = format!("Stream-{}/{}", pair + 1, pair + 2);
            labels.push(label);
        });

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, RX_MAP_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, self.1 .0.len(), &labels, None, true)
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))
    }

    fn compute_rate_index(rate: u32) -> usize {
        T::STREAM_MAPPING_RATE_TABLE
            .iter()
            .position(|rates| rates.iter().find(|r| rate.eq(r)).is_some())
            .unwrap()
    }

    pub(crate) fn read(
        &self,
        rate: u32,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            RX_MAP_NAME => {
                let rate_index = Self::compute_rate_index(rate);
                let vals: Vec<u32> = self.1 .0[rate_index]
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
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        unit: &mut SndEfw,
        rate: u32,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            RX_MAP_NAME => {
                let rate_index = Self::compute_rate_index(rate);
                let mut params = self.1.clone();
                let phys_output_pair_count = T::phys_output_count() / 2;
                params.0[rate_index]
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(entry, &pos)| {
                        if pos > phys_output_pair_count as u32 {
                            let msg = format!("Invalid value for output pair: {}", pos);
                            Err(Error::new(FileError::Inval, &msg))
                        } else {
                            *entry = if pos == 0 {
                                None
                            } else {
                                Some((pos as usize) - 1)
                            };
                            Ok(())
                        }
                    })?;
                T::update_partially(unit, &mut self.1, params, timeout_ms)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

const DIG_MODES: [(HwCap, EfwDigitalMode); 4] = [
    (HwCap::OptionalSpdifCoax, EfwDigitalMode::SpdifCoax),
    (HwCap::OptionalAesebuXlr, EfwDigitalMode::AesebuXlr),
    (HwCap::OptionalSpdifOpt, EfwDigitalMode::SpdifOpt),
    (HwCap::OptionalAdatOpt, EfwDigitalMode::AdatOpt),
];
