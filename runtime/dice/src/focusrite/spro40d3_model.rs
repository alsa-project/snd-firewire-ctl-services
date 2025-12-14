// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025 Andreas Persson

use {
    super::*,
    protocols::focusrite::spro40d3::*,
    alsa_ctl_tlv_codec::CTL_VALUE_MUTE,
};

#[derive(Default)]
pub struct SPro40D3Model {
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl<SPro40D3Protocol>,
    notified_elem_id_list: Vec<ElemId>,
    measured_elem_id_list: Vec<ElemId>,
    protocol: SPro40D3Protocol,
    gain_values: Vec<ElemValue>,
    router_out_src: ElemValue,
    router_mixer_src: ElemValue,
    router_meter_src: ElemValue,
}

const TIMEOUT_MS: u32 = 20;
    
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

impl CtlModel<(SndDice, FwNode)> for SPro40D3Model {
    fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        SPro40D3Protocol::read_general_sections(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .cache_whole_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.protocol.init_communication(&unit.1, TIMEOUT_MS)?;

        // There is not much to cache here, as I don't know any way to
        // read the current settings from the card.

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let mut notified_elem_id_list = Vec::new();
        let mut measured_elem_id_list = Vec::new();
        
        self.common_ctl.load(card_cntr)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_SRC_GAIN_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            18,
            COEF_MIN,
            COEF_MAX,
            COEF_STEP,
            16,
            Some(&Into::<Vec<u32>>::into(COEF_TLV)),
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        for elem_id in &notified_elem_id_list {
            let mut v = ElemValue::new();
            card_cntr.card.read_elem_value(elem_id, &mut v)?;
            self.gain_values.push(v);
        }
        
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_METER_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            0,
            0xfff,
            1,
            2,
            None,
            false,
        )
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MIXER_INPUT_METER_NAME, 0);
        card_cntr.add_int_elems(
            &elem_id,
            1,
            0,
            0xfff,
            1,
            18,
            None,
            false,
        )
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ROUTER_OUT_SRC_NAME, 0);
        let supported_source_labels = vec![
            "None",
            "Analog-1", "Analog-2", "Analog-3", "Analog-4",
            "Analog-5", "Analog-6", "Analog-7", "Analog-8",
            "S/PDIF-1", "S/PDIF-2",
            "ADAT-1", "ADAT-2", "ADAT-3", "ADAT-4",
            "ADAT-5", "ADAT-6", "ADAT-7", "ADAT-8",
            "Stream-1", "Stream-2", "Stream-3", "Stream-4",
            "Stream-5", "Stream-6", "Stream-7", "Stream-8",
            "Stream-9", "Stream-10", "Stream-11", "Stream-12",
            "Stream-13", "Stream-14", "Stream-15", "Stream-16",
            "Stream-17", "Stream-18", "Stream-19", "Stream-20",
            "Mixer-1", "Mixer-2", "Mixer-3", "Mixer-4",
            "Mixer-5", "Mixer-6", "Mixer-7", "Mixer-8",
            "Mixer-9", "Mixer-10", "Mixer-11", "Mixer-12",
            "Mixer-13", "Mixer-14", "Mixer-15", "Mixer-16"
        ];
        card_cntr.add_enum_elems(
            &elem_id,
            1,
            22,
            &supported_source_labels,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        card_cntr.card.read_elem_value(&elem_id, &mut self.router_out_src)?;
        
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ROUTER_MIXER_SRC_NAME, 0);
        card_cntr.add_enum_elems(
            &elem_id,
            1,
            18,
            &supported_source_labels[..39], // all, except the mixes
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        card_cntr.card.read_elem_value(&elem_id, &mut self.router_mixer_src)?;
        
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ROUTER_METER_SRC_NAME, 0);
        card_cntr.add_enum_elems(
            &elem_id,
            1,
            2,
            &supported_source_labels,
            None,
            true,
        )
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        card_cntr.card.read_elem_value(&elem_id, &mut self.router_meter_src)?;
        
        self.notified_elem_id_list = notified_elem_id_list;
        self.measured_elem_id_list = measured_elem_id_list;
        
        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            match elem_id.name().as_str() {
                OUT_METER_NAME => {
                    elem_value.set_int(&self.protocol.master_meter);
                    Ok(true)
                }
                MIXER_INPUT_METER_NAME => {
                    elem_value.set_int(&self.protocol.mixer_meter);
                    Ok(true)
                }
                _ => Ok(false),
            }
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        self.common_ctl.write(
            &unit.0,
            &self.req,
            &unit.1,
            &mut self.sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )?;

        let dst_ch = elem_id.index() as usize;
        match elem_id.name().as_str() {
            MIXER_SRC_GAIN_NAME => {
                let old_values = self.gain_values[dst_ch].int();
                let new_values = new.int();
                for i in 0..16 {
                    let old_value = old_values[i];
                    let new_value = new_values[i];
                    if new_value != old_value {
                        self.protocol.set_volume(&unit.1, dst_ch, i, new_value, TIMEOUT_MS)?;
                    }
                }
                self.gain_values[dst_ch].set_int(new_values);
                Ok(true)
            }
            ROUTER_OUT_SRC_NAME => {
                let new_values = new.enumerated();
                self.router_out_src.set_enum(new_values);
                self.protocol.set_routing(
                    &unit.1,
                    new_values,
                    self.router_mixer_src.enumerated(),
                    self.router_meter_src.enumerated(),
                    TIMEOUT_MS,
                )?;
                Ok(true)
            }
            ROUTER_MIXER_SRC_NAME => {
                let new_values = new.enumerated();
                self.router_mixer_src.set_enum(new_values);
                self.protocol.set_routing(
                    &unit.1,
                    self.router_out_src.enumerated(),
                    new_values,
                    self.router_meter_src.enumerated(),
                    TIMEOUT_MS,
                )?;
                Ok(true)
            }
            ROUTER_METER_SRC_NAME => {
                let new_values = new.enumerated();
                self.router_meter_src.set_enum(new_values);
                self.protocol.set_routing(
                    &unit.1,
                    self.router_out_src.enumerated(),
                    self.router_mixer_src.enumerated(),
                    new_values,
                    TIMEOUT_MS,
                )?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for SPro40D3Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_id_list);
        elem_id_list.extend_from_slice(&self.notified_elem_id_list);
    }

    fn parse_notification(&mut self, unit: &mut (SndDice, FwNode), msg: &u32) -> Result<(), Error> {
        self.common_ctl
            .parse_notification(&self.req, &unit.1, &mut self.sections, *msg, TIMEOUT_MS)
    }
}

impl MeasureModel<(SndDice, FwNode)> for SPro40D3Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.measured_elem_id_list);
        elem_id_list.extend_from_slice(&self.measured_elem_id_list);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .cache_partial_params(&self.req, &unit.1, &mut self.sections, TIMEOUT_MS)?;

        self.protocol.get_master_meter(&unit.1, self.common_ctl.global_params.current_rate, TIMEOUT_MS)?;
        self.protocol.get_mixer_meter(&unit.1, self.common_ctl.global_params.current_rate, TIMEOUT_MS)
    }
}

const OUT_METER_NAME: &str = "output-source-meter";
const MIXER_INPUT_METER_NAME: &str = "mixer-source-meter";

const ROUTER_OUT_SRC_NAME: &str = "output-source";
const ROUTER_MIXER_SRC_NAME: &str = "mixer-source";
const ROUTER_METER_SRC_NAME: &str = "meter-source";

const MIXER_SRC_GAIN_NAME: &str = "mixer-source-gain";
