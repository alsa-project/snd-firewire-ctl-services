// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{shell_ctl::*, *},
    protocols::tcelectronic::shell::k24d::*,
};

#[derive(Default, Debug)]
pub struct K24dModel {
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl<K24dProtocol>,
    knob_ctl: KnobCtl,
    config_ctl: ConfigCtl,
    mixer_state_ctl: MixerStateCtl,
    mixer_meter_ctl: MixerMeterCtl<K24dProtocol, K24dMixerMeter>,
    hw_state_ctl: HwStateCtl,
    reverb_state_ctl: ReverbStateCtl<K24dProtocol, K24dReverbState>,
    reverb_meter_ctl: ReverbMeterCtl<K24dProtocol, K24dReverbMeter>,
    ch_strip_state_ctl: ChStripStateCtl<K24dProtocol, K24dChStripStates>,
    ch_strip_meter_ctl: ChStripMeterCtl<K24dProtocol, K24dChStripMeters>,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<(SndDice, FwNode)> for K24dModel {
    fn cache(&mut self, (_, node): &mut (SndDice, FwNode)) -> Result<(), Error> {
        K24dProtocol::read_general_sections(&mut self.req, node, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .cache_whole_params(&mut self.req, node, &mut self.sections, TIMEOUT_MS)?;

        self.knob_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.config_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.mixer_state_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.mixer_meter_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.hw_state_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.reverb_state_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.reverb_meter_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.ch_strip_state_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;
        self.ch_strip_meter_ctl
            .cache(&mut self.req, node, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr)?;

        self.knob_ctl.load(card_cntr)?;
        self.config_ctl.load(card_cntr)?;
        self.mixer_state_ctl.load(card_cntr)?;
        self.mixer_meter_ctl.load(card_cntr)?;
        self.hw_state_ctl.load(card_cntr)?;
        self.reverb_state_ctl.load(card_cntr)?;
        self.reverb_meter_ctl.load(card_cntr)?;
        self.ch_strip_state_ctl.load(card_cntr)?;
        self.ch_strip_meter_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.knob_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.config_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_state_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.hw_state_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_state_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.reverb_meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_state_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.ch_strip_meter_ctl.read(elem_id, elem_value)? {
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
            .knob_ctl
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
            .hw_state_ctl
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
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for K24dModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_id_list);
        elem_id_list.extend_from_slice(&self.knob_ctl.1);
        elem_id_list.extend_from_slice(&self.config_ctl.1);
        elem_id_list.extend_from_slice(&self.mixer_state_ctl.1);
        elem_id_list.extend_from_slice(&self.hw_state_ctl.1);
        elem_id_list.extend_from_slice(&self.reverb_state_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.ch_strip_state_ctl.elem_id_list);
    }

    fn parse_notification(
        &mut self,
        (_, node): &mut (SndDice, FwNode),
        &msg: &u32,
    ) -> Result<(), Error> {
        self.common_ctl
            .parse_notification(&self.req, node, &mut self.sections, msg, TIMEOUT_MS)?;

        self.knob_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        self.config_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        self.mixer_state_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        self.hw_state_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        self.reverb_state_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        self.ch_strip_state_ctl
            .parse_notification(&self.req, node, msg, TIMEOUT_MS)?;
        Ok(())
    }
}

impl MeasureModel<(SndDice, FwNode)> for K24dModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.measured_elem_id_list);
        elem_id_list.extend_from_slice(&self.mixer_meter_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.reverb_meter_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.ch_strip_meter_ctl.elem_id_list);
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

#[derive(Default, Debug)]
struct KnobCtl(K24dKnobSegment, Vec<ElemId>);

impl KnobCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = K24dProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        load_knob0_target::<K24dProtocol, K24dKnob>(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        load_knob1_target::<K24dProtocol, K24dKnob>(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        load_prog::<K24dProtocol, K24dKnob>(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if read_knob0_target::<K24dProtocol, K24dKnob>(&self.0, elem_id, elem_value)? {
            Ok(true)
        } else if read_knob1_target::<K24dProtocol, K24dKnob>(&self.0, elem_id, elem_value)? {
            Ok(true)
        } else if read_prog::<K24dProtocol, K24dKnob>(&self.0, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
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
        if write_knob0_target::<K24dProtocol, K24dKnob>(
            &mut self.0,
            req,
            node,
            elem_id,
            elem_value,
            timeout_ms,
        )? {
            Ok(true)
        } else if write_knob1_target::<K24dProtocol, K24dKnob>(
            &mut self.0,
            req,
            node,
            elem_id,
            elem_value,
            timeout_ms,
        )? {
            Ok(true)
        } else if write_prog::<K24dProtocol, K24dKnob>(
            &mut self.0,
            req,
            node,
            elem_id,
            elem_value,
            timeout_ms,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn parse_notification(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if K24dProtocol::is_notified_segment(&self.0, msg) {
            let res = K24dProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct ConfigCtl(K24dConfigSegment, Vec<ElemId>);

const OUT_23_SRC_NAME: &str = "output-3/4-source";

impl ConfigCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = K24dProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        load_coax_out_src::<K24dProtocol, K24dConfig>(card_cntr)?;
        load_opt_iface_config::<K24dProtocol, K24dConfig>(card_cntr)?;
        load_standalone::<K24dProtocol, K24dConfig>(card_cntr)?;

        let labels: Vec<&str> = PHYS_OUT_SRCS
            .iter()
            .map(|s| phys_out_src_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, OUT_23_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if read_coax_out_src::<K24dProtocol, K24dConfig>(&self.0, elem_id, elem_value)? {
            Ok(true)
        } else if read_opt_iface_config::<K24dProtocol, K24dConfig>(&self.0, elem_id, elem_value)? {
            Ok(true)
        } else if read_standalone::<K24dProtocol, K24dConfig>(&self.0, elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                OUT_23_SRC_NAME => {
                    let pos = PHYS_OUT_SRCS
                        .iter()
                        .position(|s| self.0.data.out_23_src.eq(s))
                        .unwrap();
                    elem_value.set_enum(&[pos as u32]);
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
        if write_coax_out_src::<K24dProtocol, K24dConfig>(
            &mut self.0,
            req,
            node,
            elem_id,
            elem_value,
            timeout_ms,
        )? {
            Ok(true)
        } else if write_opt_iface_config::<K24dProtocol, K24dConfig>(
            &mut self.0,
            req,
            node,
            elem_id,
            elem_value,
            timeout_ms,
        )? {
            Ok(true)
        } else if write_standalone::<K24dProtocol, K24dConfig>(
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
                OUT_23_SRC_NAME => {
                    let mut params = self.0.data.clone();
                    let pos = elem_value.enumerated()[0] as usize;
                    PHYS_OUT_SRCS
                        .iter()
                        .nth(pos)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index of output source: {}", pos);
                            Error::new(FileError::Inval, &msg)
                        })
                        .map(|&s| params.out_23_src = s)?;
                    let res = K24dProtocol::update_partial_segment(
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
        if K24dProtocol::is_notified_segment(&self.0, msg) {
            let res = K24dProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct MixerStateCtl(K24dMixerStateSegment, Vec<ElemId>);

const MIXER_ENABLE_NAME: &str = "mixer-enable";
const USE_CH_STRIP_AS_PLUGIN_NAME: &str = "use-channel-strip-as-plugin";
const USE_REVERB_AT_MID_RATE: &str = "use-reverb-at-mid-rate";

impl MixerStateCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = K24dProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        load_mixer::<K24dProtocol, K24dMixerState>(&self.0, card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        load_reverb_return::<K24dProtocol, K24dMixerState>(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_ENABLE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, USE_CH_STRIP_AS_PLUGIN_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, USE_REVERB_AT_MID_RATE, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if read_mixer::<K24dProtocol, K24dMixerState>(&self.0, elem_id, elem_value)? {
            Ok(true)
        } else if read_reverb_return::<K24dProtocol, K24dMixerState>(&self.0, elem_id, elem_value)?
        {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                MIXER_ENABLE_NAME => {
                    elem_value.set_bool(&[self.0.data.enabled]);
                    Ok(true)
                }
                USE_CH_STRIP_AS_PLUGIN_NAME => {
                    elem_value.set_bool(&[self.0.data.use_ch_strip_as_plugin]);
                    Ok(true)
                }
                USE_REVERB_AT_MID_RATE => {
                    elem_value.set_bool(&[self.0.data.use_reverb_at_mid_rate]);
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
        if write_mixer::<K24dProtocol, K24dMixerState>(
            &mut self.0,
            req,
            node,
            elem_id,
            elem_value,
            timeout_ms,
        )? {
            Ok(true)
        } else if write_reverb_return::<K24dProtocol, K24dMixerState>(
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
                MIXER_ENABLE_NAME => {
                    let mut params = self.0.data.clone();
                    params.enabled = elem_value.boolean()[0];
                    let res = K24dProtocol::update_partial_segment(
                        req,
                        node,
                        &params,
                        &mut self.0,
                        timeout_ms,
                    );
                    debug!(params = ?self.0.data, ?res);
                    res.map(|_| true)
                }
                USE_CH_STRIP_AS_PLUGIN_NAME => {
                    let mut params = self.0.data.clone();
                    params.use_ch_strip_as_plugin = elem_value.boolean()[0];
                    let res = K24dProtocol::update_partial_segment(
                        req,
                        node,
                        &params,
                        &mut self.0,
                        timeout_ms,
                    );
                    debug!(params = ?self.0.data, ?res);
                    res.map(|_| true)
                }
                USE_REVERB_AT_MID_RATE => {
                    let mut params = self.0.data.clone();
                    params.use_reverb_at_mid_rate = elem_value.boolean()[0];
                    let res = K24dProtocol::update_partial_segment(
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
        if K24dProtocol::is_notified_segment(&self.0, msg) {
            let res = K24dProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug)]
struct HwStateCtl(K24dHwStateSegment, Vec<ElemId>);

impl HwStateCtl {
    fn cache(&mut self, req: &FwReq, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = K24dProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
        debug!(params = ?self.0.data, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        load_hw_state::<K24dProtocol, K24dHwState>(card_cntr)
            .map(|mut elem_id_list| self.1.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        read_hw_state::<K24dProtocol, K24dHwState>(&self.0, elem_id, elem_value)
    }

    fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        write_hw_state::<K24dProtocol, K24dHwState>(
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
        if K24dProtocol::is_notified_segment(&self.0, msg) {
            let res = K24dProtocol::cache_whole_segment(req, node, &mut self.0, timeout_ms);
            debug!(params = ?self.0.data, ?res);
            res
        } else {
            Ok(())
        }
    }
}
