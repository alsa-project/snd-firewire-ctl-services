// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

const DEFAULT_NAME: &str = "IEC958 Playback Default";
const MASK_NAME: &str = "IEC958 Playback Mask";

#[derive(Default, Debug)]
pub(crate) struct Iec60958Ctl<T>
where
    T: EfwHardwareSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwHwCtlFlags>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwHwCtlFlags>,
{
    pub elem_id_list: Vec<ElemId>,
    params: EfwHwCtlFlags,
    _phantom: PhantomData<T>,
}

impl<T> Iec60958Ctl<T>
where
    T: EfwHardwareSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwHwCtlFlags>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwHwCtlFlags>,
{
    const AES0_PROFESSIONAL: u8 = 0x1;
    const AES0_NONAUDIO: u8 = 0x2;

    pub(crate) fn cache(&mut self, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        T::cache_wholly(unit, &mut self.params, timeout_ms)
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, DEFAULT_NAME, 0);
        card_cntr
            .add_iec60958_elem(&elem_id, 1, true)
            .map(|elem_id| self.elem_id_list.push(elem_id))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MASK_NAME, 0);
        card_cntr
            .add_iec60958_elem(&elem_id, 1, false)
            .map(|elem_id| self.elem_id_list.push(elem_id))?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            DEFAULT_NAME => {
                let mut val = [0; 24];
                if self
                    .params
                    .0
                    .iter()
                    .find(|flag| HwCtlFlag::SpdifPro.eq(flag))
                    .is_some()
                {
                    val[0] |= Self::AES0_PROFESSIONAL;
                }
                if self
                    .params
                    .0
                    .iter()
                    .find(|flag| HwCtlFlag::SpdifNoneAudio.eq(flag))
                    .is_some()
                {
                    val[0] |= Self::AES0_NONAUDIO;
                }
                elem_value.set_iec60958_channel_status(&val);
                Ok(true)
            }
            MASK_NAME => {
                let mut val = [0; 24];
                val[0] = Self::AES0_PROFESSIONAL | Self::AES0_NONAUDIO;
                elem_value.set_iec60958_channel_status(&val);
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
            DEFAULT_NAME => {
                let mut params = self.params.clone();
                let vals = elem_value.iec60958_channel_status();

                if vals[0] & Self::AES0_PROFESSIONAL > 0 {
                    if params
                        .0
                        .iter()
                        .find(|f| HwCtlFlag::SpdifPro.eq(f))
                        .is_some()
                    {
                        params.0.push(HwCtlFlag::SpdifPro);
                    }
                } else {
                    params.0.retain(|f| HwCtlFlag::SpdifPro.eq(f));
                }
                if vals[0] & Self::AES0_NONAUDIO > 0 {
                    if params
                        .0
                        .iter()
                        .find(|f| HwCtlFlag::SpdifNoneAudio.eq(f))
                        .is_some()
                    {
                        params.0.push(HwCtlFlag::SpdifNoneAudio);
                    }
                } else {
                    params.0.retain(|f| HwCtlFlag::SpdifNoneAudio.eq(f));
                }

                T::update_partially(unit, &mut self.params, params, timeout_ms)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
