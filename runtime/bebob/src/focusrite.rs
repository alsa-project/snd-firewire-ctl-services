// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

pub mod saffireproio_model;

pub mod saffire_model;
pub mod saffirele_model;

use {
    super::{common_ctls::*, *},
    protocols::{
        focusrite::{saffire::*, saffireproio::*, *},
        *,
    },
};

const OUT_MUTE_NAME: &str = "phys-output-mute";
const OUT_VOL_NAME: &str = "phys-output-volume";
const OUT_HWCTL_NAME: &str = "phys-output-hwctl";
const OUT_DIM_NAME: &str = "phys-output-dim";
const OUT_PAD_NAME: &str = "phys-output-pad";

const LEVEL_TLV: DbInterval = DbInterval {
    min: -9600,
    max: 0,
    linear: false,
    mute_avail: false,
};

trait SaffireOutputCtlOperation<
    T: SaffireParametersOperation<SaffireOutputParameters> + SaffireOutputSpecification,
>
{
    const OUTPUT_LABELS: &'static [&'static str];

    fn state(&self) -> &SaffireOutputParameters;
    fn state_mut(&mut self) -> &mut SaffireOutputParameters;

    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        assert_eq!(
            Self::OUTPUT_LABELS.len(),
            T::OFFSETS.len(),
            "Programming error about labels for physical outputs",
        );

        let mut measure_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, T::MUTE_COUNT, true)
            .map(|mut elem_id_list| measure_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_VOL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                T::VOL_COUNT,
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|mut elem_id_list| measure_elem_id_list.append(&mut elem_id_list))?;

        if T::HWCTL_COUNT > 0 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_HWCTL_NAME, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, T::HWCTL_COUNT, true)
                .map(|mut elem_id_list| measure_elem_id_list.append(&mut elem_id_list))?;
        }

        if T::DIM_COUNT > 0 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_DIM_NAME, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, T::DIM_COUNT, true)
                .map(|mut elem_id_list| measure_elem_id_list.append(&mut elem_id_list))?;
        }

        if T::PAD_COUNT > 0 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_PAD_NAME, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, T::PAD_COUNT, true)
                .map(|mut elem_id_list| measure_elem_id_list.append(&mut elem_id_list))?;
        }

        Ok(measure_elem_id_list)
    }

    fn measure_params(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        T::cache(req, node, self.state_mut(), timeout_ms)
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUT_MUTE_NAME => {
                elem_value.set_bool(&self.state().mutes);
                Ok(true)
            }
            OUT_VOL_NAME => {
                let vals: Vec<i32> = self.state().vols.iter().map(|&val| val as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            OUT_HWCTL_NAME => {
                elem_value.set_bool(&self.state().hwctls);
                Ok(true)
            }
            OUT_DIM_NAME => {
                elem_value.set_bool(&self.state().dims);
                Ok(true)
            }
            OUT_PAD_NAME => {
                elem_value.set_bool(&self.state().pads);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUT_MUTE_NAME => {
                let mut params = self.state().clone();
                params
                    .mutes
                    .copy_from_slice(&elem_value.boolean()[..T::MUTE_COUNT]);
                T::update(req, node, &params, self.state_mut(), timeout_ms).map(|_| true)
            }
            OUT_VOL_NAME => {
                let mut params = self.state().clone();
                params
                    .vols
                    .iter_mut()
                    .zip(&elem_value.int()[..T::VOL_COUNT])
                    .for_each(|(vol, val)| *vol = *val as u8);
                T::update(req, node, &params, self.state_mut(), timeout_ms).map(|_| true)
            }
            OUT_HWCTL_NAME => {
                let mut params = self.state().clone();
                params
                    .hwctls
                    .copy_from_slice(&elem_value.boolean()[..T::HWCTL_COUNT]);
                T::update(req, node, &params, self.state_mut(), timeout_ms).map(|_| true)
            }
            OUT_DIM_NAME => {
                let mut params = self.state().clone();
                params
                    .dims
                    .copy_from_slice(&elem_value.boolean()[..T::DIM_COUNT]);
                T::update(req, node, &params, self.state_mut(), timeout_ms).map(|_| true)
            }
            OUT_PAD_NAME => {
                let mut params = self.state().clone();
                params
                    .pads
                    .copy_from_slice(&elem_value.boolean()[..T::PAD_COUNT]);
                T::update(req, node, &params, self.state_mut(), timeout_ms).map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const MIDI_THROUGH_NAME: &str = "MIDI-through";
const AC3_THROUGH_NAME: &str = "AC3-through";

trait SaffireThroughCtlOperation<T: SaffireParametersOperation<SaffireThroughParameters>> {
    fn state(&self) -> &SaffireThroughParameters;
    fn state_mut(&mut self) -> &mut SaffireThroughParameters;

    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MIDI_THROUGH_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, false)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, AC3_THROUGH_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, false)?;

        Ok(())
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIDI_THROUGH_NAME => {
                elem_value.set_bool(&[self.state().midi]);
                Ok(true)
            }
            AC3_THROUGH_NAME => {
                elem_value.set_bool(&[self.state().ac3]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIDI_THROUGH_NAME => {
                let mut params = self.state().clone();
                params.midi = elem_value.boolean()[0];
                T::update(req, node, &params, self.state_mut(), timeout_ms).map(|_| true)
            }
            AC3_THROUGH_NAME => {
                let mut params = self.state().clone();
                params.ac3 = elem_value.boolean()[0];
                T::update(req, node, &params, self.state_mut(), timeout_ms).map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
