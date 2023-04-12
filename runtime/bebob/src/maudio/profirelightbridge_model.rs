// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    protocols::{maudio::pfl::*, *},
};

#[derive(Default, Debug)]
pub struct PflModel {
    avc: BebobAvc,
    req: FwReq,
    clk_ctl: ClkCtl,
    meter_ctl: MeterCtl,
    input_params_ctl: InputParamsCtl,
}

const FCP_TIMEOUT_MS: u32 = 100;
const TIMEOUT_MS: u32 = 100;

#[derive(Default, Debug)]
struct ClkCtl(Vec<ElemId>, MediaClockParameters, SamplingClockParameters);

impl MediaClkFreqCtlOperation<PflClkProtocol> for ClkCtl {
    fn state(&self) -> &MediaClockParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut MediaClockParameters {
        &mut self.1
    }
}

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

    fn state(&self) -> &SamplingClockParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut SamplingClockParameters {
        &mut self.2
    }
}

#[derive(Default, Debug)]
struct MeterCtl(PflMeterState, Vec<ElemId>);

#[derive(Default, Debug)]
struct InputParamsCtl(PflInputParameters);

impl PflModel {
    pub fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.clk_ctl.cache_freq(&self.avc, FCP_TIMEOUT_MS)?;
        self.clk_ctl.cache_src(&self.avc, FCP_TIMEOUT_MS)?;
        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.input_params_ctl
            .cache(&self.req, &unit.1, TIMEOUT_MS)?;

        Ok(())
    }
}

impl CtlModel<(SndUnit, FwNode)> for PflModel {
    fn load(&mut self, _: &mut (SndUnit, FwNode), card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctl
            .load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl
            .load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.meter_ctl.load(card_cntr)?;

        self.input_params_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.read_freq(elem_id, elem_value)? {
            Ok(true)
        } else if self.clk_ctl.read_src(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_params_ctl.read(elem_id, elem_value)? {
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
            .write(unit, &self.req, elem_id, old, new, TIMEOUT_MS)?
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
        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)
    }

    fn measure_elem(
        &mut self,
        _: &(SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.meter_ctl.read(elem_id, elem_value)
    }
}

impl NotifyModel<(SndUnit, FwNode), bool> for PflModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.0);
    }

    fn parse_notification(
        &mut self,
        _: &mut (SndUnit, FwNode),
        &locked: &bool,
    ) -> Result<(), Error> {
        if locked {
            self.clk_ctl.cache_freq(&self.avc, FCP_TIMEOUT_MS)?;
        }
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.clk_ctl.read_freq(elem_id, elem_value)
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

impl MeterCtl {
    const DETECTED_RATE_NAME: &'static str = "detected-rate";
    const SYNC_STATUS_NAME: &'static str = "sync status";

    const ANALOG_OUTPUT_LABELS: &'static [&'static str; 2] =
        &["analog-output-1", "analog-output-2"];

    const METER_TLV: DbInterval = DbInterval {
        min: -14400,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    const DETECTED_INPUT_FREQ_LIST: &'static [PflDetectedInputFreq; 5] = &[
        PflDetectedInputFreq::Unavailable,
        PflDetectedInputFreq::R44100,
        PflDetectedInputFreq::R48000,
        PflDetectedInputFreq::R88200,
        PflDetectedInputFreq::R96000,
    ];

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                PflMeterProtocol::METER_MIN,
                PflMeterProtocol::METER_MAX,
                PflMeterProtocol::METER_STEP,
                Self::ANALOG_OUTPUT_LABELS.len(),
                Some(&Into::<Vec<u32>>::into(Self::METER_TLV)),
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::DETECTED_INPUT_FREQ_LIST
            .iter()
            .map(|freq| detected_input_freq_to_str(freq))
            .collect();

        // For detection of sampling clock frequency.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::DETECTED_RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        // For sync status.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::SYNC_STATUS_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        PflMeterProtocol::cache(req, node, &mut self.0, timeout_ms)
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::DETECTED_RATE_NAME => {
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
            Self::SYNC_STATUS_NAME => {
                elem_value.set_bool(&[self.0.sync_status]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

impl InputParamsCtl {
    const ADAT_MUTE_NAME: &'static str = "adat-input-mute";
    const SPDIF_MUTE_NAME: &'static str = "spdif-input-mute";
    const FORCE_SMUX_NAME: &'static str = "force-S/MUX";

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::ADAT_MUTE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 4, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::SPDIF_MUTE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::FORCE_SMUX_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        PflInputParametersProtocol::update(req, node, &mut self.0, timeout_ms)
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::ADAT_MUTE_NAME => {
                elem_value.set_bool(&self.0.adat_mute);
                Ok(true)
            }
            Self::SPDIF_MUTE_NAME => {
                elem_value.set_bool(&[self.0.spdif_mute]);
                Ok(true)
            }
            Self::FORCE_SMUX_NAME => {
                elem_value.set_bool(&[self.0.force_smux]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::ADAT_MUTE_NAME => {
                if unit.0.is_locked() {
                    Err(Error::new(FileError::Again, "Packet streaming started"))?;
                }

                let mut params = self.0.clone();
                let mutes = &mut params.adat_mute;
                let vals = &new.boolean()[..mutes.len()];
                mutes.copy_from_slice(vals);

                PflInputParametersProtocol::update(req, &unit.1, &mut params, timeout_ms)
                    .map(|_| true)
            }
            Self::SPDIF_MUTE_NAME => {
                if unit.0.is_locked() {
                    Err(Error::new(FileError::Again, "Packet streaming started"))?;
                }

                let mut params = self.0.clone();
                params.spdif_mute = new.boolean()[0];

                PflInputParametersProtocol::update(req, &unit.1, &mut params, timeout_ms)
                    .map(|_| true)
            }
            Self::FORCE_SMUX_NAME => {
                let mut params = self.0.clone();
                params.force_smux = new.boolean()[0];

                PflInputParametersProtocol::update(req, &unit.1, &mut params, timeout_ms)
                    .map(|_| true)
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
