// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{FwFcpExt, SndUnitExt};

use alsactl::CardExtManual;

use core::card_cntr;
use core::elem_value_accessor::ElemValueAccessor;

use oxfw_protocols::lacie::*;

use super::common_ctl::CommonCtl;

#[derive(Default, Debug)]
pub struct LacieModel {
    avc: hinawa::FwFcp,
    common_ctl: CommonCtl,
    voluntary: bool,
}

const FCP_TIMEOUT_MS: u32 = 100;

const VOL_NAME: &str = "PCM Playback Volume";
const MUTE_NAME: &str = "PCM Playback Switch";

impl card_cntr::CtlModel<hinawa::SndUnit> for LacieModel {
    fn load(
        &mut self,
        unit: &mut hinawa::SndUnit,
        card_cntr: &mut card_cntr::CardCntr,
    ) -> Result<(), Error> {
        self.avc.bind(&unit.get_node())?;

        self.common_ctl.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;

        // NOTE: I have a plan to remove control functionality from ALSA oxfw driver for future.
        let elem_id_list = card_cntr.card.get_elem_id_list()?;
        self.voluntary = elem_id_list
            .iter()
            .find(|elem_id| elem_id.get_name().as_str() == VOL_NAME)
            .is_none();
        if self.voluntary {
            let elem_id =
                alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, VOL_NAME, 0);
            let _ = card_cntr.add_int_elems(
                &elem_id,
                1,
                FwSpeakersProtocol::VOLUME_MIN as i32,
                FwSpeakersProtocol::VOLUME_MAX as i32,
                FwSpeakersProtocol::VOLUME_STEP as i32,
                1,
                None,
                true,
            )?;

            let elem_id =
                alsactl::ElemId::new_by_name(alsactl::ElemIfaceType::Mixer, 0, 0, MUTE_NAME, 0);
            let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;
        }

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut hinawa::SndUnit,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        if self
            .common_ctl
            .read(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.voluntary {
            match elem_id.get_name().as_str() {
                VOL_NAME => ElemValueAccessor::<i32>::set_val(elem_value, || {
                    let mut vol = 0;
                    FwSpeakersProtocol::read_volume(&mut self.avc, &mut vol, FCP_TIMEOUT_MS)
                        .map(|_| vol as i32)
                })
                .map(|_| true),
                MUTE_NAME => ElemValueAccessor::<bool>::set_val(elem_value, || {
                    let mut mute = false;
                    FwSpeakersProtocol::read_mute(&mut self.avc, &mut mute, FCP_TIMEOUT_MS)
                        .map(|_| mute)
                })
                .map(|_| true),
                _ => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut hinawa::SndUnit,
        elem_id: &alsactl::ElemId,
        _: &alsactl::ElemValue,
        new: &alsactl::ElemValue,
    ) -> Result<bool, Error> {
        if self
            .common_ctl
            .write(unit, &self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.voluntary {
            match elem_id.get_name().as_str() {
                VOL_NAME => ElemValueAccessor::<i32>::get_val(new, |val| {
                    FwSpeakersProtocol::write_volume(&mut self.avc, val as i16, FCP_TIMEOUT_MS)
                })
                .map(|_| true),
                MUTE_NAME => ElemValueAccessor::<bool>::get_val(new, |val| {
                    FwSpeakersProtocol::write_mute(&mut self.avc, val, FCP_TIMEOUT_MS)
                })
                .map(|_| true),
                _ => Ok(false),
            }
        } else {
            Ok(false)
        }
    }
}

impl card_cntr::NotifyModel<hinawa::SndUnit, bool> for LacieModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, _: &mut hinawa::SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &hinawa::SndUnit,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
    ) -> Result<bool, Error> {
        self.common_ctl
            .read(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)
    }
}
