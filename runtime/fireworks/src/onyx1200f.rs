// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2023 Takashi Sakamoto

use {super::*, protocols::onyx_f::Onyx1200fProtocol};

#[derive(Default, Debug)]
pub struct Onyx1200f {
    clk_ctl: SamplingClockCtl<Onyx1200fProtocol>,
    meter_ctl: HwMeterCtl<Onyx1200fProtocol>,
    monitor_ctl: MonitorCtl<Onyx1200fProtocol>,
    playback_ctl: PlaybackCtl<Onyx1200fProtocol>,
    output_ctl: OutCtl<Onyx1200fProtocol>,
    control_room_ctl: ControlRoomSourceCtl<Onyx1200fProtocol>,
    digital_mode_ctl: DigitalModeCtl<Onyx1200fProtocol>,
}

const TIMEOUT_MS: u32 = 100;

impl Onyx1200f {
    pub(crate) fn cache(&mut self, unit: &mut SndEfw) -> Result<(), Error> {
        self.clk_ctl.cache(unit, TIMEOUT_MS)?;
        self.meter_ctl.cache(unit, TIMEOUT_MS)?;
        self.monitor_ctl.cache(unit, TIMEOUT_MS)?;
        self.playback_ctl.cache(unit, TIMEOUT_MS)?;
        self.output_ctl.cache(unit, TIMEOUT_MS)?;
        self.control_room_ctl.cache(unit, TIMEOUT_MS)?;
        self.digital_mode_ctl.cache(unit, TIMEOUT_MS)?;

        Ok(())
    }
}

impl CtlModel<SndEfw> for Onyx1200f {
    fn load(&mut self, _: &mut SndEfw, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.clk_ctl.load(card_cntr, true)?;
        self.meter_ctl.load(card_cntr)?;
        self.monitor_ctl.load(card_cntr)?;
        self.playback_ctl.load(card_cntr)?;
        self.output_ctl.load(card_cntr)?;
        self.control_room_ctl.load(card_cntr)?;
        self.digital_mode_ctl.load(card_cntr)?;
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
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.control_room_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.digital_mode_ctl.read(elem_id, elem_value)? {
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
            .output_ctl
            .write(unit, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .control_room_ctl
            .write(unit, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .digital_mode_ctl
            .write(unit, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<SndEfw> for Onyx1200f {
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

impl NotifyModel<SndEfw, bool> for Onyx1200f {
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
                self.output_ctl.cache(unit, TIMEOUT_MS)?;
                self.control_room_ctl.cache(unit, TIMEOUT_MS)?;
                self.digital_mode_ctl.cache(unit, TIMEOUT_MS)?;
            }
        }
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndEfw,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.monitor_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.playback_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.output_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.control_room_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.digital_mode_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
