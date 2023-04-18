// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, protocols::tcelectronic::studio::*};

#[derive(Default, Debug)]
pub struct Studiok48Model {
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl<Studiok48Protocol>,
    lineout_ctl: LineoutCtl,
    remote_ctl: RemoteCtl,
    config_ctl: ConfigCtl,
    mixer_state_ctl: MixerStateCtl,
    mixer_meter_ctl: MixerMeterCtl,
    phys_out_ctl: PhysOutCtl,
    ch_strip_state_ctl: ChStripStateCtl,
    ch_strip_meter_ctl: ChStripMeterCtl,
    reverb_state_ctl: ReverbStateCtl,
    reverb_meter_ctl: ReverbMeterCtl,
    hw_state_ctl: HwStateCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<(SndDice, FwNode)> for Studiok48Model {
    fn cache(&mut self, (_, node): &mut (SndDice, FwNode)) -> Result<(), Error> {
        Studiok48Protocol::read_general_sections(
            &mut self.req,
            &node,
            &mut self.sections,
            TIMEOUT_MS,
        )?;

        self.common_ctl
            .cache_whole_params(&mut self.req, node, &mut self.sections, TIMEOUT_MS)?;

        self.lineout_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.remote_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.config_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.mixer_state_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.mixer_meter_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.phys_out_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.ch_strip_state_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.ch_strip_meter_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.reverb_state_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.reverb_meter_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.hw_state_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr)?;

        self.lineout_ctl.load(card_cntr)?;
        self.remote_ctl.load(card_cntr)?;
        self.config_ctl.load(card_cntr)?;
        self.mixer_state_ctl.load(card_cntr)?;
        self.mixer_meter_ctl.load(card_cntr)?;
        self.phys_out_ctl.load(card_cntr)?;

        self.reverb_state_ctl
            .load(card_cntr)
            .map(|notified_elem_id_list| self.reverb_state_ctl.1 = notified_elem_id_list)?;
        self.reverb_meter_ctl
            .load(card_cntr)
            .map(|measured_elem_id_list| self.reverb_meter_ctl.1 = measured_elem_id_list)?;
        self.ch_strip_state_ctl
            .load(card_cntr)
            .map(|notified_elem_id_list| self.ch_strip_state_ctl.1 = notified_elem_id_list)?;
        self.ch_strip_meter_ctl
            .load(card_cntr)
            .map(|measured_elem_id_list| {
                self.ch_strip_meter_ctl.1 = measured_elem_id_list;
            })?;

        self.hw_state_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.lineout_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.remote_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.config_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_state_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_out_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_state_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_state_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_state_ctl.read(elem_id, elem_value)? {
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
            &mut self.req,
            node,
            &mut self.sections,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .lineout_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .remote_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .config_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.mixer_state_ctl.write(
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .phys_out_ctl
            .write(&mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.reverb_state_ctl.write(
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.ch_strip_state_ctl.write(
            &mut self.req,
            node,
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
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for Studiok48Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_id_list);
        elem_id_list.extend_from_slice(&self.lineout_ctl.1);
        elem_id_list.extend_from_slice(&self.remote_ctl.1);
        elem_id_list.extend_from_slice(&self.config_ctl.1);
        elem_id_list.extend_from_slice(&self.mixer_state_ctl.1);
        elem_id_list.extend_from_slice(&self.phys_out_ctl.1);
        elem_id_list.extend_from_slice(&self.reverb_state_ctl.1);
        elem_id_list.extend_from_slice(&self.ch_strip_state_ctl.1);
        elem_id_list.extend_from_slice(&self.hw_state_ctl.1);
    }

    fn parse_notification(
        &mut self,
        (_, node): &mut (SndDice, FwNode),
        &msg: &u32,
    ) -> Result<(), Error> {
        self.common_ctl
            .parse_notification(&self.req, node, &mut self.sections, msg, TIMEOUT_MS)?;
        self.lineout_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        self.remote_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        self.config_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        self.mixer_state_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        self.phys_out_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        self.reverb_state_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        self.ch_strip_state_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        self.hw_state_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;

        Ok(())
    }
}

impl MeasureModel<(SndDice, FwNode)> for Studiok48Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.measured_elem_id_list);
        elem_id_list.extend_from_slice(&self.mixer_meter_ctl.1);
        elem_id_list.extend_from_slice(&self.reverb_meter_ctl.1);
        elem_id_list.extend_from_slice(&self.ch_strip_meter_ctl.1);
    }

    fn measure_states(&mut self, (_, node): &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .cache_partial_params(&self.req, node, &mut self.sections, TIMEOUT_MS)?;
        self.mixer_meter_ctl.cache(&self.req, node, TIMEOUT_MS)?;
        if !self.reverb_state_ctl.is_bypassed() {
            self.reverb_meter_ctl.cache(&self.req, node, TIMEOUT_MS)?;
        }
        if !self.ch_strip_state_ctl.are_bypassed() {
            self.ch_strip_meter_ctl.cache(&self.req, node, TIMEOUT_MS)?;
        }
        Ok(())
    }
}

fn nominal_signal_level_to_str(level: &NominalSignalLevel) -> &'static str {
    match level {
        NominalSignalLevel::Professional => "+4dBu",
        NominalSignalLevel::Consumer => "-10dBV",
    }
}

#[derive(Default, Debug)]
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

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = Studiok48Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
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

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            LINE_OUT_45_LEVEL_NAME => Self::read_as_index(elem_value, self.0.data.line_45),
            LINE_OUT_67_LEVEL_NAME => Self::read_as_index(elem_value, self.0.data.line_67),
            LINE_OUT_89_LEVEL_NAME => Self::read_as_index(elem_value, self.0.data.line_89),
            LINE_OUT_1011_LEVEL_NAME => Self::read_as_index(elem_value, self.0.data.line_1011),
            _ => Ok(false),
        }
    }

    fn read_as_index(elem_value: &mut ElemValue, level: NominalSignalLevel) -> Result<bool, Error> {
        let pos = Self::NOMINAL_SIGNAL_LEVELS
            .iter()
            .position(|l| level.eq(&l))
            .unwrap();
        elem_value.set_enum(&[pos as u32]);
        Ok(true)
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
            LINE_OUT_45_LEVEL_NAME => {
                self.write_as_index(req, node, elem_value, timeout_ms, |data, level| {
                    data.line_45 = level
                })
            }
            LINE_OUT_67_LEVEL_NAME => {
                self.write_as_index(req, node, elem_value, timeout_ms, |data, level| {
                    data.line_67 = level
                })
            }
            LINE_OUT_89_LEVEL_NAME => {
                self.write_as_index(req, node, elem_value, timeout_ms, |data, level| {
                    data.line_89 = level
                })
            }
            LINE_OUT_1011_LEVEL_NAME => {
                self.write_as_index(req, node, elem_value, timeout_ms, |data, level| {
                    data.line_1011 = level
                })
            }
            _ => Ok(false),
        }
    }

    fn write_as_index<F>(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_value: &ElemValue,
        timeout_ms: u32,
        cb: F,
    ) -> Result<bool, Error>
    where
        F: Fn(&mut StudioLineOutLevel, NominalSignalLevel),
    {
        let mut params = self.0.data.clone();
        let pos = elem_value.enumerated()[0] as usize;
        Self::NOMINAL_SIGNAL_LEVELS
            .iter()
            .nth(pos)
            .ok_or_else(|| {
                let msg = format!("Invalid index of nominal level: {}", pos);
                Error::new(FileError::Inval, &msg)
            })
            .map(|&l| cb(&mut params, l))?;
        let res =
            Studiok48Protocol::update_partial_segment(req, node, &params, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res.map(|_| true)
    }

    fn parse_notification(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if Studiok48Protocol::is_notified_segment(&self.0, msg) {
            let res = Studiok48Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct RemoteCtl(Studiok48RemoteSegment, Vec<ElemId>);

impl ProgramCtlOperation<StudioRemote, Studiok48Protocol> for RemoteCtl {
    fn segment(&self) -> &Studiok48RemoteSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut Studiok48RemoteSegment {
        &mut self.0
    }

    fn prog(params: &StudioRemote) -> &TcKonnektLoadedProgram {
        &params.prog
    }

    fn prog_mut(params: &mut StudioRemote) -> &mut TcKonnektLoadedProgram {
        &mut params.prog
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

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = Studiok48Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.load_prog(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let labels: Vec<String> = MixerStateCtl::SRC_PAIR_ENTRIES
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
        match elem_id.name().as_str() {
            USER_ASSIGN_NAME => {
                let params = &self.0.data;
                let vals: Vec<u32> = params
                    .user_assigns
                    .iter()
                    .map(|assign| {
                        let pos = MixerStateCtl::SRC_PAIR_ENTRIES
                            .iter()
                            .position(|a| assign.eq(a))
                            .unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            EFFECT_BUTTON_MODE_NAME => {
                let params = &self.0.data;
                let pos = Self::EFFECT_BUTTON_MODES
                    .iter()
                    .position(|m| params.effect_button_mode.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            FALLBACK_TO_MASTER_ENABLE_NAME => {
                let params = &self.0.data;
                elem_value.set_bool(&[params.fallback_to_master_enable]);
                Ok(true)
            }
            FALLBACK_TO_MASTER_DURATION_NAME => {
                let params = &self.0.data;
                elem_value.set_int(&[params.fallback_to_master_duration as i32]);
                Ok(true)
            }
            KNOB_PUSH_MODE_NAME => {
                let params = &self.0.data;
                let pos = Self::KNOB_PUSH_MODES
                    .iter()
                    .position(|m| params.knob_push_mode.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => self.read_prog(elem_id, elem_value),
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
            USER_ASSIGN_NAME => {
                let mut params = self.0.data.clone();
                params
                    .user_assigns
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(assign, &val)| {
                        let pos = val as usize;
                        MixerStateCtl::SRC_PAIR_ENTRIES
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg =
                                    format!("Invalid index of source of user assignment: {}", val);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&s| *assign = s)
                    })?;
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            EFFECT_BUTTON_MODE_NAME => {
                let mut params = self.0.data.clone();
                let val = elem_value.enumerated()[0] as usize;
                Self::EFFECT_BUTTON_MODES
                    .iter()
                    .nth(val)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of source of user assignment: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&m| params.effect_button_mode = m)?;
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            FALLBACK_TO_MASTER_ENABLE_NAME => {
                let mut params = self.0.data.clone();
                params.fallback_to_master_enable = elem_value.boolean()[0];
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            FALLBACK_TO_MASTER_DURATION_NAME => {
                let mut params = self.0.data.clone();
                params.fallback_to_master_duration = elem_value.int()[0] as u32;
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            KNOB_PUSH_MODE_NAME => {
                let mut params = self.0.data.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::KNOB_PUSH_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index of source of user assignment: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&m| params.knob_push_mode = m)?;
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            _ => self.write_prog(req, node, elem_id, elem_value, timeout_ms),
        }
    }

    fn parse_notification(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if Studiok48Protocol::is_notified_segment(&self.0, msg) {
            let res = Studiok48Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
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
    fn segment(&self) -> &Studiok48ConfigSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut Studiok48ConfigSegment {
        &mut self.0
    }

    fn standalone_rate(params: &StudioConfig) -> &TcKonnektStandaloneClockRate {
        &params.standalone_rate
    }

    fn standalone_rate_mut(params: &mut StudioConfig) -> &mut TcKonnektStandaloneClockRate {
        &mut params.standalone_rate
    }
}

impl MidiSendCtlOperation<StudioConfig, Studiok48Protocol> for ConfigCtl {
    fn segment(&self) -> &Studiok48ConfigSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut Studiok48ConfigSegment {
        &mut self.0
    }

    fn midi_sender(params: &StudioConfig) -> &TcKonnektMidiSender {
        &params.midi_send
    }

    fn midi_sender_mut(params: &mut StudioConfig) -> &mut TcKonnektMidiSender {
        &mut params.midi_send
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

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = Studiok48Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
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
            match elem_id.name().as_str() {
                OPT_IFACE_MODE_NAME => {
                    let params = &self.0.data;
                    let pos = Self::OPT_IFACE_MODES
                        .iter()
                        .position(|m| params.opt_iface_mode.eq(m))
                        .unwrap();
                    elem_value.set_enum(&[pos as u32]);
                    Ok(true)
                }
                STANDALONE_CLK_SRC_NAME => {
                    let params = &self.0.data;
                    let pos = Self::STANDALONE_CLK_SRCS
                        .iter()
                        .position(|s| params.standalone_src.eq(s))
                        .unwrap();
                    elem_value.set_enum(&[pos as u32]);
                    Ok(true)
                }
                CLOCK_RECOVERY_NAME => {
                    let params = &self.0.data;
                    elem_value.set_bool(&[params.clock_recovery]);
                    Ok(true)
                }
                _ => Ok(false),
            }
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
        if self.write_standalone_rate(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else if self.write_midi_sender(req, node, elem_id, elem_value, timeout_ms)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                OPT_IFACE_MODE_NAME => {
                    let mut params = self.0.data.clone();
                    let pos = elem_value.enumerated()[0] as usize;
                    Self::OPT_IFACE_MODES
                        .iter()
                        .nth(pos)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index of standalone clock source: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&m| params.opt_iface_mode = m)?;
                    let res = Studiok48Protocol::update_partial_segment(
                        req,
                        node,
                        &params,
                        &mut self.0,
                        timeout_ms,
                    );
                    debug!(params = ?self.0.data, ?res);
                    res.map(|_| true)
                }
                STANDALONE_CLK_SRC_NAME => {
                    let mut params = self.0.data.clone();
                    let pos = elem_value.enumerated()[0] as usize;
                    Self::STANDALONE_CLK_SRCS
                        .iter()
                        .nth(pos)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index of standalone clock source: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| params.standalone_src = s)?;
                    let res = Studiok48Protocol::update_partial_segment(
                        req,
                        node,
                        &params,
                        &mut self.0,
                        timeout_ms,
                    );
                    debug!(params = ?self.0.data, ?res);
                    res.map(|_| true)
                }
                CLOCK_RECOVERY_NAME => {
                    let mut params = self.0.data.clone();
                    params.clock_recovery = elem_value.boolean()[0];
                    let res = Studiok48Protocol::update_partial_segment(
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
        if Studiok48Protocol::is_notified_segment(&self.0, msg) {
            let res = Studiok48Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct MixerStateCtl(Studiok48MixerStateSegment, Vec<ElemId>);

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

fn mixer_monitor_src_pairs_iter(state: &StudioMixerState) -> impl Iterator<Item = &MonitorSrcPair> {
    state.src_pairs.iter()
}

fn mixer_monitor_src_pairs_iter_mut(
    state: &mut StudioMixerState,
) -> impl Iterator<Item = &mut MonitorSrcPair> {
    state.src_pairs.iter_mut()
}

fn mixer_monitor_src_params_iter(
    state: &StudioMixerState,
) -> impl Iterator<Item = &MonitorSrcParam> {
    mixer_monitor_src_pairs_iter(state).flat_map(|pair| pair.params.iter())
}

fn mixer_monitor_src_params_iter_mut(
    state: &mut StudioMixerState,
) -> impl Iterator<Item = &mut MonitorSrcParam> {
    mixer_monitor_src_pairs_iter_mut(state).flat_map(|pair| pair.params.iter_mut())
}

impl MixerStateCtl {
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

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = Studiok48Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
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
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))
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
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))
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
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))
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
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            SRC_PAIR_MODE_NAME => {
                let params = &self.0.data;
                let vals: Vec<u32> = mixer_monitor_src_pairs_iter(&params)
                    .map(|pair| {
                        let pos = Self::SRC_PAIR_MODES
                            .iter()
                            .position(|m| pair.mode.eq(m))
                            .unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            SRC_STEREO_LINK_NAME => {
                let params = &self.0.data;
                let vals: Vec<bool> = mixer_monitor_src_pairs_iter(&params)
                    .map(|pair| pair.stereo_link)
                    .collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            SRC_ENTRY_NAME => {
                let vals: Vec<u32> = mixer_monitor_src_params_iter(&self.0.data)
                    .map(|param| {
                        Self::SRC_PAIR_ENTRIES
                            .iter()
                            .position(|m| param.src.eq(m))
                            .map(|pos| pos as u32)
                            .unwrap()
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            SRC_GAIN_NAME => {
                let vals: Vec<i32> = mixer_monitor_src_params_iter(&self.0.data)
                    .map(|param| param.gain_to_main)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            SRC_PAN_NAME => {
                let vals: Vec<i32> = mixer_monitor_src_params_iter(&self.0.data)
                    .map(|param| param.pan_to_main)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            REVERB_SRC_GAIN_NAME => {
                let vals: Vec<i32> = mixer_monitor_src_params_iter(&self.0.data)
                    .map(|param| param.gain_to_reverb)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            AUX01_SRC_GAIN_NAME => {
                let vals: Vec<i32> = mixer_monitor_src_params_iter(&self.0.data)
                    .map(|param| param.gain_to_aux0)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            AUX23_SRC_GAIN_NAME => {
                let vals: Vec<i32> = mixer_monitor_src_params_iter(&self.0.data)
                    .map(|param| param.gain_to_aux1)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            SRC_MUTE_NAME => {
                elem_value.set_bool(&self.0.data.mutes);
                Ok(true)
            }
            OUT_DIM_NAME => {
                let vals: Vec<bool> = self
                    .0
                    .data
                    .mixer_out
                    .iter()
                    .map(|pair| pair.dim_enabled)
                    .collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            OUT_VOL_NAME => {
                let vals: Vec<i32> = self.0.data.mixer_out.iter().map(|pair| pair.vol).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            OUT_DIM_VOL_NAME => {
                let vals: Vec<i32> = self
                    .0
                    .data
                    .mixer_out
                    .iter()
                    .map(|pair| pair.dim_vol)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
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
                let params = &self.0.data;
                let vals: Vec<u32> = params
                    .ch_strip_src
                    .iter()
                    .map(|src| {
                        let pos = Self::SRC_PAIR_ENTRIES
                            .iter()
                            .position(|s| src.eq(s))
                            .unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
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

    fn write(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            SRC_PAIR_MODE_NAME => {
                let mut params = self.0.data.clone();
                mixer_monitor_src_pairs_iter_mut(&mut params)
                    .zip(elem_value.enumerated())
                    .try_for_each(|(pair, &val)| {
                        let pos = val as usize;
                        let mode = Self::SRC_PAIR_MODES
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg =
                                    format!("Invalid value for index of mixer source: {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .copied()?;
                        if mode == MonitorSrcPairMode::Fixed {
                            let msg = format!("The fixed mode is not newly available: {}", pos);
                            Err(Error::new(FileError::Inval, &msg))
                        } else if pair.mode == MonitorSrcPairMode::Fixed {
                            let msg = format!("The source of mixer is immutable: {}", pos);
                            Err(Error::new(FileError::Inval, &msg))
                        } else {
                            pair.mode = mode;
                            Ok(())
                        }
                    })?;
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            SRC_ENTRY_NAME => {
                let mut params = self.0.data.clone();
                mixer_monitor_src_params_iter_mut(&mut params)
                    .zip(elem_value.enumerated())
                    .try_for_each(|(entry, &val)| {
                        let pos = val as usize;
                        Self::SRC_PAIR_ENTRIES
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg =
                                    format!("Invalid value for index of mixer source: {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&s| entry.src = s)
                    })?;
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            SRC_STEREO_LINK_NAME => {
                let mut params = self.0.data.clone();
                mixer_monitor_src_pairs_iter_mut(&mut params)
                    .zip(elem_value.boolean())
                    .for_each(|(pair, val)| pair.stereo_link = val);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            SRC_GAIN_NAME => {
                let mut params = self.0.data.clone();
                mixer_monitor_src_params_iter_mut(&mut params)
                    .zip(elem_value.int())
                    .for_each(|(entry, &val)| entry.gain_to_main = val);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            SRC_PAN_NAME => {
                let mut params = self.0.data.clone();
                mixer_monitor_src_params_iter_mut(&mut params)
                    .zip(elem_value.int())
                    .for_each(|(entry, &val)| entry.pan_to_main = val);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            REVERB_SRC_GAIN_NAME => {
                let mut params = self.0.data.clone();
                mixer_monitor_src_params_iter_mut(&mut params)
                    .zip(elem_value.int())
                    .for_each(|(entry, &val)| entry.gain_to_reverb = val);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            AUX01_SRC_GAIN_NAME => {
                let mut params = self.0.data.clone();
                mixer_monitor_src_params_iter_mut(&mut params)
                    .zip(elem_value.int())
                    .for_each(|(entry, &val)| entry.gain_to_aux0 = val);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            AUX23_SRC_GAIN_NAME => {
                let mut params = self.0.data.clone();
                mixer_monitor_src_params_iter_mut(&mut params)
                    .zip(elem_value.int())
                    .for_each(|(entry, &val)| entry.gain_to_aux1 = val);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            SRC_MUTE_NAME => {
                let mut params = self.0.data.clone();
                params
                    .mutes
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            OUT_DIM_NAME => {
                let mut params = self.0.data.clone();
                params
                    .mixer_out
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(pair, val)| pair.dim_enabled = val);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            OUT_VOL_NAME => {
                let mut params = self.0.data.clone();
                params
                    .mixer_out
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(pair, &val)| pair.vol = val);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            OUT_DIM_VOL_NAME => {
                let mut params = self.0.data.clone();
                params
                    .mixer_out
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(pair, &val)| pair.dim_vol = val);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            REVERB_RETURN_MUTE_NAME => {
                let mut params = self.0.data.clone();
                params
                    .reverb_return_mute
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            REVERB_RETURN_GAIN_NAME => {
                let mut params = self.0.data.clone();
                params
                    .reverb_return_gain
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(d, s)| *d = *s);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            CH_STRIP_AS_PLUGIN_NAME => {
                let mut params = self.0.data.clone();
                params
                    .ch_strip_as_plugin
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            CH_STRIP_SRC_NAME => {
                let mut params = self.0.data.clone();
                params
                    .ch_strip_src
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(src, &val)| {
                        let pos = val as usize;
                        Self::SRC_PAIR_ENTRIES
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg =
                                    format!("Invalid value for index of ch strip source: {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&s| *src = s)
                    })?;
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            CH_STRIP_23_AT_MID_RATE => {
                let mut params = self.0.data.clone();
                params.ch_strip_23_at_mid_rate = elem_value.boolean()[0];
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            POST_FADER_NAME => {
                let mut params = self.0.data.clone();
                params
                    .post_fader
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            MIXER_ENABLE_NAME => {
                let mut params = self.0.data.clone();
                params.enabled = elem_value.boolean()[0];
                let res = Studiok48Protocol::update_partial_segment(
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

    fn parse_notification(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if Studiok48Protocol::is_notified_segment(&self.0, msg) {
            let res = Studiok48Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct MixerMeterCtl(Studiok48MixerMeterSegment, Vec<ElemId>);

const LEVEL_MIN: i32 = -1000;
const LEVEL_MAX: i32 = 0;
const LEVEL_STEP: i32 = 1;
const LEVEL_TLV: DbInterval = DbInterval {
    min: -7200,
    max: 0,
    linear: false,
    mute_avail: false,
};

impl MixerMeterCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = Studiok48Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let meter = &self.0.data;

        [
            (
                MIXER_INPUT_METER_NAME,
                meter.src_inputs.len(),
                "mixer-input",
            ),
            (
                MIXER_OUTPUT_METER_NAME,
                meter.mixer_outputs.len(),
                "mixer_output",
            ),
            (AUX_OUTPUT_METER_NAME, meter.aux_outputs.len(), "aux-output"),
        ]
        .iter()
        .try_for_each(|&(name, count, label)| {
            let labels: Vec<String> = (0..count).map(|i| format!("{}-{}", label, i)).collect();
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    LEVEL_MIN,
                    LEVEL_MAX,
                    LEVEL_STEP,
                    labels.len(),
                    Some(&Into::<Vec<u32>>::into(LEVEL_TLV)),
                    false,
                )
                .map(|mut elem_id_list| self.1.append(&mut elem_id_list))
        })
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_INPUT_METER_NAME => {
                let params = &self.0.data;
                elem_value.set_int(&params.src_inputs);
                Ok(true)
            }
            MIXER_OUTPUT_METER_NAME => {
                let params = &self.0.data;
                elem_value.set_int(&params.mixer_outputs);
                Ok(true)
            }
            AUX_OUTPUT_METER_NAME => {
                let params = &self.0.data;
                elem_value.set_int(&params.aux_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
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
    }
}

fn high_pass_freq_to_string(freq: &HighPassFreq) -> String {
    match freq {
        HighPassFreq::Off => "Off".to_string(),
        HighPassFreq::Above12 => "12HzAbove".to_string(),
        HighPassFreq::Above24 => "24HzAbove".to_string(),
    }
}

fn low_pass_freq_to_string(freq: &LowPassFreq) -> String {
    match freq {
        LowPassFreq::Below12 => "12HzBelow".to_string(),
        LowPassFreq::Below24 => "24HzBelow".to_string(),
    }
}

fn phys_out_pair_src_iter(state: &StudioPhysOut) -> impl Iterator<Item = &PhysOutPairSrc> {
    state.out_pair_srcs.iter()
}

fn phys_out_pair_src_iter_mut(
    state: &mut StudioPhysOut,
) -> impl Iterator<Item = &mut PhysOutPairSrc> {
    state.out_pair_srcs.iter_mut()
}

fn phys_out_src_params_iter(state: &StudioPhysOut) -> impl Iterator<Item = &PhysOutSrcParam> {
    phys_out_pair_src_iter(state).flat_map(|pair| pair.params.iter())
}

fn phys_out_src_params_iter_mut(
    state: &mut StudioPhysOut,
) -> impl Iterator<Item = &mut PhysOutSrcParam> {
    phys_out_pair_src_iter_mut(state).flat_map(|pair| pair.params.iter_mut())
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

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = Studiok48Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
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
        match elem_id.name().as_str() {
            MASTER_OUT_DIM_NAME => {
                let params = &self.0.data;
                elem_value.set_bool(&[params.master_out.dim_enabled]);
                Ok(true)
            }
            MASTER_OUT_VOL_NAME => {
                let params = &self.0.data;
                elem_value.set_int(&[params.master_out.vol]);
                Ok(true)
            }
            MASTER_OUT_DIM_VOL_NAME => {
                let params = &self.0.data;
                elem_value.set_int(&[params.master_out.dim_vol]);
                Ok(true)
            }
            OUT_STEREO_LINK_NAME => {
                let params = &self.0.data;
                let vals: Vec<bool> = phys_out_pair_src_iter(&params)
                    .map(|pair| pair.stereo_link)
                    .collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            OUT_MUTE_NAME => {
                let params = &self.0.data;
                elem_value.set_bool(&params.out_mutes);
                Ok(true)
            }
            OUT_SRC_NAME => {
                let params = &self.0.data;
                let vals: Vec<u32> = phys_out_src_params_iter(params)
                    .map(|params| {
                        let pos = Self::PHYS_OUT_SRCS
                            .iter()
                            .position(|s| params.src.eq(s))
                            .unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            OUT_GRP_SELECT_NAME => {
                let params = &self.0.data;
                elem_value.set_enum(&[params.selected_out_grp as u32]);
                Ok(true)
            }
            OUT_GRP_SRC_ENABLE_NAME => {
                let params = &self.0.data;
                elem_value.set_bool(&params.out_assign_to_grp);
                Ok(true)
            }
            OUT_GRP_SRC_TRIM_NAME => {
                let params = &self.0.data;
                let vals: Vec<i32> = phys_out_src_params_iter(params)
                    .map(|params| params.vol)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            OUT_GRP_SRC_DELAY_NAME => {
                let params = &self.0.data;
                let vals: Vec<i32> = phys_out_src_params_iter(params)
                    .map(|params| params.delay)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            OUT_GRP_SRC_ASSIGN_NAME => {
                let params = &self.0.data;
                let index = elem_id.index() as usize;
                elem_value.set_bool(&params.out_grps[index].assigned_phys_outs);
                Ok(true)
            }
            OUT_GRP_BASS_MANAGEMENT_NAME => {
                let params = &self.0.data;
                let vals: Vec<bool> = params
                    .out_grps
                    .iter()
                    .map(|group| group.bass_management)
                    .collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            OUT_GRP_MAIN_CROSS_OVER_FREQ_NAME => {
                let params = &self.0.data;
                let vals: Vec<u32> = params
                    .out_grps
                    .iter()
                    .map(|group| {
                        let pos = Self::CROSS_OVER_FREQS
                            .iter()
                            .position(|freq| group.main_cross_over_freq.eq(freq))
                            .unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            OUT_GRP_MAIN_LEVEL_TO_SUB_NAME => {
                let params = &self.0.data;
                let vals: Vec<i32> = params
                    .out_grps
                    .iter()
                    .map(|group| group.main_level_to_sub)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            OUT_GRP_SUB_LEVEL_TO_SUB_NAME => {
                let params = &self.0.data;
                let vals: Vec<i32> = params
                    .out_grps
                    .iter()
                    .map(|group| group.sub_level_to_sub)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            OUT_GRP_MAIN_FILTER_FOR_MAIN_NAME => {
                let params = &self.0.data;
                let vals: Vec<u32> = params
                    .out_grps
                    .iter()
                    .map(|group| {
                        let pos = Self::HIGH_PASS_FREQS
                            .iter()
                            .position(|freq| group.main_filter_for_main.eq(freq))
                            .unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            OUT_GRP_MAIN_FILTER_FOR_SUB_NAME => {
                let params = &self.0.data;
                let vals: Vec<u32> = params
                    .out_grps
                    .iter()
                    .map(|group| {
                        let pos = Self::LOW_PASS_FREQS
                            .iter()
                            .position(|freq| group.main_filter_for_sub.eq(freq))
                            .unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
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
            MASTER_OUT_DIM_NAME => {
                let mut params = self.0.data.clone();
                params.master_out.dim_enabled = elem_value.boolean()[0];
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            MASTER_OUT_VOL_NAME => {
                let mut params = self.0.data.clone();
                params.master_out.vol = elem_value.int()[0];
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            MASTER_OUT_DIM_VOL_NAME => {
                let mut params = self.0.data.clone();
                params.master_out.dim_vol = elem_value.int()[0];
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            OUT_STEREO_LINK_NAME => {
                let mut params = self.0.data.clone();
                params
                    .out_pair_srcs
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(pair_src, val)| pair_src.stereo_link = val);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            OUT_MUTE_NAME => {
                let mut params = self.0.data.clone();
                params
                    .out_mutes
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            OUT_SRC_NAME => {
                let mut params = self.0.data.clone();
                phys_out_src_params_iter_mut(&mut params)
                    .zip(elem_value.enumerated())
                    .try_for_each(|(entry, &val)| {
                        let pos = val as usize;
                        Self::PHYS_OUT_SRCS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg = format!("output source not found for position {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&s| entry.src = s)
                    })?;
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            OUT_GRP_SELECT_NAME => {
                let mut params = self.0.data.clone();
                params.selected_out_grp = elem_value.enumerated()[0] as usize;
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            OUT_GRP_SRC_ENABLE_NAME => {
                let mut params = self.0.data.clone();
                params
                    .out_assign_to_grp
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(d, s)| *d = s);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            OUT_GRP_SRC_TRIM_NAME => {
                let mut params = self.0.data.clone();
                phys_out_src_params_iter_mut(&mut params)
                    .zip(elem_value.int())
                    .for_each(|(entry, &val)| entry.vol = val);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            OUT_GRP_SRC_DELAY_NAME => {
                let mut params = self.0.data.clone();
                phys_out_src_params_iter_mut(&mut params)
                    .zip(elem_value.int())
                    .for_each(|(entry, &val)| entry.delay = val);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            OUT_GRP_SRC_ASSIGN_NAME => {
                let mut params = self.0.data.clone();
                let vals = &elem_value.boolean()[..(STUDIO_PHYS_OUT_PAIR_COUNT * 2)];
                let count = vals.iter().filter(|&v| *v).count();
                if count > STUDIO_MAX_SURROUND_CHANNELS {
                    let msg = format!(
                        "Maximum {} channels are supported for surround channels, but {} given",
                        STUDIO_MAX_SURROUND_CHANNELS, count
                    );
                    Err(Error::new(FileError::Inval, &msg))?;
                }
                let index = elem_id.index() as usize;
                params.out_grps[index]
                    .assigned_phys_outs
                    .copy_from_slice(&vals);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            OUT_GRP_BASS_MANAGEMENT_NAME => {
                let mut params = self.0.data.clone();
                params
                    .out_grps
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(out_grp, val)| out_grp.bass_management = val);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            OUT_GRP_MAIN_CROSS_OVER_FREQ_NAME => {
                let mut params = self.0.data.clone();
                params
                    .out_grps
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(out_grp, &val)| {
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
                            .map(|&freq| out_grp.main_cross_over_freq = freq)
                    })?;
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            OUT_GRP_MAIN_LEVEL_TO_SUB_NAME => {
                let mut params = self.0.data.clone();
                params
                    .out_grps
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(out_grp, &val)| out_grp.main_level_to_sub = val);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            OUT_GRP_SUB_LEVEL_TO_SUB_NAME => {
                let mut params = self.0.data.clone();
                params
                    .out_grps
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(out_grp, &val)| out_grp.sub_level_to_sub = val);
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            OUT_GRP_MAIN_FILTER_FOR_MAIN_NAME => {
                let mut params = self.0.data.clone();
                params
                    .out_grps
                    .iter_mut()
                    .zip(elem_value.int())
                    .try_for_each(|(out_grp, &val)| {
                        let pos = val as usize;
                        Self::HIGH_PASS_FREQS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg = format!(
                                    "Invalid value for index of high pass frequency: {}",
                                    pos
                                );
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&freq| out_grp.main_filter_for_main = freq)
                    })?;
                let res = Studiok48Protocol::update_partial_segment(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0.data, ?res);
                res.map(|_| true)
            }
            OUT_GRP_MAIN_FILTER_FOR_SUB_NAME => {
                let mut params = self.0.data.clone();
                params
                    .out_grps
                    .iter_mut()
                    .zip(elem_value.int())
                    .try_for_each(|(out_grp, &val)| {
                        let pos = val as usize;
                        Self::LOW_PASS_FREQS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg = format!(
                                    "Invalid value for index of low pass frequency: {}",
                                    pos
                                );
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&freq| out_grp.main_filter_for_sub = freq)
                    })?;
                let res = Studiok48Protocol::update_partial_segment(
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

    fn parse_notification(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if Studiok48Protocol::is_notified_segment(&self.0, msg) {
            Studiok48Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct ChStripStateCtl(Studiok48ChStripStatesSegment, Vec<ElemId>);

impl ChStripStateCtlOperation<StudioChStripStates, Studiok48Protocol> for ChStripStateCtl {
    fn segment(&self) -> &Studiok48ChStripStatesSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut Studiok48ChStripStatesSegment {
        &mut self.0
    }

    fn states(params: &StudioChStripStates) -> &[ChStripState] {
        &params.0
    }

    fn states_mut(params: &mut StudioChStripStates) -> &mut [ChStripState] {
        &mut params.0
    }
}

#[derive(Default, Debug)]
struct ChStripMeterCtl(Studiok48ChStripMetersSegment, Vec<ElemId>);

impl ChStripMeterCtlOperation<StudioChStripMeters, Studiok48Protocol> for ChStripMeterCtl {
    fn meters(&self) -> &[ChStripMeter] {
        &self.0.data.0
    }

    fn segment(&self) -> &TcKonnektSegment<StudioChStripMeters> {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut TcKonnektSegment<StudioChStripMeters> {
        &mut self.0
    }
}

#[derive(Default, Debug)]
struct ReverbStateCtl(Studiok48ReverbStateSegment, Vec<ElemId>);

impl ReverbStateCtlOpreation<StudioReverbState, StudioReverbMeter, Studiok48Protocol>
    for ReverbStateCtl
{
    fn segment(&self) -> &Studiok48ReverbStateSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut Studiok48ReverbStateSegment {
        &mut self.0
    }

    fn state(params: &StudioReverbState) -> &ReverbState {
        &params.0
    }

    fn state_mut(params: &mut StudioReverbState) -> &mut ReverbState {
        &mut params.0
    }
}

#[derive(Default, Debug)]
struct ReverbMeterCtl(Studiok48ReverbMeterSegment, Vec<ElemId>);

impl ReverbMeterCtlOperation<StudioReverbMeter, Studiok48Protocol> for ReverbMeterCtl {
    fn meter(&self) -> &ReverbMeter {
        &self.0.data.0
    }

    fn segment(&self) -> &TcKonnektSegment<StudioReverbMeter> {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut TcKonnektSegment<StudioReverbMeter> {
        &mut self.0
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

#[derive(Default, Debug)]
struct HwStateCtl(Studiok48HwStateSegment, Vec<ElemId>);

impl FirewireLedCtlOperation<StudioHwState, Studiok48Protocol> for HwStateCtl {
    fn segment(&self) -> &Studiok48HwStateSegment {
        &self.0
    }

    fn segment_mut(&mut self) -> &mut Studiok48HwStateSegment {
        &mut self.0
    }

    fn firewire_led(params: &StudioHwState) -> &FireWireLedState {
        &params.firewire_led
    }

    fn firewire_led_mut(params: &mut StudioHwState) -> &mut FireWireLedState {
        &mut params.firewire_led
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

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = Studiok48Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
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
        } else {
            match elem_id.name().as_str() {
                ANALOG_JACK_STATE_NAME => {
                    let params = &self.0.data;
                    let vals: Vec<u32> = params
                        .analog_jack_states
                        .iter()
                        .map(|state| {
                            let pos = Self::ANALOG_JACK_STATES
                                .iter()
                                .position(|s| state.eq(s))
                                .unwrap();
                            pos as u32
                        })
                        .collect();
                    elem_value.set_enum(&vals);
                    Ok(true)
                }
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
    }

    fn write(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        self.write_firewire_led(req, node, elem_id, elem_value, timeout_ms)
    }

    fn parse_notification(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if Studiok48Protocol::is_notified_segment(&self.0, msg) {
            let res = Studiok48Protocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}
