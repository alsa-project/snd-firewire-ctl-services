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
        let res = T::cache_freq(avc, self.state_mut(), timeout_ms);
        debug!(params = ?self.state(), ?res);
        res
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
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_RATE_NAME => {
                unit.lock()?;
                let mut params = self.state().clone();
                params.freq_idx = elem_value.enumerated()[0] as usize;
                let res = T::update_freq(avc, &params, self.state_mut(), timeout_ms);
                debug!(params = ?self.state(), ?res);
                let _ = unit.unlock();
                res.map(|_| true)
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
        let res = T::cache_src(avc, self.state_mut(), timeout_ms);
        debug!(params = ?self.state(), ?res);
        res
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
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_SRC_NAME => {
                unit.lock()?;
                let mut params = self.state().clone();
                params.src_idx = elem_value.enumerated()[0] as usize;
                let res = T::update_src(avc, &params, self.state_mut(), timeout_ms);
                debug!(params = ?self.state(), ?res);
                let _ = unit.unlock();
                res.map(|_| true)
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

    fn state(&self) -> &AvcLevelParameters;
    fn state_mut(&mut self) -> &mut AvcLevelParameters;

    fn load_level(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
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

    fn cache_levels(&mut self, avc: &BebobAvc, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache_levels(avc, self.state_mut(), timeout_ms);
        debug!(params = ?self.state(), ?res);
        res
    }

    fn read_levels(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if elem_id.name().as_str() == Self::LEVEL_NAME {
            let vals: Vec<i32> = self
                .state()
                .levels
                .iter()
                .map(|&level| level as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write_level(
        &mut self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if elem_id.name().as_str() == Self::LEVEL_NAME {
            let mut params = self.state().clone();
            let vals = &elem_value.int()[..params.levels.len()];
            params
                .levels
                .iter_mut()
                .zip(vals)
                .for_each(|(level, &val)| *level = val as i16);
            let res = T::update_levels(avc, &params, self.state_mut(), timeout_ms);
            debug!(params = ?self.state(), ?res);
            res.map(|_| true)
        } else {
            Ok(false)
        }
    }
}

pub trait AvcLrBalanceCtlOperation<T: AvcLrBalanceOperation> {
    const BALANCE_NAME: &'static str;

    const BALANCE_MIN: i32 = T::BALANCE_MIN as i32;
    const BALANCE_MAX: i32 = T::BALANCE_MAX as i32;
    const BALANCE_STEP: i32 = T::BALANCE_STEP as i32;

    fn state(&self) -> &AvcLrBalanceParameters;
    fn state_mut(&mut self) -> &mut AvcLrBalanceParameters;

    fn load_balance(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
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

    fn cache_balances(&mut self, avc: &BebobAvc, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache_lr_balances(avc, self.state_mut(), timeout_ms);
        debug!(params = ?self.state(), ?res);
        res
    }

    fn read_balances(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if elem_id.name().as_str() == Self::BALANCE_NAME {
            let vals: Vec<i32> = self
                .state()
                .balances
                .iter()
                .map(|&balance| balance as i32)
                .collect();
            elem_value.set_int(&vals);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write_balance(
        &mut self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if elem_id.name().as_str() == Self::BALANCE_NAME {
            let mut params = self.state().clone();
            let vals = &elem_value.int()[..params.balances.len()];
            params
                .balances
                .iter_mut()
                .zip(vals)
                .for_each(|(balance, &val)| *balance = val as i16);
            let res = T::update_lr_balances(avc, &params, self.state_mut(), timeout_ms);
            debug!(params = ?self.state(), ?res);
            res.map(|_| true)
        } else {
            Ok(false)
        }
    }
}

pub trait AvcMuteCtlOperation<T: AvcMuteOperation> {
    const MUTE_NAME: &'static str;

    fn state(&self) -> &AvcMuteParameters;
    fn state_mut(&mut self) -> &mut AvcMuteParameters;

    fn load_mute(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::ENTRIES.len(), true)
            .map(|_| ())
    }

    fn cache_mutes(&mut self, avc: &BebobAvc, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache_mutes(avc, self.state_mut(), timeout_ms);
        debug!(params = ?self.state(), ?res);
        res
    }

    fn read_mutes(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if elem_id.name().as_str() == Self::MUTE_NAME {
            elem_value.set_bool(&self.state().mutes);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write_mute(
        &mut self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if elem_id.name().as_str() == Self::MUTE_NAME {
            let mut params = self.state().clone();
            let vals = &elem_value.boolean()[..params.mutes.len()];
            params.mutes.copy_from_slice(&vals);
            let res = T::update_mutes(avc, &params, self.state_mut(), timeout_ms);
            debug!(params = ?self.state(), ?res);
            res.map(|_| true)
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

    fn state(&self) -> &AvcSelectorParameters;
    fn state_mut(&mut self) -> &mut AvcSelectorParameters;

    fn load_selector(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
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

    fn cache_selectors(&mut self, avc: &BebobAvc, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache_selectors(avc, self.state_mut(), timeout_ms);
        debug!(params = ?self.state(), ?res);
        res
    }

    fn read_selectors(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if elem_id.name().as_str() == Self::SELECTOR_NAME {
            let vals: Vec<u32> = self
                .state()
                .selectors
                .iter()
                .map(|&selector| selector as u32)
                .collect();
            elem_value.set_enum(&vals);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write_selector(
        &mut self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if elem_id.name().as_str() == Self::SELECTOR_NAME {
            let mut params = self.state().clone();
            let vals = &new.enumerated()[..params.selectors.len()];
            params
                .selectors
                .iter_mut()
                .zip(vals)
                .for_each(|(selector, &val)| *selector = val as usize);
            let res = T::update_selectors(avc, &params, self.state_mut(), timeout_ms);
            debug!(params = ?self.state(), ?res);
            res.map(|_| true)
        } else {
            Ok(false)
        }
    }
}
