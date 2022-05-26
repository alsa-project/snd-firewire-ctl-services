// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, dice_protocols::tcelectronic::studio::*};

#[derive(Default)]
pub struct Studiok48Model {
    req: FwReq,
    sections: GeneralSections,
    ctl: CommonCtl,
    lineout_ctl: LineoutCtl,
    remote_ctl: RemoteCtl,
    config_ctl: ConfigCtl,
    mixer_ctl: MixerCtl,
    phys_out_ctl: PhysOutCtl,
    ch_strip_ctl: ChStripCtl,
    reverb_ctl: ReverbCtl,
    hw_state_ctl: HwStateCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<(SndDice, FwNode)> for Studiok48Model {
    fn load(
        &mut self,
        unit: &mut (SndDice, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.sections =
            GeneralProtocol::read_general_sections(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        let caps = GlobalSectionProtocol::read_clock_caps(
            &mut self.req,
            &mut unit.1,
            &self.sections,
            TIMEOUT_MS,
        )?;
        let src_labels = GlobalSectionProtocol::read_clock_source_labels(
            &mut self.req,
            &mut unit.1,
            &self.sections,
            TIMEOUT_MS,
        )?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.lineout_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.remote_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.config_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.mixer_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.phys_out_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;

        self.reverb_ctl
            .load(card_cntr, &mut unit.0, &mut self.req, TIMEOUT_MS)
            .map(|(notified_elem_id_list, measured_elem_id_list)| {
                self.reverb_ctl.2 = notified_elem_id_list;
                self.reverb_ctl.3 = measured_elem_id_list;
            })?;
        self.ch_strip_ctl
            .load(card_cntr, &mut unit.0, &mut self.req, TIMEOUT_MS)
            .map(|(notified_elem_id_list, measured_elem_id_list)| {
                self.ch_strip_ctl.2 = notified_elem_id_list;
                self.ch_strip_ctl.3 = measured_elem_id_list;
            })?;

        self.hw_state_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;

        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut (SndDice, FwNode),
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
        } else if self.lineout_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.remote_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.config_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_out_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_state_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
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
            .lineout_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .remote_ctl
            .write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)?
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
            .phys_out_ctl
            .write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .reverb_ctl
            .write(&mut unit.0, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.ch_strip_ctl.write(
            &mut unit.0,
            &mut self.req,
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
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for Studiok48Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.lineout_ctl.1);
        elem_id_list.extend_from_slice(&self.remote_ctl.1);
        elem_id_list.extend_from_slice(&self.config_ctl.1);
        elem_id_list.extend_from_slice(&self.mixer_ctl.2);
        elem_id_list.extend_from_slice(&self.phys_out_ctl.1);
        elem_id_list.extend_from_slice(&self.reverb_ctl.2);
        elem_id_list.extend_from_slice(&self.ch_strip_ctl.2);
        elem_id_list.extend_from_slice(&self.hw_state_ctl.1);
    }

    fn parse_notification(&mut self, unit: &mut (SndDice, FwNode), msg: &u32) -> Result<(), Error> {
        self.ctl
            .parse_notification(unit, &mut self.req, &self.sections, *msg, TIMEOUT_MS)?;
        self.lineout_ctl
            .parse_notification(unit, &mut self.req, *msg, TIMEOUT_MS)?;
        self.remote_ctl
            .parse_notification(unit, &mut self.req, *msg, TIMEOUT_MS)?;
        self.config_ctl
            .parse_notification(unit, &mut self.req, *msg, TIMEOUT_MS)?;
        self.mixer_ctl
            .parse_notification(unit, &mut self.req, *msg, TIMEOUT_MS)?;
        self.phys_out_ctl
            .parse_notification(unit, &mut self.req, *msg, TIMEOUT_MS)?;
        self.reverb_ctl
            .parse_notification(&mut unit.0, &mut self.req, *msg, TIMEOUT_MS)?;
        self.ch_strip_ctl
            .parse_notification(&mut unit.0, &mut self.req, *msg, TIMEOUT_MS)?;
        self.hw_state_ctl
            .parse_notification(unit, &mut self.req, *msg, TIMEOUT_MS)?;

        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.lineout_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.remote_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.config_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_out_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_state_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndDice, FwNode)> for Studiok48Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.mixer_ctl.3);
        elem_id_list.extend_from_slice(&self.reverb_ctl.3);
        elem_id_list.extend_from_slice(&self.ch_strip_ctl.3);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.ctl
            .measure_states(unit, &mut self.req, &self.sections, TIMEOUT_MS)?;
        self.mixer_ctl
            .measure_states(unit, &mut self.req, TIMEOUT_MS)?;
        self.reverb_ctl
            .measure_states(&mut unit.0, &mut self.req, TIMEOUT_MS)?;
        self.ch_strip_ctl
            .measure_states(&mut unit.0, &mut self.req, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_ctl.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_ctl.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

fn nominal_signal_level_to_str(level: &NominalSignalLevel) -> &'static str {
    match level {
        NominalSignalLevel::Professional => "+4dBu",
        NominalSignalLevel::Consumer => "-10dBV",
    }
}

#[derive(Default)]
struct LineoutCtl(Studiok48LineOutLevelSegment, Vec<ElemId>);

const LINE_OUT_45_LEVEL_NAME: &str = "line-out-5/6-level";
const LINE_OUT_67_LEVEL_NAME: &str = "line-out-7/8-level";
const LINE_OUT_89_LEVEL_NAME: &str = "line-out-9/10-level";
const LINE_OUT_1011_LEVEL_NAME: &str = "line-out-11/12-level";

impl LineoutCtl {
    const NOMINAL_SIGNAL_LEVELS: [NominalSignalLevel; 2] = [
        NominalSignalLevel::Professional,
        NominalSignalLevel::Consumer,
    ];

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Studiok48Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)?;

        let labels: Vec<&str> = Self::NOMINAL_SIGNAL_LEVELS
            .iter()
            .map(|m| nominal_signal_level_to_str(m))
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, LINE_OUT_45_LEVEL_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, LINE_OUT_67_LEVEL_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, LINE_OUT_89_LEVEL_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, LINE_OUT_1011_LEVEL_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            LINE_OUT_45_LEVEL_NAME => Self::read_as_index(elem_value, self.0.data.line_45),
            LINE_OUT_67_LEVEL_NAME => Self::read_as_index(elem_value, self.0.data.line_67),
            LINE_OUT_89_LEVEL_NAME => Self::read_as_index(elem_value, self.0.data.line_89),
            LINE_OUT_1011_LEVEL_NAME => Self::read_as_index(elem_value, self.0.data.line_1011),
            _ => Ok(false),
        }
    }

    fn read_as_index(elem_value: &mut ElemValue, level: NominalSignalLevel) -> Result<bool, Error> {
        ElemValueAccessor::<u32>::set_val(elem_value, || {
            let pos = Self::NOMINAL_SIGNAL_LEVELS
                .iter()
                .position(|l| level.eq(&l))
                .unwrap();
            Ok(pos as u32)
        })
        .map(|_| true)
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            LINE_OUT_45_LEVEL_NAME => {
                self.write_as_index(unit, req, elem_value, timeout_ms, |data, level| {
                    data.line_45 = level
                })
            }
            LINE_OUT_67_LEVEL_NAME => {
                self.write_as_index(unit, req, elem_value, timeout_ms, |data, level| {
                    data.line_67 = level
                })
            }
            LINE_OUT_89_LEVEL_NAME => {
                self.write_as_index(unit, req, elem_value, timeout_ms, |data, level| {
                    data.line_89 = level
                })
            }
            LINE_OUT_1011_LEVEL_NAME => {
                self.write_as_index(unit, req, elem_value, timeout_ms, |data, level| {
                    data.line_1011 = level
                })
            }
            _ => Ok(false),
        }
    }

    fn write_as_index<F>(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_value: &ElemValue,
        timeout_ms: u32,
        cb: F,
    ) -> Result<bool, Error>
    where
        F: Fn(&mut StudioLineOutLevel, NominalSignalLevel),
    {
        ElemValueAccessor::<u32>::get_val(elem_value, |val| {
            Self::NOMINAL_SIGNAL_LEVELS
                .iter()
                .nth(val as usize)
                .ok_or_else(|| {
                    let msg = format!("Invalid index of nominal level: {}", val);
                    Error::new(FileError::Inval, &msg)
                })
                .map(|&l| cb(&mut self.0.data, l))
        })?;
        Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms).map(|_| true)
    }

    fn parse_notification(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if self.0.has_segment_change(msg) {
            Studiok48Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }
}

#[derive(Default)]
struct RemoteCtl(Studiok48RemoteSegment, Vec<ElemId>);

impl ProgramCtlOperation<StudioRemote, Studiok48Protocol> for RemoteCtl {
    fn segment_mut(&mut self) -> &mut Studiok48RemoteSegment {
        &mut self.0
    }

    fn prog(&self) -> &TcKonnektLoadedProgram {
        &self.0.data.prog
    }

    fn prog_mut(&mut self) -> &mut TcKonnektLoadedProgram {
        &mut self.0.data.prog
    }
}

const USER_ASSIGN_NAME: &str = "remote-user-assign";
const EFFECT_BUTTON_MODE_NAME: &str = "remote-effect-button-mode";
const FALLBACK_TO_MASTER_ENABLE_NAME: &str = "remote-fallback-to-master-enable";
const FALLBACK_TO_MASTER_DURATION_NAME: &str = "remote-fallback-to-master-duration";
const KNOB_PUSH_MODE_NAME: &str = "remote-knob-push-mode";

fn src_pair_mode_to_str(entry: &MonitorSrcPairMode) -> &'static str {
    match entry {
        MonitorSrcPairMode::Inactive => "Inactive",
        MonitorSrcPairMode::Active => "Active",
        MonitorSrcPairMode::Fixed => "Fixed",
    }
}

impl RemoteCtl {
    const EFFECT_BUTTON_MODES: [RemoteEffectButtonMode; 2] =
        [RemoteEffectButtonMode::Reverb, RemoteEffectButtonMode::Midi];

    // NOTE: by milisecond.
    const DURATION_MIN: i32 = 10;
    const DURATION_MAX: i32 = 1000;
    const DURATION_STEP: i32 = 1;

    const KNOB_PUSH_MODES: [KnobPushMode; 4] = [
        KnobPushMode::Pan,
        KnobPushMode::GainToReverb,
        KnobPushMode::GainToAux0,
        KnobPushMode::GainToAux1,
    ];

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Studiok48Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)?;

        self.load_prog(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let labels: Vec<String> = MixerCtl::SRC_PAIR_ENTRIES
            .iter()
            .map(|src| src_pair_entry_to_string(src))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, USER_ASSIGN_NAME, 0);
        card_cntr
            .add_enum_elems(
                &elem_id,
                1,
                STUDIO_REMOTE_USER_ASSIGN_COUNT,
                &labels,
                None,
                true,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::EFFECT_BUTTON_MODES
            .iter()
            .map(|m| effect_button_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, EFFECT_BUTTON_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            FALLBACK_TO_MASTER_ENABLE_NAME,
            0,
        );
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            FALLBACK_TO_MASTER_DURATION_NAME,
            0,
        );
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::DURATION_MIN,
                Self::DURATION_MAX,
                Self::DURATION_STEP,
                1,
                None,
                true,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::KNOB_PUSH_MODES
            .iter()
            .map(|m| knob_push_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, KNOB_PUSH_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            USER_ASSIGN_NAME => ElemValueAccessor::<u32>::set_vals(
                elem_value,
                STUDIO_REMOTE_USER_ASSIGN_COUNT,
                |idx| {
                    let pos = MixerCtl::SRC_PAIR_ENTRIES
                        .iter()
                        .position(|p| self.0.data.user_assigns[idx].eq(p))
                        .unwrap();
                    Ok(pos as u32)
                },
            )
            .map(|_| true),
            EFFECT_BUTTON_MODE_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let pos = Self::EFFECT_BUTTON_MODES
                    .iter()
                    .position(|m| self.0.data.effect_button_mode.eq(m))
                    .unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            FALLBACK_TO_MASTER_ENABLE_NAME => {
                ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(self.0.data.fallback_to_master_enable)
                })
                .map(|_| true)
            }
            FALLBACK_TO_MASTER_DURATION_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || {
                    Ok(self.0.data.fallback_to_master_duration as i32)
                })
                .map(|_| true)
            }
            KNOB_PUSH_MODE_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                let pos = Self::KNOB_PUSH_MODES
                    .iter()
                    .position(|m| self.0.data.knob_push_mode.eq(m))
                    .unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            _ => self.read_prog(elem_id, elem_value),
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            USER_ASSIGN_NAME => {
                ElemValueAccessor::<u32>::get_vals(
                    new,
                    old,
                    STUDIO_REMOTE_USER_ASSIGN_COUNT,
                    |idx, val| {
                        MixerCtl::SRC_PAIR_ENTRIES
                            .iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg =
                                    format!("Invalid index of source of user assignment: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&s| self.0.data.user_assigns[idx] = s)
                    },
                )?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            EFFECT_BUTTON_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    Self::EFFECT_BUTTON_MODES
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg =
                                format!("Invalid index of source of user assignment: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&m| self.0.data.effect_button_mode = m)
                })?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            FALLBACK_TO_MASTER_ENABLE_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    self.0.data.fallback_to_master_enable = val;
                    Ok(())
                })?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            FALLBACK_TO_MASTER_DURATION_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    self.0.data.fallback_to_master_duration = val as u32;
                    Ok(())
                })?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            KNOB_PUSH_MODE_NAME => {
                ElemValueAccessor::<u32>::get_val(new, |val| {
                    Self::KNOB_PUSH_MODES
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg =
                                format!("Invalid index of source of user assignment: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&m| self.0.data.knob_push_mode = m)
                })?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            _ => self.write_prog(&mut unit.0, req, elem_id, new, timeout_ms),
        }
    }

    fn parse_notification(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if self.0.has_segment_change(msg) {
            Studiok48Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }
}

#[derive(Default)]
struct ConfigCtl(Studiok48ConfigSegment, Vec<ElemId>);

fn opt_iface_mode_to_str(mode: &OptIfaceMode) -> &'static str {
    match mode {
        OptIfaceMode::Adat => "ADAT",
        OptIfaceMode::Spdif => "S/PDIF",
    }
}

fn standalone_clk_src_to_str(src: &StudioStandaloneClkSrc) -> &'static str {
    match src {
        StudioStandaloneClkSrc::Adat => "ADAT",
        StudioStandaloneClkSrc::SpdifOnOpt01 => "S/PDIF-opt-1/2",
        StudioStandaloneClkSrc::SpdifOnOpt23 => "S/PDIF-opt-3/4",
        StudioStandaloneClkSrc::SpdifOnCoax => "S/PDIF-coax",
        StudioStandaloneClkSrc::WordClock => "Word-clock",
        StudioStandaloneClkSrc::Internal => "Internal",
    }
}

fn effect_button_mode_to_str(mode: &RemoteEffectButtonMode) -> &'static str {
    match mode {
        RemoteEffectButtonMode::Reverb => "Reverb",
        RemoteEffectButtonMode::Midi => "Midi",
    }
}

fn knob_push_mode_to_str(mode: &KnobPushMode) -> &'static str {
    match mode {
        KnobPushMode::Pan => "Pan",
        KnobPushMode::GainToReverb => "Reverb",
        KnobPushMode::GainToAux0 => "Aux-1/2",
        KnobPushMode::GainToAux1 => "Aux-3/4",
    }
}

impl StandaloneCtlOperation<StudioConfig, Studiok48Protocol> for ConfigCtl {
    fn segment_mut(&mut self) -> &mut Studiok48ConfigSegment {
        &mut self.0
    }

    fn standalone_rate(&self) -> &TcKonnektStandaloneClkRate {
        &self.0.data.standalone_rate
    }

    fn standalone_rate_mut(&mut self) -> &mut TcKonnektStandaloneClkRate {
        &mut self.0.data.standalone_rate
    }
}

impl MidiSendCtlOperation<StudioConfig, Studiok48Protocol> for ConfigCtl {
    fn segment_mut(&mut self) -> &mut Studiok48ConfigSegment {
        &mut self.0
    }

    fn midi_sender(&self) -> &TcKonnektMidiSender {
        &self.0.data.midi_send
    }

    fn midi_sender_mut(&mut self) -> &mut TcKonnektMidiSender {
        &mut self.0.data.midi_send
    }
}

const OPT_IFACE_MODE_NAME: &str = "opt-iface-mode";
const STANDALONE_CLK_SRC_NAME: &str = "standalone-clock-source";
const CLOCK_RECOVERY_NAME: &str = "clock-recovery";

impl ConfigCtl {
    const OPT_IFACE_MODES: [OptIfaceMode; 2] = [OptIfaceMode::Adat, OptIfaceMode::Spdif];

    const STANDALONE_CLK_SRCS: [StudioStandaloneClkSrc; 6] = [
        StudioStandaloneClkSrc::Adat,
        StudioStandaloneClkSrc::SpdifOnOpt01,
        StudioStandaloneClkSrc::SpdifOnOpt23,
        StudioStandaloneClkSrc::SpdifOnCoax,
        StudioStandaloneClkSrc::WordClock,
        StudioStandaloneClkSrc::Internal,
    ];

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Studiok48Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)?;

        self.load_standalone_rate(card_cntr)?;
        self.load_midi_sender(card_cntr)?;

        let labels: Vec<&str> = Self::OPT_IFACE_MODES
            .iter()
            .map(|m| opt_iface_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OPT_IFACE_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = Self::STANDALONE_CLK_SRCS
            .iter()
            .map(|r| standalone_clk_src_to_str(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, STANDALONE_CLK_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, CLOCK_RECOVERY_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_standalone_rate(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_midi_sender(elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.get_name().as_str() {
                OPT_IFACE_MODE_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::OPT_IFACE_MODES
                        .iter()
                        .position(|m| self.0.data.opt_iface_mode.eq(m))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true),
                STANDALONE_CLK_SRC_NAME => ElemValueAccessor::<u32>::set_val(elem_value, || {
                    let pos = Self::STANDALONE_CLK_SRCS
                        .iter()
                        .position(|s| self.0.data.standalone_src.eq(s))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true),
                CLOCK_RECOVERY_NAME => ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(self.0.data.clock_recovery)
                })
                .map(|_| true),
                _ => Ok(false),
            }
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if self.write_standalone_rate(&mut unit.0, req, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.write_midi_sender(&mut unit.0, req, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else {
            match elem_id.get_name().as_str() {
                OPT_IFACE_MODE_NAME => {
                    ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                        Self::OPT_IFACE_MODES
                            .iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg =
                                    format!("Invalid index of standalone clock source: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&m| self.0.data.opt_iface_mode = m)
                    })?;
                    Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                        .map(|_| true)
                }
                STANDALONE_CLK_SRC_NAME => {
                    ElemValueAccessor::<u32>::get_val(elem_value, |val| {
                        Self::STANDALONE_CLK_SRCS
                            .iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg =
                                    format!("Invalid index of standalone clock source: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&s| self.0.data.standalone_src = s)
                    })?;
                    Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                        .map(|_| true)
                }
                CLOCK_RECOVERY_NAME => {
                    ElemValueAccessor::<bool>::get_val(elem_value, |val| {
                        self.0.data.clock_recovery = val;
                        Ok(())
                    })?;
                    Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                        .map(|_| true)
                }
                _ => Ok(false),
            }
        }
    }

    fn parse_notification(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if self.0.has_segment_change(msg) {
            Studiok48Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }
}

#[derive(Default)]
struct MixerCtl(
    Studiok48MixerStateSegment,
    Studiok48MixerMeterSegment,
    Vec<ElemId>,
    Vec<ElemId>,
);

const SRC_PAIR_MODE_NAME: &str = "mixer-input-mode";
const SRC_ENTRY_NAME: &str = "mixer-input-source";
const SRC_STEREO_LINK_NAME: &str = "mixer-input-stereo-link";
const SRC_GAIN_NAME: &str = "mixer-input-gain";
const SRC_PAN_NAME: &str = "mixer-input-pan";
const REVERB_SRC_GAIN_NAME: &str = "reverb-input-gain";
const AUX01_SRC_GAIN_NAME: &str = "aux-1/2-input-gain";
const AUX23_SRC_GAIN_NAME: &str = "aux-3/4-input-gain";
const SRC_MUTE_NAME: &str = "mixer-input-mute";

const OUT_DIM_NAME: &str = "mixer-output-dim";
const OUT_VOL_NAME: &str = "mixer-output-volume";
const OUT_DIM_VOL_NAME: &str = "mixer-output-dim-volume";
const REVERB_RETURN_MUTE_NAME: &str = "reverb-return-mute";
const REVERB_RETURN_GAIN_NAME: &str = "reverb-return-gain";

const POST_FADER_NAME: &str = "mixer-post-fader";

const CH_STRIP_AS_PLUGIN_NAME: &str = "channel-strip-as-plugin";
const CH_STRIP_SRC_NAME: &str = "channel-strip-source";
const CH_STRIP_23_AT_MID_RATE: &str = "channel-strip-3/4-at-mid-rate";

const MIXER_ENABLE_NAME: &str = "mixer-direct-monitoring";

const MIXER_INPUT_METER_NAME: &str = "mixer-input-meter";
const MIXER_OUTPUT_METER_NAME: &str = "mixer-output-meter";
const AUX_OUTPUT_METER_NAME: &str = "aux-output-meter";

impl MixerCtl {
    const SRC_PAIR_MODES: [MonitorSrcPairMode; 3] = [
        MonitorSrcPairMode::Inactive,
        MonitorSrcPairMode::Active,
        MonitorSrcPairMode::Fixed,
    ];

    const SRC_PAIR_ENTRIES: [SrcEntry; 51] = [
        SrcEntry::Unused,
        SrcEntry::Analog(0),
        SrcEntry::Analog(1),
        SrcEntry::Analog(2),
        SrcEntry::Analog(3),
        SrcEntry::Analog(4),
        SrcEntry::Analog(5),
        SrcEntry::Analog(6),
        SrcEntry::Analog(7),
        SrcEntry::Analog(8),
        SrcEntry::Analog(9),
        SrcEntry::Analog(10),
        SrcEntry::Analog(11),
        SrcEntry::Spdif(0),
        SrcEntry::Spdif(1),
        SrcEntry::Adat(0),
        SrcEntry::Adat(1),
        SrcEntry::Adat(2),
        SrcEntry::Adat(3),
        SrcEntry::Adat(4),
        SrcEntry::Adat(5),
        SrcEntry::Adat(6),
        SrcEntry::Adat(7),
        SrcEntry::StreamA(0),
        SrcEntry::StreamA(1),
        SrcEntry::StreamA(2),
        SrcEntry::StreamA(3),
        SrcEntry::StreamA(4),
        SrcEntry::StreamA(5),
        SrcEntry::StreamA(6),
        SrcEntry::StreamA(7),
        SrcEntry::StreamA(8),
        SrcEntry::StreamA(9),
        SrcEntry::StreamA(10),
        SrcEntry::StreamA(11),
        SrcEntry::StreamA(12),
        SrcEntry::StreamA(13),
        SrcEntry::StreamA(14),
        SrcEntry::StreamA(15),
        SrcEntry::StreamB(0),
        SrcEntry::StreamB(1),
        SrcEntry::StreamB(2),
        SrcEntry::StreamB(3),
        SrcEntry::StreamB(4),
        SrcEntry::StreamB(5),
        SrcEntry::StreamB(6),
        SrcEntry::StreamB(7),
        SrcEntry::StreamB(8),
        SrcEntry::StreamB(9),
        SrcEntry::StreamB(10),
        SrcEntry::StreamB(11),
    ];

    const OUT_LABELS: [&'static str; 3] = ["Main-1/2", "Aux-1/2", "Aux-3/4"];
    const SEND_TARGET_LABELS: [&'static str; 3] = ["Reverb-1/2", "Aux-1/2", "Aux-3/4"];

    const LEVEL_MIN: i32 = -1000;
    const LEVEL_MAX: i32 = 0;
    const LEVEL_STEP: i32 = 1;
    const LEVEL_TLV: DbInterval = DbInterval {
        min: -7200,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    const PAN_MIN: i32 = -50;
    const PAN_MAX: i32 = 50;
    const PAN_STEP: i32 = 1;

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Studiok48Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)?;
        Studiok48Protocol::read_segment(req, &mut unit.1, &mut self.1, timeout_ms)?;

        let labels: Vec<String> = (0..self.0.data.src_pairs.len())
            .map(|i| format!("Mixer-source-{}/{}", i + 1, i + 2))
            .collect();
        let item_labels: Vec<&str> = Self::SRC_PAIR_MODES
            .iter()
            .map(|m| src_pair_mode_to_str(m))
            .collect();
        self.state_add_elem_enum(card_cntr, SRC_PAIR_MODE_NAME, 1, labels.len(), &item_labels)?;
        self.state_add_elem_bool(card_cntr, SRC_STEREO_LINK_NAME, 1, labels.len())?;

        let labels: Vec<String> = (0..(self.0.data.src_pairs.len() * 2))
            .map(|i| format!("Mixer-source-{}", i + 1))
            .collect();
        let item_labels: Vec<String> = Self::SRC_PAIR_ENTRIES
            .iter()
            .map(|s| src_pair_entry_to_string(s))
            .collect();
        self.state_add_elem_enum(card_cntr, SRC_ENTRY_NAME, 1, labels.len(), &item_labels)?;
        self.state_add_elem_level(card_cntr, SRC_GAIN_NAME, 1, labels.len())?;
        self.state_add_elem_pan(card_cntr, SRC_PAN_NAME, 1, labels.len())?;
        self.state_add_elem_level(card_cntr, REVERB_SRC_GAIN_NAME, 1, labels.len())?;
        self.state_add_elem_level(card_cntr, AUX01_SRC_GAIN_NAME, 1, labels.len())?;
        self.state_add_elem_level(card_cntr, AUX23_SRC_GAIN_NAME, 1, labels.len())?;
        self.state_add_elem_bool(card_cntr, SRC_MUTE_NAME, 1, labels.len())?;

        let labels = &Self::OUT_LABELS;
        self.state_add_elem_bool(card_cntr, REVERB_RETURN_MUTE_NAME, 1, labels.len())?;
        self.state_add_elem_level(card_cntr, REVERB_RETURN_GAIN_NAME, 1, labels.len())?;
        self.state_add_elem_bool(card_cntr, OUT_DIM_NAME, 1, labels.len())?;
        self.state_add_elem_level(card_cntr, OUT_VOL_NAME, 1, labels.len())?;
        self.state_add_elem_level(card_cntr, OUT_DIM_VOL_NAME, 1, labels.len())?;

        let labels = &Self::SEND_TARGET_LABELS;
        self.state_add_elem_bool(card_cntr, POST_FADER_NAME, 1, labels.len())?;

        let labels: Vec<String> = (0..2)
            .map(|i| format!("Channel-strip-{}/{}", i + 1, i + 2))
            .collect();
        self.state_add_elem_bool(card_cntr, CH_STRIP_AS_PLUGIN_NAME, 1, labels.len())?;
        let labels: Vec<String> = (0..4).map(|i| format!("Channel-strip-{}", i)).collect();
        self.state_add_elem_enum(card_cntr, CH_STRIP_SRC_NAME, 1, labels.len(), &item_labels)?;
        self.state_add_elem_bool(card_cntr, CH_STRIP_23_AT_MID_RATE, 1, 1)?;

        self.state_add_elem_bool(card_cntr, MIXER_ENABLE_NAME, 1, 1)?;

        // For metering.
        let labels: Vec<String> = (0..self.1.data.src_inputs.len())
            .map(|i| format!("mixer-input-{}", i))
            .collect();
        self.meter_add_elem_level(card_cntr, MIXER_INPUT_METER_NAME, labels.len())?;

        let labels: Vec<String> = (0..self.1.data.mixer_outputs.len())
            .map(|i| format!("mixer-output-{}", i))
            .collect();
        self.meter_add_elem_level(card_cntr, MIXER_OUTPUT_METER_NAME, labels.len())?;

        let labels: Vec<String> = (0..self.1.data.mixer_outputs.len())
            .map(|i| format!("aux-output-{}", i))
            .collect();
        self.meter_add_elem_level(card_cntr, AUX_OUTPUT_METER_NAME, labels.len())?;

        Ok(())
    }

    fn state_add_elem_enum<T: AsRef<str>>(
        &mut self,
        card_cntr: &mut CardCntr,
        name: &str,
        count: usize,
        value_count: usize,
        labels: &[T],
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr
            .add_enum_elems(&elem_id, count, value_count, &labels, None, true)
            .map(|mut elem_id_list| self.2.append(&mut elem_id_list))
    }

    fn state_add_elem_bool(
        &mut self,
        card_cntr: &mut CardCntr,
        name: &str,
        count: usize,
        value_count: usize,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr
            .add_bool_elems(&elem_id, count, value_count, true)
            .map(|mut elem_id_list| self.2.append(&mut elem_id_list))
    }

    fn state_add_elem_level(
        &mut self,
        card_cntr: &mut CardCntr,
        name: &str,
        count: usize,
        value_count: usize,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                count,
                Self::LEVEL_MIN,
                Self::LEVEL_MAX,
                Self::LEVEL_STEP,
                value_count,
                Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.2.append(&mut elem_id_list))
    }

    fn state_add_elem_pan(
        &mut self,
        card_cntr: &mut CardCntr,
        name: &str,
        count: usize,
        value_count: usize,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                count,
                Self::PAN_MIN,
                Self::PAN_MAX,
                Self::PAN_STEP,
                value_count,
                None,
                true,
            )
            .map(|mut elem_id_list| self.2.append(&mut elem_id_list))
    }

    fn meter_add_elem_level(
        &mut self,
        card_cntr: &mut CardCntr,
        name: &str,
        value_count: usize,
    ) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::LEVEL_MIN,
                Self::LEVEL_MAX,
                Self::LEVEL_STEP,
                value_count,
                Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
                false,
            )
            .map(|mut elem_id_list| self.3.append(&mut elem_id_list))
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn state_read_src_param<T, F>(&self, elem_value: &ElemValue, cb: F) -> Result<bool, Error>
    where
        T: Default + Copy + Eq,
        F: Fn(&MonitorSrcParam) -> Result<T, Error>,
        ElemValue: ElemValueAccessor<T>,
    {
        let count = self.0.data.src_pairs.len() * 2;
        ElemValueAccessor::<T>::set_vals(elem_value, count, |idx| {
            let i = idx / 2;
            let ch = idx % 2;
            let param = if ch == 0 {
                &self.0.data.src_pairs[i].left
            } else {
                &self.0.data.src_pairs[i].right
            };
            cb(param)
        })
        .map(|_| true)
    }

    fn state_read_out_pair<T, F>(&self, elem_value: &ElemValue, cb: F) -> Result<bool, Error>
    where
        T: Copy + Default + Eq + PartialEq,
        F: Fn(&OutPair) -> Result<T, Error>,
        ElemValue: ElemValueAccessor<T>,
    {
        ElemValueAccessor::<T>::set_vals(elem_value, Self::OUT_LABELS.len(), |idx| {
            cb(&self.0.data.mixer_out[idx])
        })
        .map(|_| true)
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            SRC_PAIR_MODE_NAME => {
                let state = &mut self.0.data;
                ElemValueAccessor::<u32>::get_vals(new, old, state.src_pairs.len(), |idx, val| {
                    if let Some(m) = Self::SRC_PAIR_MODES.iter().nth(val as usize) {
                        if state.src_pairs[idx].mode != MonitorSrcPairMode::Fixed {
                            if *m != MonitorSrcPairMode::Fixed {
                                state.src_pairs[idx].mode = *m;
                                Ok(())
                            } else {
                                let msg = format!("The fixed mode is not newly available: {}", idx);
                                Err(Error::new(FileError::Inval, &msg))
                            }
                        } else {
                            let msg = format!("The source of mixer is immutable: {}", idx);
                            Err(Error::new(FileError::Inval, &msg))
                        }
                    } else {
                        let msg = format!("Invalid value for index of mixer source: {}", val);
                        Err(Error::new(FileError::Inval, &msg))
                    }
                })?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            SRC_ENTRY_NAME => {
                self.state_write_src_param(unit, req, new, old, timeout_ms, |param, val: u32| {
                    Self::SRC_PAIR_ENTRIES
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of mixer source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| param.src = s)
                })
            }
            SRC_STEREO_LINK_NAME => {
                let pair_count = self.0.data.src_pairs.len();
                ElemValueAccessor::<bool>::get_vals(new, old, pair_count, |idx, val| {
                    self.0.data.src_pairs[idx].stereo_link = val;
                    Ok(())
                })?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            SRC_GAIN_NAME => {
                self.state_write_src_param(unit, req, new, old, timeout_ms, |param, val| {
                    param.gain_to_main = val;
                    Ok(())
                })
            }
            SRC_PAN_NAME => {
                self.state_write_src_param(unit, req, new, old, timeout_ms, |param, val| {
                    param.pan_to_main = val;
                    Ok(())
                })
            }
            REVERB_SRC_GAIN_NAME => {
                self.state_write_src_param(unit, req, new, old, timeout_ms, |param, val| {
                    param.gain_to_reverb = val;
                    Ok(())
                })
            }
            AUX01_SRC_GAIN_NAME => {
                self.state_write_src_param(unit, req, new, old, timeout_ms, |param, val| {
                    param.gain_to_aux0 = val;
                    Ok(())
                })
            }
            AUX23_SRC_GAIN_NAME => {
                self.state_write_src_param(unit, req, new, old, timeout_ms, |param, val| {
                    param.gain_to_aux1 = val;
                    Ok(())
                })
            }
            SRC_MUTE_NAME => {
                new.get_bool(&mut self.0.data.mutes);
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            OUT_DIM_NAME => {
                self.state_write_out_pair(unit, req, new, old, timeout_ms, |pair, val| {
                    pair.dim_enabled = val;
                    Ok(())
                })
            }
            OUT_VOL_NAME => {
                self.state_write_out_pair(unit, req, new, old, timeout_ms, |pair, val| {
                    pair.vol = val;
                    Ok(())
                })
            }
            OUT_DIM_VOL_NAME => {
                self.state_write_out_pair(unit, req, new, old, timeout_ms, |pair, val| {
                    pair.dim_vol = val;
                    Ok(())
                })
            }
            REVERB_RETURN_MUTE_NAME => {
                new.get_bool(&mut self.0.data.reverb_return_mute);
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            REVERB_RETURN_GAIN_NAME => {
                new.get_int(&mut self.0.data.reverb_return_gain);
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            CH_STRIP_AS_PLUGIN_NAME => {
                new.get_bool(&mut self.0.data.ch_strip_as_plugin);
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            CH_STRIP_SRC_NAME => {
                let count = self.0.data.ch_strip_src.len();
                ElemValueAccessor::<u32>::get_vals(new, old, count, |idx, val| {
                    Self::SRC_PAIR_ENTRIES
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg =
                                format!("Invalid value for index of ch strip source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| self.0.data.ch_strip_src[idx] = s)
                })?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            CH_STRIP_23_AT_MID_RATE => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    self.0.data.ch_strip_23_at_mid_rate = val;
                    Ok(())
                })?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            POST_FADER_NAME => {
                new.get_bool(&mut self.0.data.post_fader);
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MIXER_ENABLE_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    self.0.data.enabled = val;
                    Ok(())
                })?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn state_write_src_param<T, F>(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        new: &ElemValue,
        old: &ElemValue,
        timeout_ms: u32,
        cb: F,
    ) -> Result<bool, Error>
    where
        T: Default + Copy + Eq,
        F: Fn(&mut MonitorSrcParam, T) -> Result<(), Error>,
        ElemValue: ElemValueAccessor<T>,
    {
        let count = self.0.data.src_pairs.len() * 2;
        ElemValueAccessor::<T>::get_vals(new, old, count, |idx, val| {
            let i = idx / 2;
            let ch = idx % 2;
            let param = if ch == 0 {
                &mut self.0.data.src_pairs[i].left
            } else {
                &mut self.0.data.src_pairs[i].right
            };
            cb(param, val)
        })?;
        Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms).map(|_| true)
    }

    fn state_write_out_pair<T, F>(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        new: &ElemValue,
        old: &ElemValue,
        timeout_ms: u32,
        cb: F,
    ) -> Result<bool, Error>
    where
        T: Default + Copy + Eq,
        F: Fn(&mut OutPair, T) -> Result<(), Error>,
        ElemValue: ElemValueAccessor<T>,
    {
        ElemValueAccessor::<T>::get_vals(new, old, Self::OUT_LABELS.len(), |idx, val| {
            cb(&mut self.0.data.mixer_out[idx], val)
        })?;
        Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms).map(|_| true)
    }

    fn read_notified_elem(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            SRC_PAIR_MODE_NAME => {
                let pair_count = self.0.data.src_pairs.len();
                ElemValueAccessor::<u32>::set_vals(elem_value, pair_count, |idx| {
                    let pos = Self::SRC_PAIR_MODES
                        .iter()
                        .position(|m| self.0.data.src_pairs[idx].mode.eq(m))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            SRC_STEREO_LINK_NAME => {
                let pair_count = self.0.data.src_pairs.len();
                ElemValueAccessor::<bool>::set_vals(elem_value, pair_count, |idx| {
                    Ok(self.0.data.src_pairs[idx].stereo_link)
                })
                .map(|_| true)
            }
            SRC_ENTRY_NAME => self.state_read_src_param(elem_value, |param| {
                let pos = Self::SRC_PAIR_ENTRIES
                    .iter()
                    .position(|m| param.src.eq(m))
                    .unwrap();
                Ok(pos as u32)
            }),
            SRC_GAIN_NAME => self.state_read_src_param(elem_value, |param| Ok(param.gain_to_main)),
            SRC_PAN_NAME => self.state_read_src_param(elem_value, |param| Ok(param.pan_to_main)),
            REVERB_SRC_GAIN_NAME => {
                self.state_read_src_param(elem_value, |param| Ok(param.gain_to_reverb))
            }
            AUX01_SRC_GAIN_NAME => {
                self.state_read_src_param(elem_value, |param| Ok(param.gain_to_aux0))
            }
            AUX23_SRC_GAIN_NAME => {
                self.state_read_src_param(elem_value, |param| Ok(param.gain_to_aux1))
            }
            SRC_MUTE_NAME => {
                elem_value.set_bool(&self.0.data.mutes);
                Ok(true)
            }
            OUT_DIM_NAME => self.state_read_out_pair(elem_value, |pair| Ok(pair.dim_enabled)),
            OUT_VOL_NAME => self.state_read_out_pair(elem_value, |pair| Ok(pair.vol)),
            OUT_DIM_VOL_NAME => self.state_read_out_pair(elem_value, |pair| Ok(pair.dim_vol)),
            REVERB_RETURN_MUTE_NAME => {
                elem_value.set_bool(&self.0.data.reverb_return_mute);
                Ok(true)
            }
            REVERB_RETURN_GAIN_NAME => {
                elem_value.set_int(&self.0.data.reverb_return_gain);
                Ok(true)
            }
            POST_FADER_NAME => {
                elem_value.set_bool(&self.0.data.post_fader);
                Ok(true)
            }
            CH_STRIP_AS_PLUGIN_NAME => {
                elem_value.set_bool(&self.0.data.ch_strip_as_plugin);
                Ok(true)
            }
            CH_STRIP_SRC_NAME => {
                let count = self.0.data.ch_strip_src.len();
                ElemValueAccessor::<u32>::set_vals(elem_value, count, |idx| {
                    let pos = Self::SRC_PAIR_ENTRIES
                        .iter()
                        .position(|s| self.0.data.ch_strip_src[idx].eq(s))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            CH_STRIP_23_AT_MID_RATE => {
                elem_value.set_bool(&[self.0.data.ch_strip_23_at_mid_rate]);
                Ok(true)
            }
            MIXER_ENABLE_NAME => {
                elem_value.set_bool(&[self.0.data.enabled]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn read_measured_elem(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MIXER_INPUT_METER_NAME => {
                elem_value.set_int(&self.1.data.src_inputs);
                Ok(true)
            }
            MIXER_OUTPUT_METER_NAME => {
                elem_value.set_int(&self.1.data.mixer_outputs);
                Ok(true)
            }
            AUX_OUTPUT_METER_NAME => {
                elem_value.set_int(&self.1.data.aux_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn parse_notification(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if self.0.has_segment_change(msg) {
            Studiok48Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }

    fn measure_states(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Studiok48Protocol::read_segment(req, &mut unit.1, &mut self.1, timeout_ms)
    }
}

#[derive(Default)]
struct PhysOutCtl(Studiok48PhysOutSegment, Vec<ElemId>);

const MASTER_OUT_DIM_NAME: &str = "master-out-dim";
const MASTER_OUT_VOL_NAME: &str = "master-out-volume";
const MASTER_OUT_DIM_VOL_NAME: &str = "master-out-dim-volume";

const OUT_STEREO_LINK_NAME: &str = "output-stereo-link";
const OUT_MUTE_NAME: &str = "output-mute";
const OUT_SRC_NAME: &str = "output-source";

const OUT_GRP_SELECT_NAME: &str = "output-group:select";
const OUT_GRP_SRC_ENABLE_NAME: &str = "output-group:source-enable";
const OUT_GRP_SRC_TRIM_NAME: &str = "output-group:source-trim";
const OUT_GRP_SRC_DELAY_NAME: &str = "output-group:source-delay";
const OUT_GRP_SRC_ASSIGN_NAME: &str = "output-group:source-assign";
const OUT_GRP_BASS_MANAGEMENT_NAME: &str = "output-group:bass-management";
const OUT_GRP_MAIN_CROSS_OVER_FREQ_NAME: &str = "output-group:main-cross-over-frequency";
const OUT_GRP_MAIN_LEVEL_TO_SUB_NAME: &str = "output-group:main-level-to-sub";
const OUT_GRP_SUB_LEVEL_TO_SUB_NAME: &str = "output-group:sub-level-to-sub";
const OUT_GRP_MAIN_FILTER_FOR_MAIN_NAME: &str = "output-group:main-filter-for-main";
const OUT_GRP_MAIN_FILTER_FOR_SUB_NAME: &str = "output-group:main-filter-for-sub";

fn src_pair_entry_to_string(entry: &SrcEntry) -> String {
    match entry {
        SrcEntry::Unused => "Unused".to_string(),
        SrcEntry::Analog(ch) => format!("Analog-{}", ch + 1),
        SrcEntry::Spdif(ch) => format!("S/PDIF-{}", ch + 1),
        SrcEntry::Adat(ch) => format!("ADAT-{}", ch + 1),
        SrcEntry::StreamA(ch) => format!("Stream-A-{}", ch + 1),
        SrcEntry::StreamB(ch) => format!("Stream-B-{}", ch + 1),
        SrcEntry::Mixer(ch) => {
            if *ch < 2 {
                format!("Mixer-{}", ch + 1)
            } else if *ch < 6 {
                format!("Aux-{}", ch - 1)
            } else {
                format!("Reverb-{}", ch - 5)
            }
        }
    }
}

fn cross_over_freq_to_string(freq: &CrossOverFreq) -> String {
    match freq {
        CrossOverFreq::F50 => "50Hz".to_string(),
        CrossOverFreq::F80 => "80Hz".to_string(),
        CrossOverFreq::F95 => "95Hz".to_string(),
        CrossOverFreq::F110 => "110Hz".to_string(),
        CrossOverFreq::F115 => "115Hz".to_string(),
        CrossOverFreq::F120 => "120Hz".to_string(),
        CrossOverFreq::Reserved(val) => format!("Reserved({})", val),
    }
}

fn high_pass_freq_to_string(freq: &HighPassFreq) -> String {
    match freq {
        HighPassFreq::Off => "Off".to_string(),
        HighPassFreq::Above12 => "12HzAbove".to_string(),
        HighPassFreq::Above24 => "24HzAbove".to_string(),
        HighPassFreq::Reserved(val) => format!("Reserved({})", val),
    }
}

fn low_pass_freq_to_string(freq: &LowPassFreq) -> String {
    match freq {
        LowPassFreq::Below12 => "12HzBelow".to_string(),
        LowPassFreq::Below24 => "24HzBelow".to_string(),
        LowPassFreq::Reserved(val) => format!("Reserved({})", val),
    }
}

impl PhysOutCtl {
    const PHYS_OUT_SRCS: [SrcEntry; 59] = [
        SrcEntry::Unused,
        SrcEntry::Analog(0),
        SrcEntry::Analog(1),
        SrcEntry::Analog(2),
        SrcEntry::Analog(3),
        SrcEntry::Analog(4),
        SrcEntry::Analog(5),
        SrcEntry::Analog(6),
        SrcEntry::Analog(7),
        SrcEntry::Analog(8),
        SrcEntry::Analog(9),
        SrcEntry::Analog(10),
        SrcEntry::Analog(11),
        SrcEntry::Spdif(0),
        SrcEntry::Spdif(1),
        SrcEntry::Adat(0),
        SrcEntry::Adat(1),
        SrcEntry::Adat(2),
        SrcEntry::Adat(3),
        SrcEntry::Adat(4),
        SrcEntry::Adat(5),
        SrcEntry::Adat(6),
        SrcEntry::Adat(7),
        SrcEntry::StreamA(0),
        SrcEntry::StreamA(1),
        SrcEntry::StreamA(2),
        SrcEntry::StreamA(3),
        SrcEntry::StreamA(4),
        SrcEntry::StreamA(5),
        SrcEntry::StreamA(6),
        SrcEntry::StreamA(7),
        SrcEntry::StreamA(8),
        SrcEntry::StreamA(9),
        SrcEntry::StreamA(10),
        SrcEntry::StreamA(11),
        SrcEntry::StreamA(12),
        SrcEntry::StreamA(13),
        SrcEntry::StreamA(14),
        SrcEntry::StreamA(15),
        SrcEntry::StreamB(0),
        SrcEntry::StreamB(1),
        SrcEntry::StreamB(2),
        SrcEntry::StreamB(3),
        SrcEntry::StreamB(4),
        SrcEntry::StreamB(5),
        SrcEntry::StreamB(6),
        SrcEntry::StreamB(7),
        SrcEntry::StreamB(8),
        SrcEntry::StreamB(9),
        SrcEntry::StreamB(10),
        SrcEntry::StreamB(11),
        SrcEntry::Mixer(0),
        SrcEntry::Mixer(1),
        SrcEntry::Mixer(2),
        SrcEntry::Mixer(3),
        SrcEntry::Mixer(4),
        SrcEntry::Mixer(5),
        SrcEntry::Mixer(6),
        SrcEntry::Mixer(7),
    ];

    const VOL_MIN: i32 = -1000;
    const VOL_MAX: i32 = 0;
    const VOL_STEP: i32 = 1;
    const VOL_TLV: DbInterval = DbInterval {
        min: -7200,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    const OUT_GRPS: [&'static str; 3] = ["Group-A", "Group-B", "Group-C"];

    const CROSS_OVER_FREQS: [CrossOverFreq; 6] = [
        CrossOverFreq::F50,
        CrossOverFreq::F80,
        CrossOverFreq::F95,
        CrossOverFreq::F110,
        CrossOverFreq::F115,
        CrossOverFreq::F120,
    ];

    const HIGH_PASS_FREQS: [HighPassFreq; 3] = [
        HighPassFreq::Off,
        HighPassFreq::Above12,
        HighPassFreq::Above24,
    ];

    const LOW_PASS_FREQS: [LowPassFreq; 2] = [LowPassFreq::Below12, LowPassFreq::Below24];

    const TRIM_MIN: i32 = -20;
    const TRIM_MAX: i32 = 0;
    const TRIM_STEP: i32 = 1;

    const DELAY_MIN: i32 = 0;
    const DELAY_MAX: i32 = 30;
    const DELAY_STEP: i32 = 1;

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Studiok48Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)?;

        // For master output.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MASTER_OUT_DIM_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MASTER_OUT_VOL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::VOL_MIN,
                Self::VOL_MAX,
                Self::VOL_STEP,
                1,
                Some(&Into::<Vec<u32>>::into(Self::VOL_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MASTER_OUT_DIM_VOL_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::VOL_MIN,
                Self::VOL_MAX,
                Self::VOL_STEP,
                1,
                Some(&Into::<Vec<u32>>::into(Self::VOL_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        // For source of output pair.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_STEREO_LINK_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, STUDIO_PHYS_OUT_PAIR_COUNT, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_MUTE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, STUDIO_PHYS_OUT_PAIR_COUNT * 2, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let labels: Vec<String> = Self::PHYS_OUT_SRCS
            .iter()
            .map(|src| src_pair_entry_to_string(src))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_SRC_NAME, 0);
        card_cntr
            .add_enum_elems(
                &elem_id,
                1,
                STUDIO_PHYS_OUT_PAIR_COUNT * 2,
                &labels,
                None,
                true,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        // For output group.
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_GRP_SELECT_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &Self::OUT_GRPS, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_GRP_SRC_ENABLE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, STUDIO_PHYS_OUT_PAIR_COUNT * 2, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_GRP_SRC_TRIM_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::TRIM_MIN,
                Self::TRIM_MAX,
                Self::TRIM_STEP,
                STUDIO_PHYS_OUT_PAIR_COUNT * 2,
                None,
                true,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_GRP_SRC_DELAY_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::DELAY_MIN,
                Self::DELAY_MAX,
                Self::DELAY_STEP,
                STUDIO_PHYS_OUT_PAIR_COUNT * 2,
                None,
                true,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_GRP_SRC_ASSIGN_NAME, 0);
        card_cntr
            .add_bool_elems(
                &elem_id,
                STUDIO_OUTPUT_GROUP_COUNT,
                STUDIO_PHYS_OUT_PAIR_COUNT * 2,
                true,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_GRP_BASS_MANAGEMENT_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, STUDIO_OUTPUT_GROUP_COUNT, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let labels: Vec<String> = Self::CROSS_OVER_FREQS
            .iter()
            .map(|src| cross_over_freq_to_string(src))
            .collect();
        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Card,
            0,
            0,
            OUT_GRP_MAIN_CROSS_OVER_FREQ_NAME,
            0,
        );
        card_cntr
            .add_enum_elems(&elem_id, 1, STUDIO_OUTPUT_GROUP_COUNT, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_GRP_MAIN_LEVEL_TO_SUB_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::VOL_MIN,
                Self::VOL_MAX,
                Self::VOL_STEP,
                STUDIO_OUTPUT_GROUP_COUNT,
                Some(&Into::<Vec<u32>>::into(Self::VOL_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_GRP_SUB_LEVEL_TO_SUB_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::VOL_MIN,
                Self::VOL_MAX,
                Self::VOL_STEP,
                STUDIO_OUTPUT_GROUP_COUNT,
                Some(&Into::<Vec<u32>>::into(Self::VOL_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let labels: Vec<String> = Self::HIGH_PASS_FREQS
            .iter()
            .map(|src| high_pass_freq_to_string(src))
            .collect();
        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Card,
            0,
            0,
            OUT_GRP_MAIN_FILTER_FOR_MAIN_NAME,
            0,
        );
        card_cntr
            .add_enum_elems(&elem_id, 1, STUDIO_OUTPUT_GROUP_COUNT, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let labels: Vec<String> = Self::LOW_PASS_FREQS
            .iter()
            .map(|src| low_pass_freq_to_string(src))
            .collect();
        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Card,
            0,
            0,
            OUT_GRP_MAIN_FILTER_FOR_SUB_NAME,
            0,
        );
        card_cntr
            .add_enum_elems(&elem_id, 1, STUDIO_OUTPUT_GROUP_COUNT, &labels, None, true)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MASTER_OUT_DIM_NAME => ElemValueAccessor::<bool>::set_val(elem_value, || {
                Ok(self.0.data.master_out.dim_enabled)
            })
            .map(|_| true),
            MASTER_OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || Ok(self.0.data.master_out.vol))
                    .map(|_| true)
            }
            MASTER_OUT_DIM_VOL_NAME => {
                ElemValueAccessor::<i32>::set_val(elem_value, || Ok(self.0.data.master_out.dim_vol))
                    .map(|_| true)
            }
            OUT_STEREO_LINK_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, STUDIO_PHYS_OUT_PAIR_COUNT, |idx| {
                    Ok(self.0.data.out_pair_srcs[idx].stereo_link)
                })
                .map(|_| true)
            }
            OUT_MUTE_NAME => {
                elem_value.set_bool(&self.0.data.out_mutes);
                Ok(true)
            }
            OUT_SRC_NAME => self
                .read_out_src_param(elem_value, |param| {
                    let pos = Self::PHYS_OUT_SRCS
                        .iter()
                        .position(|s| s.eq(&param.src))
                        .expect("Programming error");
                    Ok(pos as u32)
                })
                .map(|_| true),
            OUT_GRP_SELECT_NAME => {
                elem_value.set_enum(&[self.0.data.selected_out_grp as u32]);
                Ok(true)
            }
            OUT_GRP_SRC_ENABLE_NAME => {
                elem_value.set_bool(&self.0.data.out_assign_to_grp);
                Ok(true)
            }
            OUT_GRP_SRC_TRIM_NAME => self.read_out_src_param(elem_value, |param| Ok(param.vol)),
            OUT_GRP_SRC_DELAY_NAME => self.read_out_src_param(elem_value, |param| Ok(param.delay)),
            OUT_GRP_SRC_ASSIGN_NAME => {
                let index = elem_id.get_index() as usize;
                elem_value.set_bool(&self.0.data.out_grps[index].assigned_phys_outs);
                Ok(true)
            }
            OUT_GRP_BASS_MANAGEMENT_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, STUDIO_OUTPUT_GROUP_COUNT, |idx| {
                    Ok(self.0.data.out_grps[idx].bass_management)
                })
                .map(|_| true)
            }
            OUT_GRP_MAIN_CROSS_OVER_FREQ_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, STUDIO_OUTPUT_GROUP_COUNT, |idx| {
                    let pos = Self::CROSS_OVER_FREQS
                        .iter()
                        .position(|freq| freq.eq(&self.0.data.out_grps[idx].main_cross_over_freq))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            OUT_GRP_MAIN_LEVEL_TO_SUB_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, STUDIO_OUTPUT_GROUP_COUNT, |idx| {
                    Ok(self.0.data.out_grps[idx].main_level_to_sub)
                })
                .map(|_| true)
            }
            OUT_GRP_SUB_LEVEL_TO_SUB_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, STUDIO_OUTPUT_GROUP_COUNT, |idx| {
                    Ok(self.0.data.out_grps[idx].sub_level_to_sub)
                })
                .map(|_| true)
            }
            OUT_GRP_MAIN_FILTER_FOR_MAIN_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, STUDIO_OUTPUT_GROUP_COUNT, |idx| {
                    let pos = Self::HIGH_PASS_FREQS
                        .iter()
                        .position(|freq| freq.eq(&self.0.data.out_grps[idx].main_filter_for_main))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            OUT_GRP_MAIN_FILTER_FOR_SUB_NAME => {
                ElemValueAccessor::<u32>::set_vals(elem_value, STUDIO_OUTPUT_GROUP_COUNT, |idx| {
                    let pos = Self::LOW_PASS_FREQS
                        .iter()
                        .position(|freq| freq.eq(&self.0.data.out_grps[idx].main_filter_for_sub))
                        .unwrap();
                    Ok(pos as u32)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn read_out_src_param<T, F>(&self, elem_value: &mut ElemValue, cb: F) -> Result<bool, Error>
    where
        F: Fn(&PhysOutSrcParam) -> Result<T, Error>,
        T: Default + Copy + Eq,
        ElemValue: ElemValueAccessor<T>,
    {
        ElemValueAccessor::<T>::set_vals(elem_value, STUDIO_PHYS_OUT_PAIR_COUNT * 2, |idx| {
            let i = idx / 2;
            let ch = idx % 2;
            let param = if ch == 0 {
                &self.0.data.out_pair_srcs[i].left
            } else {
                &self.0.data.out_pair_srcs[i].right
            };
            cb(param)
        })
        .map(|_| true)
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MASTER_OUT_DIM_NAME => {
                ElemValueAccessor::<bool>::get_val(new, |val| {
                    self.0.data.master_out.dim_enabled = val;
                    Ok(())
                })?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MASTER_OUT_VOL_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    self.0.data.master_out.vol = val;
                    Ok(())
                })?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            MASTER_OUT_DIM_VOL_NAME => {
                ElemValueAccessor::<i32>::get_val(new, |val| {
                    self.0.data.master_out.dim_vol = val;
                    Ok(())
                })?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            OUT_STEREO_LINK_NAME => {
                ElemValueAccessor::<bool>::get_vals(
                    new,
                    old,
                    STUDIO_PHYS_OUT_PAIR_COUNT,
                    |idx, val| {
                        self.0.data.out_pair_srcs[idx].stereo_link = val;
                        Ok(())
                    },
                )?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            OUT_MUTE_NAME => {
                new.get_bool(&mut self.0.data.out_mutes);
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            OUT_SRC_NAME => {
                self.write_out_src_param(unit, req, new, old, timeout_ms, |param, val: u32| {
                    Self::PHYS_OUT_SRCS
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg =
                                format!("Invalid value for index of source of output: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| param.src = s)
                })
            }
            OUT_GRP_SELECT_NAME => {
                let mut vals = [0];
                new.get_enum(&mut vals);
                self.0.data.selected_out_grp = vals[0] as usize;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            OUT_GRP_SRC_ENABLE_NAME => {
                new.get_bool(&mut self.0.data.out_assign_to_grp);
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            OUT_GRP_SRC_TRIM_NAME => {
                self.write_out_src_param(unit, req, new, old, timeout_ms, |param, val| {
                    param.vol = val;
                    Ok(())
                })
            }
            OUT_GRP_SRC_DELAY_NAME => {
                self.write_out_src_param(unit, req, new, old, timeout_ms, |param, val| {
                    param.delay = val;
                    Ok(())
                })
            }
            OUT_GRP_SRC_ASSIGN_NAME => {
                let mut vals = [false; STUDIO_PHYS_OUT_PAIR_COUNT * 2];
                new.get_bool(&mut vals);
                let count = vals.iter().filter(|&v| *v).count();
                if count > STUDIO_MAX_SURROUND_CHANNELS {
                    let msg = format!(
                        "Maximum {} channels are supported for surround channels, but {} given",
                        STUDIO_MAX_SURROUND_CHANNELS, count
                    );
                    Err(Error::new(FileError::Inval, &msg))?;
                }
                let index = elem_id.get_index() as usize;
                self.0.data.out_grps[index]
                    .assigned_phys_outs
                    .copy_from_slice(&vals);
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            OUT_GRP_BASS_MANAGEMENT_NAME => {
                ElemValueAccessor::<bool>::get_vals(
                    new,
                    old,
                    STUDIO_OUTPUT_GROUP_COUNT,
                    |idx, val| {
                        self.0.data.out_grps[idx].bass_management = val;
                        Ok(())
                    },
                )?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            OUT_GRP_MAIN_CROSS_OVER_FREQ_NAME => {
                ElemValueAccessor::<u32>::get_vals(
                    new,
                    old,
                    STUDIO_OUTPUT_GROUP_COUNT,
                    |idx, val| {
                        Self::CROSS_OVER_FREQS
                            .iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!(
                                    "Invalid value for index of cross over frequency: {}",
                                    val
                                );
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&freq| self.0.data.out_grps[idx].main_cross_over_freq = freq)
                    },
                )?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            OUT_GRP_MAIN_LEVEL_TO_SUB_NAME => {
                ElemValueAccessor::<i32>::get_vals(
                    new,
                    old,
                    STUDIO_OUTPUT_GROUP_COUNT,
                    |idx, val| {
                        self.0.data.out_grps[idx].main_level_to_sub = val;
                        Ok(())
                    },
                )?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            OUT_GRP_SUB_LEVEL_TO_SUB_NAME => {
                ElemValueAccessor::<i32>::get_vals(
                    new,
                    old,
                    STUDIO_OUTPUT_GROUP_COUNT,
                    |idx, val| {
                        self.0.data.out_grps[idx].sub_level_to_sub = val;
                        Ok(())
                    },
                )?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            OUT_GRP_MAIN_FILTER_FOR_MAIN_NAME => {
                ElemValueAccessor::<u32>::get_vals(
                    new,
                    old,
                    STUDIO_OUTPUT_GROUP_COUNT,
                    |idx, val| {
                        Self::HIGH_PASS_FREQS
                            .iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!(
                                    "Invalid value for index of high pass frequency: {}",
                                    val
                                );
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&freq| self.0.data.out_grps[idx].main_filter_for_main = freq)
                    },
                )?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            OUT_GRP_MAIN_FILTER_FOR_SUB_NAME => {
                ElemValueAccessor::<u32>::get_vals(
                    new,
                    old,
                    STUDIO_OUTPUT_GROUP_COUNT,
                    |idx, val| {
                        Self::LOW_PASS_FREQS
                            .iter()
                            .nth(val as usize)
                            .ok_or_else(|| {
                                let msg = format!(
                                    "Invalid value for index of low pass frequency: {}",
                                    val
                                );
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&freq| self.0.data.out_grps[idx].main_filter_for_sub = freq)
                    },
                )?;
                Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write_out_src_param<T, F>(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        new: &ElemValue,
        old: &ElemValue,
        timeout_ms: u32,
        cb: F,
    ) -> Result<bool, Error>
    where
        F: Fn(&mut PhysOutSrcParam, T) -> Result<(), Error>,
        T: Default + Copy + Eq,
        ElemValue: ElemValueAccessor<T>,
    {
        ElemValueAccessor::<T>::get_vals(new, old, STUDIO_PHYS_OUT_PAIR_COUNT * 2, |idx, val| {
            let i = idx / 2;
            let ch = idx % 2;
            let param = if ch == 0 {
                &mut self.0.data.out_pair_srcs[i].left
            } else {
                &mut self.0.data.out_pair_srcs[i].right
            };
            cb(param, val)
        })?;
        Studiok48Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms).map(|_| true)
    }

    fn parse_notification(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if self.0.has_segment_change(msg) {
            Studiok48Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }
}

#[derive(Default)]
struct ChStripCtl(
    Studiok48ChStripStatesSegment,
    Studiok48ChStripMetersSegment,
    Vec<ElemId>,
    Vec<ElemId>,
);

impl ChStripCtlOperation<StudioChStripStates, StudioChStripMeters, Studiok48Protocol>
    for ChStripCtl
{
    fn states_segment(&self) -> &Studiok48ChStripStatesSegment {
        &self.0
    }

    fn states_segment_mut(&mut self) -> &mut Studiok48ChStripStatesSegment {
        &mut self.0
    }

    fn meters_segment_mut(&mut self) -> &mut Studiok48ChStripMetersSegment {
        &mut self.1
    }

    fn states(&self) -> &[ChStripState] {
        &self.0.data.0
    }

    fn states_mut(&mut self) -> &mut [ChStripState] {
        &mut self.0.data.0
    }

    fn meters(&self) -> &[ChStripMeter] {
        &self.1.data.0
    }
}

#[derive(Default)]
struct ReverbCtl(
    Studiok48ReverbStateSegment,
    Studiok48ReverbMeterSegment,
    Vec<ElemId>,
    Vec<ElemId>,
);

impl ReverbCtlOperation<StudioReverbState, StudioReverbMeter, Studiok48Protocol> for ReverbCtl {
    fn state_segment(&self) -> &Studiok48ReverbStateSegment {
        &self.0
    }

    fn state_segment_mut(&mut self) -> &mut Studiok48ReverbStateSegment {
        &mut self.0
    }

    fn meter_segment_mut(&mut self) -> &mut Studiok48ReverbMeterSegment {
        &mut self.1
    }

    fn state(&self) -> &ReverbState {
        &self.0.data.0
    }

    fn state_mut(&mut self) -> &mut ReverbState {
        &mut self.0.data.0
    }

    fn meter(&self) -> &ReverbMeter {
        &self.1.data.0
    }
}

fn analog_jack_state_to_str(state: &StudioAnalogJackState) -> &'static str {
    match state {
        StudioAnalogJackState::FrontSelected => "front-selected",
        StudioAnalogJackState::FrontInserted => "front-inserted",
        StudioAnalogJackState::RearSelected => "rear-selected",
        StudioAnalogJackState::RearInserted => "rear-inserted",
    }
}

#[derive(Default)]
struct HwStateCtl(Studiok48HwStateSegment, Vec<ElemId>);

impl FirewireLedCtlOperation<StudioHwState, Studiok48Protocol> for HwStateCtl {
    fn segment_mut(&mut self) -> &mut Studiok48HwStateSegment {
        &mut self.0
    }

    fn firewire_led(&self) -> &FireWireLedState {
        &self.0.data.firewire_led
    }

    fn firewire_led_mut(&mut self) -> &mut FireWireLedState {
        &mut self.0.data.firewire_led
    }
}

// TODO: For Jack detection in ALSA applications.
const ANALOG_JACK_STATE_NAME: &str = "analog-jack-state";
const HP_JACK_STATE_NAME: &str = "headphone-jack-state";
const VALID_MASTER_LEVEL_NAME: &str = "valid-master-level";

impl HwStateCtl {
    const ANALOG_JACK_STATES: [StudioAnalogJackState; 4] = [
        StudioAnalogJackState::FrontSelected,
        StudioAnalogJackState::FrontInserted,
        StudioAnalogJackState::RearSelected,
        StudioAnalogJackState::RearInserted,
    ];

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Studiok48Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)?;

        self.load_firewire_led(card_cntr)?;

        let labels = Self::ANALOG_JACK_STATES
            .iter()
            .map(|s| analog_jack_state_to_str(s))
            .collect::<Vec<_>>();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, ANALOG_JACK_STATE_NAME, 0);
        card_cntr
            .add_enum_elems(
                &elem_id,
                1,
                STUDIO_ANALOG_JACK_STATE_COUNT,
                &labels,
                None,
                false,
            )
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, HP_JACK_STATE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 2, false)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, VALID_MASTER_LEVEL_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_firewire_led(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        self.write_firewire_led(&mut unit.0, req, elem_id, elem_value, timeout_ms)
    }

    fn read_notified_elem(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            ANALOG_JACK_STATE_NAME => ElemValueAccessor::<u32>::set_vals(
                elem_value,
                STUDIO_ANALOG_JACK_STATE_COUNT,
                |idx| {
                    let pos = Self::ANALOG_JACK_STATES
                        .iter()
                        .position(|s| self.0.data.analog_jack_states[idx].eq(s))
                        .unwrap();
                    Ok(pos as u32)
                },
            )
            .map(|_| true),
            HP_JACK_STATE_NAME => {
                elem_value.set_bool(&self.0.data.hp_state);
                Ok(true)
            }
            VALID_MASTER_LEVEL_NAME => {
                elem_value.set_bool(&[self.0.data.valid_master_level]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn parse_notification(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if self.0.has_segment_change(msg) {
            Studiok48Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }
}
