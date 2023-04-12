// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2023 Takashi Sakamoto

use {super::*, protocols::rip::RipProtocol};

#[derive(Default, Debug)]
pub struct Rip {
    clk_ctl: SamplingClockCtl<RipProtocol>,
    meter_ctl: HwMeterCtl<RipProtocol>,
    monitor_ctl: MonitorCtl<RipProtocol>,
    playback_ctl: PlaybackCtl<RipProtocol>,
    playback_solo_ctl: PlaybackSoloCtl<RipProtocol>,
    output_ctl: OutCtl<RipProtocol>,
    guitar_ctl: RobotGuitarCtl<RipProtocol>,
}

const TIMEOUT_MS: u32 = 100;

impl CtlModel<SndEfw> for Rip {
    fn cache(&mut self, unit: &mut SndEfw) -> Result<(), Error> {
        self.clk_ctl.cache(unit, TIMEOUT_MS)?;
        self.meter_ctl.cache(unit, TIMEOUT_MS)?;
        self.monitor_ctl.cache(unit, TIMEOUT_MS)?;
        self.playback_ctl.cache(unit, TIMEOUT_MS)?;
        self.playback_solo_ctl.cache(unit, TIMEOUT_MS)?;
        self.output_ctl.cache(unit, TIMEOUT_MS)?;
        self.guitar_ctl.cache(unit, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctl.load(card_cntr, false)?;
        self.meter_ctl.load(card_cntr)?;
        self.monitor_ctl.load(card_cntr)?;
        self.playback_ctl.load(card_cntr)?;
        self.playback_solo_ctl.load(card_cntr)?;
        self.output_ctl.load(card_cntr)?;
        self.guitar_ctl.load(card_cntr)?;
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
        } else if self.guitar_ctl.read(elem_id, elem_value)? {
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
            .guitar_ctl
            .write(unit, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<SndEfw> for Rip {
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

impl NotifyModel<SndEfw, bool> for Rip {
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
                self.output_ctl.cache(unit, TIMEOUT_MS)?;
                self.guitar_ctl.cache(unit, TIMEOUT_MS)?;
            }
        }
        Ok(())
    }
}
