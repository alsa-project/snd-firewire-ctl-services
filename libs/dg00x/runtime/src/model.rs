// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use core::card_cntr::*;

use hinawa::FwReq;
use hinawa::{SndDg00x, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use core::elem_value_accessor::*;

use alsa_ctl_tlv_codec::items::DbInterval;

use dg00x_protocols::*;

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct Dg00xCommonCtl(ClockRate, Vec<ElemId>);

#[derive(Default)]
pub struct Dg00xMeterCtl(Option<ClockRate>, Vec<ElemId>);

#[derive(Default)]
pub struct Digi002Model {
    req: FwReq,
    common_ctl: Digi002CommonCtl,
    meter_ctl: Digi002MeterCtl,
    monitor_ctl: Digi002MonitorCtl,
}

#[derive(Default)]
pub struct Digi002CommonCtl(Dg00xCommonCtl);

impl Dg00xCommonCtlOperation<Digi002Protocol> for Digi002CommonCtl {
    fn state(&self) -> &Dg00xCommonCtl {
        &self.0
    }

    fn state_mut(&mut self) -> &mut Dg00xCommonCtl {
        &mut self.0
    }
}

#[derive(Default)]
pub struct Digi002MeterCtl(Dg00xMeterCtl);

impl Dg00xMeterCtlOperation<Digi002Protocol> for Digi002MeterCtl {
    fn meter(&self) -> &Dg00xMeterCtl {
        &self.0
    }

    fn meter_mut(&mut self) -> &mut Dg00xMeterCtl {
        &mut self.0
    }
}

#[derive(Default)]
struct Digi002MonitorCtl(Dg00xMonitorState, Vec<ElemId>);

impl AsRef<Dg00xMonitorState> for Digi002MonitorCtl {
    fn as_ref(&self) -> &Dg00xMonitorState {
        &self.0
    }
}

impl AsMut<Dg00xMonitorState> for Digi002MonitorCtl {
    fn as_mut(&mut self) -> &mut Dg00xMonitorState {
        &mut self.0
    }
}

impl Dg00xMonitorCtl<Digi002Protocol> for Digi002MonitorCtl {}

impl NotifyModel<SndDg00x, bool> for Digi002Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.state().1);
        elem_id_list.extend_from_slice(&self.monitor_ctl.1);
    }

    fn parse_notification(&mut self, unit: &mut SndDg00x, &locked: &bool) -> Result<(), Error> {
        self.common_ctl.handle_lock_notification(locked, unit, &mut self.req, TIMEOUT_MS)?;
        self.monitor_ctl.handle_streaming_event(locked, unit, &mut self.req, TIMEOUT_MS)?;
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndDg00x,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read_notified_elems(elem_id, elem_value)? {
            Ok(true)
        } else if self.monitor_ctl.read(elem_id, elem_value)? {
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
        self.common_ctl.load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.meter_ctl.load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.monitor_ctl.load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.monitor_ctl.1.append(&mut elem_id_list))?;
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
        } else if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.monitor_ctl.read(elem_id, elem_value)? {
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
        } else if self.monitor_ctl.write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<SndDg00x> for Digi002Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.meter().1);
    }

    fn measure_states(&mut self, unit: &mut SndDg00x) -> Result<(), Error> {
        self.meter_ctl.measure_states(unit, &mut self.req, TIMEOUT_MS)
    }

    fn measure_elem(
        &mut self,
        _: &SndDg00x,
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

#[derive(Default)]
pub struct Digi003Model {
    req: FwReq,
    common_ctl: Digi003CommonCtl,
    meter_ctl: Digi003MeterCtl,
    monitor_ctl: Digi003MonitorCtl,
}

#[derive(Default)]
pub struct Digi003CommonCtl(Dg00xCommonCtl);

impl Dg00xCommonCtlOperation<Digi003Protocol> for Digi003CommonCtl {
    fn state(&self) -> &Dg00xCommonCtl {
        &self.0
    }

    fn state_mut(&mut self) -> &mut Dg00xCommonCtl {
        &mut self.0
    }
}

#[derive(Default)]
pub struct Digi003MeterCtl(Dg00xMeterCtl);

impl Dg00xMeterCtlOperation<Digi003Protocol> for Digi003MeterCtl {
    fn meter(&self) -> &Dg00xMeterCtl {
        &self.0
    }

    fn meter_mut(&mut self) -> &mut Dg00xMeterCtl {
        &mut self.0
    }
}

#[derive(Default)]
struct Digi003MonitorCtl(Dg00xMonitorState, Vec<ElemId>);

impl AsRef<Dg00xMonitorState> for Digi003MonitorCtl {
    fn as_ref(&self) -> &Dg00xMonitorState {
        &self.0
    }
}

impl AsMut<Dg00xMonitorState> for Digi003MonitorCtl {
    fn as_mut(&mut self) -> &mut Dg00xMonitorState {
        &mut self.0
    }
}

impl Dg00xMonitorCtl<Digi003Protocol> for Digi003MonitorCtl {}

impl NotifyModel<SndDg00x, bool> for Digi003Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.state().1);
        elem_id_list.extend_from_slice(&self.monitor_ctl.1);
    }

    fn parse_notification(&mut self, unit: &mut SndDg00x, &locked: &bool) -> Result<(), Error> {
        self.common_ctl.handle_lock_notification(locked, unit, &mut self.req, TIMEOUT_MS)?;
        self.monitor_ctl.handle_streaming_event(locked, unit, &mut self.req, TIMEOUT_MS)?;
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndDg00x,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read_notified_elems(elem_id, elem_value)? {
            Ok(true)
        } else if self.monitor_ctl.read(elem_id, elem_value)? {
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
        self.common_ctl.load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.meter_ctl.load(card_cntr, unit, &mut self.req, TIMEOUT_MS)?;
        self.monitor_ctl.load(card_cntr, unit, &mut self.req, TIMEOUT_MS)
            .map(|mut elem_id_list| self.monitor_ctl.1.append(&mut elem_id_list))?;
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
        } else if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.monitor_ctl.read(elem_id, elem_value)? {
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
        } else if self.monitor_ctl.write(unit, &mut self.req, elem_id, old, new, TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<SndDg00x> for Digi003Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.meter().1);
    }

    fn measure_states(&mut self, unit: &mut SndDg00x) -> Result<(), Error> {
        self.meter_ctl.measure_states(unit, &mut self.req, TIMEOUT_MS)
    }

    fn measure_elem(
        &mut self,
        _: &SndDg00x,
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

pub trait Dg00xCommonCtlOperation<T: Dg00xCommonOperation> {
    fn state(&self) -> &Dg00xCommonCtl;
    fn state_mut(&mut self) -> &mut Dg00xCommonCtl;

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
    ) -> Result<(), Error> {
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

        self.state_mut().1 = notified_elem_id_list;

        T::read_media_clock_rate(req, &mut unit.get_node(), timeout_ms)
            .map(|src| self.state_mut().0 = src)?;

        Ok(())
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
                        self.state_mut().0 = rate;
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
                .map(|rate| self.state_mut().0 = rate)
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
                    .position(|r| self.state().0.eq(r))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

fn optional_clock_rate_to_str(rate: &Option<ClockRate>) -> &'static str {
    if let Some(r) = rate {
        clock_rate_to_str(r)
    } else {
        "N/A"
    }
}

const CLK_EXT_RATE_NAME: &str = "external-clock-rate";

pub trait Dg00xMeterCtlOperation<T: Dg00xCommonOperation> {
    fn meter(&self) -> &Dg00xMeterCtl;
    fn meter_mut(&mut self) -> &mut Dg00xMeterCtl;

    const OPTIONAL_CLOCK_RATES: [Option<ClockRate>; 5] = [
        None,
        Some(ClockRate::R44100),
        Some(ClockRate::R48000),
        Some(ClockRate::R88200),
        Some(ClockRate::R96000),
    ];

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDg00x,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut measured_elem_id_list = Vec::new();

        let labels: Vec<&str> = Self::OPTIONAL_CLOCK_RATES
            .iter()
            .map(|r| optional_clock_rate_to_str(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, CLK_EXT_RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        self.meter_mut().1 = measured_elem_id_list;

        self.measure_states(unit, req, timeout_ms)?;

        Ok(())
    }

    fn measure_states(
        &mut self,
        unit: &mut SndDg00x,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::read_external_clock_source_rate(req, &mut unit.get_node(), timeout_ms)
            .map(|rate| self.meter_mut().0 = rate)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            CLK_EXT_RATE_NAME => {
                let pos = Self::OPTIONAL_CLOCK_RATES
                    .iter()
                    .position(|r| self.meter().0.eq(r))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

const MONITOR_ENABLE_NAME: &str = "monitor-enable";
const MONITOR_SRC_GAIN_NAME: &str = "monitor-source-gain";

trait Dg00xMonitorCtl<T: Dg00xMonitorOperation>:
    AsRef<Dg00xMonitorState> + AsMut<Dg00xMonitorState>
{
    const DST_LABELS: [&'static str; 2] = ["monitor-output-1", "monitor-output-2"];
    const SRC_LABELS: [&'static str; 18] = [
        "analog-input-1",
        "analog-input-2",
        "analog-input-3",
        "analog-input-4",
        "analog-input-5",
        "analog-input-6",
        "analog-input-7",
        "analog-input-8",
        "spdif-input-1",
        "spdif-input-2",
        "adat-input-1",
        "adat-input-2",
        "adat-input-3",
        "adat-input-4",
        "adat-input-5",
        "adat-input-6",
        "adat-input-7",
        "adat-input-8",
    ];

    const GAIN_TLV: DbInterval = DbInterval {
        min: -4800,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        unit: &mut SndDg00x,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<Vec<ElemId>, Error> {
        assert_eq!(Self::DST_LABELS.len(), T::MONITOR_DST_COUNT);
        assert_eq!(Self::SRC_LABELS.len(), T::MONITOR_SRC_COUNT);

        let mut measured_elem_id_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_ENABLE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_SRC_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                Self::DST_LABELS.len(),
                T::GAIN_MIN as i32,
                T::GAIN_MAX as i32,
                T::GAIN_STEP as i32,
                Self::SRC_LABELS.len(),
                Some(&Into::<Vec<u32>>::into(Self::GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| measured_elem_id_list.append(&mut elem_id_list))?;

        T::read_monitor_state(req, &mut unit.get_node(), self.as_mut(), timeout_ms)?;

        if !unit.get_property_streaming() {
            self.as_mut().enabled = false;
        }

        Ok(measured_elem_id_list)
    }

    fn handle_streaming_event(
        &mut self,
        locked: bool,
        unit: &mut SndDg00x,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        // Just during packet streaming, any write transaction to register has effect to configure
        // internal multiplexer. Without packet streaming, the transaction has no effect against
        // the multiplexer even if it's successful to change the value of register.
        if !locked {
            self.as_mut().enabled = false;
            Ok(())
        } else {
            // Attempt to update the registers with cached value at the beginning of packet
            // streaming.
            T::write_monitor_state(req, &mut unit.get_node(), self.as_mut(), timeout_ms)
        }
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MONITOR_ENABLE_NAME => {
                elem_value.set_bool(&[self.as_ref().enabled]);
                Ok(true)
            }
            MONITOR_SRC_GAIN_NAME => {
                let dst = elem_id.get_index() as usize;
                ElemValueAccessor::<i32>::set_vals(elem_value, Self::SRC_LABELS.len(), |src| {
                    Ok(self.as_ref().src_gains[dst][src] as i32)
                })
                .map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDg00x,
        req: &mut FwReq,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.get_name().as_str() {
            MONITOR_ENABLE_NAME => {
                if !unit.get_property_streaming() {
                    let msg = "Monitor function is configurable during packet streaming.";
                    Err(Error::new(FileError::Again, &msg))
                } else {
                    let mut vals = [false];
                    new.get_bool(&mut vals);
                    T::write_monitor_enable(req, &mut unit.get_node(), vals[0], timeout_ms).map(
                        |_| {
                            self.as_mut().enabled = vals[0];
                            true
                        },
                    )
                }
            }
            MONITOR_SRC_GAIN_NAME => {
                if !self.as_ref().enabled {
                    let msg = "Monitor is disabled.";
                    Err(Error::new(FileError::Again, &msg))
                } else {
                    let mut node = unit.get_node();
                    let dst = elem_id.get_index() as usize;
                    ElemValueAccessor::<i32>::get_vals(
                        new,
                        old,
                        Self::SRC_LABELS.len(),
                        |src, val| {
                            T::write_monitor_source_gain(
                                req, &mut node, dst, src, val as u8, timeout_ms,
                            )
                        },
                    )
                    .map(|_| true)
                }
            }
            _ => Ok(false),
        }
    }
}
