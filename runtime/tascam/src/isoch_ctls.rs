// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {
    super::*,
    alsa_ctl_tlv_codec::DbInterval,
    alsactl::{prelude::*, *},
    core::elem_value_accessor::*,
    protocols::isoch::*,
};

const MONITOR_ROTARY_NAME: &str = "monitor-rotary";
const SOLO_ROTARY_NAME: &str = "solo-rotary";
const INPUT_METER_NAME: &str = "input-meters";
const OUTPUT_METER_NAME: &str = "output-meters";
const DETECTED_CLK_SRC_NAME: &str = "detected-clock-source";
const DETECTED_CLK_RATE_NAME: &str = "detected-clock-rate";
const MONITOR_METER_NAME: &str = "monitor-meters";
const ANALOG_MIXER_METER_NAME: &str = "analog-mixer-meters";
const MONITOR_MODE_NAME: &str = "monitor-mode";

fn clk_src_to_str(src: &Option<ClkSrc>) -> &'static str {
    match src {
        Some(ClkSrc::Internal) => "Internal",
        Some(ClkSrc::Wordclock) => "Word-clock",
        Some(ClkSrc::Spdif) => "S/PDIF",
        Some(ClkSrc::Adat) => "ADAT",
        None => "N/A",
    }
}

fn clk_rate_to_str(rate: &Option<ClkRate>) -> &'static str {
    match rate {
        Some(ClkRate::R44100) => "44100",
        Some(ClkRate::R48000) => "48000",
        Some(ClkRate::R88200) => "88200",
        Some(ClkRate::R96000) => "i96000",
        None => "N/A",
    }
}

fn monitor_mode_to_str(mode: &MonitorMode) -> &'static str {
    match mode {
        MonitorMode::Computer => "computer",
        MonitorMode::Inputs => "input",
        MonitorMode::Both => "both",
    }
}

pub trait IsochMeterCtlOperation<T: IsochMeterOperation> {
    const INPUT_LABELS: &'static [&'static str];
    const OUTPUT_LABELS: &'static [&'static str];

    fn meter(&self) -> &IsochMeterState;
    fn meter_mut(&mut self) -> &mut IsochMeterState;

    const CLK_SRCS: [Option<ClkSrc>; 5] = [
        Some(ClkSrc::Internal),
        Some(ClkSrc::Wordclock),
        Some(ClkSrc::Spdif),
        Some(ClkSrc::Adat),
        None,
    ];

    const CLK_RATES: [Option<ClkRate>; 5] = [
        Some(ClkRate::R44100),
        Some(ClkRate::R48000),
        Some(ClkRate::R88200),
        Some(ClkRate::R96000),
        None,
    ];

    const MONITOR_MODES: [MonitorMode; 3] = [
        MonitorMode::Computer,
        MonitorMode::Inputs,
        MonitorMode::Both,
    ];

    fn parse_state(&mut self, image: &[u32]) -> Result<(), Error> {
        T::parse_meter_state(self.meter_mut(), image)
    }

    fn load_state(
        &mut self,
        card_cntr: &mut CardCntr,
        image: &[u32],
    ) -> Result<Vec<ElemId>, Error> {
        assert_eq!(Self::INPUT_LABELS.len(), T::INPUT_COUNT);
        assert_eq!(Self::OUTPUT_LABELS.len(), T::OUTPUT_COUNT);

        let mut measured_elem_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_ROTARY_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::ROTARY_MIN as i32,
                T::ROTARY_MAX as i32,
                T::ROTARY_STEP as i32,
                1,
                None,
                false,
            )
            .map(|mut elem_id_list| measured_elem_list.append(&mut elem_id_list))?;

        if T::HAS_SOLO {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, SOLO_ROTARY_NAME, 0);
            card_cntr
                .add_int_elems(
                    &elem_id,
                    1,
                    T::ROTARY_MIN as i32,
                    T::ROTARY_MAX as i32,
                    T::ROTARY_STEP as i32,
                    1,
                    None,
                    false,
                )
                .map(|mut elem_id_list| measured_elem_list.append(&mut elem_id_list))?;
        }

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                T::INPUT_COUNT,
                None,
                false,
            )
            .map(|mut elem_id_list| measured_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OUTPUT_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                T::OUTPUT_COUNT,
                None,
                false,
            )
            .map(|mut elem_id_list| measured_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                2,
                None,
                false,
            )
            .map(|mut elem_id_list| measured_elem_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, ANALOG_MIXER_METER_NAME, 0);
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                T::LEVEL_MIN as i32,
                T::LEVEL_MAX as i32,
                T::LEVEL_STEP as i32,
                2,
                None,
                false,
            )
            .map(|mut elem_id_list| measured_elem_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::CLK_SRCS.iter().map(|s| clk_src_to_str(s)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, DETECTED_CLK_SRC_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| measured_elem_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::CLK_RATES.iter().map(|s| clk_rate_to_str(s)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, DETECTED_CLK_RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| measured_elem_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = Self::MONITOR_MODES
            .iter()
            .map(|s| monitor_mode_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MONITOR_MODE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, false)
            .map(|mut elem_id_list| measured_elem_list.append(&mut elem_id_list))?;

        *self.meter_mut() = T::create_meter_state();
        self.parse_state(image)?;

        Ok(measured_elem_list)
    }

    fn read_state(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MONITOR_ROTARY_NAME => {
                elem_value.set_int(&[self.meter().monitor as i32]);
                Ok(true)
            }
            SOLO_ROTARY_NAME => {
                elem_value.set_int(&[self.meter().solo.unwrap() as i32]);
                Ok(true)
            }
            INPUT_METER_NAME => {
                let vals: Vec<i32> = self.meter().inputs.iter().map(|&l| l as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            OUTPUT_METER_NAME => {
                let vals: Vec<i32> = self.meter().outputs.iter().map(|&l| l as i32).collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            DETECTED_CLK_SRC_NAME => {
                let pos = Self::CLK_SRCS
                    .iter()
                    .position(|s| s.eq(&self.meter().src))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            DETECTED_CLK_RATE_NAME => {
                let pos = Self::CLK_RATES
                    .iter()
                    .position(|r| r.eq(&self.meter().rate))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            MONITOR_METER_NAME => {
                let vals: Vec<i32> = self
                    .meter()
                    .monitor_meters
                    .iter()
                    .map(|&l| l as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            ANALOG_MIXER_METER_NAME => {
                let vals: Vec<i32> = self
                    .meter()
                    .analog_mixer_meters
                    .iter()
                    .map(|&l| l as i32)
                    .collect();
                elem_value.set_int(&vals);
                Ok(true)
            }
            MONITOR_MODE_NAME => {
                let pos = Self::MONITOR_MODES
                    .iter()
                    .position(|m| self.meter().monitor_mode.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct ClockCtl<T>
where
    T: IsochCommonOperation,
{
    elem_id_list: Vec<ElemId>,
    params: TascamClockParameters,
    _phantom: PhantomData<T>,
}

const CLK_SRC_NAME: &str = "clock-source";
const CLK_RATE_NAME: &str = "clock-rate";

const CLOCK_RATES: &[ClkRate] = &[
    ClkRate::R44100,
    ClkRate::R48000,
    ClkRate::R88200,
    ClkRate::R96000,
];

impl<T> ClockCtl<T>
where
    T: IsochCommonOperation,
{
    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::get_sampling_clock_source(req, node, timeout_ms)
            .map(|src| self.params.sampling_clock_source = src)?;
        T::get_media_clock_rate(req, node, timeout_ms)
            .map(|rate| self.params.media_clock_rate = rate)?;
        Ok(())
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = T::SAMPLING_CLOCK_SOURCES
            .iter()
            .map(|&s| clk_src_to_str(&Some(s)))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, CLK_SRC_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = CLOCK_RATES
            .iter()
            .map(|&r| clk_rate_to_str(&Some(r)))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, CLK_RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_SRC_NAME => {
                let pos = T::SAMPLING_CLOCK_SOURCES
                    .iter()
                    .position(|s| self.params.sampling_clock_source.eq(s))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            CLK_RATE_NAME => {
                let pos = CLOCK_RATES
                    .iter()
                    .position(|r| self.params.media_clock_rate.eq(r))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        unit: &mut SndTascam,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_SRC_NAME => {
                let pos = elem_value.enumerated()[0] as usize;
                let mut params = self.params.clone();
                T::SAMPLING_CLOCK_SOURCES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of clock sources: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&src| params.sampling_clock_source = src)?;
                unit.lock()?;
                let res = T::set_sampling_clock_source(
                    req,
                    node,
                    params.sampling_clock_source,
                    timeout_ms,
                )
                .map(|_| self.params = params);
                let _ = unit.unlock();
                res.map(|_| true)
            }
            CLK_RATE_NAME => {
                let pos = elem_value.enumerated()[0] as usize;
                let mut params = self.params.clone();
                CLOCK_RATES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of clock rates: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .map(|&rate| params.media_clock_rate = rate)?;
                unit.lock()?;
                let res = T::set_media_clock_rate(req, node, params.media_clock_rate, timeout_ms)
                    .map(|_| self.params = params);
                let _ = unit.unlock();
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const SIGNAL_DETECTION_THRESHOLD_NAME: &str = "signal-detection-threshold";
const OVER_LEVEL_DETECTION_THRESHOLD_NAME: &str = "over-level-detection-threshold";

#[derive(Default, Debug)]
pub(crate) struct InputDetectionThreshold<T>
where
    T: IsochCommonOperation,
{
    elem_id_list: Vec<ElemId>,
    params: TascamInputDetectionThreshold,
    _phantom: PhantomData<T>,
}

impl<T> InputDetectionThreshold<T>
where
    T: IsochCommonOperation,
{
    const THRESHOLD_MIN: i32 = T::THRESHOLD_MIN as i32;
    const THRESHOLD_MAX: i32 = T::THRESHOLD_MAX as i32;
    const THRESHOLD_STEP: i32 = 1;
    const THRESHOLD_TLV: DbInterval = DbInterval {
        min: -9038,
        max: 0,
        linear: false,
        mute_avail: false,
    };

    pub(crate) fn cache(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        T::get_analog_input_threshold_for_signal_detection(req, node, timeout_ms)
            .map(|threshold| self.params.signal = threshold)?;
        T::get_analog_input_threshold_for_over_level_detection(req, node, timeout_ms)
            .map(|threshold| self.params.over_level = threshold)?;
        Ok(())
    }

    pub(crate) fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            SIGNAL_DETECTION_THRESHOLD_NAME,
            0,
        );
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::THRESHOLD_MIN,
                Self::THRESHOLD_MAX,
                Self::THRESHOLD_STEP,
                1,
                Some(&Into::<Vec<u32>>::into(Self::THRESHOLD_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        let elem_id = ElemId::new_by_name(
            ElemIfaceType::Mixer,
            0,
            0,
            OVER_LEVEL_DETECTION_THRESHOLD_NAME,
            0,
        );
        card_cntr
            .add_int_elems(
                &elem_id,
                1,
                Self::THRESHOLD_MIN,
                Self::THRESHOLD_MAX,
                Self::THRESHOLD_STEP,
                1,
                Some(&Into::<Vec<u32>>::into(Self::THRESHOLD_TLV)),
                true,
            )
            .map(|mut elem_id_list| self.elem_id_list.append(&mut elem_id_list))?;

        Ok(())
    }

    pub(crate) fn read(
        &mut self,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            SIGNAL_DETECTION_THRESHOLD_NAME => {
                elem_value.set_int(&[self.params.signal as i32]);
                Ok(true)
            }
            OVER_LEVEL_DETECTION_THRESHOLD_NAME => {
                elem_value.set_int(&[self.params.over_level as i32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn write(
        &mut self,
        req: &mut FwReq,
        node: &mut FwNode,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            SIGNAL_DETECTION_THRESHOLD_NAME => {
                let mut params = self.params.clone();
                params.signal = elem_value.int()[0] as u16;
                T::set_analog_input_threshold_for_signal_detection(
                    req,
                    node,
                    params.signal,
                    timeout_ms,
                )
                .map(|_| self.params = params)?;
                Ok(true)
            }
            OVER_LEVEL_DETECTION_THRESHOLD_NAME => {
                let mut params = self.params.clone();
                params.over_level = elem_value.int()[0] as u16;
                T::set_analog_input_threshold_for_over_level_detection(
                    req,
                    node,
                    params.over_level,
                    timeout_ms,
                )
                .map(|_| self.params = params)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

const COAX_OUT_SRC_NAME: &str = "coax-output-source";

fn coaxial_output_source_to_str(src: &CoaxialOutputSource) -> &str {
    match src {
        CoaxialOutputSource::StreamInputPair => "stream-input",
        CoaxialOutputSource::AnalogOutputPair0 => "analog-output-1/2",
    }
}

pub trait IsochCommonCtlOperation<T: IsochCommonOperation> {
    const COAXIAL_OUTPUT_SOURCES: [CoaxialOutputSource; 2] = [
        CoaxialOutputSource::StreamInputPair,
        CoaxialOutputSource::AnalogOutputPair0,
    ];

    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::COAXIAL_OUTPUT_SOURCES
            .iter()
            .map(|s| coaxial_output_source_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, COAX_OUT_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read_params(
        &mut self,
        node: &mut FwNode,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            COAX_OUT_SRC_NAME => {
                let src = T::get_coaxial_output_source(req, node, timeout_ms)?;
                let pos = Self::COAXIAL_OUTPUT_SOURCES
                    .iter()
                    .position(|s| s.eq(&src))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(true),
        }
    }

    fn write_params(
        &mut self,
        unit: &mut (SndTascam, FwNode),
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            COAX_OUT_SRC_NAME => {
                let val = elem_value.enumerated()[0];
                let &src = Self::COAXIAL_OUTPUT_SOURCES
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of clock rates: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })?;
                T::set_coaxial_output_source(req, &mut unit.1, src, timeout_ms).map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const OPT_OUT_SRC_NAME: &str = "opt-output-source";
const SPDIF_IN_SRC_NAME: &str = "spdif-input-source";

fn spdif_capture_source_to_str(src: &SpdifCaptureSource) -> &'static str {
    match src {
        SpdifCaptureSource::Coaxial => "coaxial",
        SpdifCaptureSource::Optical => "optical",
    }
}

fn optical_output_source_to_str(src: &OpticalOutputSource) -> &'static str {
    match src {
        OpticalOutputSource::StreamInputPairs => "stream-input",
        OpticalOutputSource::CoaxialOutputPair0 => "coaxial-output-1/2",
        OpticalOutputSource::AnalogInputPair0 => "analog-input-1/2",
        OpticalOutputSource::AnalogOutputPairs => "analog-output-1/2",
    }
}

pub trait IsochOpticalCtlOperation<T: IsochOpticalOperation> {
    const SPDIF_INPUT_SOURCES: [SpdifCaptureSource; 2] =
        [SpdifCaptureSource::Coaxial, SpdifCaptureSource::Optical];

    const OPTICAL_OUTPUT_SOURCES: &'static [OpticalOutputSource];

    fn load_params(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::SPDIF_INPUT_SOURCES
            .iter()
            .map(|s| spdif_capture_source_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, OPT_OUT_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = Self::OPTICAL_OUTPUT_SOURCES
            .iter()
            .map(|s| optical_output_source_to_str(s))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, SPDIF_IN_SRC_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        Ok(())
    }

    fn read_params(
        &mut self,
        node: &mut FwNode,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OPT_OUT_SRC_NAME => {
                let src = T::get_opt_output_source(req, node, timeout_ms)?;
                let pos = Self::OPTICAL_OUTPUT_SOURCES
                    .iter()
                    .position(|s| s.eq(&src))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            SPDIF_IN_SRC_NAME => {
                let src = T::get_spdif_capture_source(req, node, timeout_ms)?;
                let pos = Self::SPDIF_INPUT_SOURCES
                    .iter()
                    .position(|s| s.eq(&src))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        node: &mut FwNode,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            OPT_OUT_SRC_NAME => {
                let val = elem_value.enumerated()[0];
                let &src = Self::OPTICAL_OUTPUT_SOURCES
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index for optical output sources: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })?;
                T::set_opt_output_source(req, node, src, timeout_ms).map(|_| true)
            }
            SPDIF_IN_SRC_NAME => {
                let val = elem_value.enumerated()[0];
                let &src = Self::SPDIF_INPUT_SOURCES
                    .iter()
                    .nth(val as usize)
                    .ok_or_else(|| {
                        let msg = format!("Invalid index for spdif input sources: {}", val);
                        Error::new(FileError::Inval, &msg)
                    })?;
                T::set_spdif_capture_source(req, node, src, timeout_ms).map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const MASTER_FADER_ASSIGN_NAME: &str = "master-fader-assign";
const HOST_MODE_NAME: &str = "host-mode";

pub trait IsochConsoleCtlOperation<T: IsochConsoleOperation> {
    fn state(&self) -> &IsochConsoleState;
    fn state_mut(&mut self) -> &mut IsochConsoleState;

    fn load_params(
        &mut self,
        card_cntr: &mut CardCntr,
        image: &[u32],
    ) -> Result<Vec<ElemId>, Error> {
        let mut measured_elem_list = Vec::new();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MASTER_FADER_ASSIGN_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, HOST_MODE_NAME, 0);
        card_cntr
            .add_bool_elems(&elem_id, 1, 1, false)
            .map(|mut elem_id_list| measured_elem_list.append(&mut elem_id_list))?;

        self.parse_states(image)?;

        Ok(measured_elem_list)
    }

    fn parse_states(&mut self, image: &[u32]) -> Result<(), Error> {
        T::parse_console_state(self.state_mut(), image)
    }

    fn read_states(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            HOST_MODE_NAME => {
                elem_value.set_bool(&[self.state().host_mode]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn read_params(
        &mut self,
        node: &mut FwNode,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MASTER_FADER_ASSIGN_NAME => {
                let enabled = T::get_master_fader_assign(req, node, timeout_ms)?;
                elem_value.set_bool(&[enabled]);
                Ok(true)
            }
            _ => self.read_states(elem_id, elem_value),
        }
    }

    fn write_params(
        &mut self,
        node: &mut FwNode,
        req: &mut FwReq,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            MASTER_FADER_ASSIGN_NAME => {
                let val = elem_value.boolean()[0];
                T::set_master_fader_assign(req, node, val, timeout_ms).map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

const INPUT_GAIN_NAME: &str = "input-gain";
const INPUT_BALANCE_NAME: &str = "input-balance";
const INPUT_MUTE_NAME: &str = "input-mute";

pub trait IsochRackCtlOperation<T: IsochRackOperation> {
    fn state(&self) -> &IsochRackState;
    fn state_mut(&mut self) -> &mut IsochRackState;

    const INPUT_LABELS: [&'static str; 18] = [
        "Analog-1", "Analog-2", "Analog-3", "Analog-4", "Analog-5", "Analog-6", "Analog-7",
        "Analog-8", "ADAT-1", "ADAT-2", "ADAT-3", "ADAT-4", "ADAT-5", "ADAT-6", "ADAT-7", "ADAT-8",
        "S/PDIF-1", "S/PDIF-2",
    ];

    fn load_params(
        &mut self,
        card_cntr: &mut CardCntr,
        node: &mut FwNode,
        req: &mut FwReq,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(Self::INPUT_LABELS.len(), T::CHANNEL_COUNT);

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_GAIN_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::GAIN_MIN as i32,
            T::GAIN_MAX as i32,
            T::GAIN_STEP as i32,
            Self::INPUT_LABELS.len(),
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_BALANCE_NAME, 0);
        let _ = card_cntr.add_int_elems(
            &elem_id,
            1,
            T::BALANCE_MIN as i32,
            T::BALANCE_MAX as i32,
            T::BALANCE_STEP as i32,
            Self::INPUT_LABELS.len(),
            None,
            true,
        )?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, INPUT_MUTE_NAME, 0);
        let _ = card_cntr.add_bool_elems(&elem_id, 1, T::CHANNEL_COUNT, true)?;

        T::init_input_state(req, node, self.state_mut(), timeout_ms)
    }

    fn read_params(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_GAIN_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, Self::INPUT_LABELS.len(), |idx| {
                    Ok(T::get_input_gain(self.state(), idx) as i32)
                })?;
                Ok(true)
            }
            INPUT_BALANCE_NAME => {
                ElemValueAccessor::<i32>::set_vals(elem_value, Self::INPUT_LABELS.len(), |idx| {
                    Ok(T::get_input_balance(self.state(), idx) as i32)
                })?;
                Ok(true)
            }
            INPUT_MUTE_NAME => {
                ElemValueAccessor::<bool>::set_vals(elem_value, Self::INPUT_LABELS.len(), |idx| {
                    Ok(T::get_input_mute(self.state(), idx))
                })?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write_params(
        &mut self,
        node: &mut FwNode,
        req: &mut FwReq,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            INPUT_GAIN_NAME => ElemValueAccessor::<i32>::get_vals(
                new,
                old,
                Self::INPUT_LABELS.len(),
                |idx, val| {
                    T::set_input_gain(req, node, idx, val as i16, self.state_mut(), timeout_ms)
                },
            )
            .map(|_| true),
            INPUT_BALANCE_NAME => ElemValueAccessor::<i32>::get_vals(
                new,
                old,
                Self::INPUT_LABELS.len(),
                |idx, val| {
                    T::set_input_balance(req, node, idx, val as u8, self.state_mut(), timeout_ms)
                },
            )
            .map(|_| true),
            INPUT_MUTE_NAME => ElemValueAccessor::<bool>::get_vals(
                new,
                old,
                Self::INPUT_LABELS.len(),
                |idx, val| T::set_input_mute(req, node, idx, val, self.state_mut(), timeout_ms),
            )
            .map(|_| true),
            _ => Ok(false),
        }
    }
}
