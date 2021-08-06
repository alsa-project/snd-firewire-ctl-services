// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::Error;

use hinawa::{FwFcpExt, FwReq};
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use alsa_ctl_tlv_codec::items::DbInterval;

use core::card_cntr::*;
use core::elem_value_accessor::ElemValueAccessor;

use bebob_protocols::{*, maudio::pfl::*};

use crate::common_ctls::*;
use crate::model::OUT_METER_NAME;

use super::common_proto::CommonProto;

pub struct PflModel {
    avc: BebobAvc,
    req: FwReq,
    clk_ctl: ClkCtl,
    meter_ctl: MeterCtl,
    input_ctl: InputCtl,
}

const FCP_TIMEOUT_MS: u32 = 100;
const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
struct ClkCtl(Vec<ElemId>);

impl MediaClkFreqCtlOperation<PflClkProtocol> for ClkCtl {}

impl SamplingClkSrcCtlOperation<PflClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &[
        "Internal",
        "S/PDIF",
        "ADAT-1",
        "ADAT-2",
        "ADAT-3",
        "ADAT-4",
        "Word-clock",
    ];
}

#[derive(Default)]
struct MeterCtl(PflMeterState, Vec<ElemId>);

impl Default for PflModel {
    fn default() -> Self {
        Self{
            avc: Default::default(),
            req: Default::default(),
            clk_ctl: Default::default(),
            meter_ctl: Default::default(),
            input_ctl: InputCtl::new(),
        }
    }
}

impl CtlModel<SndUnit> for PflModel {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.get_node())?;

        self.clk_ctl.load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl.load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.meter_ctl.load_state(card_cntr, unit, &self.req, TIMEOUT_MS)?;

        self.input_ctl.load(unit, &self.req, card_cntr)?;

        Ok(())
    }

    fn read(&mut self, _: &mut SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.clk_ctl.read_src(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctl.read(elem_id, elem_value)? {
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
        } else if self.clk_ctl.write_src(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS * 3)? {
            Ok(true)
        } else if self.input_ctl.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<SndUnit> for PflModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut SndUnit) -> Result<(), Error> {
        self.meter_ctl.measure_state(unit, &self.req, TIMEOUT_MS)
    }

    fn measure_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.meter_ctl.read_state(elem_id, elem_value)
    }
}

impl NotifyModel<SndUnit, bool> for PflModel {
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

fn detected_input_freq_to_str(freq: &PflDetectedInputFreq) -> &str {
    match freq {
        PflDetectedInputFreq::Unavailable => "N/A",
        PflDetectedInputFreq::R44100 => "44100",
        PflDetectedInputFreq::R48000 => "48000",
        PflDetectedInputFreq::R88200 => "88200",
        PflDetectedInputFreq::R96000 => "96000",
    }
}

const DETECTED_RATE_NAME: &str = "detected-rate";
const SYNC_STATUS_NAME: &str = "sync status";

const ANALOG_OUTPUT_LABELS: [&str; 2] = ["analog-output-1", "analog-output-2"];

impl MeterCtl {
    const METER_TLV: DbInterval = DbInterval {
        min: -14400,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    const DETECTED_INPUT_FREQ_LIST: [PflDetectedInputFreq; 5] = [
        PflDetectedInputFreq::Unavailable,
        PflDetectedInputFreq::R44100,
        PflDetectedInputFreq::R48000,
        PflDetectedInputFreq::R88200,
        PflDetectedInputFreq::R96000,
    ];

    fn load_state(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &SndUnit,
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                PflMeterProtocol::METER_MIN,
                PflMeterProtocol::METER_MAX,
                PflMeterProtocol::METER_STEP,
                ANALOG_OUTPUT_LABELS.len(),
                Some(&Into::<Vec<u32>>::into(Self::METER_TLV)),
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::DETECTED_INPUT_FREQ_LIST
            .iter()
            .map(|freq| detected_input_freq_to_str(freq))
            .collect();

        // For detection of sampling clock frequency.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, DETECTED_RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        // For sync status.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, SYNC_STATUS_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        self.measure_state(unit, req, timeout_ms)
    }

    fn measure_state(
        &mut self,
        unit: &SndUnit,
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        PflMeterProtocol::read_meter(req, &unit.get_node(), &mut self.0, timeout_ms)
    }

    fn read_state(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            DETECTED_RATE_NAME => {
                let freq = Self::DETECTED_INPUT_FREQ_LIST
                    .iter()
                    .position(|&f| f == self.0.detected_input_freq)
                    .unwrap();
                elem_value.set_enum(&[freq as u32]);
                Ok(true)
            }
            OUT_METER_NAME => {
                elem_value.set_int(&self.0.phys_outputs);
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

struct InputCtl {
    cache: [u8; Self::FRAME_COUNT],
}

impl<'a> InputCtl {
    const FRAME_COUNT: usize = 24;

    const MUTE_NAME: &'a str = "Input-mute";
    const FORCE_SMUX_NAME: &'a str = "Force-S/MUX";

    const INPUT_LABELS: &'a [&'a str] = &[
        "ADAT_1-8",
        "ADAT_9-16",
        "ADAT_17-24",
        "ADAT_25-32",
        "S/PDIF-1/2",
    ];

    const OFFSET: u64 = 0;

    fn new() -> Self {
        InputCtl {
            cache: [0;Self::FRAME_COUNT],
        }
    }

    fn load(&mut self, unit: &SndUnit, req: &FwReq, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        // For mute of input for ADAT interfaces.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer,
                                                   0, 0, Self::MUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, Self::INPUT_LABELS.len(), true)?;

        // For switch to force S/MUX.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer,
                                                   0, 0, Self::FORCE_SMUX_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        // Initialize cache.
        let val = 1 as u32;
        (0..Self::INPUT_LABELS.len()).for_each(|i| {
            let pos = i * 4;
            self.cache[pos..(pos + 4)].copy_from_slice(&val.to_be_bytes());
        });

        req.write_block(unit, Self::OFFSET, &mut self.cache)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            Self::MUTE_NAME => {
                let mut quadlet = [0;4];
                ElemValueAccessor::<bool>::set_vals(elem_value, Self::INPUT_LABELS.len(), |idx| {
                    let pos = idx * 4;
                    let bytes = &self.cache[pos..(pos + 4)];
                    quadlet.copy_from_slice(bytes);
                    Ok(u32::from_be_bytes(quadlet) > 0)
                })?;
                Ok(true)
            }
            Self::FORCE_SMUX_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    let mut quadlet = [0;4];
                    let pos = Self::INPUT_LABELS.len() * 4;
                    quadlet.copy_from_slice(&self.cache[pos..(pos + 4)]);
                    Ok(u32::from_be_bytes(quadlet) > 0)
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(&mut self, unit: &SndUnit, req: &FwReq, elem_id: &ElemId,
             old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        match elem_id.get_name().as_str() {
            Self::MUTE_NAME => {
                if unit.get_property_streaming() {
                    Ok(false)
                } else {
                    ElemValueAccessor::<bool>::get_vals(new, old, Self::INPUT_LABELS.len(), |idx, val| {
                        let val = val as u32;
                        let pos = idx * 4;
                        self.cache[pos..(pos + 4)].copy_from_slice(&val.to_be_bytes());
                        Ok(())
                    })?;
                    req.write_block(unit, Self::OFFSET, &mut self.cache)?;
                    Ok(true)
                }
            }
            Self::FORCE_SMUX_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    let val = val as u32;
                    let pos = Self::INPUT_LABELS.len() * 4;
                    self.cache[pos..(pos + 4)].copy_from_slice(&val.to_be_bytes());
                    req.write_block(unit, Self::OFFSET, &mut self.cache)
                })?;
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
        let mut ctl = ClkCtl::default();

        let error = ctl.load_freq(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let error = ctl.load_src(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
