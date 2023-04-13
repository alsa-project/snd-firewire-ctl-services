// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use super::*;

#[derive(Default, Debug)]
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

#[derive(Default, Debug)]
struct ClkCtl(Vec<ElemId>, MediaClockParameters, SamplingClockParameters);

impl MediaClkFreqCtlOperation<SaffireClkProtocol> for ClkCtl {
    fn state(&self) -> &MediaClockParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut MediaClockParameters {
        &mut self.1
    }
}

impl SamplingClkSrcCtlOperation<SaffireClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal", "S/PDIF"];

    fn state(&self) -> &SamplingClockParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut SamplingClockParameters {
        &mut self.2
    }
}

#[derive(Default, Debug)]
struct MeterCtl(Vec<ElemId>, SaffireMeter);

#[derive(Debug)]
struct OutputCtl(Vec<ElemId>, SaffireOutputParameters);

impl Default for OutputCtl {
    fn default() -> Self {
        Self(
            Default::default(),
            SaffireOutputProtocol::create_output_parameters(),
        )
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

    fn state(&self) -> &SaffireOutputParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut SaffireOutputParameters {
        &mut self.1
    }
}

#[derive(Default, Debug)]
struct SpecificCtl(SaffireSpecificParameters);

#[derive(Debug)]
struct SeparatedMixerCtl(Vec<ElemId>, SaffireMixerState);

impl Default for SeparatedMixerCtl {
    fn default() -> Self {
        Self(
            Default::default(),
            SaffireSeparatedMixerProtocol::create_mixer_state(),
        )
    }
}

impl SaffireMixerCtlOperation<SaffireSeparatedMixerProtocol> for SeparatedMixerCtl {
    const PHYS_INPUT_GAIN_NAME: &'static str = "mixer:separated:phys-input-gain";
    const REVERB_RETURN_GAIN_NAME: &'static str = "mixer:separated:reverb-return-gain";
    const STREAM_SRC_GAIN_NAME: &'static str = "mixer:separated:stream-source-gain";

    const MIXER_MODE: SaffireMixerMode = SaffireMixerMode::StereoSeparated;

    fn state(&self) -> &SaffireMixerState {
        &self.1
    }

    fn state_mut(&mut self) -> &mut SaffireMixerState {
        &mut self.1
    }
}

#[derive(Debug)]
struct PairedMixerCtl(Vec<ElemId>, SaffireMixerState);

impl Default for PairedMixerCtl {
    fn default() -> Self {
        Self(
            Default::default(),
            SaffirePairedMixerProtocol::create_mixer_state(),
        )
    }
}

impl SaffireMixerCtlOperation<SaffirePairedMixerProtocol> for PairedMixerCtl {
    const PHYS_INPUT_GAIN_NAME: &'static str = "mixer:paired:phys-input-gain";
    const REVERB_RETURN_GAIN_NAME: &'static str = "mixer:paired:reverb-return-gain";
    const STREAM_SRC_GAIN_NAME: &'static str = "mixer:paired:stream-source-gain";

    const MIXER_MODE: SaffireMixerMode = SaffireMixerMode::StereoPaired;

    fn state(&self) -> &SaffireMixerState {
        &self.1
    }

    fn state_mut(&mut self) -> &mut SaffireMixerState {
        &mut self.1
    }
}

#[derive(Default, Debug)]
struct ReverbCtl(SaffireReverbParameters);

impl CtlModel<(SndUnit, FwNode)> for SaffireModel {
    fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.clk_ctl.cache_freq(&self.avc, FCP_TIMEOUT_MS)?;
        self.clk_ctl.cache_src(&self.avc, FCP_TIMEOUT_MS)?;
        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.out_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.specific_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.separated_mixer_ctl
            .cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.paired_mixer_ctl
            .cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.reverb_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;

        Ok(())
    }
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctl
            .load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl
            .load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.meter_ctl
            .load_meter(card_cntr)
            .map(|mut elem_id_list| self.meter_ctl.0.append(&mut elem_id_list))?;

        self.out_ctl
            .load_params(card_cntr)
            .map(|mut elem_id_list| self.out_ctl.0.append(&mut elem_id_list))?;

        self.specific_ctl.load_params(card_cntr)?;

        self.separated_mixer_ctl
            .load_src_levels(card_cntr)
            .map(|measured_elem_id_list| self.separated_mixer_ctl.0 = measured_elem_id_list)?;

        self.paired_mixer_ctl
            .load_src_levels(card_cntr)
            .map(|measured_elem_id_list| self.paired_mixer_ctl.0 = measured_elem_id_list)?;

        self.reverb_ctl.load_params(card_cntr)?;

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
        unit: &mut (SndUnit, FwNode),
        elem_id: &ElemId,
        _: &ElemValue,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.write_freq(
            &mut unit.0,
            &self.avc,
            elem_id,
            elem_value,
            FCP_TIMEOUT_MS * 3,
        )? {
            Ok(true)
        } else if self.clk_ctl.write_src(
            &mut unit.0,
            &self.avc,
            elem_id,
            elem_value,
            FCP_TIMEOUT_MS * 3,
        )? {
            Ok(true)
        } else if self
            .out_ctl
            .write_params(&self.req, &unit.1, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.specific_ctl.write_params(
            &self.req,
            &unit.1,
            elem_id,
            elem_value,
            &mut self.separated_mixer_ctl.1,
            &mut self.paired_mixer_ctl.1,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.separated_mixer_ctl.write_src_levels(
            self.specific_ctl.0.mixer_mode,
            unit,
            &self.req,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.paired_mixer_ctl.write_src_levels(
            self.specific_ctl.0.mixer_mode,
            unit,
            &self.req,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.reverb_ctl.write_params(
            unit,
            &mut self.req,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndUnit, FwNode), bool> for SaffireModel {
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
}

impl MeasureModel<(SndUnit, FwNode)> for SaffireModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
        elem_id_list.extend_from_slice(&self.out_ctl.0);
        elem_id_list.extend_from_slice(&self.separated_mixer_ctl.0);
        elem_id_list.extend_from_slice(&self.paired_mixer_ctl.0);
    }

    fn measure_states(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.out_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        Ok(())
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

    fn load_meter(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
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

        Ok(measured_elem_id_list)
    }

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = SaffireMeterProtocol::cache(req, node, &mut self.1, timeout_ms);
        debug!(params = ?self.1, ?res);
        res
    }

    fn read_meter(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
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

    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
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

        Ok(())
    }

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = SaffireSpecificProtocol::cache(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
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
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        separated_params: &mut SaffireMixerState,
        paired_params: &mut SaffireMixerState,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MODE_192_KHZ_NAME => {
                let mut params = self.0.clone();
                params.mode_192khz = elem_value.boolean()[0];
                let res =
                    SaffireSpecificProtocol::update(req, node, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            INPUT_PAIR_1_SRC_NAME => {
                let mut params = self.0.clone();
                let val = elem_value.enumerated()[0];
                params.input_pair_1_src = Self::INPUT_PAIR_1_SRCS
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index for source of input pair 1: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res =
                    SaffireSpecificProtocol::update(req, node, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            MIXER_MODE_NAME => {
                let mut params = self.0.clone();
                let val = elem_value.enumerated()[0];
                params.mixer_mode = Self::MIXER_MODES
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index for mode of mixer: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res =
                    SaffireSpecificProtocol::update(req, node, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.and_then(|_| {
                    if params.mixer_mode == SaffireMixerMode::StereoSeparated {
                        SaffireSeparatedMixerProtocol::cache(
                            req,
                            node,
                            separated_params,
                            timeout_ms,
                        )
                    } else {
                        SaffirePairedMixerProtocol::cache(req, node, paired_params, timeout_ms)
                    }
                })
                .map(|_| true)
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
    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
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

        Ok(())
    }

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = SaffireReverbProtocol::cache(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
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
        unit: &mut (SndUnit, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            REVERB_AMOUNT_NAME => {
                let mut params = self.0.clone();
                let amounts = &mut params.amounts;
                let vals = &elem_value.int()[..amounts.len()];
                amounts.copy_from_slice(&vals);
                let res = SaffireReverbProtocol::update(
                    req,
                    &mut unit.1,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            REVERB_ROOM_SIZE_NAME => {
                let mut params = self.0.clone();
                let room_sizes = &mut params.room_sizes;
                let vals = &elem_value.int()[..room_sizes.len()];
                room_sizes.copy_from_slice(&vals);
                let res = SaffireReverbProtocol::update(
                    req,
                    &mut unit.1,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            REVERB_DIFFUSION_NAME => {
                let mut params = self.0.clone();
                let diffusion = &mut params.room_sizes;
                let vals = &elem_value.int()[..diffusion.len()];
                diffusion.copy_from_slice(&vals);
                let res = SaffireReverbProtocol::update(
                    req,
                    &mut unit.1,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            REVERB_TONE_NAME => {
                let mut params = self.0.clone();
                let tones = &mut params.room_sizes;
                let vals = &elem_value.int()[..tones.len()];
                tones.copy_from_slice(&vals);
                let res = SaffireReverbProtocol::update(
                    req,
                    &mut unit.1,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

trait SaffireMixerCtlOperation<T: SaffireMixerOperation> {
    const PHYS_INPUT_GAIN_NAME: &'static str;
    const REVERB_RETURN_GAIN_NAME: &'static str;
    const STREAM_SRC_GAIN_NAME: &'static str;

    const MIXER_MODE: SaffireMixerMode;

    fn state(&self) -> &SaffireMixerState;
    fn state_mut(&mut self) -> &mut SaffireMixerState;

    fn load_src_levels(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let mut measured_elem_id_list = Vec::new();

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::PHYS_INPUT_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                T::OUTPUT_PAIR_COUNT,
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                T::PHYS_INPUT_COUNT,
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::REVERB_RETURN_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                T::OUTPUT_PAIR_COUNT,
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                T::REVERB_RETURN_COUNT,
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::STREAM_SRC_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                T::OUTPUT_PAIR_COUNT,
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                T::STREAM_INPUT_COUNT,
                Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                true,
            )
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        Ok(measured_elem_id_list)
    }

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache(req, node, self.state_mut(), timeout_ms);
        debug!(params = ?self.state(), ?res);
        res
    }

    fn read_src_levels(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        let name = elem_id.name();

        if name.as_str() == Self::PHYS_INPUT_GAIN_NAME {
            read_mixer_src_levels(elem_value, elem_id, &self.state().phys_inputs)
        } else if name.as_str() == Self::REVERB_RETURN_GAIN_NAME {
            read_mixer_src_levels(elem_value, elem_id, &self.state().reverb_returns)
        } else if name.as_str() == Self::STREAM_SRC_GAIN_NAME {
            read_mixer_src_levels(elem_value, elem_id, &self.state().stream_inputs)
        } else {
            Ok(false)
        }
    }

    fn write_src_levels(
        &mut self,
        mixer_mode: SaffireMixerMode,
        unit: &(SndUnit, FwNode),
        req: &FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        let name = &elem_id.name();

        if name.as_str() == Self::PHYS_INPUT_GAIN_NAME {
            if Self::MIXER_MODE != mixer_mode {
                Err(Error::new(
                    FileError::Inval,
                    "Not available at current mixer mode",
                ))
            } else {
                let mut params = self.state().clone();
                let index = elem_id.index() as usize;
                let levels = &mut params.phys_inputs[index];
                let vals = &elem_value.int()[..levels.len()];
                levels
                    .iter_mut()
                    .zip(vals)
                    .for_each(|(level, &val)| *level = val as i16);
                let res = T::update(req, &unit.1, &params, self.state_mut(), timeout_ms);
                debug!(params = ?self.state(), ?res);
                res.map(|_| true)
            }
        } else if name.as_str() == Self::REVERB_RETURN_GAIN_NAME {
            if Self::MIXER_MODE != mixer_mode {
                Err(Error::new(
                    FileError::Inval,
                    "Not available at current mixer mode",
                ))
            } else {
                let mut params = self.state().clone();
                let index = elem_id.index() as usize;
                let levels = &mut params.reverb_returns[index];
                let vals = &elem_value.int()[..levels.len()];
                levels
                    .iter_mut()
                    .zip(vals)
                    .for_each(|(level, &val)| *level = val as i16);
                let res = T::update(req, &unit.1, &params, self.state_mut(), timeout_ms);
                debug!(params = ?self.state(), ?res);
                res.map(|_| true)
            }
        } else if name.as_str() == Self::STREAM_SRC_GAIN_NAME {
            if Self::MIXER_MODE != mixer_mode {
                Err(Error::new(
                    FileError::Inval,
                    "Not available at current mixer mode",
                ))
            } else {
                let mut params = self.state().clone();
                let index = elem_id.index() as usize;
                let levels = &mut params.stream_inputs[index];
                let vals = &elem_value.int()[..levels.len()];
                levels
                    .iter_mut()
                    .zip(vals)
                    .for_each(|(level, &val)| *level = val as i16);
                let res = T::update(req, &unit.1, &params, self.state_mut(), timeout_ms);
                debug!(params = ?self.state(), ?res);
                res.map(|_| true)
            }
        } else {
            Ok(false)
        }
    }
}

fn read_mixer_src_levels(
    elem_value: &mut ElemValue,
    elem_id: &ElemId,
    levels_list: &[Vec<i16>],
) -> Result<bool, Error> {
    let index = elem_id.index() as usize;
    levels_list
        .iter()
        .nth(index)
        .ok_or_else(|| {
            let msg = format!("Invalid index of source level list {}", index);
            Error::new(FileError::Inval, &msg)
        })
        .map(|levels| {
            let vals: Vec<i32> = levels.iter().fold(Vec::new(), |mut vals, &level| {
                vals.push(level as i32);
                vals
            });
            elem_value.set_int(&vals);
            true
        })
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
