// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol about hardware control.
//!
//! The module includes protocol about hardware control defined by Echo Audio Digital Corporation
//! for Fireworks board module.

use super::*;

const CATEGORY_HWCTL: u32 = 3;

const CMD_SET_CLOCK: u32 = 0;
const CMD_GET_CLOCK: u32 = 1;
const CMD_SET_FLAGS: u32 = 3;
const CMD_GET_FLAGS: u32 = 4;
const CMD_BLINK_LED: u32 = 5;
const CMD_RECONNECT: u32 = 6;

/// The type of hardware control.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HwCtlFlag {
    /// Whether multiplexer is enabled or not for audio signal.
    MixerEnabled,
    /// Whether channel status block of IEC 60958-1 is for professional use or not.
    SpdifPro,
    /// Whether main data field of IEC 60958-1 is not for linear PCM samples.
    SpdifNoneAudio,
    /// Whether control room B is selected or not.
    CtlRoomSelect,
    /// Whether physical knob is bypass to adjust any output.
    OutputLevelBypass,
    /// The mode of metering for input in surface.
    MeterInMode,
    /// The mode of metering for output in surface.
    MeterOutMode,
    /// Whether to enable soft clip.
    SoftClip,
    /// The hex mode of robot guitar.
    GuitarHexInput,
    /// Whether to enable automatic charging or not.
    GuitarAutoCharging,
    /// Whether to enable phantom powering or not.
    PhantomPowering,
    Reserved(usize),
}

impl From<HwCtlFlag> for usize {
    fn from(flag: HwCtlFlag) -> Self {
        match flag {
            HwCtlFlag::MixerEnabled => 0,
            HwCtlFlag::SpdifPro => 1,
            HwCtlFlag::SpdifNoneAudio => 2,
            HwCtlFlag::CtlRoomSelect => 8, // B if it stands, else A.
            HwCtlFlag::OutputLevelBypass => 9,
            HwCtlFlag::MeterInMode => 12,
            HwCtlFlag::MeterOutMode => 13,
            HwCtlFlag::SoftClip => 18,
            HwCtlFlag::GuitarHexInput => 29,
            HwCtlFlag::GuitarAutoCharging => 30,
            HwCtlFlag::PhantomPowering => 31,
            HwCtlFlag::Reserved(pos) => pos,
        }
    }
}

impl From<usize> for HwCtlFlag {
    fn from(pos: usize) -> Self {
        match pos {
            0 => HwCtlFlag::MixerEnabled,
            1 => HwCtlFlag::SpdifPro,
            2 => HwCtlFlag::SpdifNoneAudio,
            8 => HwCtlFlag::CtlRoomSelect,
            9 => HwCtlFlag::OutputLevelBypass,
            12 => HwCtlFlag::MeterInMode,
            13 => HwCtlFlag::MeterOutMode,
            18 => HwCtlFlag::SoftClip,
            29 => HwCtlFlag::GuitarHexInput,
            30 => HwCtlFlag::GuitarAutoCharging,
            31 => HwCtlFlag::PhantomPowering,
            _ => HwCtlFlag::Reserved(pos),
        }
    }
}

/// Protocol about hardware control for Fireworks board module.
pub trait HwCtlProtocol: EfwProtocol {
    fn set_clock(
        &mut self,
        src: Option<ClkSrc>,
        rate: Option<u32>,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut args = [0; 3];
        let (current_src, current_rate) = self.get_clock(timeout_ms)?;
        args[0] = usize::from(match src {
            Some(s) => s,
            None => current_src,
        }) as u32;
        args[1] = match rate {
            Some(r) => r,
            None => current_rate,
        };
        self.transaction(
            CATEGORY_HWCTL,
            CMD_SET_CLOCK,
            &args,
            &mut vec![0; 3],
            timeout_ms,
        )
    }

    fn get_clock(&mut self, timeout_ms: u32) -> Result<(ClkSrc, u32), Error> {
        let mut params = vec![0; 3];
        self.transaction(CATEGORY_HWCTL, CMD_GET_CLOCK, &[], &mut params, timeout_ms)
            .map(|_| (ClkSrc::from(params[0] as usize), params[1]))
    }

    fn set_flags(
        &mut self,
        enables: Option<&[HwCtlFlag]>,
        disables: Option<&[HwCtlFlag]>,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut args = [0; 2];
        if let Some(flags) = enables {
            args[0] = flags
                .iter()
                .fold(0, |mask, flag| mask | (1 << usize::from(*flag)));
        }
        if let Some(flags) = disables {
            args[1] = flags
                .iter()
                .fold(0, |mask, flag| mask | (1 << usize::from(*flag)));
        }
        self.transaction(
            CATEGORY_HWCTL,
            CMD_SET_FLAGS,
            &args,
            &mut Vec::new(),
            timeout_ms,
        )
    }

    fn get_flags(&mut self, timeout_ms: u32) -> Result<Vec<HwCtlFlag>, Error> {
        let mut params = vec![0];
        self.transaction(CATEGORY_HWCTL, CMD_GET_FLAGS, &[], &mut params, timeout_ms)
            .map(|_| {
                (0..32)
                    .filter(|i| params[0] & (1 << i) > 0)
                    .map(|i| HwCtlFlag::from(i))
                    .collect()
            })
    }

    /// Blink LEDs on device.
    fn blink_led(&mut self, timeout_ms: u32) -> Result<(), Error> {
        self.transaction(
            CATEGORY_HWCTL,
            CMD_BLINK_LED,
            &[],
            &mut Vec::new(),
            timeout_ms,
        )
    }

    /// Take the device to disappear from IEEE 1394 bus, then to appear again.
    fn reconnect(&mut self, timeout_ms: u32) -> Result<(), Error> {
        self.transaction(
            CATEGORY_HWCTL,
            CMD_RECONNECT,
            &[],
            &mut Vec::new(),
            timeout_ms,
        )
    }
}

impl<O: EfwProtocol> HwCtlProtocol for O {}
