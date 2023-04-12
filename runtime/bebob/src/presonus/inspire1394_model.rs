// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::*,
    protocols::{presonus::inspire1394::*, *},
};

#[derive(Default, Debug)]
pub struct Inspire1394Model {
    avc: BebobAvc,
    req: FwReq,
    clk_ctl: ClkCtl,
    meter_ctl: MeterCtl,
    phys_in_ctl: PhysInputCtl,
    phys_out_ctl: PhysOutputCtl,
    hp_ctl: HeadphoneCtl,
    mixer_phys_src_ctl: MixerPhysSourceCtl,
    mixer_stream_src_ctl: MixerStreamSourceCtl,
    input_switch_ctl: InputSwitchCtl,
}

const FCP_TIMEOUT_MS: u32 = 100;
const TIMEOUT_MS: u32 = 50;

#[derive(Default, Debug)]
struct ClkCtl(Vec<ElemId>, MediaClockParameters, SamplingClockParameters);

impl MediaClkFreqCtlOperation<Inspire1394ClkProtocol> for ClkCtl {
    fn state(&self) -> &MediaClockParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut MediaClockParameters {
        &mut self.1
    }
}

impl SamplingClkSrcCtlOperation<Inspire1394ClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal"];

    fn state(&self) -> &SamplingClockParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut SamplingClockParameters {
        &mut self.2
    }
}

#[derive(Default, Debug)]
struct MeterCtl(Vec<ElemId>, Inspire1394Meter);

impl MeterCtl {
    const STREAM_INPUT_METER_NAME: &'static str = "stream-input";

    const METER_TLV: DbInterval = DbInterval {
        min: -14400,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, IN_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Inspire1394MeterProtocol::LEVEL_MIN,
                Inspire1394MeterProtocol::LEVEL_MAX,
                Inspire1394MeterProtocol::LEVEL_STEP,
                4,
                Some(&Into::<Vec<u32>>::into(Self::METER_TLV)),
                false,
            )
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id =
            ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, Self::STREAM_INPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Inspire1394MeterProtocol::LEVEL_MIN,
                Inspire1394MeterProtocol::LEVEL_MAX,
                Inspire1394MeterProtocol::LEVEL_STEP,
                2,
                Some(&Into::<Vec<u32>>::into(Self::METER_TLV)),
                false,
            )
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Inspire1394MeterProtocol::LEVEL_MIN,
                Inspire1394MeterProtocol::LEVEL_MAX,
                Inspire1394MeterProtocol::LEVEL_STEP,
                2,
                Some(&Into::<Vec<u32>>::into(Self::METER_TLV)),
                false,
            )
            .map(|mut elem_id_list| self.0.append(&mut elem_id_list))?;

        Ok(())
    }

    fn cache(
        &mut self,
        req: &FwReq,
        unit: &(SndUnit, FwNode),
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = Inspire1394MeterProtocol::cache(req, &unit.1, &mut self.1, timeout_ms);
        debug!(params = ?self.1, ?res);
        res
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            IN_METER_NAME => {
                elem_value.set_int(&self.1.phys_inputs);
                Ok(true)
            }
            Self::STREAM_INPUT_METER_NAME => {
                elem_value.set_int(&self.1.stream_inputs);
                Ok(true)
            }
            OUT_METER_NAME => {
                elem_value.set_int(&self.1.phys_outputs);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Debug)]
struct PhysInputCtl(AvcLevelParameters, AvcMuteParameters);

impl Default for PhysInputCtl {
    fn default() -> Self {
        Self(
            Inspire1394PhysInputProtocol::create_level_parameters(),
            Inspire1394PhysInputProtocol::create_mute_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<Inspire1394PhysInputProtocol> for PhysInputCtl {
    const LEVEL_NAME: &'static str = "analog-input-gain";
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-input-1",
        "analog-input-2",
        "analog-input-3",
        "analog-input-4",
    ];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcMuteCtlOperation<Inspire1394PhysInputProtocol> for PhysInputCtl {
    const MUTE_NAME: &'static str = "analog-input-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.1
    }
}

#[derive(Debug)]
struct PhysOutputCtl(AvcLevelParameters, AvcMuteParameters, AvcSelectorParameters);

impl Default for PhysOutputCtl {
    fn default() -> Self {
        Self(
            Inspire1394PhysOutputProtocol::create_level_parameters(),
            Inspire1394PhysOutputProtocol::create_mute_parameters(),
            Inspire1394PhysOutputProtocol::create_selector_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<Inspire1394PhysOutputProtocol> for PhysOutputCtl {
    const LEVEL_NAME: &'static str = "analog-output-volume";
    const PORT_LABELS: &'static [&'static str] = &["analog-output-1", "analog-output-2"];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcMuteCtlOperation<Inspire1394PhysOutputProtocol> for PhysOutputCtl {
    const MUTE_NAME: &'static str = "analog-output-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.1
    }
}

impl AvcSelectorCtlOperation<Inspire1394PhysOutputProtocol> for PhysOutputCtl {
    const SELECTOR_NAME: &'static str = "output-source";
    const SELECTOR_LABELS: &'static [&'static str] = &["analog-output-1/2"];
    const ITEM_LABELS: &'static [&'static str] = &["mixer-output-1/2", "stream-input-1/2"];

    fn state(&self) -> &AvcSelectorParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut AvcSelectorParameters {
        &mut self.2
    }
}

#[derive(Debug)]
struct HeadphoneCtl(AvcLevelParameters, AvcMuteParameters);

impl Default for HeadphoneCtl {
    fn default() -> Self {
        Self(
            Inspire1394HeadphoneProtocol::create_level_parameters(),
            Inspire1394HeadphoneProtocol::create_mute_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<Inspire1394HeadphoneProtocol> for HeadphoneCtl {
    const LEVEL_NAME: &'static str = "headphone-volume";
    const PORT_LABELS: &'static [&'static str] = &["headphone-1", "headphone-2"];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcMuteCtlOperation<Inspire1394HeadphoneProtocol> for HeadphoneCtl {
    const MUTE_NAME: &'static str = "headphone-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.1
    }
}

#[derive(Debug)]
struct MixerPhysSourceCtl(
    AvcLevelParameters,
    AvcLrBalanceParameters,
    AvcMuteParameters,
);

impl Default for MixerPhysSourceCtl {
    fn default() -> Self {
        Self(
            Inspire1394MixerAnalogSourceProtocol::create_level_parameters(),
            Inspire1394MixerAnalogSourceProtocol::create_lr_balance_parameters(),
            Inspire1394MixerAnalogSourceProtocol::create_mute_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<Inspire1394MixerAnalogSourceProtocol> for MixerPhysSourceCtl {
    const LEVEL_NAME: &'static str = "mixer-analog-source-gain";
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-input-1",
        "analog-input-2",
        "analog-input-3",
        "analog-input-4",
    ];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcLrBalanceCtlOperation<Inspire1394MixerAnalogSourceProtocol> for MixerPhysSourceCtl {
    const BALANCE_NAME: &'static str = "mixer-analog-source-balance";

    fn state(&self) -> &AvcLrBalanceParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcLrBalanceParameters {
        &mut self.1
    }
}

impl AvcMuteCtlOperation<Inspire1394MixerAnalogSourceProtocol> for MixerPhysSourceCtl {
    const MUTE_NAME: &'static str = "mixer-analog-source-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.2
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.2
    }
}

#[derive(Debug)]
struct MixerStreamSourceCtl(AvcLevelParameters, AvcMuteParameters);

impl Default for MixerStreamSourceCtl {
    fn default() -> Self {
        Self(
            Inspire1394MixerStreamSourceProtocol::create_level_parameters(),
            Inspire1394MixerStreamSourceProtocol::create_mute_parameters(),
        )
    }
}

impl AvcLevelCtlOperation<Inspire1394MixerStreamSourceProtocol> for MixerStreamSourceCtl {
    const LEVEL_NAME: &'static str = "mixer-stream-source-gain";
    const PORT_LABELS: &'static [&'static str] = &["stream-input-1/2"];

    fn state(&self) -> &AvcLevelParameters {
        &self.0
    }

    fn state_mut(&mut self) -> &mut AvcLevelParameters {
        &mut self.0
    }
}

impl AvcMuteCtlOperation<Inspire1394MixerStreamSourceProtocol> for MixerStreamSourceCtl {
    const MUTE_NAME: &'static str = "mixer-stream-source-mute";

    fn state(&self) -> &AvcMuteParameters {
        &self.1
    }

    fn state_mut(&mut self) -> &mut AvcMuteParameters {
        &mut self.1
    }
}

#[derive(Default, Debug)]
struct InputSwitchCtl(Inspire1394SwitchParameters);

impl InputSwitchCtl {
    const MIC_PHANTOM_NAME: &'static str = "mic-phantom";
    const MIC_BOOST_NAME: &'static str = "mic-boost";
    const MIC_LIMIT_NAME: &'static str = "mic-limit";
    const LINE_PHONO_NAME: &'static str = "line-input-phono";

    const MIC_PHANTOM_LABELS: &'static [&'static str] = &["analog-input-1", "analog-input-2"];
    const MIC_BOOST_LABELS: &'static [&'static str] = &["analog-input-1", "analog-input-2"];
    const MIC_LIMIT_LABELS: &'static [&'static str] = &["analog-input-1", "analog-input-2"];
    const LINE_PHONO_LABELS: &'static [&'static str] = &["analog-input-3/4"];

    fn load(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        [
            (Self::MIC_PHANTOM_NAME, Self::MIC_PHANTOM_LABELS),
            (Self::MIC_BOOST_NAME, Self::MIC_BOOST_LABELS),
            (Self::MIC_LIMIT_NAME, Self::MIC_LIMIT_LABELS),
            (Self::LINE_PHONO_NAME, Self::LINE_PHONO_LABELS),
        ]
        .iter()
        .try_for_each(|(name, labels)| {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, name, 0);
            card_cntr
                .add_bool_elems(&elem_id, 1, labels.len(), true)
                .map(|_| ())
        })
    }

    fn cache(&mut self, avc: &BebobAvc, timeout_ms: u32) -> Result<(), Error> {
        let res = Inspire1394SwitchProtocol::cache(avc, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::MIC_PHANTOM_NAME => {
                elem_value.set_bool(&self.0.pair0_phantom);
                Ok(true)
            }
            Self::MIC_BOOST_NAME => {
                elem_value.set_bool(&self.0.pair0_boost);
                Ok(true)
            }
            Self::MIC_LIMIT_NAME => {
                elem_value.set_bool(&self.0.pair0_limit);
                Ok(true)
            }
            Self::LINE_PHONO_NAME => {
                elem_value.set_bool(&[self.0.pair1_phono]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        avc: &BebobAvc,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            Self::MIC_PHANTOM_NAME => {
                let mut params = self.0.clone();
                let vals = &new.boolean()[..2];
                params.pair0_phantom.copy_from_slice(&vals);
                let res = Inspire1394SwitchProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::MIC_BOOST_NAME => {
                let mut params = self.0.clone();
                let vals = &new.boolean()[..2];
                params.pair0_boost.copy_from_slice(&vals);
                let res = Inspire1394SwitchProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::MIC_LIMIT_NAME => {
                let mut params = self.0.clone();
                let vals = &new.boolean()[..2];
                params.pair0_limit.copy_from_slice(&vals);
                let res = Inspire1394SwitchProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            Self::LINE_PHONO_NAME => {
                let mut params = self.0.clone();
                params.pair1_phono = new.boolean()[0];
                let res = Inspire1394SwitchProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

impl CtlModel<(SndUnit, FwNode)> for Inspire1394Model {
    fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.clk_ctl.cache_freq(&self.avc, FCP_TIMEOUT_MS)?;
        self.clk_ctl.cache_src(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_out_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.hp_ctl.cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_phys_src_ctl
            .cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_stream_src_ctl
            .cache_levels(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_phys_src_ctl
            .cache_balances(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_out_ctl.cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;
        self.hp_ctl.cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_phys_src_ctl
            .cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;
        self.mixer_stream_src_ctl
            .cache_mutes(&self.avc, FCP_TIMEOUT_MS)?;
        self.phys_out_ctl
            .cache_selectors(&self.avc, FCP_TIMEOUT_MS)?;
        self.input_switch_ctl.cache(&self.avc, FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctl
            .load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl
            .load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.meter_ctl.load(card_cntr)?;

        self.phys_in_ctl.load_level(card_cntr)?;
        self.phys_in_ctl.load_mute(card_cntr)?;
        self.phys_out_ctl.load_level(card_cntr)?;
        self.phys_out_ctl.load_mute(card_cntr)?;
        self.phys_out_ctl.load_selector(card_cntr)?;
        self.hp_ctl.load_level(card_cntr)?;
        self.hp_ctl.load_mute(card_cntr)?;
        self.mixer_phys_src_ctl.load_level(card_cntr)?;
        self.mixer_phys_src_ctl.load_balance(card_cntr)?;
        self.mixer_phys_src_ctl.load_mute(card_cntr)?;
        self.mixer_stream_src_ctl.load_level(card_cntr)?;
        self.mixer_stream_src_ctl.load_mute(card_cntr)?;
        self.input_switch_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.read_freq(elem_id, elem_value)? {
            Ok(true)
        } else if self.clk_ctl.read_src(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_in_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_in_ctl.read_mutes(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_out_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_out_ctl.read_mutes(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_out_ctl.read_selectors(elem_id, elem_value)? {
            Ok(true)
        } else if self.hp_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.hp_ctl.read_mutes(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_phys_src_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_phys_src_ctl.read_balances(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_phys_src_ctl.read_mutes(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_stream_src_ctl.read_levels(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_stream_src_ctl.read_mutes(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_switch_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.write_freq(
            &mut unit.0,
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS * 3,
        )? {
            Ok(true)
        } else if self.clk_ctl.write_src(
            &mut unit.0,
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .phys_in_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_in_ctl
            .write_mute(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_out_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_out_ctl
            .write_mute(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_out_ctl
            .write_selector(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .hp_ctl
            .write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .hp_ctl
            .write_mute(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.mixer_phys_src_ctl.write_level(
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_phys_src_ctl.write_balance(
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_phys_src_ctl.write_mute(
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_stream_src_ctl.write_level(
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.mixer_stream_src_ctl.write_mute(
            &self.avc,
            elem_id,
            old,
            new,
            FCP_TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self
            .input_switch_ctl
            .write(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndUnit, FwNode), bool> for Inspire1394Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.0);
    }

    fn parse_notification(
        &mut self,
        _: &mut (SndUnit, FwNode),
        &locked: &bool,
    ) -> Result<(), Error> {
        if locked {
            self.clk_ctl.cache_freq(&self.avc, FCP_TIMEOUT_MS)?;
        }
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.clk_ctl.read_freq(elem_id, elem_value)
    }
}

impl MeasureModel<(SndUnit, FwNode)> for Inspire1394Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
    }

    fn measure_states(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.meter_ctl.cache(&self.req, unit, TIMEOUT_MS)
    }

    fn measure_elem(
        &mut self,
        _: &(SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.meter_ctl.read(elem_id, elem_value)
    }
}
