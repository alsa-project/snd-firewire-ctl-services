// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2023 Takashi Sakamoto

use {super::*, protocols::audiofire::Audiofire9Protocol};

#[derive(Default, Debug)]
pub struct Audiofire9Model {
    clk_ctl: SamplingClockCtl<Audiofire9Protocol>,
    meter_ctl: HwMeterCtl<Audiofire9Protocol>,
    monitor_ctl: MonitorCtl<Audiofire9Protocol>,
    playback_ctl: PlaybackCtl<Audiofire9Protocol>,
    playback_solo_ctl: PlaybackSoloCtl<Audiofire9Protocol>,
    output_ctl: OutCtl<Audiofire9Protocol>,
    phys_output_ctl: PhysOutputCtl<Audiofire9Protocol>,
    phys_input_ctl: PhysInputCtl<Audiofire9Protocol>,
    digital_mode_ctl: DigitalModeCtl<Audiofire9Protocol>,
    iec60958_ctl: Iec60958Ctl<Audiofire9Protocol>,
}

const TIMEOUT_MS: u32 = 100;

impl CtlModel<SndEfw> for Audiofire9Model {
    fn cache(&mut self, unit: &mut SndEfw) -> Result<(), Error> {
        self.clk_ctl.cache(unit, TIMEOUT_MS)?;
        self.meter_ctl.cache(unit, TIMEOUT_MS)?;
        self.monitor_ctl.cache(unit, TIMEOUT_MS)?;
        self.playback_ctl.cache(unit, TIMEOUT_MS)?;
        self.playback_solo_ctl.cache(unit, TIMEOUT_MS)?;
        self.output_ctl.cache(unit, TIMEOUT_MS)?;
        self.phys_output_ctl.cache(unit, TIMEOUT_MS)?;
        self.phys_input_ctl.cache(unit, TIMEOUT_MS)?;
        self.digital_mode_ctl.cache(unit, TIMEOUT_MS)?;
        self.iec60958_ctl.cache(unit, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctl.load(card_cntr, false)?;
        self.meter_ctl.load(card_cntr)?;
        self.monitor_ctl.load(card_cntr)?;
        self.playback_ctl.load(card_cntr)?;
        self.playback_solo_ctl.load(card_cntr)?;
        self.output_ctl.load(card_cntr)?;
        self.phys_output_ctl.load(card_cntr)?;
        self.phys_input_ctl.load(card_cntr)?;
        self.digital_mode_ctl.load(card_cntr)?;
        self.iec60958_ctl.load(card_cntr)?;
        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
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
        } else if self.digital_mode_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.iec60958_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut SndEfw,
        elem_id: &ElemId,
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
        } else if self
            .digital_mode_ctl
            .write(unit, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .iec60958_ctl
            .write(unit, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<SndEfw> for Audiofire9Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.0);
    }

    fn measure_states(&mut self, unit: &mut SndEfw) -> Result<(), Error> {
        self.meter_ctl.cache(unit, TIMEOUT_MS)?;
        Ok(())
    }
}

impl NotifyModel<SndEfw, bool> for Audiofire9Model {
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
                self.phys_output_ctl.cache(unit, TIMEOUT_MS)?;
                self.phys_input_ctl.cache(unit, TIMEOUT_MS)?;
                self.digital_mode_ctl.cache(unit, TIMEOUT_MS)?;
                self.iec60958_ctl.cache(unit, TIMEOUT_MS)?;
            }
        }
        Ok(())
    }
}
