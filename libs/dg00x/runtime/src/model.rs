// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use core::card_cntr::*;

use hinawa::FwReq;
use hinawa::{SndDg00x, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use dg00x_protocols::*;

use super::monitor_ctl::MonitorCtl;

const TIMEOUT_MS: u32 = 100;

pub struct Digi002Model {
    req: FwReq,
    common_ctl: Digi002CommonCtl,
    monitor: MonitorCtl,
}

impl Default for Digi002Model {
    fn default() -> Self {
        Self {
            req: FwReq::new(),
            common_ctl: Default::default(),
            monitor: MonitorCtl::new(),
        }
    }
}

#[derive(Default)]
struct Digi002CommonCtl(ClockRate, Vec<ElemId>);

impl AsRef<ClockRate> for Digi002CommonCtl {
    fn as_ref(&self) -> &ClockRate {
        &self.0
    }
}

impl AsMut<ClockRate> for Digi002CommonCtl {
    fn as_mut(&mut self) -> &mut ClockRate {
        &mut self.0
    }
}

impl Dg00xCommonCtl<Digi002Protocol> for Digi002CommonCtl {}

impl NotifyModel<SndDg00x, bool> for Digi002Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.1);
        elem_id_list.extend_from_slice(&self.monitor.notified_elems);
    }

    fn parse_notification(&mut self, unit: &mut SndDg00x, &locked: &bool) -> Result<(), Error> {
        self.common_ctl.handle_lock_notification(locked, unit, &mut self.req, TIMEOUT_MS)?;
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        unit: &SndDg00x,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read_notified_elems(elem_id, elem_value)? {
            Ok(true)
        } else if self.monitor.read_notified_elems(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl CtlModel<SndDg00x> for Digi002Model {
    fn load(
        &mut self,
        unit: &mut SndDg00x,
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.common_ctl.load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.common_ctl.1.append(&mut elem_id_list))?;
        self.monitor.load(&unit, &self.req, card_cntr)?;
        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut SndDg00x,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.monitor.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDg00x,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.monitor.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

pub struct Digi003Model {
    req: FwReq,
    common_ctl: Digi003CommonCtl,
    monitor: MonitorCtl,
}

impl Default for Digi003Model {
    fn default() -> Self {
        Self {
            req: FwReq::new(),
            common_ctl: Default::default(),
            monitor: MonitorCtl::new(),
        }
    }
}

#[derive(Default)]
struct Digi003CommonCtl(ClockRate, Vec<ElemId>);

impl AsRef<ClockRate> for Digi003CommonCtl {
    fn as_ref(&self) -> &ClockRate {
        &self.0
    }
}

impl AsMut<ClockRate> for Digi003CommonCtl {
    fn as_mut(&mut self) -> &mut ClockRate {
        &mut self.0
    }
}

impl Dg00xCommonCtl<Digi003Protocol> for Digi003CommonCtl {}

impl NotifyModel<SndDg00x, bool> for Digi003Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.1);
        elem_id_list.extend_from_slice(&self.monitor.notified_elems);
    }

    fn parse_notification(&mut self, unit: &mut SndDg00x, &locked: &bool) -> Result<(), Error> {
        self.common_ctl.handle_lock_notification(locked, unit, &mut self.req, TIMEOUT_MS)?;
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        unit: &SndDg00x,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read_notified_elems(elem_id, elem_value)? {
            Ok(true)
        } else if self.monitor.read_notified_elems(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl CtlModel<SndDg00x> for Digi003Model {
    fn load(
        &mut self,
        unit: &mut SndDg00x,
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        self.common_ctl.load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.common_ctl.1.append(&mut elem_id_list))?;
        self.monitor.load(&unit, &self.req, card_cntr)?;
        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut SndDg00x,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(unit, &mut self.req, elem_id, elem_value, TIMEOUT_MS)? {
            Ok(true)
        } else if self.monitor.read(unit, &self.req, elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDg00x,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.write(unit, &mut self.req, elem_id, new, TIMEOUT_MS)? {
            Ok(true)
        } else if self.monitor.write(unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

fn clock_rate_to_str(rate: &ClockRate) -> &'static str {
    match rate {
        ClockRate::R44100 => "44100",
        ClockRate::R48000 => "48000",
        ClockRate::R88200 => "88200",
        ClockRate::R96000 => "96000",
    }
}

fn clock_source_to_str(src: &ClockSource) -> &'static str {
    match src {
        ClockSource::Internal => "Internal",
        ClockSource::Spdif => "S/PDIF",
        ClockSource::Adat => "ADAT",
        ClockSource::WordClock => "WordClock",
    }
}

fn optical_interface_mode_to_str(mode: &OpticalInterfaceMode) -> &'static str {
    match mode {
        OpticalInterfaceMode::Adat => "ADAT",
        OpticalInterfaceMode::Spdif => "S/PDIF",
    }
}

const CLK_LOCAL_RATE_NAME: &str = "local-clock-rate";
const CLK_SRC_NAME: &str = "clock-source";
const OPT_IFACE_NAME: &str = "optical-interface";

trait Dg00xCommonCtl<T: Dg00xCommonOperation>: AsRef<ClockRate> + AsMut<ClockRate> {
    const CLOCK_RATES: [ClockRate; 4] = [
        ClockRate::R44100,
        ClockRate::R48000,
        ClockRate::R88200,
        ClockRate::R96000,
    ];

    const OPTICAL_INTERFACE_MODES: [OpticalInterfaceMode; 2] =
        [OpticalInterfaceMode::Adat, OpticalInterfaceMode::Spdif];

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDg00x,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        let mut notified_elem_id_list = Vec::new();

        let labels: Vec<&str> = T::SAMPLING_CLOCK_SOURCES
            .iter()
            .map(|s| clock_source_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, CLK_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = Self::CLOCK_RATES
            .iter()
            .map(|r| clock_rate_to_str(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, CLK_LOCAL_RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| notified_elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::OPTICAL_INTERFACE_MODES
            .iter()
            .map(|m| optical_interface_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OPT_IFACE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        T::read_media_clock_rate(req, &mut unit.get_node(), timeout_ms)
            .map(|src| *self.as_mut() = src)?;

        Ok(notified_elem_id_list)
    }

    fn read(
        &mut self,
        unit: &mut SndDg00x,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            CLK_SRC_NAME => {
                let src = T::read_sampling_clock_source(req, &mut unit.get_node(), timeout_ms)?;
                let pos = T::SAMPLING_CLOCK_SOURCES
                    .iter()
                    .position(|s| s.eq(&src))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            OPT_IFACE_NAME => {
                let mode = T::read_optical_interface_mode(req, &mut unit.get_node(), timeout_ms)?;
                let pos = Self::OPTICAL_INTERFACE_MODES
                    .iter()
                    .position(|r| r.eq(&mode))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => self.read_notified_elems(elem_id, elem_value),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDg00x,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            CLK_SRC_NAME => {
                if unit.get_property_streaming() {
                    let msg = "Not configurable during packet streaming";
                    Err(Error::new(FileError::Again, &msg))
                } else {
                    let mut vals = [0];
                    elem_value.get_enum(&mut vals);
                    let &src = T::SAMPLING_CLOCK_SOURCES
                        .iter()
                        .nth(vals[0] as usize)
                        .ok_or_else(|| {
                            let msg =
                                format!("Invalid index for sampling clock sources: {}", vals[0]);
                            Error::new(FileError::Inval, &msg)
                        })?;
                    T::write_sampling_clock_source(req, &mut unit.get_node(), src, timeout_ms)
                        .map(|_| true)
                }
            }
            CLK_LOCAL_RATE_NAME => {
                if unit.get_property_streaming() {
                    let msg = "Not configurable during packet streaming";
                    Err(Error::new(FileError::Again, &msg))
                } else {
                    let mut vals = [0];
                    elem_value.get_enum(&mut vals);
                    let &rate = Self::CLOCK_RATES
                        .iter()
                        .nth(vals[0] as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index for media clock rates: {}",
                                              vals[0]);
                            Error::new(FileError::Inval, &msg)
                        })?;
                    T::write_media_clock_rate(req, &mut unit.get_node(), rate, timeout_ms).map(|_| {
                        *self.as_mut() = rate;
                        true
                    })
                }
            }
            OPT_IFACE_NAME => {
                if unit.get_property_streaming() {
                    let msg = "Not configurable during packet streaming";
                    Err(Error::new(FileError::Again, &msg))
                } else {
                    let mut vals = [0];
                    elem_value.get_enum(&mut vals);
                    let &mode = Self::OPTICAL_INTERFACE_MODES
                        .iter()
                        .nth(vals[0] as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index for optical interface mode: {}", vals[0]);
                            Error::new(FileError::Inval, &msg)
                        })?;
                    T::write_optical_interface_mode(req, &mut unit.get_node(), mode, timeout_ms)
                        .map(|_| true)
                }
            }
            _ => Ok(false),
        }
    }

    fn handle_lock_notification(
        &mut self,
        locked: bool,
        unit: &mut SndDg00x,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if locked {
            T::read_media_clock_rate(req, &mut unit.get_node(), timeout_ms)
                .map(|src| *self.as_mut() = src)
        } else {
            Ok(())
        }
    }

    fn read_notified_elems(
        &self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            CLK_LOCAL_RATE_NAME => {
                let pos = Self::CLOCK_RATES
                    .iter()
                    .position(|r| r.eq(&self.as_ref()))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
