// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, protocols::*};

pub trait MediaClkFreqCtlOperation<T: MediaClockFrequencyOperation> {
    fn state(&self) -> &MediaClockParameters;
    fn state_mut(&mut self) -> &mut MediaClockParameters;

    fn load_freq(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let labels: Vec<String> = T::FREQ_LIST.iter().map(|&r| r.to_string()).collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_RATE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)
    }

    fn cache_freq(&mut self, avc: &BebobAvc, timeout_ms: u32) -> Result<(), Error> {
        T::cache_freq(avc, self.state_mut(), timeout_ms)
    }

    fn read_freq(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_RATE_NAME => {
                elem_value.set_enum(&[self.state().freq_idx as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_freq(
        &mut self,
        unit: &mut SndUnit,
        avc: &BebobAvc,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_RATE_NAME => {
                unit.lock()?;
                let mut params = self.state().clone();
                params.freq_idx = new.enumerated()[0] as usize;
                let res = T::update_freq(avc, &params, self.state_mut(), timeout_ms).map(|_| true);
                let _ = unit.unlock();
                res
            }
            _ => Ok(false),
        }
    }
}

pub trait SamplingClkSrcCtlOperation<T: SamplingClockSourceOperation> {
    const SRC_LABELS: &'static [&'static str];

    fn state(&self) -> &SamplingClockParameters;
    fn state_mut(&mut self) -> &mut SamplingClockParameters;

    fn load_src(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        assert_eq!(
            Self::SRC_LABELS.len(),
            T::SRC_LIST.len(),
            "Programming error for count of clock source"
        );

        let mut elem_id_list = Vec::new();

        if T::SRC_LIST.len() > 1 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_SRC_NAME, 0);
            card_cntr
                .add_enum_elems(&elem_id, 1, 1, &Self::SRC_LABELS, None, true)
                .map(|mut elem_id| elem_id_list.append(&mut elem_id))?;
        }

        Ok(elem_id_list)
    }

    fn cache_src(&mut self, avc: &BebobAvc, timeout_ms: u32) -> Result<(), Error> {
        T::cache_src(avc, self.state_mut(), timeout_ms)
    }

    fn read_src(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_SRC_NAME => {
                elem_value.set_enum(&[self.state().src_idx as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_src(
        &mut self,
        unit: &mut SndUnit,
        avc: &BebobAvc,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_SRC_NAME => {
                unit.lock()?;
                let mut params = self.state().clone();
                params.src_idx = new.enumerated()[0] as usize;
                let res = T::update_src(avc, &params, self.state_mut(), timeout_ms).map(|_| true);
                let _ = unit.unlock();
                res
            }
            _ => Ok(false),
        }
    }
}

pub trait AvcLevelCtlOperation<T: AvcLevelOperation> {
    const LEVEL_NAME: &'static str;

    const PORT_LABELS: &'static [&'static str];

    const LEVEL_MIN: i32 = T::LEVEL_MIN as i32;
    const LEVEL_MAX: i32 = T::LEVEL_MAX as i32;
    const LEVEL_STEP: i32 = T::LEVEL_STEP as i32;
    const LEVEL_TLV: DbInterval = DbInterval {
        min: -12800,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    fn load_level(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        assert_eq!(
            Self::PORT_LABELS.len(),
            T::ENTRIES.len(),
            "Programming error for count of channels: {}",
            Self::LEVEL_NAME
        );

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::LEVEL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::LEVEL_MIN,
                Self::LEVEL_MAX,
                Self::LEVEL_STEP,
                T::ENTRIES.len(),
                Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
                true,
            )
            .map(|_| ())
    }

    fn read_level(
        &self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if elem_id.name().as_str() == Self::LEVEL_NAME {
            ElemValueAccessor::<i32>::set_vals(elem_value, T::ENTRIES.len(), |idx| {
                T::read_level(avc, idx, timeout_ms).map(|level| level as i32)
            })
            .map(|_| true)
        } else {
            Ok(false)
        }
    }

    fn write_level(
        &self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if elem_id.name().as_str() == Self::LEVEL_NAME {
            ElemValueAccessor::<i32>::get_vals(new, old, T::ENTRIES.len(), |idx, val| {
                T::write_level(avc, idx, val as i16, timeout_ms)
            })
            .map(|_| true)
        } else {
            Ok(false)
        }
    }
}

pub trait AvcLrBalanceCtlOperation<T: AvcLevelOperation + AvcLrBalanceOperation>:
    AvcLevelCtlOperation<T>
{
    const BALANCE_NAME: &'static str;

    const BALANCE_MIN: i32 = T::BALANCE_MIN as i32;
    const BALANCE_MAX: i32 = T::BALANCE_MAX as i32;
    const BALANCE_STEP: i32 = T::BALANCE_STEP as i32;

    fn load_balance(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::BALANCE_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::BALANCE_MIN,
                Self::BALANCE_MAX,
                Self::BALANCE_STEP,
                T::ENTRIES.len(),
                None,
                true,
            )
            .map(|_| ())
    }

    fn read_balance(
        &self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if elem_id.name().as_str() == Self::BALANCE_NAME {
            ElemValueAccessor::<i32>::set_vals(elem_value, T::ENTRIES.len(), |idx| {
                T::read_lr_balance(avc, idx, timeout_ms).map(|balance| balance as i32)
            })
            .map(|_| true)
        } else {
            Ok(false)
        }
    }

    fn write_balance(
        &self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if elem_id.name().as_str() == Self::BALANCE_NAME {
            ElemValueAccessor::<i32>::get_vals(new, old, T::ENTRIES.len(), |idx, val| {
                T::write_lr_balance(avc, idx, val as i16, timeout_ms)
            })
            .map(|_| true)
        } else {
            Ok(false)
        }
    }
}

pub trait AvcMuteCtlOperation<T: AvcLevelOperation + AvcMuteOperation>:
    AvcLevelCtlOperation<T>
{
    const MUTE_NAME: &'static str;

    fn load_mute(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::ENTRIES.len(), true)
            .map(|_| ())
    }

    fn read_mute(
        &self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if elem_id.name().as_str() == Self::MUTE_NAME {
            ElemValueAccessor::<bool>::set_vals(elem_value, T::ENTRIES.len(), |idx| {
                T::read_mute(avc, idx, timeout_ms)
            })
            .map(|_| true)
        } else {
            Ok(false)
        }
    }

    fn write_mute(
        &self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if elem_id.name().as_str() == Self::MUTE_NAME {
            ElemValueAccessor::<bool>::get_vals(new, old, T::ENTRIES.len(), |idx, val| {
                T::write_mute(avc, idx, val, timeout_ms)
            })
            .map(|_| true)
        } else {
            Ok(false)
        }
    }
}

/// The trait for operation to selector control.
pub trait AvcSelectorCtlOperation<T: AvcSelectorOperation> {
    const SELECTOR_NAME: &'static str;
    const SELECTOR_LABELS: &'static [&'static str];
    const ITEM_LABELS: &'static [&'static str];

    const CH_COUNT: usize = T::FUNC_BLOCK_ID_LIST.len();

    fn load_selector(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        assert_eq!(
            Self::SELECTOR_LABELS.len(),
            T::FUNC_BLOCK_ID_LIST.len(),
            "Programming error for count of selectors: {}",
            Self::SELECTOR_NAME
        );
        assert_eq!(
            Self::ITEM_LABELS.len(),
            T::INPUT_PLUG_ID_LIST.len(),
            "Programming error for count of values: {}",
            Self::SELECTOR_NAME
        );

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::SELECTOR_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, Self::CH_COUNT, Self::ITEM_LABELS, None, true)
            .map(|_| ())
    }

    fn read_selector(
        &self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if elem_id.name().as_str() == Self::SELECTOR_NAME {
            ElemValueAccessor::<u32>::set_vals(elem_value, Self::CH_COUNT, |idx| {
                T::read_selector(avc, idx, timeout_ms).map(|val| val as u32)
            })
            .map(|_| true)
        } else {
            Ok(false)
        }
    }

    fn write_selector(
        &self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if elem_id.name().as_str() == Self::SELECTOR_NAME {
            ElemValueAccessor::<u32>::get_vals(new, old, Self::CH_COUNT, |idx, val| {
                T::write_selector(avc, idx, val as usize, timeout_ms)
            })
            .map(|_| true)
        } else {
            Ok(false)
        }
    }
}
