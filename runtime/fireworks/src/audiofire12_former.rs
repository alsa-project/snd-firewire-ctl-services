// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2023 Takashi Sakamoto

use {super::*, protocols::audiofire::Audiofire12FormerProtocol};

#[derive(Default, Debug)]
pub struct Audiofire12Former {
    higher_rates_supported: bool,
    clk_ctl: SamplingClockCtl<Audiofire12FormerProtocol>,
    meter_ctl: HwMeterCtl<Audiofire12FormerProtocol>,
    monitor_ctl: MonitorCtl<Audiofire12FormerProtocol>,
    playback_ctl: PlaybackCtl<Audiofire12FormerProtocol>,
    playback_solo_ctl: PlaybackSoloCtl<Audiofire12FormerProtocol>,
    output_ctl: OutCtl<Audiofire12FormerProtocol>,
    phys_output_ctl: PhysOutputCtl<Audiofire12FormerProtocol>,
    phys_input_ctl: PhysInputCtl<Audiofire12FormerProtocol>,
}

const TIMEOUT_MS: u32 = 100;

impl Audiofire12Former {
    pub(crate) fn is_later_model(&mut self, unit: &mut SndEfw) -> Result<bool, Error> {
        let mut hw_info = HwInfo::default();
        let res = Audiofire12FormerProtocol::cache_wholly(unit, &mut hw_info, TIMEOUT_MS);
        debug!(params = ?hw_info, ?res);
        res?;

        Ok(hw_info
            .caps
            .iter()
            .find(|cap| HwCap::InputGain.eq(cap))
            .is_some())
    }
}

impl CtlModel<SndEfw> for Audiofire12Former {
    fn cache(&mut self, unit: &mut SndEfw) -> Result<(), Error> {
        let mut hw_info = HwInfo::default();
        let res = Audiofire12FormerProtocol::cache_wholly(unit, &mut hw_info, TIMEOUT_MS);
        debug!(params = ?hw_info, ?res);
        res?;

        self.higher_rates_supported = hw_info
            .clk_rates
            .iter()
            .find(|&rate| *rate >= 176400)
            .is_some();

        self.clk_ctl.cache(unit, TIMEOUT_MS)?;
        self.meter_ctl.cache(unit, TIMEOUT_MS)?;
        self.monitor_ctl.cache(unit, TIMEOUT_MS)?;
        self.playback_ctl.cache(unit, TIMEOUT_MS)?;
        self.playback_solo_ctl.cache(unit, TIMEOUT_MS)?;
        self.output_ctl.cache(unit, TIMEOUT_MS)?;
        self.phys_output_ctl.cache(unit, TIMEOUT_MS)?;
        self.phys_input_ctl.cache(unit, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctl.load(card_cntr, self.higher_rates_supported)?;
        self.meter_ctl.load(card_cntr)?;
        self.monitor_ctl.load(card_cntr)?;
        self.playback_ctl.load(card_cntr)?;
        self.playback_solo_ctl.load(card_cntr)?;
        self.output_ctl.load(card_cntr)?;
        self.phys_output_ctl.load(card_cntr)?;
        self.phys_input_ctl.load(card_cntr)?;
        Ok(())
    }

    fn read(
        &mut self,
        _: &mut SndEfw,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.monitor_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.playback_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.playback_solo_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.phys_input_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
        _: &ElemValue,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.write(unit, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self
            .monitor_ctl
            .write(unit, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .playback_ctl
            .write(unit, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .playback_solo_ctl
            .write(unit, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .output_ctl
            .write(unit, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_output_ctl
            .write(unit, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .phys_input_ctl
            .write(unit, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<SndEfw> for Audiofire12Former {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
    }

    fn measure_states(&mut self, unit: &mut SndEfw) -> Result<(), Error> {
        self.meter_ctl.cache(unit, TIMEOUT_MS)?;
        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &SndEfw,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndEfw, bool> for Audiofire12Former {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.elem_id_list);
    }

    fn parse_notification(&mut self, unit: &mut SndEfw, &locked: &bool) -> Result<(), Error> {
        if locked {
            let rate = self.clk_ctl.params.rate;
            self.clk_ctl.cache(unit, TIMEOUT_MS)?;
            if self.clk_ctl.params.rate != rate {
                self.monitor_ctl.cache(unit, TIMEOUT_MS)?;
                self.playback_ctl.cache(unit, TIMEOUT_MS)?;
                self.playback_solo_ctl.cache(unit, TIMEOUT_MS)?;
                self.phys_input_ctl.cache(unit, TIMEOUT_MS)?;
            }
        }
        Ok(())
    }
}
