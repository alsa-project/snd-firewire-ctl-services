// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

#[derive(Default, Debug)]
pub(crate) struct Iec958Ctl<T>
where
    T: EfwHardwareSpecification
        + EfwWhollyCachableParamsOperation<SndEfw, EfwHwCtlFlags>
        + EfwPartiallyUpdatableParamsOperation<SndEfw, EfwHwCtlFlags>,
{
    pub elem_id_list: Vec<ElemId>,
    params: EfwHwCtlFlags,
    _phantom: PhantomData<T>,
}

impl<T> Iec958Ctl<T>
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

#[derive(Default)]
pub struct Iec60958Ctl(EfwHwCtlFlags);

const DEFAULT_NAME: &str = "IEC958 Playback Default";
const MASK_NAME: &str = "IEC958 Playback Mask";

impl Iec60958Ctl {
    const AES0_PROFESSIONAL: u8 = 0x1;
    const AES0_NONAUDIO: u8 = 0x2;

    fn cache(&mut self, unit: &mut SndEfw, timeout_ms: u32) -> Result<(), Error> {
        unit.get_flags(timeout_ms).map(|flags| self.0 .0 = flags)
    }

    pub fn load(
        &mut self,
        hwinfo: &HwInfo,
        unit: &mut SndEfw,
        card_cntr: &mut CardCntr,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.cache(unit, timeout_ms)?;

        let has_spdif = hwinfo
            .clk_srcs
            .iter()
            .find(|src| ClkSrc::Spdif.eq(src))
            .is_some();

        if has_spdif {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, DEFAULT_NAME, 0);
            let _ = card_cntr.add_iec60958_elem(&elem_id, 1, true)?;

            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MASK_NAME, 0);
            let _ = card_cntr.add_iec60958_elem(&elem_id, 1, false)?;
        }

        Ok(())
    }

    pub fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            DEFAULT_NAME => {
                let mut val = [0; 24];
                if self
                    .0
                     .0
                    .iter()
                    .find(|&flag| *flag == HwCtlFlag::SpdifPro)
                    .is_some()
                {
                    val[0] |= Self::AES0_PROFESSIONAL;
                }
                if self
                    .0
                     .0
                    .iter()
                    .find(|&flag| *flag == HwCtlFlag::SpdifNoneAudio)
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

    pub fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            DEFAULT_NAME => {
                let vals = elem_value.iec60958_channel_status();

                let mut enable = vec![];
                let mut disable = vec![];
                if vals[0] & Self::AES0_PROFESSIONAL > 0 {
                    enable.push(HwCtlFlag::SpdifPro);
                } else {
                    disable.push(HwCtlFlag::SpdifPro);
                }
                if vals[0] & Self::AES0_NONAUDIO > 0 {
                    enable.push(HwCtlFlag::SpdifNoneAudio);
                } else {
                    disable.push(HwCtlFlag::SpdifNoneAudio);
                }

                unit.set_flags(Some(&enable), Some(&disable), timeout_ms)
                    .map(|_| {
                        enable.iter().for_each(|&flag| {
                            if self.0 .0.iter().find(|f| flag.eq(f)).is_none() {
                                self.0 .0.push(flag);
                            }
                        });
                        disable.iter().for_each(|&flag| {
                            self.0 .0.retain(|f| flag.eq(f));
                        });
                    })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
