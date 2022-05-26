// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use super::*;

#[derive(Default)]
pub struct SaffireModel {
    req: FwReq,
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    meter_ctl: MeterCtl,
    out_ctl: OutputCtl,
    specific_ctl: SpecificCtl,
    separated_mixer_ctl: SeparatedMixerCtl,
    paired_mixer_ctl: PairedMixerCtl,
    reverb_ctl: ReverbCtl,
}

const FCP_TIMEOUT_MS: u32 = 100;
const TIMEOUT_MS: u32 = 50;

#[derive(Default)]
struct ClkCtl(Vec<ElemId>);

impl MediaClkFreqCtlOperation<SaffireClkProtocol> for ClkCtl {}

impl SamplingClkSrcCtlOperation<SaffireClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal", "S/PDIF"];
}

#[derive(Default)]
struct MeterCtl(Vec<ElemId>, SaffireMeter);

#[derive(Default)]
struct OutputCtl(Vec<ElemId>, SaffireOutputParameters);

impl AsRef<SaffireOutputParameters> for OutputCtl {
    fn as_ref(&self) -> &SaffireOutputParameters {
        &self.1
    }
}

impl AsMut<SaffireOutputParameters> for OutputCtl {
    fn as_mut(&mut self) -> &mut SaffireOutputParameters {
        &mut self.1
    }
}

impl SaffireOutputCtlOperation<SaffireOutputProtocol> for OutputCtl {
    const OUTPUT_LABELS: &'static [&'static str] = &[
        "analog-output-1/2",
        "analog-output-3/4",
        "analog-output-5/6",
        "analog-output-7/8",
        "digital-output-1/2",
    ];
}

#[derive(Default)]
struct SpecificCtl(SaffireSpecificParameters);

#[derive(Default)]
struct SeparatedMixerCtl(Vec<ElemId>, SaffireMixerState);

impl AsRef<SaffireMixerState> for SeparatedMixerCtl {
    fn as_ref(&self) -> &SaffireMixerState {
        &self.1
    }
}

impl AsMut<SaffireMixerState> for SeparatedMixerCtl {
    fn as_mut(&mut self) -> &mut SaffireMixerState {
        &mut self.1
    }
}

impl SaffireMixerCtlOperation<SaffireSeparatedMixerProtocol> for SeparatedMixerCtl {
    const PHYS_INPUT_GAIN_NAME: &'static str = "mixer:separated:phys-input-gain";
    const REVERB_RETURN_GAIN_NAME: &'static str = "mixer:separated:reverb-return-gain";
    const STREAM_SRC_GAIN_NAME: &'static str = "mixer:separated:stream-source-gain";

    const MIXER_MODE: SaffireMixerMode = SaffireMixerMode::StereoSeparated;
}

#[derive(Default)]
struct PairedMixerCtl(Vec<ElemId>, SaffireMixerState);

impl AsRef<SaffireMixerState> for PairedMixerCtl {
    fn as_ref(&self) -> &SaffireMixerState {
        &self.1
    }
}

impl AsMut<SaffireMixerState> for PairedMixerCtl {
    fn as_mut(&mut self) -> &mut SaffireMixerState {
        &mut self.1
    }
}

impl SaffireMixerCtlOperation<SaffirePairedMixerProtocol> for PairedMixerCtl {
    const PHYS_INPUT_GAIN_NAME: &'static str = "mixer:paired:phys-input-gain";
    const REVERB_RETURN_GAIN_NAME: &'static str = "mixer:paired:reverb-return-gain";
    const STREAM_SRC_GAIN_NAME: &'static str = "mixer:paired:stream-source-gain";

    const MIXER_MODE: SaffireMixerMode = SaffireMixerMode::StereoPaired;
}

#[derive(Default)]
struct ReverbCtl(SaffireReverbParameters);

impl CtlModel<SndUnit> for SaffireModel {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.get_node())?;

        self.clk_ctl
            .load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl
            .load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.meter_ctl
            .load_meter(card_cntr, unit, &self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.meter_ctl.0.append(&mut elem_id_list))?;

        self.out_ctl
            .load_params(card_cntr, unit, &self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.out_ctl.0.append(&mut elem_id_list))?;

        self.specific_ctl
            .load_params(card_cntr, unit, &self.req, TIMEOUT_MS)?;

        self.separated_mixer_ctl
            .load_src_levels(
                card_cntr,
                self.specific_ctl.0.mixer_mode,
                unit,
                &self.req,
                TIMEOUT_MS,
            )
            .map(|measured_elem_id_list| self.separated_mixer_ctl.0 = measured_elem_id_list)?;

        self.paired_mixer_ctl
            .load_src_levels(
                card_cntr,
                self.specific_ctl.0.mixer_mode,
                unit,
                &self.req,
                TIMEOUT_MS,
            )
            .map(|measured_elem_id_list| self.paired_mixer_ctl.0 = measured_elem_id_list)?;

        self.reverb_ctl
            .load_params(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut SndUnit,
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
        } else if self.meter_ctl.read_meter(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else if self.specific_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else if self
            .separated_mixer_ctl
            .read_src_levels(elem_id, elem_value)?
        {
            Ok(true)
        } else if self.paired_mixer_ctl.read_src_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut SndUnit,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .clk_ctl
            .write_freq(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS * 3)?
        {
            Ok(true)
        } else if self
            .clk_ctl
            .write_src(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS * 3)?
        {
            Ok(true)
        } else if self
            .out_ctl
            .write_params(unit, &self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.specific_ctl.write_params(
            &mut self.separated_mixer_ctl,
            &mut self.paired_mixer_ctl,
            unit,
            &self.req,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.separated_mixer_ctl.write_src_levels(
            self.specific_ctl.0.mixer_mode,
            unit,
            &self.req,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.paired_mixer_ctl.write_src_levels(
            self.specific_ctl.0.mixer_mode,
            unit,
            &self.req,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .reverb_ctl
            .write_params(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndUnit, bool> for SaffireModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.0);
    }

    fn parse_notification(&mut self, _: &mut SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndUnit,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.clk_ctl
            .read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)
    }
}

impl MeasureModel<SndUnit> for SaffireModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
        elem_id_list.extend_from_slice(&self.out_ctl.0);
        elem_id_list.extend_from_slice(&self.separated_mixer_ctl.0);
        elem_id_list.extend_from_slice(&self.paired_mixer_ctl.0);
    }

    fn measure_states(&mut self, unit: &mut SndUnit) -> Result<(), Error> {
        self.meter_ctl.measure_meter(unit, &self.req, TIMEOUT_MS)?;
        self.out_ctl.measure_params(unit, &self.req, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &SndUnit,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.meter_ctl.read_meter(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_ctl.read_params(elem_id, elem_value)? {
            Ok(true)
        } else if self
            .separated_mixer_ctl
            .read_src_levels(elem_id, elem_value)?
        {
            Ok(true)
        } else if self.paired_mixer_ctl.read_src_levels(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeterCtl {
    const METER_DIG_INPUT_DETECT_NAME: &'static str = "digital-input-detection";

    const PHYS_INPUT_LABELS: &'static [&'static str] = &[
        "analog-input-1",
        "analog-input-2",
        "analog-input-3",
        "analog-input-4",
    ];

    fn load_meter(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &SndUnit,
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        let mut measured_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, IN_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                SaffireMeterProtocol::LEVEL_MIN as i32,
                SaffireMeterProtocol::LEVEL_MAX as i32,
                SaffireMeterProtocol::LEVEL_STEP as i32,
                Self::PHYS_INPUT_LABELS.len(),
                None,
                false,
            )
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Card,
            0,
            0,
            Self::METER_DIG_INPUT_DETECT_NAME,
            0,
        );
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        SaffireMeterProtocol::read_meter(req, &unit.get_node(), &mut self.1, timeout_ms)?;

        Ok(measured_elem_id_list)
    }

    fn measure_meter(&mut self, unit: &SndUnit, req: &FwReq, timeout_ms: u32) -> Result<(), Error> {
        SaffireMeterProtocol::read_meter(req, &unit.get_node(), &mut self.1, timeout_ms)
    }

    fn read_meter(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            IN_METER_NAME => {
                elem_value.set_int(&self.1.phys_inputs);
                Ok(true)
            }
            Self::METER_DIG_INPUT_DETECT_NAME => {
                elem_value.set_bool(&[self.1.dig_input_detect]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

const MODE_192_KHZ_NAME: &str = "mode-192khz";
const INPUT_PAIR_1_SRC_NAME: &str = "input-3/4-source";
const MIXER_MODE_NAME: &str = "mixer-mode";

fn input_pair_1_src_to_str(src: &SaffireInputPair1Source) -> &'static str {
    match src {
        SaffireInputPair1Source::AnalogInputPair0 => "analog-input-1/2",
        SaffireInputPair1Source::DigitalInputPair0 => "digital-input-1/2",
    }
}

fn mixer_mode_to_str(mode: &SaffireMixerMode) -> &'static str {
    match mode {
        SaffireMixerMode::StereoPaired => "stereo-paired",
        SaffireMixerMode::StereoSeparated => "stereo-separated",
    }
}

impl SpecificCtl {
    const INPUT_PAIR_1_SRCS: [SaffireInputPair1Source; 2] = [
        SaffireInputPair1Source::AnalogInputPair0,
        SaffireInputPair1Source::DigitalInputPair0,
    ];

    const MIXER_MODES: [SaffireMixerMode; 2] = [
        SaffireMixerMode::StereoPaired,
        SaffireMixerMode::StereoSeparated,
    ];

    fn load_params(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &SndUnit,
        req: &FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MODE_192_KHZ_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true).map(|_| ())?;

        let labels: Vec<&str> = Self::INPUT_PAIR_1_SRCS
            .iter()
            .map(|src| input_pair_1_src_to_str(src))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_PAIR_1_SRC_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|_| ())?;

        let labels: Vec<&str> = Self::MIXER_MODES
            .iter()
            .map(|mode| mixer_mode_to_str(mode))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|_| ())?;

        SaffireSpecificProtocol::read_params(req, &unit.get_node(), &mut self.0, timeout_ms)?;

        Ok(())
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MODE_192_KHZ_NAME => {
                elem_value.set_bool(&[self.0.mode_192khz]);
                Ok(true)
            }
            INPUT_PAIR_1_SRC_NAME => {
                let pos = Self::INPUT_PAIR_1_SRCS
                    .iter()
                    .position(|s| s.eq(&self.0.input_pair_1_src))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            MIXER_MODE_NAME => {
                let pos = Self::MIXER_MODES
                    .iter()
                    .position(|m| m.eq(&self.0.mixer_mode))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        separated_mixer_ctl: &mut SeparatedMixerCtl,
        paired_mixer_ctl: &mut PairedMixerCtl,
        unit: &SndUnit,
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MODE_192_KHZ_NAME => {
                let mut vals = [false];
                elem_value.get_bool(&mut vals);
                SaffireSpecificProtocol::write_192khz_mode(
                    req,
                    &unit.get_node(),
                    vals[0],
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            INPUT_PAIR_1_SRC_NAME => {
                let mut vals = [0];
                elem_value.get_enum(&mut vals);
                let &src = Self::INPUT_PAIR_1_SRCS
                    .iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index for source of input pair 1: {}", vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })?;
                SaffireSpecificProtocol::write_input_pair_1_src(
                    req,
                    &unit.get_node(),
                    src,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            MIXER_MODE_NAME => {
                let mut vals = [0];
                elem_value.get_enum(&mut vals);
                let &mode = Self::MIXER_MODES
                    .iter()
                    .nth(vals[0] as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index for mode of mixer: {}", vals[0]);
                        Error::new(FileError::Inval, &msg)
                    })?;
                SaffireSpecificProtocol::write_mixer_mode(
                    req,
                    &unit.get_node(),
                    mode,
                    &mut self.0,
                    timeout_ms,
                )?;
                if mode == SaffireMixerMode::StereoSeparated {
                    separated_mixer_ctl.write_state(unit, req, timeout_ms)?;
                } else {
                    paired_mixer_ctl.write_state(unit, req, timeout_ms)?;
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

const REVERB_AMOUNT_NAME: &str = "reverb-amount";
const REVERB_ROOM_SIZE_NAME: &str = "reverb-room-size";
const REVERB_DIFFUSION_NAME: &str = "reverb-diffusion";
const REVERB_TONE_NAME: &str = "reverb-tone";

impl ReverbCtl {
    fn load_params(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndUnit,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_AMOUNT_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            SaffireReverbProtocol::AMOUNT_MIN,
            SaffireReverbProtocol::AMOUNT_MAX,
            SaffireReverbProtocol::AMOUNT_STEP,
            2,
            None,
            false,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_ROOM_SIZE_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            SaffireReverbProtocol::ROOM_SIZE_MIN,
            SaffireReverbProtocol::ROOM_SIZE_MAX,
            SaffireReverbProtocol::ROOM_SIZE_STEP,
            2,
            None,
            false,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_DIFFUSION_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            SaffireReverbProtocol::DIFFUSION_MIN,
            SaffireReverbProtocol::DIFFUSION_MAX,
            SaffireReverbProtocol::DIFFUSION_STEP,
            2,
            None,
            false,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, REVERB_TONE_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            SaffireReverbProtocol::TONE_MIN,
            SaffireReverbProtocol::TONE_MAX,
            SaffireReverbProtocol::TONE_STEP,
            2,
            None,
            false,
        )?;

        SaffireReverbProtocol::read_params(req, &mut unit.get_node(), &mut self.0, timeout_ms)
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            REVERB_AMOUNT_NAME => {
                elem_value.set_int(&self.0.amounts);
                Ok(true)
            }
            REVERB_ROOM_SIZE_NAME => {
                elem_value.set_int(&self.0.room_sizes);
                Ok(true)
            }
            REVERB_DIFFUSION_NAME => {
                elem_value.set_int(&self.0.diffusions);
                Ok(true)
            }
            REVERB_TONE_NAME => {
                elem_value.set_int(&self.0.tones);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        unit: &mut SndUnit,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            REVERB_AMOUNT_NAME => {
                let mut vals = [0; 2];
                elem_value.get_int(&mut vals);
                SaffireReverbProtocol::write_amounts(
                    req,
                    &mut unit.get_node(),
                    &vals,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            REVERB_ROOM_SIZE_NAME => {
                let mut vals = [0; 2];
                elem_value.get_int(&mut vals);
                SaffireReverbProtocol::write_room_sizes(
                    req,
                    &mut unit.get_node(),
                    &vals,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            REVERB_DIFFUSION_NAME => {
                let mut vals = [0; 2];
                elem_value.get_int(&mut vals);
                SaffireReverbProtocol::write_diffusions(
                    req,
                    &mut unit.get_node(),
                    &vals,
                    &mut self.0,
                    timeout_ms,
                )
                .map(|_| true)
            }
            REVERB_TONE_NAME => {
                let mut vals = [0; 2];
                elem_value.get_int(&mut vals);
                SaffireReverbProtocol::write_tones(
                    req,
                    &mut unit.get_node(),
                    &vals,
                    &mut self.0,
                    timeout_ms,
                )
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
        let mut card_cntr = CardCntr::new();
        let mut ctl = ClkCtl::default();

        let error = ctl.load_freq(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let error = ctl.load_src(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }

    #[test]
    fn test_output_params_definition() {
        let mut card_cntr = CardCntr::new();
        let mut ctl = OutputCtl::default();
        let unit = SndUnit::default();
        let req = FwReq::default();

        let error = ctl
            .load_params(&mut card_cntr, &unit, &req, TIMEOUT_MS)
            .unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
