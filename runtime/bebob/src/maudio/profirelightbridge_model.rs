// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    protocols::{maudio::pfl::*, *},
};

#[derive(Default)]
pub struct PflModel {
    avc: BebobAvc,
    req: FwReq,
    clk_ctl: ClkCtl,
    meter_ctl: MeterCtl,
    input_params_ctl: InputParamsCtl,
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

#[derive(Default)]
struct InputParamsCtl(PflInputParameters);

impl CtlModel<(SndUnit, FwNode)> for PflModel {
    fn load(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.clk_ctl
            .load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl
            .load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.meter_ctl
            .load_state(card_cntr, unit, &self.req, TIMEOUT_MS)?;

        self.input_params_ctl
            .load_params(card_cntr, unit, &self.req, TIMEOUT_MS)?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self
            .clk_ctl
            .read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .clk_ctl
            .read_src(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.meter_ctl.read_state(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_params_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.write_freq(
            &mut unit.0,
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS * 3,
        )? {
            Ok(true)
        } else if self.clk_ctl.write_src(
            &mut unit.0,
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS * 3,
        )? {
            Ok(true)
        } else if self
            .input_params_ctl
            .write_params(unit, &self.req, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndUnit, FwNode)> for PflModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.meter_ctl.measure_state(unit, &self.req, TIMEOUT_MS)
    }

    fn measure_elem(
        &mut self,
        _: &(SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.meter_ctl.read_state(elem_id, elem_value)
    }
}

impl NotifyModel<(SndUnit, FwNode), bool> for PflModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.0);
    }

    fn parse_notification(&mut self, _: &mut (SndUnit, FwNode), _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.clk_ctl
            .read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)
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
        unit: &(SndUnit, FwNode),
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
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        PflMeterProtocol::read_meter(req, &unit.1, &mut self.0, timeout_ms)
    }

    fn read_state(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
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

const ADAT_MUTE_NAME: &str = "adat-input-mute";
const SPDIF_MUTE_NAME: &str = "spdif-input-mute";
const FORCE_SMUX_NAME: &str = "force-S/MUX";

impl InputParamsCtl {
    fn load_params(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ADAT_MUTE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 4, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, SPDIF_MUTE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, FORCE_SMUX_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        PflInputParametersProtocol::write_input_parameters(req, &unit.1, &mut self.0, timeout_ms)
    }

    fn read_params(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ADAT_MUTE_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, 4, |idx| Ok(self.0.adat_mute[idx]))
                    .map(|_| true)
            }
            SPDIF_MUTE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || Ok(self.0.spdif_mute))
                    .map(|_| true)
            }
            FORCE_SMUX_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || Ok(self.0.force_smux))
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ADAT_MUTE_NAME => {
                if unit.0.is_locked() {
                    Err(Error::new(FileError::Again, "Packet streaming started"))?;
                }

                let mut params = self.0.clone();

                ElemValueAccessor::<bool>::get_vals(new, old, 4, |idx, val| {
                    params.adat_mute[idx] = val;
                    Ok(())
                })
                .and_then(|_| {
                    PflInputParametersProtocol::write_input_parameters(
                        req,
                        &unit.1,
                        &mut params,
                        timeout_ms,
                    )?;
                    self.0 = params;
                    Ok(true)
                })
            }
            SPDIF_MUTE_NAME => {
                if unit.0.is_locked() {
                    Err(Error::new(FileError::Again, "Packet streaming started"))?;
                }

                let mut params = self.0.clone();

                ElemValueAccessor::<bool>::get_val(new, |val| {
                    params.spdif_mute = val;
                    Ok(())
                })
                .and_then(|_| {
                    PflInputParametersProtocol::write_input_parameters(
                        req,
                        &unit.1,
                        &mut params,
                        timeout_ms,
                    )?;
                    self.0 = params;
                    Ok(true)
                })
            }
            FORCE_SMUX_NAME => {
                let mut params = self.0.clone();

                ElemValueAccessor::<bool>::get_val(new, |val| {
                    params.force_smux = val;
                    Ok(())
                })
                .and_then(|_| {
                    PflInputParametersProtocol::write_input_parameters(
                        req,
                        &unit.1,
                        &mut params,
                        timeout_ms,
                    )?;
                    self.0 = params;
                    Ok(true)
                })
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
        let mut card_cntr = CardCntr::default();
        let mut ctl = ClkCtl::default();

        let error = ctl.load_freq(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let error = ctl.load_src(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
