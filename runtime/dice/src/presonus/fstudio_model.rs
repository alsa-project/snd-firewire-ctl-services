// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {super::*, protocols::presonus::fstudio::*};

#[derive(Default)]
pub struct FStudioModel {
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl<FStudioProtocol>,
    meter_ctl: MeterCtl,
    out_ctl: OutputCtl,
    mixer_ctl: MixerCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<(SndDice, FwNode)> for FStudioModel {
    fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        FStudioProtocol::read_general_sections(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .cache_whole_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.out_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        self.mixer_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr)?;

        self.meter_ctl.load(card_cntr)?;
        self.out_ctl.load(card_cntr)?;
        self.mixer_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.write(
            &unit.0,
            &self.req,
            &unit.1,
            &mut self.sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .out_ctl
            .write(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_ctl
            .write(&self.req, &unit.1, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for FStudioModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_id_list);
    }

    fn parse_notification(
        &mut self,
        unit: &mut (SndDice, FwNode),
        &msg: &u32,
    ) -> Result<(), Error> {
        self.common_ctl
            .parse_notification(&self.req, &unit.1, &mut self.sections, msg, TIMEOUT_MS)
    }
}

impl MeasureModel<(SndDice, FwNode)> for FStudioModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.measured_elem_id_list);
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .cache_partial_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;
        self.meter_ctl.cache(&self.req, &unit.1, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Debug)]
struct MeterCtl(FStudioMeter, Vec<ElemId>);

impl MeterCtl {
    const ANALOG_INPUT_NAME: &'static str = "analog-input-meter";
    const STREAM_INPUT_NAME: &'static str = "stream-input-meter";
    const MIXER_OUTPUT_NAME: &'static str = "mixer-output-meter";

    const LEVEL_MIN: i32 = 0x00;
    const LEVEL_MAX: i32 = 0xff;
    const LEVEL_STEP: i32 = 1;
    const LEVEL_TLV: DbInterval = DbInterval {
        min: -9600,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = FStudioProtocol::cache_whole_parameters(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        [
            (Self::ANALOG_INPUT_NAME, self.0.analog_inputs.len()),
            (Self::STREAM_INPUT_NAME, self.0.stream_inputs.len()),
            (Self::MIXER_OUTPUT_NAME, self.0.mixer_outputs.len()),
        ]
        .iter()
        .try_for_each(|&(name, count)| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    Self::LEVEL_MIN,
                    Self::LEVEL_MAX,
                    Self::LEVEL_STEP,
                    count,
                    Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
                    false,
                )
                .map(|mut elem_id_list| self.1.append(&mut elem_id_list))
        })
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::ANALOG_INPUT_NAME => {
                let vals: Vec<i32> = self.0.analog_inputs.iter().map(|&l| l as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::STREAM_INPUT_NAME => {
                let vals: Vec<i32> = self.0.stream_inputs.iter().map(|&l| l as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::MIXER_OUTPUT_NAME => {
                let vals: Vec<i32> = self.0.mixer_outputs.iter().map(|&l| l as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct OutputCtl(OutputParameters);

fn output_src_to_string(src: &OutputSrc) -> String {
    match src {
        OutputSrc::Analog(ch) => format!("Analog-{}", ch + 1),
        OutputSrc::Adat0(ch) => format!("ADAT-A-{}", ch + 1),
        OutputSrc::Spdif(ch) => format!("S/PDIF-{}", ch + 1),
        OutputSrc::Stream(ch) => format!("Stream-{}", ch + 1),
        OutputSrc::StreamAdat1(ch) => format!("Stream-{}/ADAT-B-{}", ch + 11, ch + 1),
        OutputSrc::MixerOut(ch) => format!("Mixer-{}", ch + 1),
    }
}

impl OutputCtl {
    const SRC_NAME: &'static str = "output-source";
    const VOL_NAME: &'static str = "output-volume";
    const MUTE_NAME: &'static str = "output-mute";
    const LINK_NAME: &'static str = "output-link";
    const MAIN_NAME: &'static str = "main-assign";
    const HP_NAME: &'static str = "headphone-assign";
    const TERMINATE_BNC_NAME: &'static str = "terminate-bnc";

    const VOL_MIN: i32 = 0;
    const VOL_MAX: i32 = 0xff;
    const VOL_STEP: i32 = 1;
    const VOL_TLV: DbInterval = DbInterval {
        min: -9600,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    const SRCS: [OutputSrc; 54] = [
        OutputSrc::Analog(0),
        OutputSrc::Analog(1),
        OutputSrc::Analog(2),
        OutputSrc::Analog(3),
        OutputSrc::Analog(4),
        OutputSrc::Analog(5),
        OutputSrc::Analog(6),
        OutputSrc::Analog(7),
        OutputSrc::Adat0(0),
        OutputSrc::Adat0(1),
        OutputSrc::Adat0(2),
        OutputSrc::Adat0(3),
        OutputSrc::Adat0(4),
        OutputSrc::Adat0(5),
        OutputSrc::Adat0(6),
        OutputSrc::Adat0(7),
        OutputSrc::Spdif(0),
        OutputSrc::Spdif(1),
        OutputSrc::Stream(0),
        OutputSrc::Stream(1),
        OutputSrc::Stream(2),
        OutputSrc::Stream(3),
        OutputSrc::Stream(4),
        OutputSrc::Stream(5),
        OutputSrc::Stream(6),
        OutputSrc::Stream(7),
        OutputSrc::Stream(8),
        OutputSrc::Stream(9),
        OutputSrc::StreamAdat1(0),
        OutputSrc::StreamAdat1(1),
        OutputSrc::StreamAdat1(2),
        OutputSrc::StreamAdat1(3),
        OutputSrc::StreamAdat1(4),
        OutputSrc::StreamAdat1(5),
        OutputSrc::StreamAdat1(6),
        OutputSrc::StreamAdat1(7),
        OutputSrc::MixerOut(0),
        OutputSrc::MixerOut(1),
        OutputSrc::MixerOut(2),
        OutputSrc::MixerOut(3),
        OutputSrc::MixerOut(4),
        OutputSrc::MixerOut(5),
        OutputSrc::MixerOut(6),
        OutputSrc::MixerOut(7),
        OutputSrc::MixerOut(8),
        OutputSrc::MixerOut(9),
        OutputSrc::MixerOut(10),
        OutputSrc::MixerOut(11),
        OutputSrc::MixerOut(12),
        OutputSrc::MixerOut(13),
        OutputSrc::MixerOut(14),
        OutputSrc::MixerOut(15),
        OutputSrc::MixerOut(16),
        OutputSrc::MixerOut(17),
    ];

    const HP_LABELS: [&'static str; 3] = ["HP-1/2", "HP-3/4", "HP-5/6"];

    const TARGETS: [AssignTarget; 9] = [
        AssignTarget::Analog01,
        AssignTarget::Analog23,
        AssignTarget::Analog56,
        AssignTarget::Analog78,
        AssignTarget::AdatA01,
        AssignTarget::AdatA23,
        AssignTarget::AdatA45,
        AssignTarget::AdatA67,
        AssignTarget::Spdif01,
    ];

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = FStudioProtocol::cache_whole_parameters(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::VOL_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::VOL_MIN,
            Self::VOL_MAX,
            Self::VOL_STEP,
            MIXER_COUNT * 2,
            Some(&Into::<Vec<u32>>::into(Self::VOL_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MUTE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, MIXER_COUNT * 2, true)?;

        let labels: Vec<String> = Self::SRCS.iter().map(|s| output_src_to_string(s)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SRC_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, MIXER_COUNT, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::LINK_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, MIXER_COUNT, true)?;

        let labels: Vec<&str> = Self::TARGETS
            .iter()
            .map(|s| assign_target_to_str(s))
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MAIN_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::HP_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, Self::HP_LABELS.len(), &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::TERMINATE_BNC_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::VOL_NAME => {
                let params = &self.0;
                let vals: Vec<i32> = params
                    .pairs
                    .iter()
                    .flat_map(|pair| pair.volumes.iter())
                    .map(|&vol| vol as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::MUTE_NAME => {
                let params = &self.0;
                let vals: Vec<bool> = params
                    .pairs
                    .iter()
                    .flat_map(|pair| pair.mutes.iter())
                    .copied()
                    .collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::SRC_NAME => {
                let params = &self.0;
                let vals: Vec<u32> = params
                    .pairs
                    .iter()
                    .map(|pair| {
                        let pos = Self::SRCS.iter().position(|s| pair.src.eq(s)).unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            Self::LINK_NAME => {
                let params = &self.0;
                let vals: Vec<bool> = params.pairs.iter().map(|pair| pair.link).collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::MAIN_NAME => {
                let params = &self.0;
                let pos = Self::TARGETS
                    .iter()
                    .position(|t| params.main_assign.eq(t))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            Self::HP_NAME => {
                let params = &self.0;
                let vals: Vec<u32> = params
                    .headphone_assigns
                    .iter()
                    .map(|assign| {
                        let pos = Self::TARGETS.iter().position(|t| assign.eq(t)).unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            Self::TERMINATE_BNC_NAME => {
                let params = &self.0;
                elem_value.set_bool(&[params.bnc_terminate]);
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
            Self::VOL_NAME => {
                let mut params = self.0.clone();
                params
                    .pairs
                    .iter_mut()
                    .flat_map(|pair| pair.volumes.iter_mut())
                    .zip(elem_value.int())
                    .for_each(|(vol, &val)| *vol = val as u8);
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::MUTE_NAME => {
                let mut params = self.0.clone();
                params
                    .pairs
                    .iter_mut()
                    .flat_map(|pair| pair.mutes.iter_mut())
                    .zip(elem_value.boolean())
                    .for_each(|(mute, val)| *mute = val);
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::SRC_NAME => {
                let mut params = self.0.clone();
                params
                    .pairs
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(pair, &val)| {
                        let pos = val as usize;
                        Self::SRCS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg =
                                    format!("Invalid value for index of output source: {}", pos);
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&s| pair.src = s)
                    })?;
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::LINK_NAME => {
                let mut params = self.0.clone();
                params
                    .pairs
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(pair, val)| pair.link = val);
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::MAIN_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.main_assign = Self::TARGETS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of assignment target: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::HP_NAME => {
                let mut params = self.0.clone();
                params
                    .headphone_assigns
                    .iter_mut()
                    .zip(elem_value.enumerated())
                    .try_for_each(|(assign, &val)| {
                        let pos = val as usize;
                        Self::TARGETS
                            .iter()
                            .nth(pos)
                            .ok_or_else(|| {
                                let msg = format!(
                                    "Invalid value for index of assignment target: {}",
                                    pos
                                );
                                Error::new(FileError::Inval, &msg)
                            })
                            .map(|&t| *assign = t)
                    })?;
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::TERMINATE_BNC_NAME => {
                let mut params = self.0.clone();
                params.bnc_terminate = elem_value.boolean()[0];
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
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

fn assign_target_to_str(target: &AssignTarget) -> &'static str {
    match target {
        AssignTarget::Analog01 => "Analog-output-1/2",
        AssignTarget::Analog23 => "Analog-output-3/4",
        AssignTarget::Analog56 => "Analog-output-5/6",
        AssignTarget::Analog78 => "Analog-output-7/8",
        AssignTarget::AdatA01 => "ADAT-output-1/2",
        AssignTarget::AdatA23 => "ADAT-output-3/4",
        AssignTarget::AdatA45 => "ADAT-output-5/6",
        AssignTarget::AdatA67 => "ADAT-output-7/8",
        AssignTarget::Spdif01 => "S/PDIF-output-1/2",
    }
}

fn expansion_mode_to_str(mode: &ExpansionMode) -> &'static str {
    match mode {
        ExpansionMode::StreamB0_7 => "Stream-B-0|8",
        ExpansionMode::AdatB0_7 => "ADAT-B-1|8",
    }
}

#[derive(Default, Debug)]
struct MixerCtl(MixerParameters);

impl MixerCtl {
    const PHYS_SRC_GAIN_NAME: &'static str = "mixer-phys-source-gain";
    const PHYS_SRC_PAN_NAME: &'static str = "mixer-phys-source-pan";
    const PHYS_SRC_MUTE_NAME: &'static str = "mixer-phys-source-mute";
    const PHYS_SRC_LINK_NAME: &'static str = "mixer-phys-source-link";
    const STREAM_SRC_GAIN_NAME: &'static str = "mixer-stream-source-gain";
    const STREAM_SRC_PAN_NAME: &'static str = "mixer-stream-source-pan";
    const STREAM_SRC_MUTE_NAME: &'static str = "mixer-stream-source-mute";
    const STREAM_SRC_LINK_NAME: &'static str = "mixer-stream-source-link";
    const SELECTABLE_SRC_GAIN_NAME: &'static str = "mixer-selectable-source-gain";
    const SELECTABLE_SRC_PAN_NAME: &'static str = "mixer-selectable-source-pan";
    const SELECTABLE_SRC_MUTE_NAME: &'static str = "mixer-selectable-source-mute";
    const SELECTABLE_SRC_LINK_NAME: &'static str = "mixer-selectable-source-link";
    const OUT_VOL_NAME: &'static str = "mixer-output-volume";
    const OUT_MUTE_NAME: &'static str = "mixer-output-mute";
    const EXPANSION_MODE_NAME: &'static str = "mixer-expansion-mode";

    const LEVEL_MIN: i32 = 0x00;
    const LEVEL_MAX: i32 = 0xff;
    const LEVEL_STEP: i32 = 1;
    const LEVEL_TLV: DbInterval = DbInterval {
        min: -9600,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    const PAN_MIN: i32 = 0x00;
    const PAN_MAX: i32 = 0x7f;
    const PAN_STEP: i32 = 1;

    const EXPANSION_MODES: [ExpansionMode; 2] =
        [ExpansionMode::StreamB0_7, ExpansionMode::AdatB0_7];

    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = FStudioProtocol::cache_whole_parameters(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        [
            (
                Self::PHYS_SRC_GAIN_NAME,
                Self::PHYS_SRC_PAN_NAME,
                Self::PHYS_SRC_MUTE_NAME,
                Self::PHYS_SRC_LINK_NAME,
                18,
            ),
            (
                Self::STREAM_SRC_GAIN_NAME,
                Self::STREAM_SRC_PAN_NAME,
                Self::STREAM_SRC_MUTE_NAME,
                Self::STREAM_SRC_LINK_NAME,
                10,
            ),
            (
                Self::SELECTABLE_SRC_GAIN_NAME,
                Self::SELECTABLE_SRC_PAN_NAME,
                Self::SELECTABLE_SRC_MUTE_NAME,
                Self::SELECTABLE_SRC_LINK_NAME,
                8,
            ),
        ]
        .iter()
        .try_for_each(
            |&(gain_name, balance_name, mute_name, link_name, value_count)| {
                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, gain_name, 0);
                let _ = card_cntr.add_int_elems(
                    &elem_id,
                    MIXER_COUNT,
                    Self::LEVEL_MIN,
                    Self::LEVEL_MAX,
                    Self::LEVEL_STEP,
                    value_count,
                    Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
                    true,
                )?;

                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, balance_name, 0);
                let _ = card_cntr.add_int_elems(
                    &elem_id,
                    MIXER_COUNT,
                    Self::PAN_MIN,
                    Self::PAN_MAX,
                    Self::PAN_STEP,
                    value_count,
                    None,
                    true,
                )?;

                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, mute_name, 0);
                let _ = card_cntr.add_bool_elems(&elem_id, MIXER_COUNT, value_count, true)?;

                let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, link_name, 0);
                card_cntr
                    .add_bool_elems(&elem_id, MIXER_COUNT, value_count / 2, true)
                    .map(|_| ())
            },
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::OUT_VOL_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::LEVEL_MIN,
            Self::LEVEL_MAX,
            Self::LEVEL_STEP,
            9,
            Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::OUT_MUTE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, MIXER_COUNT, true)?;

        let labels: Vec<&str> = Self::EXPANSION_MODES
            .iter()
            .map(|m| expansion_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EXPANSION_MODE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::PHYS_SRC_GAIN_NAME => {
                let params = &self.0;
                let index = elem_id.index() as usize;
                let srcs = &params.sources[index];
                let vals: Vec<i32> = srcs
                    .analog_pairs
                    .iter()
                    .chain(srcs.adat_0_pairs.iter())
                    .chain(srcs.spdif_pairs.iter())
                    .flat_map(|pair| pair.gains.iter())
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::PHYS_SRC_PAN_NAME => {
                let params = &self.0;
                let index = elem_id.index() as usize;
                let srcs = &params.sources[index];
                let vals: Vec<i32> = srcs
                    .analog_pairs
                    .iter()
                    .chain(srcs.adat_0_pairs.iter())
                    .chain(srcs.spdif_pairs.iter())
                    .flat_map(|pair| pair.balances.iter())
                    .map(|&balance| balance as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::PHYS_SRC_MUTE_NAME => {
                let params = &self.0;
                let index = elem_id.index() as usize;
                let srcs = &params.sources[index];
                let vals: Vec<bool> = srcs
                    .analog_pairs
                    .iter()
                    .chain(srcs.adat_0_pairs.iter())
                    .chain(srcs.spdif_pairs.iter())
                    .flat_map(|pair| pair.mutes.iter())
                    .copied()
                    .collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::PHYS_SRC_LINK_NAME => {
                let params = &self.0;
                let index = elem_id.index() as usize;
                let srcs = &params.sources[index];
                let vals: Vec<bool> = srcs
                    .analog_pairs
                    .iter()
                    .chain(srcs.adat_0_pairs.iter())
                    .chain(srcs.spdif_pairs.iter())
                    .map(|pair| pair.link)
                    .collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::STREAM_SRC_GAIN_NAME => {
                let params = &self.0;
                let index = elem_id.index() as usize;
                let srcs = &params.sources[index];
                let vals: Vec<i32> = srcs
                    .stream_pairs
                    .iter()
                    .flat_map(|pair| pair.gains.iter())
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::STREAM_SRC_PAN_NAME => {
                let params = &self.0;
                let index = elem_id.index() as usize;
                let srcs = &params.sources[index];
                let vals: Vec<i32> = srcs
                    .stream_pairs
                    .iter()
                    .flat_map(|pair| pair.balances.iter())
                    .map(|&balance| balance as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::STREAM_SRC_MUTE_NAME => {
                let params = &self.0;
                let index = elem_id.index() as usize;
                let srcs = &params.sources[index];
                let vals: Vec<bool> = srcs
                    .stream_pairs
                    .iter()
                    .flat_map(|pair| pair.mutes.iter())
                    .copied()
                    .collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::STREAM_SRC_LINK_NAME => {
                let params = &self.0;
                let index = elem_id.index() as usize;
                let srcs = &params.sources[index];
                let vals: Vec<bool> = srcs.stream_pairs.iter().map(|pair| pair.link).collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }

            Self::SELECTABLE_SRC_GAIN_NAME => {
                let params = &self.0;
                let index = elem_id.index() as usize;
                let srcs = &params.sources[index];
                let vals: Vec<i32> = srcs
                    .selectable_pairs
                    .iter()
                    .flat_map(|pair| pair.gains.iter())
                    .map(|&gain| gain as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::SELECTABLE_SRC_PAN_NAME => {
                let params = &self.0;
                let index = elem_id.index() as usize;
                let srcs = &params.sources[index];
                let vals: Vec<i32> = srcs
                    .selectable_pairs
                    .iter()
                    .flat_map(|pair| pair.balances.iter())
                    .map(|&balance| balance as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::SELECTABLE_SRC_MUTE_NAME => {
                let params = &self.0;
                let index = elem_id.index() as usize;
                let srcs = &params.sources[index];
                let vals: Vec<bool> = srcs
                    .selectable_pairs
                    .iter()
                    .flat_map(|pair| pair.mutes.iter())
                    .copied()
                    .collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::SELECTABLE_SRC_LINK_NAME => {
                let params = &self.0;
                let index = elem_id.index() as usize;
                let srcs = &params.sources[index];
                let vals: Vec<bool> = srcs.selectable_pairs.iter().map(|pair| pair.link).collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }

            Self::OUT_VOL_NAME => {
                let params = &self.0;
                let vals: Vec<i32> = params
                    .outputs
                    .iter()
                    .map(|pair| pair.volume as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::OUT_MUTE_NAME => {
                let params = &self.0;
                let vals: Vec<bool> = params.outputs.iter().map(|pair| pair.mute).collect();
                elem_value.set_bool(&vals);
                Ok(true)
            }
            Self::EXPANSION_MODE_NAME => {
                let params = &self.0;
                let pos = Self::EXPANSION_MODES
                    .iter()
                    .position(|m| params.expansion_mode.eq(m))
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
            Self::PHYS_SRC_GAIN_NAME => {
                let mut params = self.0.clone();
                let index = elem_id.index() as usize;
                let srcs = &mut params.sources[index];
                srcs.analog_pairs
                    .iter_mut()
                    .chain(srcs.adat_0_pairs.iter_mut())
                    .chain(srcs.spdif_pairs.iter_mut())
                    .flat_map(|pair| pair.gains.iter_mut())
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val as u8);
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::PHYS_SRC_PAN_NAME => {
                let mut params = self.0.clone();
                let index = elem_id.index() as usize;
                let srcs = &mut params.sources[index];
                srcs.analog_pairs
                    .iter_mut()
                    .chain(srcs.adat_0_pairs.iter_mut())
                    .chain(srcs.spdif_pairs.iter_mut())
                    .flat_map(|pair| pair.balances.iter_mut())
                    .zip(elem_value.int())
                    .for_each(|(balances, &val)| *balances = val as u8);
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::PHYS_SRC_MUTE_NAME => {
                let mut params = self.0.clone();
                let index = elem_id.index() as usize;
                let srcs = &mut params.sources[index];
                srcs.analog_pairs
                    .iter_mut()
                    .chain(srcs.adat_0_pairs.iter_mut())
                    .chain(srcs.spdif_pairs.iter_mut())
                    .flat_map(|pair| pair.mutes.iter_mut())
                    .zip(elem_value.boolean())
                    .for_each(|(mute, val)| *mute = val);
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::PHYS_SRC_LINK_NAME => {
                let mut params = self.0.clone();
                let index = elem_id.index() as usize;
                let srcs = &mut params.sources[index];
                srcs.analog_pairs
                    .iter_mut()
                    .chain(srcs.adat_0_pairs.iter_mut())
                    .chain(srcs.spdif_pairs.iter_mut())
                    .zip(elem_value.boolean())
                    .for_each(|(pair, val)| pair.link = val);
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::STREAM_SRC_GAIN_NAME => {
                let mut params = self.0.clone();
                let index = elem_id.index() as usize;
                let srcs = &mut params.sources[index];
                srcs.stream_pairs
                    .iter_mut()
                    .flat_map(|pair| pair.gains.iter_mut())
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val as u8);
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::STREAM_SRC_PAN_NAME => {
                let mut params = self.0.clone();
                let index = elem_id.index() as usize;
                let srcs = &mut params.sources[index];
                srcs.stream_pairs
                    .iter_mut()
                    .flat_map(|pair| pair.balances.iter_mut())
                    .zip(elem_value.int())
                    .for_each(|(balances, &val)| *balances = val as u8);
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::STREAM_SRC_MUTE_NAME => {
                let mut params = self.0.clone();
                let index = elem_id.index() as usize;
                let srcs = &mut params.sources[index];
                srcs.stream_pairs
                    .iter_mut()
                    .flat_map(|pair| pair.mutes.iter_mut())
                    .zip(elem_value.boolean())
                    .for_each(|(mute, val)| *mute = val);
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::STREAM_SRC_LINK_NAME => {
                let mut params = self.0.clone();
                let index = elem_id.index() as usize;
                let srcs = &mut params.sources[index];
                srcs.stream_pairs
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(pair, val)| pair.link = val);
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::SELECTABLE_SRC_GAIN_NAME => {
                let mut params = self.0.clone();
                let index = elem_id.index() as usize;
                let srcs = &mut params.sources[index];
                srcs.selectable_pairs
                    .iter_mut()
                    .flat_map(|pair| pair.gains.iter_mut())
                    .zip(elem_value.int())
                    .for_each(|(gain, &val)| *gain = val as u8);
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::SELECTABLE_SRC_PAN_NAME => {
                let mut params = self.0.clone();
                let index = elem_id.index() as usize;
                let srcs = &mut params.sources[index];
                srcs.selectable_pairs
                    .iter_mut()
                    .flat_map(|pair| pair.balances.iter_mut())
                    .zip(elem_value.int())
                    .for_each(|(balances, &val)| *balances = val as u8);
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::SELECTABLE_SRC_MUTE_NAME => {
                let mut params = self.0.clone();
                let index = elem_id.index() as usize;
                let srcs = &mut params.sources[index];
                srcs.selectable_pairs
                    .iter_mut()
                    .flat_map(|pair| pair.mutes.iter_mut())
                    .zip(elem_value.boolean())
                    .for_each(|(mute, val)| *mute = val);
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::SELECTABLE_SRC_LINK_NAME => {
                let mut params = self.0.clone();
                let index = elem_id.index() as usize;
                let srcs = &mut params.sources[index];
                srcs.selectable_pairs
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(pair, val)| pair.link = val);
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::OUT_VOL_NAME => {
                let mut params = self.0.clone();
                params
                    .outputs
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(pair, &val)| pair.volume = val as u8);
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::OUT_MUTE_NAME => {
                let mut params = self.0.clone();
                params
                    .outputs
                    .iter_mut()
                    .zip(elem_value.boolean())
                    .for_each(|(pair, val)| pair.mute = val);
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
                    &params,
                    &mut self.0,
                    timeout_ms,
                );
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::EXPANSION_MODE_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.expansion_mode = Self::EXPANSION_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of expansion mode: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = FStudioProtocol::update_partial_parameters(
                    req,
                    node,
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
