// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{shell_ctl::*, *},
    protocols::tcelectronic::shell::{k8::*, *},
};

#[derive(Default)]
pub struct K8Model {
    req: FwReq,
    sections: GeneralSections,
    ctl: CommonCtl,
    knob_ctl: KnobCtl,
    config_ctl: ConfigCtl,
    mixer_ctl: MixerCtl,
    hw_state_ctl: HwStateCtl,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<(SndDice, FwNode)> for K8Model {
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

        self.knob_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.config_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.mixer_ctl
            .load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
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
        } else if self.knob_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.config_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read(elem_id, elem_value)? {
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
            .knob_ctl
            .write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .config_ctl
            .write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .mixer_ctl
            .write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .hw_state_ctl
            .write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for K8Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        elem_id_list.extend_from_slice(&self.knob_ctl.1);
        elem_id_list.extend_from_slice(&self.config_ctl.1);
        elem_id_list.extend_from_slice(&self.mixer_ctl.2);
        elem_id_list.extend_from_slice(&self.hw_state_ctl.1);
    }

    fn parse_notification(&mut self, unit: &mut (SndDice, FwNode), msg: &u32) -> Result<(), Error> {
        self.ctl
            .parse_notification(unit, &mut self.req, &self.sections, *msg, TIMEOUT_MS)?;
        self.knob_ctl
            .parse_notification(unit, &mut self.req, *msg, TIMEOUT_MS)?;
        self.config_ctl
            .parse_notification(unit, &mut self.req, *msg, TIMEOUT_MS)?;
        self.mixer_ctl
            .parse_notification(unit, &mut self.req, *msg, TIMEOUT_MS)?;
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
        } else if self.knob_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.config_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_state_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndDice, FwNode)> for K8Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        elem_id_list.extend_from_slice(&self.mixer_ctl.3);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.ctl
            .measure_states(unit, &mut self.req, &self.sections, TIMEOUT_MS)?;
        self.mixer_ctl
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
        } else if self.mixer_ctl.read_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default)]
struct KnobCtl(K8KnobSegment, Vec<ElemId>);

impl ShellKnobCtlOperation<K8Knob, K8Protocol> for KnobCtl {
    const TARGETS: [&'static str; 4] = ["Analog-1", "Analog-2", "S/PDIF-1/2", "Configurable"];

    fn segment_mut(&mut self) -> &mut K8KnobSegment {
        &mut self.0
    }

    fn knob_target(&self) -> &ShellKnobTarget {
        &self.0.data.target
    }

    fn knob_target_mut(&mut self) -> &mut ShellKnobTarget {
        &mut self.0.data.target
    }
}

impl ShellKnob2CtlOperation<K8Knob, K8Protocol> for KnobCtl {
    const TARGETS: &'static [&'static str] = &["Stream-input-1/2", "Mixer-1/2"];

    fn segment_mut(&mut self) -> &mut K8KnobSegment {
        &mut self.0
    }

    fn knob2_target(&self) -> &ShellKnob2Target {
        &self.0.data.knob2_target
    }

    fn knob2_target_mut(&mut self) -> &mut ShellKnob2Target {
        &mut self.0.data.knob2_target
    }
}

impl KnobCtl {
    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        K8Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)?;

        self.load_knob_target(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;
        self.load_knob2_target(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_knob_target(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_knob2_target(elem_id, elem_value)? {
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
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if self.write_knob_target(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else if self.write_knob2_target(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else {
            Ok(false)
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
            K8Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }

    fn read_notified_elem(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.read_knob_target(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_knob2_target(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default)]
struct ConfigCtl(K8ConfigSegment, Vec<ElemId>);

impl ShellCoaxIfaceCtlOperation<K8Config, K8Protocol> for ConfigCtl {
    fn segment_mut(&mut self) -> &mut K8ConfigSegment {
        &mut self.0
    }

    fn coax_out_src(&self) -> &ShellCoaxOutPairSrc {
        &self.0.data.coax_out_src
    }

    fn coax_out_src_mut(&mut self) -> &mut ShellCoaxOutPairSrc {
        &mut self.0.data.coax_out_src
    }
}

impl StandaloneCtlOperation<K8Config, K8Protocol> for ConfigCtl {
    fn segment_mut(&mut self) -> &mut K8ConfigSegment {
        &mut self.0
    }

    fn standalone_rate(&self) -> &TcKonnektStandaloneClkRate {
        &self.0.data.standalone_rate
    }

    fn standalone_rate_mut(&mut self) -> &mut TcKonnektStandaloneClkRate {
        &mut self.0.data.standalone_rate
    }
}

impl ShellStandaloneCtlOperation<K8Config, K8Protocol> for ConfigCtl {
    fn standalone_src(&self) -> &ShellStandaloneClkSrc {
        &self.0.data.standalone_src
    }

    fn standalone_src_mut(&mut self) -> &mut ShellStandaloneClkSrc {
        &mut self.0.data.standalone_src
    }
}

impl ConfigCtl {
    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        K8Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)?;

        self.load_coax_out_src(card_cntr)?;
        self.load_standalone(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_coax_out_src(elem_id, elem_value)? {
            Ok(true)
        } else if self.read_standalone(elem_id, elem_value)? {
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
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if self.write_coax_out_src(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else if self.write_standalone(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else {
            Ok(false)
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
            K8Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }

    fn read_notified_elem(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.read_coax_out_src(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default)]
struct MixerCtl(
    K8MixerStateSegment,
    K8MixerMeterSegment,
    Vec<ElemId>,
    Vec<ElemId>,
);

impl ShellMixerCtlOperation<K8MixerState, K8MixerMeter, K8Protocol> for MixerCtl {
    fn state_segment(&self) -> &K8MixerStateSegment {
        &self.0
    }

    fn state_segment_mut(&mut self) -> &mut K8MixerStateSegment {
        &mut self.0
    }

    fn meter_segment_mut(&mut self) -> &mut K8MixerMeterSegment {
        &mut self.1
    }

    fn state(&self) -> &ShellMixerState {
        &self.0.data.mixer
    }

    fn state_mut(&mut self) -> &mut ShellMixerState {
        &mut self.0.data.mixer
    }

    fn meter(&self) -> &ShellMixerMeter {
        &self.1.data.0
    }

    fn meter_mut(&mut self) -> &mut ShellMixerMeter {
        &mut self.1.data.0
    }

    fn enabled(&self) -> bool {
        self.0.data.enabled
    }
}

const MIXER_ENABLE_NAME: &str = "mixer-enable";

impl MixerCtl {
    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        K8Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)?;
        K8Protocol::read_segment(req, &mut unit.1, &mut self.1, timeout_ms)?;

        self.load_mixer(card_cntr)
            .map(|(notified_elem_id_list, measured_elem_id_list)| {
                self.2 = notified_elem_id_list;
                self.3 = measured_elem_id_list;
            })?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_ENABLE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_mixer(elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                MIXER_ENABLE_NAME => {
                    ElemValueAccessor::<bool>::set_val(elem_value, || Ok(self.0.data.enabled))
                        .map(|_| true)
                }
                _ => Ok(false),
            }
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
        if self.write_mixer(unit, req, elem_id, old, new, timeout_ms)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                MIXER_ENABLE_NAME => {
                    ElemValueAccessor::<bool>::get_val(new, |val| {
                        self.0.data.enabled = val;
                        Ok(())
                    })?;
                    K8Protocol::write_segment(req, &mut unit.1, &mut self.0, timeout_ms)
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
            K8Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }

    fn read_notified_elem(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.read_mixer_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn measure_states(
        &mut self,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        K8Protocol::read_segment(req, &mut unit.1, &mut self.1, timeout_ms)
    }

    fn read_measured_elem(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.read_mixer_measured_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default)]
struct HwStateCtl(K8HwStateSegment, Vec<ElemId>);

impl FirewireLedCtlOperation<K8HwState, K8Protocol> for HwStateCtl {
    fn segment_mut(&mut self) -> &mut K8HwStateSegment {
        &mut self.0
    }

    fn firewire_led(&self) -> &FireWireLedState {
        &self.0.data.hw_state.firewire_led
    }

    fn firewire_led_mut(&mut self) -> &mut FireWireLedState {
        &mut self.0.data.hw_state.firewire_led
    }
}

impl ShellHwStateCtlOperation<K8HwState, K8Protocol> for HwStateCtl {
    fn hw_state(&self) -> &ShellHwState {
        &self.0.data.hw_state
    }

    fn hw_state_mut(&mut self) -> &mut ShellHwState {
        &mut self.0.data.hw_state
    }
}

const AUX_IN_ENABLED_NAME: &str = "aux-input-enable";

impl HwStateCtl {
    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut (SndDice, FwNode),
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        K8Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)?;

        self.load_hw_state(card_cntr)
            .map(|mut notified_elem_id_list| self.1.append(&mut notified_elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, AUX_IN_ENABLED_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.read_hw_state(elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                AUX_IN_ENABLED_NAME => ElemValueAccessor::<bool>::set_val(elem_value, || {
                    Ok(self.0.data.aux_input_enabled)
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
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if self.write_hw_state(unit, req, elem_id, new, timeout_ms)? {
            Ok(true)
        } else {
            Ok(false)
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
            K8Protocol::read_segment(req, &mut unit.1, &mut self.0, timeout_ms)
        } else {
            Ok(())
        }
    }
}
