// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::Error;

use hinawa::{FwFcpExt, FwNode, FwReq};
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExtManual};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;
use core::elem_value_accessor::*;

use ta1394::*;

use bebob_protocols::{*, maudio::special::*};

use crate::common_ctls::*;
use super::special_ctls::{StateCache, MixerCtl, InputCtl, OutputCtl, AuxCtl, HpCtl};

pub type Fw1814Model = SpecialModel<Fw1814ClkProtocol>;
pub type ProjectMixModel = SpecialModel<ProjectMixClkProtocol>;

pub struct SpecialModel<T: MediaClockFrequencyOperation + Default> {
    avc: BebobAvc,
    req: FwReq,
    clk_ctl: ClkCtl<T>,
    meter_ctl: MeterCtl,
    cache: StateCache,
}

const FCP_TIMEOUT_MS: u32 = 200;
const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
struct ClkCtl<T: MediaClockFrequencyOperation + Default>(Vec<ElemId>, T);

impl<T: MediaClockFrequencyOperation + Default> MediaClkFreqCtlOperation<T> for ClkCtl<T> {}

impl<T: MediaClockFrequencyOperation + Default> Default for SpecialModel<T> {
    fn default() -> Self {
        Self {
            avc: Default::default(),
            req: Default::default(),
            clk_ctl: Default::default(),
            meter_ctl: Default::default(),
            cache: StateCache::new(),
        }
    }
}

#[derive(Default)]
struct MeterCtl(MaudioSpecialMeterState, Vec<ElemId>);

impl<T: MediaClockFrequencyOperation + Default> CtlModel<SndUnit> for SpecialModel<T> {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.get_node())?;

        self.clk_ctl.load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.meter_ctl.load_state(card_cntr, &self.req, &unit.get_node(), TIMEOUT_MS)?;

        MixerCtl::load(&mut self.cache, card_cntr)?;
        InputCtl::load(&mut self.cache, card_cntr)?;
        OutputCtl::load(&mut self.cache, card_cntr)?;
        AuxCtl::load(&mut self.cache, card_cntr)?;
        HpCtl::load(&mut self.cache, card_cntr)?;

        self.cache.upload(unit, &self.req)?;

        Ok(())
    }

    fn read(&mut self, _: &mut SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if MixerCtl::read(&mut self.cache, elem_id, elem_value)? {
            Ok(true)
        } else if InputCtl::read(&mut self.cache, elem_id, elem_value)? {
            Ok(true)
        } else if OutputCtl::read(&mut self.cache, elem_id, elem_value)? {
            Ok(true)
        } else if AuxCtl::read(&mut self.cache, elem_id, elem_value)? {
            Ok(true)
        } else if HpCtl::read(&mut self.cache, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndUnit, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.write_freq(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS * 3)? {
            Ok(true)
        } else if MixerCtl::write(&mut self.cache, unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if InputCtl::write(&mut self.cache, unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if OutputCtl::write(&mut self.cache, unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if AuxCtl::write(&mut self.cache, unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if HpCtl::write(&mut self.cache, unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<T: MediaClockFrequencyOperation + Default> MeasureModel<SndUnit> for SpecialModel<T> {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut SndUnit) -> Result<(), Error> {
        let switch = self.meter_ctl.0.switch;

        self.meter_ctl.measure_state(&self.req, &unit.get_node(), TIMEOUT_MS)?;

        if switch != self.meter_ctl.0.switch {
            let mut op = MaudioSpecialLedSwitch::new(self.meter_ctl.0.switch);
            self.avc.control(&AvcAddr::Unit, &mut op, FCP_TIMEOUT_MS)?;
        }

        Ok(())
    }

    fn measure_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.meter_ctl.read_state(elem_id, elem_value)
    }
}

impl<T: MediaClockFrequencyOperation + Default> NotifyModel<SndUnit, bool> for SpecialModel<T> {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.0);
    }

    fn parse_notification(&mut self, _: &mut SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)
    }
}

const ANALOG_INPUT_METER_NAME: &str = "meter:analog-input";
const SPDIF_INPUT_METER_NAME: &str = "meter:spdif-input";
const ADAT_INPUT_METER_NAME: &str = "meter:adat-input";
const ANALOG_OUTPUT_METER_NAME: &str = "meter:analog-output";
const SPDIF_OUTPUT_METER_NAME: &str = "meter:spdif-output";
const ADAT_OUTPUT_METER_NAME: &str = "meter:adat-output";
const HP_METER_NAME: &str = "meter:headhpone";
const AUX_OUT_METER_NAME: &str = "meter:aux-output";
const ROTARY_NAME: &str = "rotary";
const SWITCH_NAME: &str = "switch";
const SYNC_STATUS_NAME: &str = "sync-status";

const ANALOG_INPUT_LABELS: [&str; 8] = [
    "analog-input-1", "analog-input-2", "analog-input-3", "analog-input-4",
    "analog-input-5", "analog-input-6", "analog-input-7", "analog-input-8",
];

const SPDIF_INPUT_LABELS: [&str; 2] = [
    "spdif-input-1", "spdif-input-2",
];

const ADAT_INPUT_LABELS: [&str; 8] = [
    "adat-input-1", "adat-input-2", "adat-input-3", "adat-input-4",
    "adat-input-5", "adat-input-6", "adat-input-7", "adat-input-8",
];

const ANALOG_OUTPUT_LABELS: [&str; 4] = [
    "analog-output-1", "analog-output-2", "analog-output-3", "analog-output-4",
];

const SPDIF_OUTPUT_LABELS: [&str; 2] = [
    "spdif-output-1", "spdif-input-2",
];

const ADAT_OUTPUT_LABELS: [&str; 8] = [
    "adat-output-1", "adat-output-2", "adat-output-3", "adat-output-4",
    "adat-output-5", "adat-output-6", "adat-output-7", "adat-output-8",
];

const HEADPHONE_LABELS: [&'static str; 4] = [
    "headphone-1", "headphone-2", "headphone-3", "headphone-4",
];

const AUX_OUTPUT_LABELS: [&'static str; 2] = [
    "aux-output-1", "aux-output-2",
];

impl MeterCtl {
    const METER_TLV: DbInterval = DbInterval {
        min: -14400,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    fn add_level_int_elems(
        card_cntr: &mut CardCntr,
        name: &str,
        labels: &[&str],
    ) -> Result<Vec<ElemId>, Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                MaudioSpecialMeterProtocol::LEVEL_MIN as i32,
                MaudioSpecialMeterProtocol::LEVEL_MAX as i32,
                MaudioSpecialMeterProtocol::LEVEL_STEP as i32,
                labels.len(),
                Some(&Into::<Vec<u32>>::into(Self::METER_TLV)),
                false,
            )
    }

    fn load_state(
        &mut self,
        card_cntr: &mut CardCntr,
        req: &FwReq,
        node: &FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        [
            (ANALOG_INPUT_METER_NAME, &ANALOG_INPUT_LABELS[..]),
            (SPDIF_INPUT_METER_NAME, &SPDIF_INPUT_LABELS[..]),
            (ADAT_INPUT_METER_NAME, &ADAT_INPUT_LABELS[..]),
            (ANALOG_OUTPUT_METER_NAME, &ANALOG_OUTPUT_LABELS[..]),
            (SPDIF_OUTPUT_METER_NAME, &SPDIF_OUTPUT_LABELS[..]),
            (ADAT_OUTPUT_METER_NAME, &ADAT_OUTPUT_LABELS[..]),
            (HP_METER_NAME, &HEADPHONE_LABELS[..]),
            (AUX_OUT_METER_NAME, &AUX_OUTPUT_LABELS[..]),
        ].iter()
            .try_for_each(|(name, labels)| {
                Self::add_level_int_elems(card_cntr, name, labels)
                    .map(|mut elem_id_list| self.1.append(&mut elem_id_list))
            })?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ROTARY_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                MaudioSpecialMeterProtocol::ROTARY_MIN as i32,
                MaudioSpecialMeterProtocol::ROTARY_MAX as i32,
                MaudioSpecialMeterProtocol::ROTARY_STEP as i32,
                3,
                None,
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SWITCH_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SYNC_STATUS_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        self.measure_state(req, node, timeout_ms)
    }

    fn measure_state(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        MaudioSpecialMeterProtocol::read_state(req, node, &mut self.0, timeout_ms)
    }

    fn read_state(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            ANALOG_INPUT_METER_NAME => {
                ElemValueAccessor::set_vals(
                    elem_value,
                    ANALOG_INPUT_LABELS.len(),
                    |idx| Ok(self.0.analog_inputs[idx] as i32)
                )
                    .map(|_| true)
            }
            SPDIF_INPUT_METER_NAME => {
                ElemValueAccessor::set_vals(
                    elem_value,
                    SPDIF_INPUT_LABELS.len(),
                    |idx| Ok(self.0.spdif_inputs[idx] as i32)
                )
                    .map(|_| true)
            }
            ADAT_INPUT_METER_NAME => {
                ElemValueAccessor::set_vals(
                    elem_value,
                    ADAT_INPUT_LABELS.len(),
                    |idx| Ok(self.0.adat_inputs[idx] as i32)
                )
                    .map(|_| true)
            }
            ANALOG_OUTPUT_METER_NAME => {
                ElemValueAccessor::set_vals(
                    elem_value,
                    ANALOG_OUTPUT_LABELS.len(),
                    |idx| Ok(self.0.analog_outputs[idx] as i32)
                )
                    .map(|_| true)
            }
            SPDIF_OUTPUT_METER_NAME => {
                ElemValueAccessor::set_vals(
                    elem_value,
                    SPDIF_OUTPUT_LABELS.len(),
                    |idx| Ok(self.0.spdif_outputs[idx] as i32)
                )
                    .map(|_| true)
            }
            ADAT_OUTPUT_METER_NAME => {
                ElemValueAccessor::set_vals(
                    elem_value,
                    ADAT_OUTPUT_LABELS.len(),
                    |idx| Ok(self.0.adat_outputs[idx] as i32)
                )
                    .map(|_| true)
            }
            HP_METER_NAME => {
                ElemValueAccessor::set_vals(
                    elem_value,
                    HEADPHONE_LABELS.len(),
                    |idx| Ok(self.0.headphone[idx] as i32)
                )
                    .map(|_| true)
            }
            AUX_OUT_METER_NAME => {
                ElemValueAccessor::set_vals(
                    elem_value,
                    AUX_OUTPUT_LABELS.len(),
                    |idx| Ok(self.0.aux_outputs[idx] as i32)
                )
                    .map(|_| true)
            }
            ROTARY_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 3, |idx| {
                    Ok(self.0.rotaries[idx] as i32)
                })
                    .map(|_| true)
            }
            SWITCH_NAME => {
                elem_value.set_bool(&[self.0.switch]);
                Ok(true)
            }
            SYNC_STATUS_NAME => {
                elem_value.set_bool(&[self.0.sync_status]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alsactl::CardError;

    #[test]
    fn test_clk_ctl_definition() {
        let mut card_cntr = CardCntr::new();
        let mut ctl = ClkCtl::<Fw1814ClkProtocol>::default();

        let error = ctl.load_freq(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = ClkCtl::<ProjectMixClkProtocol>::default();

        let error = ctl.load_freq(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }

    #[test]
    fn test_meter_label_count() {
        let meter_ctl = MeterCtl::default();
        assert_eq!(meter_ctl.0.analog_inputs.len(), ANALOG_INPUT_LABELS.len());
        assert_eq!(meter_ctl.0.spdif_inputs.len(), SPDIF_INPUT_LABELS.len());
        assert_eq!(meter_ctl.0.adat_inputs.len(), ADAT_INPUT_LABELS.len());
        assert_eq!(meter_ctl.0.analog_outputs.len(), ANALOG_OUTPUT_LABELS.len());
        assert_eq!(meter_ctl.0.spdif_outputs.len(), SPDIF_OUTPUT_LABELS.len());
        assert_eq!(meter_ctl.0.adat_outputs.len(), ADAT_OUTPUT_LABELS.len());
        assert_eq!(meter_ctl.0.headphone.len(), HEADPHONE_LABELS.len());
        assert_eq!(meter_ctl.0.aux_outputs.len(), AUX_OUTPUT_LABELS.len());
    }
}
