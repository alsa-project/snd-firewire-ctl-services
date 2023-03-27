// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, alsa_ctl_tlv_codec::DbInterval, protocols::phys_output::*};

#[derive(Debug)]
pub(crate) struct OutCtl<T>
where
    T: EfwHardwareSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwOutputParameters>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwOutputParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: EfwOutputParameters,
    _phantom: PhantomData<T>,
}

impl<T> Default for OutCtl<T>
where
    T: EfwHardwareSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwOutputParameters>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwOutputParameters>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_output_parameters(),
            _phantom: Default::default(),
        }
    }
}

impl<T> OutCtl<T>
where
    T: EfwHardwareSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwOutputParameters>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwOutputParameters>,
{
    // The fixed point number of 8.24 format.
    const COEF_MIN: i32 = 0x00000000;
    const COEF_MAX: i32 = 0x02000000;
    const COEF_STEP: i32 = 0x00000001;
    const COEF_TLV: DbInterval = DbInterval {
        min: -14400,
        max: 600,
        linear: false,
        mute_avail: false,
    };

    pub(crate) fn cache(&mut self, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        T::cache_wholly(unit, &mut self.params, timeout_ms)
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_VOL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::COEF_MIN,
                Self::COEF_MAX,
                Self::COEF_STEP,
                self.params.volumes.len(),
                Some(&Into::<Vec<u32>>::into(Self::COEF_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, self.params.mutes.len(), true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUT_VOL_NAME => {
                elem_value.set_int(&self.params.volumes);
                Ok(true)
            }
            OUT_MUTE_NAME => {
                elem_value.set_bool(&self.params.mutes);
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
            OUT_VOL_NAME => {
                let mut params = self.params.clone();
                let vals = &elem_value.int()[..T::phys_output_count()];
                params.volumes.copy_from_slice(&vals);
                T::update_partially(unit, &mut self.params, params, timeout_ms)?;
                Ok(true)
            }
            OUT_MUTE_NAME => {
                let mut params = self.params.clone();
                let vals = &elem_value.boolean()[..T::phys_output_count()];
                params.mutes.copy_from_slice(&vals);
                T::update_partially(unit, &mut self.params, params, timeout_ms)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Debug)]
pub(crate) struct PhysOutputCtl<T>
where
    T: EfwPhysOutputSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwPhysOutputParameters>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwPhysOutputParameters>,
{
    pub elem_id_list: Vec<ElemId>,
    params: EfwPhysOutputParameters,
    _phantom: PhantomData<T>,
}

impl<T> Default for PhysOutputCtl<T>
where
    T: EfwPhysOutputSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwPhysOutputParameters>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwPhysOutputParameters>,
{
    fn default() -> Self {
        Self {
            elem_id_list: Default::default(),
            params: T::create_phys_output_parameters(),
            _phantom: Default::default(),
        }
    }
}

impl<T> PhysOutputCtl<T>
where
    T: EfwPhysOutputSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwPhysOutputParameters>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwPhysOutputParameters>,
{
    const OUT_NOMINAL_LABELS: [&'static str; 2] = ["+4dBu", "-10dBV"];
    const OUT_NOMINAL_LEVELS: [NominalSignalLevel; 2] = [
        NominalSignalLevel::Professional,
        NominalSignalLevel::Consumer,
    ];

    pub(crate) fn cache(&mut self, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        T::cache_wholly(unit, &mut self.params, timeout_ms)
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_NOMINAL_NAME, 0);
        card_cntr
            .add_enum_elems(
                &elem_id,
                1,
                self.params.nominals.len(),
                &Self::OUT_NOMINAL_LABELS,
                None,
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUT_NOMINAL_NAME => {
                let vals: Vec<u32> = self
                    .params
                    .nominals
                    .iter()
                    .map(|level| {
                        Self::OUT_NOMINAL_LEVELS
                            .iter()
                            .position(|l| l.eq(level))
                            .map(|pos| pos as u32)
                            .unwrap()
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
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUT_NOMINAL_NAME => {
                let mut params = self.params.clone();
                params
                    .nominals
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(level, &val)| {
                        let pos = val as usize;
                        Self::OUT_NOMINAL_LEVELS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg =
                                    format!("Invalid value for nominal level of output: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&l| *level = l)
                    })?;
                T::update_partially(unit, &mut self.params, params, timeout_ms)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default)]
pub struct OutputCtl {
    all_params: EfwOutputParameters,
    phys_params: EfwPhysOutputParameters,
}

const OUT_VOL_NAME: &str = "output-volume";
const OUT_MUTE_NAME: &str = "output-mute";
const OUT_NOMINAL_NAME: &str = "output-nominal";

impl OutputCtl {
    // The fixed point number of 8.24 format.
    const COEF_MIN: i32 = 0x00000000;
    const COEF_MAX: i32 = 0x02000000;
    const COEF_STEP: i32 = 0x00000001;
    const COEF_TLV: DbInterval = DbInterval {
        min: -14400,
        max: 600,
        linear: false,
        mute_avail: false,
    };

    const OUT_NOMINAL_LABELS: [&'static str; 2] = ["+4dBu", "-10dBV"];
    const OUT_NOMINAL_LEVELS: [NominalSignalLevel; 2] = [
        NominalSignalLevel::Professional,
        NominalSignalLevel::Consumer,
    ];

    fn cache(&mut self, hw_info: &HwInfo, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        let count = hw_info.phys_outputs.iter().fold(0, |accm, entry| {
            if entry.group_type != PhysGroupType::AnalogMirror {
                accm + entry.group_count
            } else {
                accm
            }
        });

        self.all_params = Default::default();
        (0..count).try_for_each(|ch| {
            unit.get_vol(ch, timeout_ms)
                .map(|vol| self.all_params.volumes.push(vol))?;
            unit.get_mute(ch, timeout_ms)
                .map(|enabled| self.all_params.mutes.push(enabled))?;
            Ok::<(), Error>(())
        })?;

        if hw_info
            .caps
            .iter()
            .find(|&cap| *cap == HwCap::NominalOutput)
            .is_some()
        {
            (0..count).try_for_each(|ch| {
                unit.get_nominal(ch, timeout_ms)
                    .map(|nominal| self.phys_params.nominals.push(nominal))
            })?;
        }

        Ok(())
    }

    pub fn load(
        &mut self,
        hwinfo: &HwInfo,
        unit: &mut SndEfw,
        card_cntr: &mut CardCntr,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.cache(hwinfo, unit, timeout_ms)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::COEF_MIN,
            Self::COEF_MAX,
            Self::COEF_STEP,
            self.all_params.volumes.len(),
            Some(&Into::<Vec<u32>>::into(Self::COEF_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_MUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, self.all_params.mutes.len(), true)?;

        if hwinfo
            .caps
            .iter()
            .find(|&cap| *cap == HwCap::NominalOutput)
            .is_some()
        {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_NOMINAL_NAME, 0);
            let _ = card_cntr.add_enum_elems(
                &elem_id,
                1,
                self.phys_params.nominals.len(),
                &Self::OUT_NOMINAL_LABELS,
                None,
                true,
            )?;
        }

        Ok(())
    }

    pub fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUT_VOL_NAME => {
                elem_value.set_int(&self.all_params.volumes);
                Ok(true)
            }
            OUT_MUTE_NAME => {
                elem_value.set_bool(&self.all_params.mutes);
                Ok(true)
            }
            OUT_NOMINAL_NAME => {
                let vals: Vec<u32> = self
                    .phys_params
                    .nominals
                    .iter()
                    .map(|level| {
                        Self::OUT_NOMINAL_LEVELS
                            .iter()
                            .position(|l| l.eq(level))
                            .map(|pos| pos as u32)
                            .unwrap()
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUT_VOL_NAME => {
                self.all_params
                    .volumes
                    .iter_mut()
                    .zip(elem_value.int())
                    .enumerate()
                    .filter(|(_, (o, n))| !o.eq(n))
                    .try_for_each(|(ch, (o, &val))| {
                        unit.set_vol(ch, val, timeout_ms).map(|_| *o = val)
                    })?;
                Ok(true)
            }
            OUT_MUTE_NAME => {
                self.all_params
                    .mutes
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .enumerate()
                    .filter(|(_, (o, n))| !o.eq(&n))
                    .try_for_each(|(ch, (o, val))| {
                        unit.set_mute(ch, val, timeout_ms).map(|_| *o = val)
                    })?;
                Ok(true)
            }
            OUT_NOMINAL_NAME => {
                self.phys_params
                    .nominals
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .enumerate()
                    .try_for_each(|(ch, (o, &val))| {
                        let pos = val as usize;
                        let level = Self::OUT_NOMINAL_LEVELS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg =
                                    format!("Invalid value for nominal level of output: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .copied()?;
                        if level.eq(o) {
                            unit.set_nominal(ch, level, timeout_ms).map(|_| *o = level)
                        } else {
                            Ok(())
                        }
                    })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
