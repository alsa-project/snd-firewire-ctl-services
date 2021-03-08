// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};
use hinawa::{FwNode, SndUnit, SndUnitExt};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;

use ff_protocols::former::*;

const VOL_MIN: i32 = 0x00000000;
const VOL_ZERO: i32 = 0x00008000;
const VOL_MAX: i32 = 0x00010000;
const VOL_STEP: i32 = 1;
const VOL_TLV: DbInterval = DbInterval{min: -9000, max: 600, linear: false, mute_avail: false};

#[derive(Default, Debug)]
pub struct FormerOutCtl<V>
    where V: AsRef<[i32]> + AsMut<[i32]>,
{
    state: V,
}

impl<'a, V> FormerOutCtl<V>
    where V: AsRef<[i32]> + AsMut<[i32]>,
{
    const VOL_NAME: &'a str = "output-volume";

    pub fn load<U>(&mut self, unit: &SndUnit, proto: &U, card_cntr: &mut CardCntr, timeout_ms: u32)
        -> Result<(), Error>
        where U: RmeFormerOutputProtocol<FwNode, V>,
              V: AsRef<[i32]> + AsMut<[i32]>,
    {
        self.state.as_mut().iter_mut()
            .for_each(|vol| *vol = VOL_ZERO);
        proto.init_output_vols(&unit.get_node(), &self.state, timeout_ms)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::VOL_NAME, 0);
        let _ = card_cntr.add_int_elems(&elem_id, 1, VOL_MIN, VOL_MAX, VOL_STEP,
                                        self.state.as_ref().len(), Some(&Vec::<u32>::from(&VOL_TLV)), true)?;

        Ok(())
    }

    pub fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::VOL_NAME => {
                elem_value.set_int(&self.state.as_ref());
                Ok(true)
            },
            _ => Ok(false),
        }
    }

    pub fn write<U>(&mut self, unit: &SndUnit, proto: &U, elem_id: &ElemId, new: &alsactl::ElemValue,
                    timeout_ms: u32)
        -> Result<bool, Error>
        where U: RmeFormerOutputProtocol<FwNode, V>,
              V: AsRef<[i32]> + AsMut<[i32]>,
    {
        match elem_id.get_name().as_str() {
            Self::VOL_NAME => {
                let mut vals = self.state.as_ref().to_vec();
                new.get_int(&mut vals);
                proto.write_output_vols(&unit.get_node(), &mut self.state, &vals, timeout_ms)
                    .map(|_| true)
            },
            _ => Ok(false),
        }
    }
}
