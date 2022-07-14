// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {super::*, dice_protocols::presonus::fstudio::*};

#[derive(Default)]
pub struct FStudioModel {
    req: FwReq,
    sections: GeneralSections,
    ctl: CommonCtl,
    meter_ctl: MeterCtl,
    out_ctl: OutputCtl,
    assign_ctl: AssignCtl,
    mixer_ctl: MixerCtl,
}

const TIMEOUT_MS: u32 = 20;

// MEMO: the device returns 'SPDIF\ADAT\Word Clock\Unused\Unused\Unused\Unused\Internal\\'.
const AVAIL_CLK_SRC_LABELS: [&str; 13] = [
    "S/PDIF",
    "Unused",
    "Unused",
    "Unused",
    "Unused",
    "ADAT",
    "Unused",
    "WordClock",
    "Unused",
    "Unused",
    "Unused",
    "Unused",
    "Internal",
];

impl CtlModel<(SndDice, FwNode)> for FStudioModel {
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
        let entries: Vec<_> = AVAIL_CLK_SRC_LABELS.iter().map(|l| l.to_string()).collect();
        let src_labels = ClockSourceLabels { entries };
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        self.meter_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.out_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.assign_ctl.load(card_cntr)?;
        self.mixer_ctl
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
        } else if self
            .out_ctl
            .read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .assign_ctl
            .read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_ctl
            .read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)?
        {
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
            .out_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .assign_ctl
            .write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_ctl
            .write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for FStudioModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, unit: &mut (SndDice, FwNode), msg: &u32) -> Result<(), Error> {
        self.ctl
            .parse_notification(unit, &mut self.req, &self.sections, *msg, TIMEOUT_MS)
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.ctl.read_notified_elem(elem_id, elem_value)
    }
}

impl MeasureModel<(SndDice, FwNode)> for FStudioModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.meter_ctl.1);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.ctl
            .measure_states(unit, &mut self.req, &self.sections, TIMEOUT_MS)?;
        self.meter_ctl
            .measure_states(unit, &mut self.req, TIMEOUT_MS)?;
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
        } else if self.meter_ctl.read_measured_elem(elem_id, elem_value)? {
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

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        FStudioProtocol::read_meter(req, &mut unit.1, &mut self.0, timeout_ms)?;

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
        })?;

        Ok(())
    }

    fn measure_states(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        FStudioProtocol::read_meter(req, &mut unit.1, &mut self.0, timeout_ms)
    }

    fn read_measured_elem(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
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
struct OutputCtl(OutputState);

fn output_src_to_string(src: &OutputSrc) -> String {
    match src {
        OutputSrc::Analog(ch) => format!("Analog-{}", ch + 1),
        OutputSrc::Adat0(ch) => format!("ADAT-A-{}", ch + 1),
        OutputSrc::Spdif(ch) => format!("S/PDIF-{}", ch + 1),
        OutputSrc::Stream(ch) => format!("Stream-{}", ch + 1),
        OutputSrc::StreamAdat1(ch) => format!("Stream-{}/ADAT-B-{}", ch + 11, ch + 1),
        OutputSrc::MixerOut(ch) => format!("Mixer-{}", ch + 1),
        OutputSrc::Reserved(val) => format!("Reserved({})", val),
    }
}

impl OutputCtl {
    const SRC_NAME: &'static str = "output-source";
    const VOL_NAME: &'static str = "output-volume";
    const MUTE_NAME: &'static str = "output-mute";
    const LINK_NAME: &'static str = "output-link";
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

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        FStudioProtocol::read_output_states(req, &mut unit.1, &mut self.0, timeout_ms)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::VOL_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::VOL_MIN,
            Self::VOL_MAX,
            Self::VOL_STEP,
            self.0.vols.len(),
            Some(&Into::<Vec<u32>>::into(Self::VOL_TLV)),
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MUTE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, self.0.mutes.len(), true)?;

        let labels: Vec<String> = Self::SRCS.iter().map(|s| output_src_to_string(s)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::SRC_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, self.0.srcs.len(), &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::LINK_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, self.0.links.len(), true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::TERMINATE_BNC_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::VOL_NAME => {
                let vals: Vec<i32> = self.0.vols.iter().map(|&vol| vol as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::MUTE_NAME => {
                elem_value.set_bool(&self.0.mutes);
                Ok(true)
            }
            Self::SRC_NAME => {
                let vals: Vec<u32> = self
                    .0
                    .srcs
                    .iter()
                    .map(|src| {
                        let pos = Self::SRCS.iter().position(|s| s.eq(src)).unwrap();
                        pos as u32
                    })
                    .collect();
                elem_value.set_enum(&vals);
                Ok(true)
            }
            Self::LINK_NAME => {
                elem_value.set_bool(&self.0.links);
                Ok(true)
            }
            Self::TERMINATE_BNC_NAME => {
                FStudioProtocol::read_bnc_terminate(req, &mut unit.1, timeout_ms).map(|terminate| {
                    elem_value.set_bool(&[terminate]);
                    true
                })
            }
            _ => Ok(false),
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
        match elem_id.name().as_str() {
            Self::VOL_NAME => {
                let vals = &elem_value.int()[..self.0.vols.len()];
                let vols: Vec<u8> = vals.iter().map(|&val| val as u8).collect();
                FStudioProtocol::write_output_vols(req, &mut unit.1, &mut self.0, &vols, timeout_ms)
                    .map(|_| true)
            }
            Self::MUTE_NAME => {
                let vals = &elem_value.boolean()[..self.0.mutes.len()];
                FStudioProtocol::write_output_mute(req, &mut unit.1, &mut self.0, &vals, timeout_ms)
                    .map(|_| true)
            }
            Self::SRC_NAME => {
                let vals = &elem_value.enumerated()[..self.0.srcs.len()];

                let mut srcs = self.0.srcs.clone();
                vals.iter().enumerate().try_for_each(|(i, &val)| {
                    Self::SRCS
                        .iter()
                        .nth(val as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid value for index of output source: {}", val);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&src| srcs[i] = src)
                })?;
                FStudioProtocol::write_output_src(req, &mut unit.1, &mut self.0, &srcs, timeout_ms)
                    .map(|_| true)
            }
            Self::LINK_NAME => {
                let vals = &elem_value.boolean()[..self.0.links.len()];
                FStudioProtocol::write_output_link(req, &mut unit.1, &mut self.0, &vals, timeout_ms)
                    .map(|_| true)
            }
            Self::TERMINATE_BNC_NAME => {
                let val = elem_value.boolean()[0];
                FStudioProtocol::write_bnc_terminalte(req, &mut unit.1, val, timeout_ms)
                    .map(|_| true)
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
        AssignTarget::Reserved(_) => "Reserved",
    }
}

#[derive(Default, Debug)]
struct AssignCtl;

impl AssignCtl {
    const MAIN_NAME: &'static str = "main-assign";
    const HP_NAME: &'static str = "headphone-assign";

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

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::TARGETS
            .iter()
            .map(|s| assign_target_to_str(s))
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::MAIN_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, Self::HP_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, Self::HP_LABELS.len(), &labels, None, true)?;

        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::MAIN_NAME => {
                let target =
                    FStudioProtocol::read_main_assign_target(req, &mut unit.1, timeout_ms)?;
                let pos = Self::TARGETS.iter().position(|t| t.eq(&target)).unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            Self::HP_NAME => ElemValueAccessor::<u32>::set_vals(elem_value, 3, |idx| {
                let target =
                    FStudioProtocol::read_hp_assign_target(req, &mut unit.1, idx, timeout_ms)?;
                let pos = Self::TARGETS.iter().position(|t| t.eq(&target)).unwrap();
                Ok(pos as u32)
            })
            .map(|_| true),
            _ => Ok(false),
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
        match elem_id.name().as_str() {
            Self::MAIN_NAME => {
                let val = new.enumerated()[0];
                let target = Self::TARGETS.iter().nth(val as usize).ok_or_else(|| {
                    let msg = format!("Invalid value for index of assignment target: {}", val);
                    Error::new(FileError::Inval, &msg)
                })?;
                FStudioProtocol::write_main_assign_target(req, &mut unit.1, *target, timeout_ms)
                    .map(|_| true)
            }
            Self::HP_NAME => ElemValueAccessor::<u32>::get_vals(new, old, 3, |idx, val| {
                let target = Self::TARGETS.iter().nth(val as usize).ok_or_else(|| {
                    let msg = format!("Invalid value for index of assignment target: {}", val);
                    Error::new(FileError::Inval, &msg)
                })?;
                FStudioProtocol::write_hp_assign_target(req, &mut unit.1, idx, *target, timeout_ms)
            })
            .map(|_| true),
            _ => Ok(false),
        }
    }
}

fn expansion_mode_to_str(mode: &ExpansionMode) -> &'static str {
    match mode {
        ExpansionMode::Stream10_17 => "Stream-11|18",
        ExpansionMode::AdatB0_7 => "ADAT-B-1|8",
    }
}

#[derive(Default, Debug)]
struct MixerCtl {
    phys_src_params: [SrcParams; MIXER_COUNT],
    stream_src_params: [SrcParams; MIXER_COUNT],
    phys_src_links: [[bool; 9]; MIXER_COUNT],
    stream_src_links: [[bool; 9]; MIXER_COUNT],
    outs: OutParams,
}

impl MixerCtl {
    const PHYS_SRC_GAIN_NAME: &'static str = "mixer-phys-source-gain";
    const PHYS_SRC_PAN_NAME: &'static str = "mixer-phys-source-pan";
    const PHYS_SRC_MUTE_NAME: &'static str = "mixer-phys-source-mute";
    const PHYS_SRC_LINK_NAME: &'static str = "mixer-phys-source-link";
    const STREAM_SRC_GAIN_NAME: &'static str = "mixer-stream-source-gain";
    const STREAM_SRC_PAN_NAME: &'static str = "mixer-stream-source-pan";
    const STREAM_SRC_MUTE_NAME: &'static str = "mixer-stream-source-mute";
    const STREAM_SRC_LINK_NAME: &'static str = "mixer-stream-source-link";
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
        [ExpansionMode::Stream10_17, ExpansionMode::AdatB0_7];

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.phys_src_params
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, params)| {
                FStudioProtocol::read_mixer_phys_src_params(req, &mut unit.1, params, i, timeout_ms)
            })?;

        self.stream_src_params
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, params)| {
                FStudioProtocol::read_mixer_stream_src_params(
                    req,
                    &mut unit.1,
                    params,
                    i,
                    timeout_ms,
                )
            })?;

        self.phys_src_links
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, links)| {
                FStudioProtocol::read_mixer_phys_src_links(req, &mut unit.1, links, i, timeout_ms)
            })?;

        self.stream_src_links
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, links)| {
                FStudioProtocol::read_mixer_stream_src_links(req, &mut unit.1, links, i, timeout_ms)
            })?;

        FStudioProtocol::read_mixer_out_params(req, &mut unit.1, &mut self.outs, timeout_ms)?;

        [
            (
                Self::PHYS_SRC_GAIN_NAME,
                self.phys_src_params.len(),
                self.phys_src_params[0].gains.len(),
            ),
            (
                Self::STREAM_SRC_GAIN_NAME,
                self.stream_src_params.len(),
                self.stream_src_params[0].gains.len(),
            ),
        ]
        .iter()
        .try_for_each(|&(name, elem_count, value_count)| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    elem_count,
                    Self::LEVEL_MIN,
                    Self::LEVEL_MAX,
                    Self::LEVEL_STEP,
                    value_count,
                    Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
                    true,
                )
                .map(|_| ())
        })?;

        [
            (
                Self::PHYS_SRC_PAN_NAME,
                self.phys_src_params.len(),
                self.phys_src_params[0].pans.len(),
            ),
            (
                Self::STREAM_SRC_PAN_NAME,
                self.stream_src_params.len(),
                self.stream_src_params[0].pans.len(),
            ),
        ]
        .iter()
        .try_for_each(|&(name, elem_count, value_count)| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    elem_count,
                    Self::PAN_MIN,
                    Self::PAN_MAX,
                    Self::PAN_STEP,
                    value_count,
                    None,
                    true,
                )
                .map(|_| ())
        })?;

        [
            (
                Self::PHYS_SRC_MUTE_NAME,
                self.phys_src_params.len(),
                self.phys_src_params[0].mutes.len(),
            ),
            (
                Self::STREAM_SRC_MUTE_NAME,
                self.stream_src_params.len(),
                self.stream_src_params[0].mutes.len(),
            ),
        ]
        .iter()
        .try_for_each(|&(name, elem_count, value_count)| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_bool_elems(&elem_id, elem_count, value_count, true)
                .map(|_| ())
        })?;

        [
            (
                Self::PHYS_SRC_LINK_NAME,
                self.phys_src_links.len(),
                self.phys_src_links[0].len(),
            ),
            (
                Self::STREAM_SRC_LINK_NAME,
                self.stream_src_links.len(),
                self.stream_src_links[0].len(),
            ),
        ]
        .iter()
        .try_for_each(|&(name, elem_count, value_count)| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_bool_elems(&elem_id, elem_count, value_count, true)
                .map(|_| ())
        })?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::OUT_VOL_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            Self::LEVEL_MIN,
            Self::LEVEL_MAX,
            Self::LEVEL_STEP,
            self.outs.vols.len(),
            Some(&Into::<Vec<u32>>::into(Self::LEVEL_TLV)),
            true,
        )?;
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::OUT_MUTE_NAME, 0);
        card_cntr.add_bool_elems(&elem_id, 1, self.outs.mutes.len(), true)?;

        let labels: Vec<&str> = Self::EXPANSION_MODES
            .iter()
            .map(|m| expansion_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::EXPANSION_MODE_NAME, 0);
        card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::PHYS_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let params = &self.phys_src_params[index];
                let vals: Vec<i32> = params.gains.iter().map(|&gain| gain as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::PHYS_SRC_PAN_NAME => {
                let index = elem_id.index() as usize;
                let params = &self.phys_src_params[index];
                let vals: Vec<i32> = params.pans.iter().map(|&pan| pan as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::PHYS_SRC_MUTE_NAME => {
                let index = elem_id.index() as usize;
                let params = &self.phys_src_params[index];
                elem_value.set_bool(&params.mutes);
                Ok(true)
            }
            Self::PHYS_SRC_LINK_NAME => {
                let index = elem_id.index() as usize;
                let links = &self.phys_src_links[index];
                elem_value.set_bool(links);
                Ok(true)
            }
            Self::STREAM_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let params = &self.stream_src_params[index];
                let vals: Vec<i32> = params.gains.iter().map(|&gain| gain as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::STREAM_SRC_PAN_NAME => {
                let index = elem_id.index() as usize;
                let params = &self.stream_src_params[index];
                let vals: Vec<i32> = params.pans.iter().map(|&pan| pan as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::STREAM_SRC_MUTE_NAME => {
                let index = elem_id.index() as usize;
                let params = &self.stream_src_params[index];
                elem_value.set_bool(&params.mutes);
                Ok(true)
            }
            Self::STREAM_SRC_LINK_NAME => {
                let index = elem_id.index() as usize;
                let links = &self.stream_src_links[index];
                elem_value.set_bool(links);
                Ok(true)
            }
            Self::OUT_VOL_NAME => {
                let vals: Vec<i32> = self.outs.vols.iter().map(|&v| v as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            Self::OUT_MUTE_NAME => {
                elem_value.set_bool(&self.outs.mutes);
                Ok(true)
            }
            Self::EXPANSION_MODE_NAME => {
                let mode =
                    FStudioProtocol::read_mixer_expansion_mode(req, &mut unit.1, timeout_ms)?;
                let pos = Self::EXPANSION_MODES
                    .iter()
                    .position(|m| m.eq(&mode))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
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
        match elem_id.name().as_str() {
            Self::PHYS_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let params = &mut self.phys_src_params[index];
                let vals = &elem_value.int()[..params.gains.len()];
                let gains: Vec<u8> = vals.iter().map(|&v| v as u8).collect();
                FStudioProtocol::write_mixer_phys_src_gains(
                    req,
                    &mut unit.1,
                    params,
                    index,
                    &gains,
                    timeout_ms,
                )
                .map(|_| true)
            }
            Self::PHYS_SRC_PAN_NAME => {
                let index = elem_id.index() as usize;
                let params = &mut self.phys_src_params[index];
                let vals = &elem_value.int()[..params.pans.len()];
                let pans: Vec<u8> = vals.iter().map(|&v| v as u8).collect();
                FStudioProtocol::write_mixer_phys_src_pans(
                    req,
                    &mut unit.1,
                    params,
                    index,
                    &pans,
                    timeout_ms,
                )
                .map(|_| true)
            }
            Self::PHYS_SRC_MUTE_NAME => {
                let index = elem_id.index() as usize;
                let params = &mut self.phys_src_params[index];
                let vals = &elem_value.boolean()[..params.mutes.len()];
                FStudioProtocol::write_mixer_phys_src_mutes(
                    req,
                    &mut unit.1,
                    params,
                    index,
                    &vals,
                    timeout_ms,
                )
                .map(|_| true)
            }
            Self::PHYS_SRC_LINK_NAME => {
                let index = elem_id.index() as usize;
                let links = &mut self.phys_src_links[index];
                let vals = &elem_value.boolean()[..links.len()];
                FStudioProtocol::write_mixer_phys_src_links(
                    req,
                    &mut unit.1,
                    &vals,
                    index,
                    timeout_ms,
                )
                .map(|_| true)
            }
            Self::STREAM_SRC_GAIN_NAME => {
                let index = elem_id.index() as usize;
                let params = &mut self.stream_src_params[index];
                let vals = &elem_value.int()[..params.gains.len()];
                let gains: Vec<u8> = vals.iter().map(|&v| v as u8).collect();
                FStudioProtocol::write_mixer_stream_src_gains(
                    req,
                    &mut unit.1,
                    params,
                    index,
                    &gains,
                    timeout_ms,
                )
                .map(|_| true)
            }
            Self::STREAM_SRC_PAN_NAME => {
                let index = elem_id.index() as usize;
                let params = &mut self.stream_src_params[index];
                let vals = &elem_value.int()[..params.pans.len()];
                let pans: Vec<u8> = vals.iter().map(|&v| v as u8).collect();
                FStudioProtocol::write_mixer_stream_src_pans(
                    req,
                    &mut unit.1,
                    params,
                    index,
                    &pans,
                    timeout_ms,
                )
                .map(|_| true)
            }
            Self::STREAM_SRC_MUTE_NAME => {
                let index = elem_id.index() as usize;
                let params = &mut self.stream_src_params[index];
                let vals = &elem_value.boolean()[..params.mutes.len()];
                FStudioProtocol::write_mixer_stream_src_mutes(
                    req,
                    &mut unit.1,
                    params,
                    index,
                    &vals,
                    timeout_ms,
                )
                .map(|_| true)
            }
            Self::STREAM_SRC_LINK_NAME => {
                let index = elem_id.index() as usize;
                let links = &mut self.stream_src_links[index];
                let vals = &elem_value.boolean()[..links.len()];
                FStudioProtocol::write_mixer_stream_src_links(
                    req,
                    &mut unit.1,
                    &vals,
                    index,
                    timeout_ms,
                )
                .map(|_| true)
            }
            Self::OUT_VOL_NAME => {
                let vals = &elem_value.int()[..self.outs.vols.len()];
                let vols: Vec<u8> = vals.iter().map(|&v| v as u8).collect();
                FStudioProtocol::write_mixer_out_vol(
                    req,
                    &mut unit.1,
                    &mut self.outs,
                    &vols,
                    timeout_ms,
                )
                .map(|_| true)
            }
            Self::OUT_MUTE_NAME => {
                let vals = &elem_value.boolean()[..self.outs.mutes.len()];
                FStudioProtocol::write_mixer_out_mute(
                    req,
                    &mut unit.1,
                    &mut self.outs,
                    &vals,
                    timeout_ms,
                )
                .map(|_| true)
            }
            Self::EXPANSION_MODE_NAME => {
                let val = elem_value.enumerated()[0];
                let &mode = Self::EXPANSION_MODES
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of expansion mode: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })?;
                FStudioProtocol::write_mixer_expansion_mode(req, &mut unit.1, mode, timeout_ms)
                    .map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
