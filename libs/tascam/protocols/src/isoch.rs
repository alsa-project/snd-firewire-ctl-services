// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocols defined for Tascam for FireWire series with isochronous communication.
//!
//! The module includes protocol implementation defined by Tascam for FireWire series with
//! isochronous communication.

pub mod fw1082;
pub mod fw1804;
pub mod fw1884;

use glib::Error;

/// The enumeration for source of sampling clock.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ClkSrc {
    Internal,
    Wordclock,
    Spdif,
    Adat,
}

impl Default for ClkSrc {
    fn default() -> Self {
        Self::Internal
    }
}

/// The enumeration for frequency of media clock.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ClkRate {
    R44100,
    R48000,
    R88200,
    R96000,
}

impl Default for ClkRate {
    fn default() -> Self {
        Self::R44100
    }
}

/// The enumeration for mode of monitor.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MonitorMode {
    Computer,
    Inputs,
    Both,
}

impl Default for MonitorMode {
    fn default() -> Self {
        Self::Computer
    }
}

/// The structure for state of meter.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct IsochMeterState {
    pub monitor: i16,
    pub solo: Option<i16>,
    pub inputs: Vec<i32>,
    pub outputs: Vec<i32>,
    pub rate: Option<ClkRate>,
    pub src: Option<ClkSrc>,
    pub monitor_meters: [i32; 2],
    pub analog_mixer_meters: [i32; 2],
    pub monitor_mode: MonitorMode,
}

/// The trait for meter operation.
pub trait IsochMeterOperation {
    const INPUT_COUNT: usize;
    const OUTPUT_COUNT: usize;
    const HAS_SOLO: bool;

    const ROTARY_MIN: i16 = 0;
    const ROTARY_MAX: i16 = 1023;
    const ROTARY_STEP: i16 = 2;

    const LEVEL_MIN: i32 = 0;
    const LEVEL_MAX: i32 = 0x7fffff00;
    const LEVEL_STEP: i32 = 0x100;

    fn create_meter_state() -> IsochMeterState {
        IsochMeterState {
            monitor: Default::default(),
            solo: if Self::HAS_SOLO {
                Some(Default::default())
            } else {
                None
            },
            inputs: vec![Default::default(); Self::INPUT_COUNT],
            outputs: vec![Default::default(); Self::OUTPUT_COUNT],
            rate: Default::default(),
            src: Default::default(),
            monitor_meters: Default::default(),
            analog_mixer_meters: Default::default(),
            monitor_mode: Default::default(),
        }
    }

    fn parse_meter_state(state: &mut IsochMeterState, image: &[u32]) -> Result<(), Error> {
        let monitor = (image[5] & 0x0000ffff) as i16;
        if (state.monitor - monitor).abs() > Self::ROTARY_STEP {
            state.monitor = monitor;
        }

        if let Some(solo) = &mut state.solo {
            let val = ((image[4] >> 16) & 0x0000ffff) as i16;
            if (*solo - val).abs() > Self::ROTARY_STEP {
                *solo = val;
            }
        }

        state
            .inputs
            .iter_mut()
            .take(Self::INPUT_COUNT)
            .enumerate()
            .for_each(|(i, input)| {
                let pos = if Self::INPUT_COUNT == 10 && i >= 8 {
                    i + 16
                } else {
                    i
                } + 16;
                *input = image[pos] as i32;
            });

        state
            .outputs
            .iter_mut()
            .take(Self::OUTPUT_COUNT)
            .enumerate()
            .for_each(|(i, output)| {
                let pos = if Self::OUTPUT_COUNT == 4 && i >= 2 {
                    i + 16
                } else {
                    i
                } + 34;
                *output = image[pos] as i32;
            });

        let bits = (image[52] & 0x0000000f) as u8;
        state.src = match bits {
            0x04 => Some(ClkSrc::Adat),
            0x03 => Some(ClkSrc::Spdif),
            0x02 => Some(ClkSrc::Wordclock),
            0x01 => Some(ClkSrc::Internal),
            _ => None,
        };

        let bits = ((image[52] >> 8) & 0x000000ff) as u8;
        state.rate = match bits {
            0x82 => Some(ClkRate::R96000),
            0x81 => Some(ClkRate::R88200),
            0x02 => Some(ClkRate::R48000),
            0x01 => Some(ClkRate::R44100),
            _ => None,
        };

        state
            .monitor_meters
            .iter_mut()
            .enumerate()
            .for_each(|(i, m)| {
                *m = image[i + 54] as i32;
            });

        state
            .analog_mixer_meters
            .iter_mut()
            .enumerate()
            .for_each(|(i, m)| {
                *m = image[i + 57] as i32;
            });

        if image[59] > 0 && image[59] < 4 {
            state.monitor_mode = match image[59] {
                2 => MonitorMode::Both,
                1 => MonitorMode::Inputs,
                _ => MonitorMode::Computer,
            };
        }

        Ok(())
    }
}
