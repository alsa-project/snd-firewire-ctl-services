// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, protocols::tcelectronic::desktop::*};

#[derive(Default, Debug)]
pub struct Desktopk6Model {
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl<Desktopk6Protocol>,
    hw_state_ctl: HwStateCtl,
    config_ctl: ConfigCtl,
    mixer_ctl: MixerCtl,
    panel_ctl: PanelCtl,
    meter_ctl: MeterCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<(SndDice, FwNode)> for Desktopk6Model {
    fn cache(&mut self, (_, node): &mut (SndDice, FwNode)) -> Result<(), Error> {
        Desktopk6Protocol::read_general_sections(
            &mut self.req,
            node,
            &mut self.sections,
            TIMEOUT_MS,
        )?;

        self.common_ctl
            .cache_whole_params(&mut self.req, node, &mut self.sections, TIMEOUT_MS)?;
        self.hw_state_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.config_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.mixer_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.panel_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.meter_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr)?;

        self.hw_state_ctl.load(card_cntr)?;
        self.config_ctl.load(card_cntr)?;
        self.mixer_ctl.load(card_cntr)?;
        self.panel_ctl.load(card_cntr)?;
        self.meter_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_state_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.config_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.panel_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        (unit, node): &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.write(
            unit,
            &self.req,
            node,
            &mut self.sections,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .hw_state_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .config_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .panel_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for Desktopk6Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_id_list);
        elem_id_list.extend_from_slice(&self.hw_state_ctl.1);
        elem_id_list.extend_from_slice(&self.panel_ctl.1);
    }

    fn parse_notification(
        &mut self,
        (_, node): &mut (SndDice, FwNode),
        &msg: &u32,
    ) -> Result<(), Error> {
        self.common_ctl
            .parse_notification(&self.req, node, &mut self.sections, msg, TIMEOUT_MS)?;
        self.hw_state_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        self.config_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        self.panel_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;

        Ok(())
    }
}

impl MeasureModel<(SndDice, FwNode)> for Desktopk6Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.measured_elem_id_list);
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
    }

    fn measure_states(&mut self, (_, node): &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .cache_partial_params(&self.req, node, &mut self.sections, TIMEOUT_MS)?;
        self.meter_ctl.cache(&self.req, node, TIMEOUT_MS)?;
        Ok(())
    }
}

fn meter_target_to_str(target: &MeterTarget) -> &'static str {
    match target {
        MeterTarget::Input => "Input",
        MeterTarget::Pre => "Pre",
        MeterTarget::Post => "Post",
    }
}

fn input_scene_to_str(scene: &InputScene) -> &'static str {
    match scene {
        InputScene::MicInst => "Mic-inst",
        InputScene::DualInst => "Dual-inst",
        InputScene::StereoIn => "Stereo-in",
    }
}

#[derive(Default, Debug)]
struct HwStateCtl(Desktopk6HwStateSegment, Vec<ElemId>);

const METER_TARGET_NAME: &str = "meter-target";
const MIXER_OUT_MONAURAL_NAME: &str = "mixer-out-monaural";
const KNOB_ASSIGN_TO_HP_NAME: &str = "knob-assign-to-headphone";
const MIXER_OUTPUT_DIM_ENABLE_NAME: &str = "mixer-output-dim-enable";
const MIXER_OUTPUT_DIM_LEVEL_NAME: &str = "mixer-output-dim-level";
const SCENE_NAME: &str = "scene-select";
const REVERB_TO_MAIN_NAME: &str = "reverb-to-main";
const REVERB_TO_HP_NAME: &str = "reverb-to-hp";
const KNOB_BACKLIGHT_NAME: &str = "knob-backlight";
const MIC_0_PHANTOM_NAME: &str = "mic-1-phantom";
const MIC_0_BOOST_NAME: &str = "mic-1-boost";

impl HwStateCtl {
    const METER_TARGETS: [MeterTarget; 3] =
        [MeterTarget::Input, MeterTarget::Pre, MeterTarget::Post];

    const INPUT_SCENES: [InputScene; 3] = [
        InputScene::MicInst,
        InputScene::DualInst,
        InputScene::StereoIn,
    ];

    const DIM_LEVEL_MIN: i32 = -1000;
    const DIM_LEVEL_MAX: i32 = -60;
    const DIM_LEVEL_STEP: i32 = 1;
    const DIM_LEVEL_TLV: DbInterval = DbInterval {
        min: -9400,
        max: -600,
        linear: false,
        mute_avail: false,
    };

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = Desktopk6Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::METER_TARGETS
            .iter()
            .map(|l| meter_target_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, METER_TARGET_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MIXER_OUT_MONAURAL_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, KNOB_ASSIGN_TO_HP_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUTPUT_DIM_ENABLE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUTPUT_DIM_LEVEL_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::DIM_LEVEL_MIN,
            Self::DIM_LEVEL_MAX,
            Self::DIM_LEVEL_STEP,
            1,
            Some(&Into::<Vec<u32>>::into(Self::DIM_LEVEL_TLV)),
            true,
        )?;

        let labels: Vec<&str> = Self::INPUT_SCENES
            .iter()
            .map(|l| input_scene_to_str(l))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SCENE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, REVERB_TO_MAIN_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, REVERB_TO_HP_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, KNOB_BACKLIGHT_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MIC_0_PHANTOM_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MIC_0_BOOST_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            METER_TARGET_NAME => {
                let params = &self.0.data;
                let pos = Self::METER_TARGETS
                    .iter()
                    .position(|t| params.meter_target.eq(t))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            MIXER_OUT_MONAURAL_NAME => {
                let params = &self.0.data;
                elem_value.set_bool(&[params.mixer_output_monaural]);
                Ok(true)
            }
            KNOB_ASSIGN_TO_HP_NAME => {
                let params = &self.0.data;
                elem_value.set_bool(&[params.knob_assign_to_hp]);
                Ok(true)
            }
            MIXER_OUTPUT_DIM_ENABLE_NAME => {
                let params = &self.0.data;
                elem_value.set_bool(&[params.mixer_output_dim_enabled]);
                Ok(true)
            }
            MIXER_OUTPUT_DIM_LEVEL_NAME => {
                let params = &self.0.data;
                elem_value.set_int(&[params.mixer_output_dim_volume]);
                Ok(true)
            }
            SCENE_NAME => {
                let params = &self.0.data;
                let pos = Self::INPUT_SCENES
                    .iter()
                    .position(|s| params.input_scene.eq(s))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            REVERB_TO_MAIN_NAME => {
                let params = &self.0.data;
                elem_value.set_bool(&[params.reverb_to_master]);
                Ok(true)
            }
            REVERB_TO_HP_NAME => {
                let params = &self.0.data;
                elem_value.set_bool(&[params.reverb_to_hp]);
                Ok(true)
            }
            KNOB_BACKLIGHT_NAME => {
                let params = &self.0.data;
                elem_value.set_bool(&[params.master_knob_backlight]);
                Ok(true)
            }
            MIC_0_PHANTOM_NAME => {
                let params = &self.0.data;
                elem_value.set_bool(&[params.mic_0_phantom]);
                Ok(true)
            }
            MIC_0_BOOST_NAME => {
                let params = &self.0.data;
                elem_value.set_bool(&[params.mic_0_boost]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            METER_TARGET_NAME => {
                let mut params = self.0.data.clone();
                let val = elem_value.enumerated()[0] as usize;
                params.meter_target = Self::METER_TARGETS
                    .iter()
                    .nth(val)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of meter target: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            MIXER_OUT_MONAURAL_NAME => {
                let mut params = self.0.data.clone();
                params.mixer_output_monaural = elem_value.boolean()[0];
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            KNOB_ASSIGN_TO_HP_NAME => {
                let mut params = self.0.data.clone();
                params.knob_assign_to_hp = elem_value.boolean()[0];
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            MIXER_OUTPUT_DIM_ENABLE_NAME => {
                let mut params = self.0.data.clone();
                params.mixer_output_dim_enabled = elem_value.boolean()[0];
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            MIXER_OUTPUT_DIM_LEVEL_NAME => {
                let mut params = self.0.data.clone();
                params.mixer_output_dim_volume = elem_value.int()[0];
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            SCENE_NAME => {
                let mut params = self.0.data.clone();
                let val = elem_value.enumerated()[0] as usize;
                params.input_scene = Self::INPUT_SCENES
                    .iter()
                    .nth(val)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of input scene: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            REVERB_TO_MAIN_NAME => {
                let mut params = self.0.data.clone();
                params.reverb_to_master = elem_value.boolean()[0];
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            REVERB_TO_HP_NAME => {
                let mut params = self.0.data.clone();
                params.reverb_to_hp = elem_value.boolean()[0];
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            KNOB_BACKLIGHT_NAME => {
                let mut params = self.0.data.clone();
                params.master_knob_backlight = elem_value.boolean()[0];
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            MIC_0_PHANTOM_NAME => {
                let mut params = self.0.data.clone();
                params.mic_0_phantom = elem_value.boolean()[0];
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            MIC_0_BOOST_NAME => {
                let mut params = self.0.data.clone();
                params.mic_0_boost = elem_value.boolean()[0];
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn parse_notification(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if Desktopk6Protocol::is_notified_segment(&self.0, msg) {
            let res = Desktopk6Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct ConfigCtl(Desktopk6ConfigSegment, Vec<ElemId>);

impl ConfigCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = Desktopk6Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        load_standalone_rate::<Desktopk6Protocol, DesktopConfig>(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        read_standalone_rate::<Desktopk6Protocol, DesktopConfig>(&self.0, elem_id, elem_value)
    }

    fn write(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        write_standalone_rate::<Desktopk6Protocol, DesktopConfig>(
            &mut self.0,
            req,
            node,
            elem_id,
            elem_value,
            timeout_ms,
        )
    }

    fn parse_notification(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if Desktopk6Protocol::is_notified_segment(&self.0, msg) {
            let res = Desktopk6Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct MixerCtl(Desktopk6MixerStateSegment);

const MIXER_MIC_INST_SRC_LEVEL_NAME: &str = "mixer-mic-inst-source-level";
const MIXER_MIC_INST_SRC_BALANCE_NAME: &str = "mixer-mic-inst-source-pan";
const MIXER_MIC_INST_SRC_SEND_NAME: &str = "mixer-mic-inst-source-send";

const MIXER_DUAL_INST_SRC_LEVEL_NAME: &str = "mixer-dual-inst-source-level";
const MIXER_DUAL_INST_SRC_BALANCE_NAME: &str = "mixer-dual-inst-source-pan";
const MIXER_DUAL_INST_SRC_SEND_NAME: &str = "mixer-dual-inst-source-send";

const MIXER_STEREO_IN_SRC_LEVEL_NAME: &str = "mixer-stereo-input-source-level";
const MIXER_STEREO_IN_SRC_BALANCE_NAME: &str = "mixer-stereo-input-source-pan";
const MIXER_STEREO_IN_SRC_SEND_NAME: &str = "mixer-stereo-input-source-send";

const HP_SRC_NAME: &str = "headphone-source";

fn hp_src_to_str(src: &DesktopHpSrc) -> &'static str {
    match src {
        DesktopHpSrc::Stream23 => "Stream-3/4",
        DesktopHpSrc::Mixer01 => "Mixer-out-1/2",
    }
}
impl MixerCtl {
    const LEVEL_MIN: i32 = -1000;
    const LEVEL_MAX: i32 = 0;
    const LEVEL_STEP: i32 = 1;
    const LEVEL_TLV: DbInterval = DbInterval {
        min: -9400,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    const BALANCE_MIN: i32 = -50;
    const BALANCE_MAX: i32 = 50;
    const BALANCE_STEP: i32 = 1;

    const HP_SRCS: [DesktopHpSrc; 2] = [DesktopHpSrc::Stream23, DesktopHpSrc::Mixer01];

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = Desktopk6Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_MIC_INST_SRC_LEVEL_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::LEVEL_MIN,
            Self::LEVEL_MAX,
            Self::LEVEL_STEP,
            2,
            Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
            true,
        )?;
        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            MIXER_MIC_INST_SRC_BALANCE_NAME,
            0,
        );
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::BALANCE_MIN,
            Self::BALANCE_MAX,
            Self::BALANCE_STEP,
            2,
            None,
            true,
        )?;
        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_MIC_INST_SRC_SEND_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::LEVEL_MIN,
            Self::LEVEL_MAX,
            Self::LEVEL_STEP,
            2,
            Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            MIXER_DUAL_INST_SRC_LEVEL_NAME,
            0,
        );
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::LEVEL_MIN,
            Self::LEVEL_MAX,
            Self::LEVEL_STEP,
            2,
            Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
            true,
        )?;
        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            MIXER_DUAL_INST_SRC_BALANCE_NAME,
            0,
        );
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::BALANCE_MIN,
            Self::BALANCE_MAX,
            Self::BALANCE_STEP,
            2,
            None,
            true,
        )?;
        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_DUAL_INST_SRC_SEND_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::LEVEL_MIN,
            Self::LEVEL_MAX,
            Self::LEVEL_STEP,
            2,
            Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            MIXER_STEREO_IN_SRC_LEVEL_NAME,
            0,
        );
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::LEVEL_MIN,
            Self::LEVEL_MAX,
            Self::LEVEL_STEP,
            1,
            Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
            true,
        )?;
        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            MIXER_STEREO_IN_SRC_BALANCE_NAME,
            0,
        );
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::BALANCE_MIN,
            Self::BALANCE_MAX,
            Self::BALANCE_STEP,
            1,
            None,
            true,
        )?;
        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_STEREO_IN_SRC_SEND_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::LEVEL_MIN,
            Self::LEVEL_MAX,
            Self::LEVEL_STEP,
            1,
            Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
            true,
        )?;

        let labels: Vec<&str> = Self::HP_SRCS.iter().map(|s| hp_src_to_str(s)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, HP_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_MIC_INST_SRC_LEVEL_NAME => {
                let params = &self.0.data;
                elem_value.set_int(&params.mic_inst_level);
                Ok(true)
            }
            MIXER_MIC_INST_SRC_BALANCE_NAME => {
                let params = &self.0.data;
                elem_value.set_int(&params.mic_inst_pan);
                Ok(true)
            }
            MIXER_MIC_INST_SRC_SEND_NAME => {
                let params = &self.0.data;
                elem_value.set_int(&params.mic_inst_send);
                Ok(true)
            }
            MIXER_DUAL_INST_SRC_LEVEL_NAME => {
                let params = &self.0.data;
                elem_value.set_int(&params.dual_inst_level);
                Ok(true)
            }
            MIXER_DUAL_INST_SRC_BALANCE_NAME => {
                let params = &self.0.data;
                elem_value.set_int(&params.dual_inst_pan);
                Ok(true)
            }
            MIXER_DUAL_INST_SRC_SEND_NAME => {
                let params = &self.0.data;
                elem_value.set_int(&params.dual_inst_send);
                Ok(true)
            }
            MIXER_STEREO_IN_SRC_LEVEL_NAME => {
                let params = &self.0.data;
                elem_value.set_int(&[params.stereo_in_level]);
                Ok(true)
            }
            MIXER_STEREO_IN_SRC_BALANCE_NAME => {
                let params = &self.0.data;
                elem_value.set_int(&[params.stereo_in_pan]);
                Ok(true)
            }
            MIXER_STEREO_IN_SRC_SEND_NAME => {
                let params = &self.0.data;
                elem_value.set_int(&[params.stereo_in_send]);
                Ok(true)
            }
            HP_SRC_NAME => {
                let params = &self.0.data;
                let pos = Self::HP_SRCS
                    .iter()
                    .position(|s| params.hp_src.eq(s))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_MIC_INST_SRC_LEVEL_NAME => {
                let mut params = self.0.data.clone();
                let levels = &mut params.mic_inst_level;
                let vals = &elem_value.int()[..levels.len()];
                levels.copy_from_slice(&vals);
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIXER_MIC_INST_SRC_BALANCE_NAME => {
                let mut params = self.0.data.clone();
                let pans = &mut params.mic_inst_pan;
                let vals = &elem_value.int()[..pans.len()];
                pans.copy_from_slice(&vals);
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIXER_MIC_INST_SRC_SEND_NAME => {
                let mut params = self.0.data.clone();
                let sends = &mut params.mic_inst_send;
                let vals = &elem_value.int()[..sends.len()];
                sends.copy_from_slice(&vals);
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIXER_DUAL_INST_SRC_LEVEL_NAME => {
                let mut params = self.0.data.clone();
                let levels = &mut params.mic_inst_level;
                let vals = &elem_value.int()[..levels.len()];
                levels.copy_from_slice(&vals);
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIXER_DUAL_INST_SRC_BALANCE_NAME => {
                let mut params = self.0.data.clone();
                let pans = &mut params.mic_inst_pan;
                let vals = &elem_value.int()[..pans.len()];
                pans.copy_from_slice(&vals);
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIXER_DUAL_INST_SRC_SEND_NAME => {
                let mut params = self.0.data.clone();
                let sends = &mut params.mic_inst_send;
                let vals = &elem_value.int()[..sends.len()];
                sends.copy_from_slice(&vals);
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIXER_STEREO_IN_SRC_LEVEL_NAME => {
                let mut params = self.0.data.clone();
                params.stereo_in_level = elem_value.int()[0];
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIXER_STEREO_IN_SRC_BALANCE_NAME => {
                let mut params = self.0.data.clone();
                params.stereo_in_pan = elem_value.int()[0];
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            MIXER_STEREO_IN_SRC_SEND_NAME => {
                let mut params = self.0.data.clone();
                params.stereo_in_send = elem_value.int()[0];
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            HP_SRC_NAME => {
                let mut params = self.0.data.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.hp_src = Self::HP_SRCS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of headphone source: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = Desktopk6Protocol::update_partial_segment(
                    req,
                    &node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct PanelCtl(Desktopk6PanelSegment, Vec<ElemId>);

const PANEL_BUTTON_COUNT_NAME: &str = "panel-button-count";
const MIXER_OUT_VOL: &str = "mixer-output-volume";
const PHONE_KNOB_VALUE_NAME: &str = "phone-knob-value";
const MIX_KNOB_VALUE_NAME: &str = "mix-knob-value";
const REVERB_LED_STATE_NAME: &str = "reverb-led-state";
const REVERB_KNOB_VALUE_NAME: &str = "reverb-knob-value";

impl PanelCtl {
    const KNOB_MIN: i32 = -1000;
    const KNOB_MAX: i32 = 0;
    const KNOB_STEP: i32 = 1;

    const MIX_MIN: i32 = 0;
    const MIX_MAX: i32 = 1000;
    const MIX_STEP: i32 = 1;

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = Desktopk6Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        load_firewire_led::<Desktopk6Protocol, DesktopPanel>(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, PANEL_BUTTON_COUNT_NAME, 0);
        card_cntr
            .add_int_elems(&elem_id, 1, 0, i32::MAX, 1, 1, None, false)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_OUT_VOL, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::KNOB_MIN,
                Self::KNOB_MAX,
                Self::KNOB_STEP,
                1,
                None,
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, PHONE_KNOB_VALUE_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::KNOB_MIN,
                Self::KNOB_MAX,
                Self::KNOB_STEP,
                1,
                None,
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MIX_KNOB_VALUE_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::MIX_MIN,
                Self::MIX_MAX,
                Self::MIX_STEP,
                1,
                None,
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, REVERB_LED_STATE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, REVERB_KNOB_VALUE_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::KNOB_MIN,
                Self::KNOB_MAX,
                Self::KNOB_STEP,
                1,
                None,
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if read_firewire_led::<Desktopk6Protocol, DesktopPanel>(&self.0, elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                PANEL_BUTTON_COUNT_NAME => {
                    let params = &self.0.data;
                    elem_value.set_int(&[params.panel_button_count as i32]);
                    Ok(true)
                }
                .map(|_| true),
                MIXER_OUT_VOL => {
                    let params = &self.0.data;
                    elem_value.set_int(&[params.main_knob_value]);
                    Ok(true)
                }
                .map(|_| true),
                PHONE_KNOB_VALUE_NAME => {
                    let params = &self.0.data;
                    elem_value.set_int(&[params.phone_knob_value]);
                    Ok(true)
                }
                .map(|_| true),
                MIX_KNOB_VALUE_NAME => {
                    let params = &self.0.data;
                    elem_value.set_int(&[params.mix_knob_value as i32]);
                    Ok(true)
                }
                .map(|_| true),
                REVERB_LED_STATE_NAME => {
                    let params = &self.0.data;
                    elem_value.set_bool(&[params.reverb_led_on]);
                    Ok(true)
                }
                REVERB_KNOB_VALUE_NAME => {
                    let params = &self.0.data;
                    elem_value.set_int(&[params.reverb_knob_value]);
                    Ok(true)
                }
                _ => Ok(false),
            }
        }
    }

    fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if write_firewire_led::<Desktopk6Protocol, DesktopPanel>(
            &mut self.0,
            req,
            node,
            elem_id,
            elem_value,
            timeout_ms,
        )? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                REVERB_LED_STATE_NAME => {
                    let mut params = self.0.data.clone();
                    params.reverb_led_on = elem_value.boolean()[0];
                    let res = Desktopk6Protocol::update_partial_segment(
                        req,
                        node,
                        &params,
                        &mut self.0,
                        timeout_ms,
                    );
                    debug!(params = ?self.0.data, ?res);
                    res.map(|_| true)
                }
                _ => Ok(false),
            }
        }
    }

    fn parse_notification(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if Desktopk6Protocol::is_notified_segment(&self.0, msg) {
            let res = Desktopk6Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct MeterCtl(Desktopk6MeterSegment, Vec<ElemId>);

const ANALOG_IN_NAME: &str = "analog-input-meters";
const MIXER_OUT_NAME: &str = "mixer-output-meters";
const STREAM_IN_NAME: &str = "stream-input-meters";

impl MeterCtl {
    const METER_MIN: i32 = -1000;
    const METER_MAX: i32 = 0;
    const METER_STEP: i32 = 1;
    const METER_TLV: DbInterval = DbInterval {
        min: -9400,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = Desktopk6Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels = (0..self.0.data.analog_inputs.len())
            .map(|i| format!("Analog-input-{}", i))
            .collect::<Vec<_>>();
        Self::add_meter_elem(card_cntr, ANALOG_IN_NAME, &labels, &mut self.1)?;

        let labels = (0..self.0.data.mixer_outputs.len())
            .map(|i| format!("Mixer-output-{}", i))
            .collect::<Vec<_>>();
        Self::add_meter_elem(card_cntr, MIXER_OUT_NAME, &labels, &mut self.1)?;

        let labels = (0..self.0.data.stream_inputs.len())
            .map(|i| format!("Stream-input-{}", i))
            .collect::<Vec<_>>();
        Self::add_meter_elem(card_cntr, STREAM_IN_NAME, &labels, &mut self.1)?;

        Ok(())
    }

    fn add_meter_elem<T: AsRef<str>>(
        card_cntr: &mut CardCntr,
        name: &str,
        labels: &[T],
        measured_elem_id_list: &mut Vec<ElemId>,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::METER_MIN,
                Self::METER_MAX,
                Self::METER_STEP,
                labels.len(),
                Some(&Into::<Vec<u32>>::into(Self::METER_TLV)),
                false,
            )
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ANALOG_IN_NAME => {
                let params = &self.0.data;
                elem_value.set_int(&params.analog_inputs);
                Ok(true)
            }
            MIXER_OUT_NAME => {
                let params = &self.0.data;
                elem_value.set_int(&params.mixer_outputs);
                Ok(true)
            }
            STREAM_IN_NAME => {
                let params = &self.0.data;
                elem_value.set_int(&params.stream_inputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
