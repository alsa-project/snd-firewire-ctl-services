// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025 Andreas Persson

use {super::*, alsa_ctl_tlv_codec::CTL_VALUE_MUTE, protocols::focusrite::spro40d3::*};

#[derive(Default)]
struct MixerCtls {
    gain_values: Vec<[i32; MIX_COUNT]>,
}

impl MixerCtls {
    const COEF_MIN: i32 = 0;
    const COEF_MAX: i32 = 0x7fff;
    const COEF_STEP: i32 = 1;
    // 0dB is 0x1fff
    const COEF_TLV: DbInterval = DbInterval {
        min: CTL_VALUE_MUTE,
        max: 1204,
        linear: true,
        mute_avail: false,
    };

    fn cache(
        &mut self,
        protocol: &mut SPro40D3Protocol,
        node: &FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.gain_values
            .resize_with(CHANNEL_COUNT, Default::default);

        // There doesn't seem to be a way to read the current mixer
        // levels and routing tables from the card, so we instead
        // initialize the hardware and the cache with some default
        // values. This ensures the alsa values will be in sync with
        // the hardware.

        for mix in 0..MIX_COUNT {
            for channel in 0..CHANNEL_COUNT {
                let volume = if mix < 2 && channel < 8 {
                    0x12f3
                } else if (channel == 16 && mix == 0) || (channel == 17 && mix == 1) {
                    0x1ff6
                } else {
                    0
                };
                self.gain_values[channel][mix] = volume;
                protocol.set_volume(node, channel, mix, volume, timeout_ms)?;
            }
        }
        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SRC_GAIN_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            CHANNEL_COUNT,
            Self::COEF_MIN,
            Self::COEF_MAX,
            Self::COEF_STEP,
            MIX_COUNT,
            Some(&Into::<Vec<u32>>::into(Self::COEF_TLV)),
            true,
        )
    }

    fn read(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_SRC_GAIN_NAME => {
                let dst_ch = elem_id.index() as usize;
                elem_value.set_int(&self.gain_values[dst_ch]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        protocol: &mut SPro40D3Protocol,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MIXER_SRC_GAIN_NAME => {
                let dst_ch = elem_id.index() as usize;
                let old_values = &mut self.gain_values[dst_ch];
                let new_values = elem_value.int();

                for i in 0..MIX_COUNT {
                    let old_value = old_values[i];
                    let new_value = new_values[i];
                    if new_value != old_value {
                        protocol.set_volume(node, dst_ch, i, new_value, timeout_ms)?;
                        old_values[i] = new_value;
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default)]
struct RouterCtls {
    router_out_src: Vec<u32>,
    router_mixer_src: Vec<u32>,
    router_meter_src: [u32; MASTER_METER_COUNT],
}

impl RouterCtls {
    fn cache(
        &mut self,
        protocol: &mut SPro40D3Protocol,
        node: &FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.router_out_src
            .resize_with(OUTPUT_COUNT, Default::default);
        self.router_mixer_src
            .resize_with(CHANNEL_COUNT, Default::default);

        self.router_out_src[0..10].copy_from_slice(&[39, 40, 39, 40, 39, 40, 39, 40, 39, 40]);
        self.router_mixer_src
            .copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0, 19, 20]);
        self.router_meter_src.copy_from_slice(&[39, 40]);

        self.set_routing(protocol, node, timeout_ms)?;
        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let mut notified_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ROUTER_OUT_SRC_NAME, 0);
        let supported_source_labels = vec![
            "None",
            "Analog-1",
            "Analog-2",
            "Analog-3",
            "Analog-4",
            "Analog-5",
            "Analog-6",
            "Analog-7",
            "Analog-8",
            "S/PDIF-1",
            "S/PDIF-2",
            "ADAT-1",
            "ADAT-2",
            "ADAT-3",
            "ADAT-4",
            "ADAT-5",
            "ADAT-6",
            "ADAT-7",
            "ADAT-8",
            "Stream-1",
            "Stream-2",
            "Stream-3",
            "Stream-4",
            "Stream-5",
            "Stream-6",
            "Stream-7",
            "Stream-8",
            "Stream-9",
            "Stream-10",
            "Stream-11",
            "Stream-12",
            "Stream-13",
            "Stream-14",
            "Stream-15",
            "Stream-16",
            "Stream-17",
            "Stream-18",
            "Stream-19",
            "Stream-20",
            "Mixer-1",
            "Mixer-2",
            "Mixer-3",
            "Mixer-4",
            "Mixer-5",
            "Mixer-6",
            "Mixer-7",
            "Mixer-8",
            "Mixer-9",
            "Mixer-10",
            "Mixer-11",
            "Mixer-12",
            "Mixer-13",
            "Mixer-14",
            "Mixer-15",
            "Mixer-16",
        ];
        card_cntr
            .add_enum_elems(
                &elem_id,
                1,
                OUTPUT_COUNT,
                &supported_source_labels,
                None,
                true,
            )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ROUTER_MIXER_SRC_NAME, 0);
        card_cntr
            .add_enum_elems(
                &elem_id,
                1,
                CHANNEL_COUNT,
                &supported_source_labels[..39], // all, except the mixes
                None,
                true,
            )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ROUTER_METER_SRC_NAME, 0);
        card_cntr
            .add_enum_elems(
                &elem_id,
                1,
                MASTER_METER_COUNT,
                &supported_source_labels,
                None,
                true,
            )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        Ok(notified_elem_id_list)
    }

    fn read(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ROUTER_OUT_SRC_NAME => {
                elem_value.set_enum(&self.router_out_src);
                Ok(true)
            }
            ROUTER_MIXER_SRC_NAME => {
                elem_value.set_enum(&self.router_mixer_src);
                Ok(true)
            }
            ROUTER_METER_SRC_NAME => {
                elem_value.set_enum(&self.router_meter_src);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        protocol: &mut SPro40D3Protocol,
        node: &FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            ROUTER_OUT_SRC_NAME => {
                self.router_out_src
                    .copy_from_slice(&elem_value.enumerated()[0..OUTPUT_COUNT]);
                self.set_routing(protocol, node, timeout_ms)?;
                Ok(true)
            }
            ROUTER_MIXER_SRC_NAME => {
                self.router_mixer_src
                    .copy_from_slice(&elem_value.enumerated()[0..CHANNEL_COUNT]);
                self.set_routing(protocol, node, timeout_ms)?;
                Ok(true)
            }
            ROUTER_METER_SRC_NAME => {
                self.router_meter_src
                    .copy_from_slice(&elem_value.enumerated()[0..MASTER_METER_COUNT]);
                self.set_routing(protocol, node, timeout_ms)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn set_routing(
        &mut self,
        protocol: &mut SPro40D3Protocol,
        node: &FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        protocol.set_routing(
            node,
            &self.router_out_src,
            &self.router_mixer_src,
            &self.router_meter_src,
            timeout_ms,
        )
    }
}

#[derive(Default)]
struct MeterCtls {
    master_meter: [i32; MASTER_METER_COUNT],
    mixer_meter: Vec<i32>,
}

impl MeterCtls {
    fn cache(
        &mut self,
        protocol: &mut SPro40D3Protocol,
        node: &FwNode,
        current_rate: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.mixer_meter
            .resize_with(CHANNEL_COUNT, Default::default);
        protocol.get_master_meter(node, current_rate, &mut self.master_meter, timeout_ms)?;
        protocol.get_mixer_meter(node, current_rate, &mut self.mixer_meter, timeout_ms)
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<Vec<ElemId>, Error> {
        let mut measured_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(&elem_id, 1, 0, 0xfff, 1, 2, None, false)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_INPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(&elem_id, 1, 0, 0xfff, 1, CHANNEL_COUNT, None, false)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        Ok(measured_elem_id_list)
    }

    fn read(&self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OUT_METER_NAME => {
                elem_value.set_int(&self.master_meter);
                Ok(true)
            }
            MIXER_INPUT_METER_NAME => {
                elem_value.set_int(&self.mixer_meter);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default)]
pub struct SPro40D3Model {
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl<SPro40D3Protocol>,
    notified_elem_id_list: Vec<ElemId>,
    measured_elem_id_list: Vec<ElemId>,
    protocol: SPro40D3Protocol,
    mixer_ctls: MixerCtls,
    router_ctls: RouterCtls,
    meter_ctls: MeterCtls,
}

const TIMEOUT_MS: u32 = 20;

impl CtlModel<(SndDice, FwNode)> for SPro40D3Model {
    fn cache(&mut self, (_, node): &mut (SndDice, FwNode)) -> Result<(), Error> {
        SPro40D3Protocol::read_general_sections(&self.req, node, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .cache_whole_params(&self.req, node, &mut self.sections, TIMEOUT_MS)?;

        self.protocol.init_communication(node, TIMEOUT_MS)?;

        self.mixer_ctls
            .cache(&mut self.protocol, node, TIMEOUT_MS)?;
        self.router_ctls.cache(&mut self.protocol, node, TIMEOUT_MS)
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let mut notified_elem_id_list = Vec::new();
        let mut measured_elem_id_list = Vec::new();

        self.common_ctl.load(card_cntr)?;

        self.mixer_ctls
            .load(card_cntr)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        self.router_ctls
            .load(card_cntr)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        self.meter_ctls
            .load(card_cntr)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        self.notified_elem_id_list = notified_elem_id_list;
        self.measured_elem_id_list = measured_elem_id_list;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        let res = self.common_ctl.read(elem_id, elem_value)?
            || self.mixer_ctls.read(elem_id, elem_value)?
            || self.router_ctls.read(elem_id, elem_value)?
            || self.meter_ctls.read(elem_id, elem_value)?;
        Ok(res)
    }

    fn write(
        &mut self,
        (unit, node): &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        let res = self.common_ctl.write(
            unit,
            &self.req,
            node,
            &mut self.sections,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? || self.mixer_ctls.write(
            &mut self.protocol,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? || self.router_ctls.write(
            &mut self.protocol,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )?;
        Ok(res)
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for SPro40D3Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_id_list);
        elem_id_list.extend_from_slice(&self.notified_elem_id_list);
    }

    fn parse_notification(
        &mut self,
        (_, node): &mut (SndDice, FwNode),
        msg: &u32,
    ) -> Result<(), Error> {
        self.common_ctl
            .parse_notification(&self.req, node, &mut self.sections, *msg, TIMEOUT_MS)
    }
}

impl MeasureModel<(SndDice, FwNode)> for SPro40D3Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.measured_elem_id_list);
        elem_id_list.extend_from_slice(&self.measured_elem_id_list);
    }

    fn measure_states(&mut self, (_, node): &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .cache_partial_params(&self.req, node, &mut self.sections, TIMEOUT_MS)?;

        self.meter_ctls.cache(
            &mut self.protocol,
            node,
            self.common_ctl.global_params.current_rate,
            TIMEOUT_MS,
        )
    }
}

const OUT_METER_NAME: &str = "output-source-meter";
const MIXER_INPUT_METER_NAME: &str = "mixer-source-meter";

const ROUTER_OUT_SRC_NAME: &str = "output-source";
const ROUTER_MIXER_SRC_NAME: &str = "mixer-source";
const ROUTER_METER_SRC_NAME: &str = "meter-source";

const MIXER_SRC_GAIN_NAME: &str = "mixer-source-gain";
