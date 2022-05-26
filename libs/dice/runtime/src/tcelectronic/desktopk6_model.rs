// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, dice_protocols::tcelectronic::desktop::*};

#[derive(Default)]
pub struct Desktopk6Model {
    req: FwReq,
    sections: GeneralSections,
    ctl: CommonCtl,
    hw_state_ctl: HwStateCtl,
    config_ctl: ConfigCtl,
    mixer_ctl: MixerCtl,
    panel_ctl: PanelCtl,
    meter_ctl: MeterCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<SndDice> for Desktopk6Model {
    fn load(&mut self, unit: &mut SndDice, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let mut node = unit.get_node();

        self.sections =
            GeneralProtocol::read_general_sections(&mut self.req, &mut node, TIMEOUT_MS)?;
        let caps = GlobalSectionProtocol::read_clock_caps(
            &mut self.req,
            &mut node,
            &self.sections,
            TIMEOUT_MS,
        )?;
        let src_labels = GlobalSectionProtocol::read_clock_source_labels(
            &mut self.req,
            &mut node,
            &self.sections,
            TIMEOUT_MS,
        )?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.hw_state_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.config_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.mixer_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.panel_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.meter_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;

        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut SndDice,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.ctl.read(
            unit,
            &mut self.req,
            &self.sections,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
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
        unit: &mut SndDice,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.ctl.write(
            unit,
            &mut self.req,
            &self.sections,
            elem_id,
            old,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .hw_state_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .config_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_ctl
            .write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .panel_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndDice, u32> for Desktopk6Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.hw_state_ctl.1);
        elem_id_list.extend_from_slice(&self.panel_ctl.1);
    }

    fn parse_notification(&mut self, unit: &mut SndDice, msg: &u32) -> Result<(), Error> {
        self.ctl
            .parse_notification(unit, &mut self.req, &self.sections, *msg, TIMEOUT_MS)?;

        self.hw_state_ctl
            .parse_notification(unit, &mut self.req, TIMEOUT_MS, *msg)?;
        self.config_ctl
            .parse_notification(unit, &mut self.req, TIMEOUT_MS, *msg)?;
        self.panel_ctl
            .parse_notification(unit, &mut self.req, TIMEOUT_MS, *msg)?;

        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndDice,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_state_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.panel_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<SndDice> for Desktopk6Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut SndDice) -> Result<(), Error> {
        self.ctl
            .measure_states(unit, &mut self.req, &self.sections, TIMEOUT_MS)?;
        self.meter_ctl
            .measure_states(unit, &mut self.req, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &SndDice,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
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

#[derive(Default)]
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

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Desktopk6Protocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)?;

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
        match elem_id.get_name().as_str() {
            METER_TARGET_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let pos = Self::METER_TARGETS
                    .iter()
                    .position(|&t| t == self.0.data.meter_target)
                    .unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            MIXER_OUT_MONAURAL_NAME => ElemValueAccessor::<bool>::set_val(elem_value, || {
                Ok(self.0.data.mixer_output_monaural)
            })
            .map(|_| true),
            KNOB_ASSIGN_TO_HP_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || Ok(self.0.data.knob_assign_to_hp))
                    .map(|_| true)
            }
            MIXER_OUTPUT_DIM_ENABLE_NAME => ElemValueAccessor::<bool>::set_val(elem_value, || {
                Ok(self.0.data.mixer_output_dim_enabled)
            })
            .map(|_| true),
            MIXER_OUTPUT_DIM_LEVEL_NAME => ElemValueAccessor::<i32>::set_val(elem_value, || {
                Ok(self.0.data.mixer_output_dim_volume)
            })
            .map(|_| true),
            SCENE_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let pos = Self::INPUT_SCENES
                    .iter()
                    .position(|&s| s == self.0.data.input_scene)
                    .unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            REVERB_TO_MAIN_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || Ok(self.0.data.reverb_to_master))
                    .map(|_| true)
            }
            REVERB_TO_HP_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || Ok(self.0.data.reverb_to_hp))
                    .map(|_| true)
            }
            KNOB_BACKLIGHT_NAME => ElemValueAccessor::<bool>::set_val(elem_value, || {
                Ok(self.0.data.master_knob_backlight)
            })
            .map(|_| true),
            MIC_0_PHANTOM_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || Ok(self.0.data.mic_0_phantom))
                    .map(|_| true)
            }
            MIC_0_BOOST_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || Ok(self.0.data.mic_0_boost))
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            METER_TARGET_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::METER_TARGETS
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of meter target: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&target| self.0.data.meter_target = target)
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MIXER_OUT_MONAURAL_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    self.0.data.mixer_output_monaural = val;
                    Ok(())
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            KNOB_ASSIGN_TO_HP_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    self.0.data.knob_assign_to_hp = val;
                    Ok(())
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MIXER_OUTPUT_DIM_ENABLE_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    self.0.data.mixer_output_dim_enabled = val;
                    Ok(())
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MIXER_OUTPUT_DIM_LEVEL_NAME => {
                ElemValueAccessor::<i32>::get_val(elem_value, |val| {
                    self.0.data.mixer_output_dim_volume = val;
                    Ok(())
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            SCENE_NAME => {
                ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                    Self::INPUT_SCENES
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of input scene: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&scene| self.0.data.input_scene = scene)
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            REVERB_TO_MAIN_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    self.0.data.reverb_to_master = val;
                    Ok(())
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            REVERB_TO_HP_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    self.0.data.reverb_to_hp = val;
                    Ok(())
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            KNOB_BACKLIGHT_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    self.0.data.master_knob_backlight = val;
                    Ok(())
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MIC_0_PHANTOM_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    self.0.data.mic_0_phantom = val;
                    Ok(())
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MIC_0_BOOST_NAME => {
                ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                    self.0.data.mic_0_boost = val;
                    Ok(())
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn parse_notification(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32,
        msg: u32,
    ) -> Result<(), Error> {
        if self.0.has_segment_change(msg) {
            Desktopk6Protocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }
}

#[derive(Default)]
struct ConfigCtl(Desktopk6ConfigSegment);

impl StandaloneCtlOperation<DesktopConfig, Desktopk6Protocol> for ConfigCtl {
    fn segment_mut(&mut self) -> &mut Desktopk6ConfigSegment {
        &mut self.0
    }

    fn standalone_rate(&self) -> &TcKonnektStandaloneClkRate {
        &self.0.data.standalone_rate
    }

    fn standalone_rate_mut(&mut self) -> &mut TcKonnektStandaloneClkRate {
        &mut self.0.data.standalone_rate
    }
}

impl ConfigCtl {
    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Desktopk6Protocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)?;

        self.load_standalone_rate(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        self.read_standalone_rate(elem_id, elem_value)
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        self.write_standalone_rate(unit, req, elem_id, elem_value, timeout_ms)
    }

    fn parse_notification(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32,
        msg: u32,
    ) -> Result<(), Error> {
        if self.0.has_segment_change(msg) {
            Desktopk6Protocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }
}

#[derive(Default)]
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

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Desktopk6Protocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)?;

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
        match elem_id.get_name().as_str() {
            MIXER_MIC_INST_SRC_LEVEL_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| {
                    Ok(self.0.data.mic_inst_level[idx])
                })
                .map(|_| true)
            }
            MIXER_MIC_INST_SRC_BALANCE_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| {
                    Ok(self.0.data.mic_inst_pan[idx])
                })
                .map(|_| true)
            }
            MIXER_MIC_INST_SRC_SEND_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| {
                    Ok(self.0.data.mic_inst_send[idx])
                })
                .map(|_| true)
            }
            MIXER_DUAL_INST_SRC_LEVEL_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| {
                    Ok(self.0.data.dual_inst_level[idx])
                })
                .map(|_| true)
            }
            MIXER_DUAL_INST_SRC_BALANCE_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| {
                    Ok(self.0.data.dual_inst_pan[idx])
                })
                .map(|_| true)
            }
            MIXER_DUAL_INST_SRC_SEND_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, 2, |idx| {
                    Ok(self.0.data.dual_inst_send[idx])
                })
                .map(|_| true)
            }
            MIXER_STEREO_IN_SRC_LEVEL_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || Ok(self.0.data.stereo_in_level))
                    .map(|_| true)
            }
            MIXER_STEREO_IN_SRC_BALANCE_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || Ok(self.0.data.stereo_in_pan))
                    .map(|_| true)
            }
            MIXER_STEREO_IN_SRC_SEND_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || Ok(self.0.data.stereo_in_send))
                    .map(|_| true)
            }
            HP_SRC_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let pos = Self::HP_SRCS
                    .iter()
                    .position(|&s| s == self.0.data.hp_src)
                    .unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIXER_MIC_INST_SRC_LEVEL_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    self.0.data.mic_inst_level[idx] = val;
                    Ok(())
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MIXER_MIC_INST_SRC_BALANCE_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    self.0.data.mic_inst_pan[idx] = val;
                    Ok(())
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MIXER_MIC_INST_SRC_SEND_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    self.0.data.mic_inst_send[idx] = val;
                    Ok(())
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MIXER_DUAL_INST_SRC_LEVEL_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    self.0.data.dual_inst_level[idx] = val;
                    Ok(())
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MIXER_DUAL_INST_SRC_BALANCE_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    self.0.data.dual_inst_pan[idx] = val;
                    Ok(())
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MIXER_DUAL_INST_SRC_SEND_NAME => {
                ElemValueAccessor::<i32>::get_vals(new, old, 2, |idx, val| {
                    self.0.data.dual_inst_send[idx] = val;
                    Ok(())
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MIXER_STEREO_IN_SRC_LEVEL_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    self.0.data.stereo_in_level = val;
                    Ok(())
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MIXER_STEREO_IN_SRC_BALANCE_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    self.0.data.stereo_in_pan = val;
                    Ok(())
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MIXER_STEREO_IN_SRC_SEND_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    self.0.data.stereo_in_send = val;
                    Ok(())
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            HP_SRC_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    Self::HP_SRCS
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg =
                                format!("Invalid value for index of headphone source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| self.0.data.hp_src = s)
                })?;
                Desktopk6Protocol::write_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default)]
struct PanelCtl(Desktopk6PanelSegment, Vec<ElemId>);

impl FirewireLedCtlOperation<DesktopPanel, Desktopk6Protocol> for PanelCtl {
    fn segment_mut(&mut self) -> &mut Desktopk6PanelSegment {
        &mut self.0
    }

    fn firewire_led(&self) -> &FireWireLedState {
        &self.0.data.firewire_led
    }

    fn firewire_led_mut(&mut self) -> &mut FireWireLedState {
        &mut self.0.data.firewire_led
    }
}

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

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Desktopk6Protocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)?;

        self.load_firewire_led(card_cntr)
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
        if self.read_firewire_led(elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.get_name().as_str() {
                PANEL_BUTTON_COUNT_NAME => ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(self.0.data.panel_button_count as i32)
                })
                .map(|_| true),
                MIXER_OUT_VOL => ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(self.0.data.main_knob_value)
                })
                .map(|_| true),
                PHONE_KNOB_VALUE_NAME => ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(self.0.data.phone_knob_value)
                })
                .map(|_| true),
                MIX_KNOB_VALUE_NAME => ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(self.0.data.mix_knob_value as i32)
                })
                .map(|_| true),
                REVERB_LED_STATE_NAME => {
                    ElemValueAccessor::<bool>::set_val(elem_value, || Ok(self.0.data.reverb_led_on))
                        .map(|_| true)
                }
                REVERB_KNOB_VALUE_NAME => ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(self.0.data.reverb_knob_value)
                })
                .map(|_| true),
                _ => Ok(false),
            }
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if self.write_firewire_led(unit, req, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else {
            match elem_id.get_name().as_str() {
                REVERB_LED_STATE_NAME => {
                    ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                        self.0.data.reverb_led_on = val;
                        Ok(())
                    })?;
                    Desktopk6Protocol::write_segment(
                        req,
                        &mut unit.get_node(),
                        &mut self.0,
                        timeout_ms,
                    )
                    .map(|_| true)
                }
                _ => Ok(false),
            }
        }
    }

    fn parse_notification(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32,
        msg: u32,
    ) -> Result<(), Error> {
        if self.0.has_segment_change(msg) {
            Desktopk6Protocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }
}

#[derive(Default)]
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

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.measure_states(unit, req, timeout_ms)?;

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

    fn measure_states(
        &mut self,
        unit: &mut SndDice,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Desktopk6Protocol::read_segment(req, &mut unit.get_node(), &mut self.0, timeout_ms)
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            ANALOG_IN_NAME => {
                elem_value.set_int(&self.0.data.analog_inputs);
                Ok(true)
            }
            MIXER_OUT_NAME => {
                elem_value.set_int(&self.0.data.mixer_outputs);
                Ok(true)
            }
            STREAM_IN_NAME => {
                elem_value.set_int(&self.0.data.stream_inputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
