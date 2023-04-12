// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, alsa_ctl_tlv_codec::DbInterval, protocols::*, std::marker::PhantomData};

const TIMEOUT_MS: u32 = 100;

#[derive(Default, Debug)]
pub struct Digi002Model {
    req: FwReq,
    common_ctl: CommonCtl<Digi002Protocol>,
    meter_ctl: MeterCtl<Digi002Protocol>,
    monitor_ctl: MonitorCtl<Digi002Protocol>,
}

impl CtlModel<(SndDigi00x, FwNode)> for Digi002Model {
    fn cache(&mut self, (unit, node): &mut (SndDigi00x, FwNode)) -> Result<(), Error> {
        self.common_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.meter_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.monitor_ctl
            .cache(unit, &mut self.req, node, TIMEOUT_MS)?;
        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr)?;
        self.meter_ctl.load(card_cntr)?;
        self.monitor_ctl.load(card_cntr)?;
        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndDigi00x, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
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
        (unit, node): &mut (SndDigi00x, FwNode),
        elem_id: &ElemId,
        _: &ElemValue,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .common_ctl
            .write(unit, &mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.monitor_ctl.write(
            unit,
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

impl MeasureModel<(SndDigi00x, FwNode)> for Digi002Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.elem_id_list);
    }

    fn measure_states(&mut self, (_, node): &mut (SndDigi00x, FwNode)) -> Result<(), Error> {
        self.meter_ctl.cache(&mut self.req, node, TIMEOUT_MS)
    }
}

impl NotifyModel<(SndDigi00x, FwNode), bool> for Digi002Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.monitor_ctl.elem_id_list);
    }

    fn parse_notification(
        &mut self,
        (unit, node): &mut (SndDigi00x, FwNode),
        &locked: &bool,
    ) -> Result<(), Error> {
        if locked {
            self.common_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        }
        self.monitor_ctl
            .cache(unit, &mut self.req, node, TIMEOUT_MS)?;
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct Digi003Model {
    req: FwReq,
    common_ctl: CommonCtl<Digi003Protocol>,
    meter_ctl: MeterCtl<Digi003Protocol>,
    monitor_ctl: MonitorCtl<Digi003Protocol>,
    opt_iface_ctl: OpticalIfaceCtl,
}

impl CtlModel<(SndDigi00x, FwNode)> for Digi003Model {
    fn cache(&mut self, (unit, node): &mut (SndDigi00x, FwNode)) -> Result<(), Error> {
        self.common_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.meter_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        self.monitor_ctl
            .cache(unit, &mut self.req, node, TIMEOUT_MS)?;
        self.opt_iface_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr)?;
        self.meter_ctl.load(card_cntr)?;
        self.monitor_ctl.load(card_cntr)?;
        self.opt_iface_ctl.load(card_cntr)?;
        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndDigi00x, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.monitor_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.opt_iface_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        (unit, node): &mut (SndDigi00x, FwNode),
        elem_id: &ElemId,
        _: &ElemValue,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .common_ctl
            .write(unit, &mut self.req, node, elem_id, elem_value, TIMEOUT_MS)?
        {
            Ok(true)
        } else if self.monitor_ctl.write(
            unit,
            &mut self.req,
            node,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.opt_iface_ctl.write(
            unit,
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

impl MeasureModel<(SndDigi00x, FwNode)> for Digi003Model {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.elem_id_list);
    }

    fn measure_states(&mut self, (_, node): &mut (SndDigi00x, FwNode)) -> Result<(), Error> {
        self.meter_ctl.cache(&mut self.req, node, TIMEOUT_MS)
    }
}

impl NotifyModel<(SndDigi00x, FwNode), bool> for Digi003Model {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.elem_id_list);
        elem_id_list.extend_from_slice(&self.monitor_ctl.elem_id_list);
    }

    fn parse_notification(
        &mut self,
        (unit, node): &mut (SndDigi00x, FwNode),
        &locked: &bool,
    ) -> Result<(), Error> {
        if locked {
            self.common_ctl.cache(&mut self.req, node, TIMEOUT_MS)?;
        }
        self.monitor_ctl
            .cache(unit, &mut self.req, node, TIMEOUT_MS)?;
        Ok(())
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

const CLK_LOCAL_RATE_NAME: &str = "local-clock-rate";
const CLK_SRC_NAME: &str = "clock-source";

#[derive(Default, Debug)]
struct CommonCtl<T>
where
    T: Dg00xHardwareSpecification
        + Dg00xWhollyCachableParamsOperation<Dg00xSamplingClockParameters>
        + Dg00xWhollyUpdatableParamsOperation<Dg00xSamplingClockParameters>
        + Dg00xWhollyCachableParamsOperation<Dg00xMediaClockParameters>
        + Dg00xWhollyUpdatableParamsOperation<Dg00xMediaClockParameters>,
{
    elem_id_list: Vec<ElemId>,
    sampling_clock_source: Dg00xSamplingClockParameters,
    media_clock_rate: Dg00xMediaClockParameters,
    _phantom: PhantomData<T>,
}

impl<T> CommonCtl<T>
where
    T: Dg00xHardwareSpecification
        + Dg00xWhollyCachableParamsOperation<Dg00xSamplingClockParameters>
        + Dg00xWhollyUpdatableParamsOperation<Dg00xSamplingClockParameters>
        + Dg00xWhollyCachableParamsOperation<Dg00xMediaClockParameters>
        + Dg00xWhollyUpdatableParamsOperation<Dg00xMediaClockParameters>,
{
    const CLOCK_RATES: [ClockRate; 4] = [
        ClockRate::R44100,
        ClockRate::R48000,
        ClockRate::R88200,
        ClockRate::R96000,
    ];

    fn cache(&mut self, req: &mut FwReq, node: &mut FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache_wholly(req, node, &mut self.sampling_clock_source, timeout_ms);
        debug!(params = ?self.sampling_clock_source, ?res);
        res?;

        let res = T::cache_wholly(req, node, &mut self.media_clock_rate, timeout_ms);
        debug!(params = ?self.sampling_clock_source, ?res);
        res?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = T::SAMPLING_CLOCK_SOURCES
            .iter()
            .map(|s| clock_source_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, CLK_SRC_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::CLOCK_RATES
            .iter()
            .map(|r| clock_rate_to_str(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, CLK_LOCAL_RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_SRC_NAME => {
                let pos = T::SAMPLING_CLOCK_SOURCES
                    .iter()
                    .position(|s| self.sampling_clock_source.source.eq(s))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            CLK_LOCAL_RATE_NAME => {
                let pos = Self::CLOCK_RATES
                    .iter()
                    .position(|r| self.media_clock_rate.rate.eq(r))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDigi00x,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_SRC_NAME => {
                if unit.is_locked() {
                    let msg = "Not configurable during packet streaming";
                    Err(Error::new(FileError::Again, &msg))?;
                }

                let mut params = self.sampling_clock_source.clone();
                let pos = elem_value.enumerated()[0] as usize;
                T::SAMPLING_CLOCK_SOURCES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index for sampling clock sources: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&s| params.source = s)?;
                let res = T::update_wholly(req, node, &params, timeout_ms)
                    .map(|_| self.sampling_clock_source = params);
                debug!(params = ?self.sampling_clock_source, ?res);
                res.map(|_| true)
            }
            CLK_LOCAL_RATE_NAME => {
                if unit.is_locked() {
                    let msg = "Not configurable during packet streaming";
                    Err(Error::new(FileError::Again, &msg))?;
                }

                let mut params = self.media_clock_rate.clone();
                let pos = elem_value.enumerated()[0] as usize;
                Self::CLOCK_RATES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index for media clock rates: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&r| params.rate = r)?;
                let res = T::update_wholly(req, node, &params, timeout_ms)
                    .map(|_| self.media_clock_rate = params);
                debug!(params = ?self.media_clock_rate, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
struct OpticalIfaceCtl {
    elem_id_list: Vec<ElemId>,
    opt_iface_mode: OpticalInterfaceMode,
}

fn optical_interface_mode_to_str(mode: &OpticalInterfaceMode) -> &'static str {
    match mode {
        OpticalInterfaceMode::Adat => "ADAT",
        OpticalInterfaceMode::Spdif => "S/PDIF",
    }
}

const OPT_IFACE_NAME: &str = "optical-interface";

impl OpticalIfaceCtl {
    const OPTICAL_INTERFACE_MODES: &[OpticalInterfaceMode] =
        &[OpticalInterfaceMode::Adat, OpticalInterfaceMode::Spdif];

    fn cache(&mut self, req: &mut FwReq, node: &mut FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = Digi003Protocol::cache_wholly(req, node, &mut self.opt_iface_mode, timeout_ms);
        debug!(params = ?self.opt_iface_mode, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::OPTICAL_INTERFACE_MODES
            .iter()
            .map(|m| optical_interface_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OPT_IFACE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OPT_IFACE_NAME => {
                let pos = Self::OPTICAL_INTERFACE_MODES
                    .iter()
                    .position(|m| self.opt_iface_mode.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDigi00x,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OPT_IFACE_NAME => {
                if unit.is_locked() {
                    let msg = "Not configurable during packet streaming";
                    Err(Error::new(FileError::Again, &msg))?;
                }

                let pos = elem_value.enumerated()[0] as usize;
                let params = Self::OPTICAL_INTERFACE_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index for optical interface mode: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = Digi003Protocol::update_wholly(req, node, &params, timeout_ms)
                    .map(|_| self.opt_iface_mode = params);
                debug!(params = ?self.opt_iface_mode, ?res);
                res.map(|_| true)
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

#[derive(Default, Debug)]
pub struct MeterCtl<T>
where
    T: Dg00xHardwareSpecification
        + Dg00xWhollyCachableParamsOperation<Dg00xExternalClockParameters>,
{
    elem_id_list: Vec<ElemId>,
    external_clock_rate: Dg00xExternalClockParameters,
    _phantom: PhantomData<T>,
}

impl<T> MeterCtl<T>
where
    T: Dg00xHardwareSpecification
        + Dg00xWhollyCachableParamsOperation<Dg00xExternalClockParameters>,
{
    const OPTIONAL_CLOCK_RATES: &[Option<ClockRate>] = &[
        None,
        Some(ClockRate::R44100),
        Some(ClockRate::R48000),
        Some(ClockRate::R88200),
        Some(ClockRate::R96000),
    ];

    fn cache(&mut self, req: &mut FwReq, node: &mut FwNode, timeout_ms: u32) -> Result<(), Error> {
        let res = T::cache_wholly(req, node, &mut self.external_clock_rate, timeout_ms);
        debug!(params = ?self.external_clock_rate, ?res);
        res
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::OPTIONAL_CLOCK_RATES
            .iter()
            .map(|r| optional_clock_rate_to_str(r))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, CLK_EXT_RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_EXT_RATE_NAME => {
                let pos = Self::OPTIONAL_CLOCK_RATES
                    .iter()
                    .position(|r| self.external_clock_rate.rate.eq(r))
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

#[derive(Default, Debug)]
pub struct MonitorCtl<T>
where
    T: Dg00xHardwareSpecification
        + Dg00xWhollyCachableParamsOperation<Dg00xMonitorState>
        + Dg00xPartiallyUpdatableParamsOperation<Dg00xMonitorState>,
{
    elem_id_list: Vec<ElemId>,
    states: Dg00xMonitorState,
    _phantom: PhantomData<T>,
}

impl<T> MonitorCtl<T>
where
    T: Dg00xHardwareSpecification
        + Dg00xWhollyCachableParamsOperation<Dg00xMonitorState>
        + Dg00xPartiallyUpdatableParamsOperation<Dg00xMonitorState>,
{
    const DST_LABELS: &'static [&'static str] = &["monitor-output-1", "monitor-output-2"];
    const SRC_LABELS: &'static [&'static str] = &[
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

    fn cache(
        &mut self,
        unit: &mut SndDigi00x,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if unit.is_locked() {
            let res = if self.states.enabled {
                T::update_wholly(req, node, &self.states, timeout_ms)
            } else {
                T::cache_wholly(req, node, &mut self.states, timeout_ms)
            };
            debug!(params = ?self.states, ?res);
            res?;
        }

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_ENABLE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_SRC_GAIN_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                Self::DST_LABELS.len(),
                T::MONITOR_SOURCE_GAIN_MIN as i32,
                T::MONITOR_SOURCE_GAIN_MAX as i32,
                T::MONITOR_SOURCE_GAIN_STEP as i32,
                Self::SRC_LABELS.len(),
                Some(&Into::<Vec<u32>>::into(Self::GAIN_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MONITOR_ENABLE_NAME => {
                elem_value.set_bool(&[self.states.enabled]);
                Ok(true)
            }
            MONITOR_SRC_GAIN_NAME => {
                let dst = elem_id.index() as usize;
                let gains = self.states.src_gains.iter().nth(dst).ok_or_else(|| {
                    let msg = format!("Invalid value for monitor destination: {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                let vals: Vec<i32> = gains.iter().map(|&gain| gain as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &mut SndDigi00x,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MONITOR_ENABLE_NAME => {
                if !unit.is_locked() {
                    let msg = "Monitor function is configurable just during packet streaming.";
                    Err(Error::new(FileError::Again, &msg))?;
                }

                let mut params = self.states.clone();
                params.enabled = elem_value.boolean()[0];
                let res = if params.enabled {
                    // Restore previous settings when enabling again.
                    T::update_wholly(req, node, &params, timeout_ms).map(|_| self.states = params)
                } else {
                    T::update_partially(req, node, &mut self.states, params, timeout_ms)
                };
                debug!(params = ?self.states, ?res);
                res.map(|_| true)
            }
            MONITOR_SRC_GAIN_NAME => {
                if !self.states.enabled {
                    let msg = "Monitor is disabled.";
                    Err(Error::new(FileError::Again, &msg))?;
                }

                let dst = elem_id.index() as usize;
                let mut params = self.states.clone();
                let gains = params.src_gains.iter_mut().nth(dst).ok_or_else(|| {
                    let msg = format!("Invalid value for monitor destination: {}", dst);
                    Error::new(FileError::Inval, &msg)
                })?;
                gains
                    .iter_mut()
                    .zip(elem_value.int())
                    .for_each(|(o, &val)| *o = val as u8);

                let res = T::update_partially(req, node, &mut self.states, params, timeout_ms);
                debug!(params = ?self.states, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
