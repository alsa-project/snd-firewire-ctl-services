// SPDX-License-Identifier: LGPL-3.0-or-later
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

/// The parameters of sampling clock configuration.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct EfwSamplingClockParameters {
    /// The frequency.
    pub rate: u32,
    /// The source.
    pub source: ClkSrc,
}

impl<O, P> EfwWhollyCachableParamsOperation<P, EfwSamplingClockParameters> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn cache_wholly(
        proto: &mut P,
        states: &mut EfwSamplingClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = Vec::new();
        let mut params = vec![0; 3];
        proto
            .transaction(
                CATEGORY_HWCTL,
                CMD_GET_CLOCK,
                &args,
                &mut params,
                timeout_ms,
            )
            .map(|_| {
                deserialize_clock_source(&mut states.source, params[0]);
                states.rate = params[1];
            })
    }
}

impl<O, P> EfwWhollyUpdatableParamsOperation<P, EfwSamplingClockParameters> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn update_wholly(
        proto: &mut P,
        states: &EfwSamplingClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = [serialize_clock_source(&states.source), states.rate, 0];
        let mut params = vec![0; 3];
        proto.transaction(
            CATEGORY_HWCTL,
            CMD_SET_CLOCK,
            &args,
            &mut params,
            timeout_ms,
        )
    }
}

/// The type of hardware control.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

impl Default for HwCtlFlag {
    fn default() -> Self {
        Self::Reserved(usize::MAX)
    }
}

fn serialize_hw_ctl_flag(flag: &HwCtlFlag) -> usize {
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
        HwCtlFlag::Reserved(pos) => *pos,
    }
}

fn deserialize_hw_ctl_flag(flag: &mut HwCtlFlag, pos: usize) {
    *flag = match pos {
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
    };
}

/// The parameter of flags for hardware control;
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct EfwHwCtlFlags(pub Vec<HwCtlFlag>);

impl<O, P> EfwWhollyCachableParamsOperation<P, EfwHwCtlFlags> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn cache_wholly(
        proto: &mut P,
        states: &mut EfwHwCtlFlags,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = Vec::new();
        let mut params = vec![0];
        proto.transaction(
            CATEGORY_HWCTL,
            CMD_GET_FLAGS,
            &args,
            &mut params,
            timeout_ms,
        )?;

        (0..32).filter(|i| params[0] & (1 << i) > 0).for_each(|i| {
            let mut flag = HwCtlFlag::default();
            deserialize_hw_ctl_flag(&mut flag, i);
            if states.0.iter().find(|f| flag.eq(f)).is_none() {
                states.0.push(flag);
            }
        });

        Ok(())
    }
}

impl<O, P> EfwPartiallyUpdatableParamsOperation<P, EfwHwCtlFlags> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn update_partially(
        proto: &mut P,
        states: &mut EfwHwCtlFlags,
        updates: EfwHwCtlFlags,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut args = [0; 2];
        let mut params = Vec::new();
        // Enable.
        updates
            .0
            .iter()
            .for_each(|flag| args[0] |= 1 << serialize_hw_ctl_flag(flag));
        // Disable.
        states.0.iter().for_each(|flag| {
            if updates.0.iter().find(|f| flag.eq(f)).is_none() {
                args[1] = 1 << serialize_hw_ctl_flag(flag);
            }
        });
        proto.transaction(
            CATEGORY_HWCTL,
            CMD_SET_FLAGS,
            &args,
            &mut params,
            timeout_ms,
        )
    }
}

/// The parameter to blink LED.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct EfwLedBlink;

impl<O, P> EfwWhollyUpdatableParamsOperation<P, EfwLedBlink> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn update_wholly(proto: &mut P, _: &EfwLedBlink, timeout_ms: u32) -> Result<(), Error> {
        let args = Vec::new();
        let mut params = Vec::new();
        proto.transaction(
            CATEGORY_HWCTL,
            CMD_BLINK_LED,
            &args,
            &mut params,
            timeout_ms,
        )
    }
}

/// Protocol about hardware control for Fireworks board module.
pub trait HwCtlProtocol: EfwProtocolExtManual {
    fn set_clock(
        &mut self,
        src: Option<ClkSrc>,
        rate: Option<u32>,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut args = [0; 3];
        let (current_src, current_rate) = self.get_clock(timeout_ms)?;
        let s = match &src {
            Some(s) => s,
            None => &current_src,
        };
        args[0] = serialize_clock_source(s);
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
            .map(|_| {
                let mut src = ClkSrc::default();
                deserialize_clock_source(&mut src, params[0]);
                (src, params[1])
            })
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
                .fold(0, |mask, flag| mask | (1 << serialize_hw_ctl_flag(flag)))
        }
        if let Some(flags) = disables {
            args[1] = flags
                .iter()
                .fold(0, |mask, flag| mask | (1 << serialize_hw_ctl_flag(flag)));
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
                    .map(|i| {
                        let mut flag = HwCtlFlag::default();
                        deserialize_hw_ctl_flag(&mut flag, i);
                        flag
                    })
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

impl<O: EfwProtocolExtManual> HwCtlProtocol for O {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hw_ctl_flag_serdes() {
        [
            HwCtlFlag::MixerEnabled,
            HwCtlFlag::SpdifPro,
            HwCtlFlag::SpdifNoneAudio,
            HwCtlFlag::CtlRoomSelect,
            HwCtlFlag::OutputLevelBypass,
            HwCtlFlag::MeterInMode,
            HwCtlFlag::MeterOutMode,
            HwCtlFlag::SoftClip,
            HwCtlFlag::GuitarHexInput,
            HwCtlFlag::GuitarAutoCharging,
            HwCtlFlag::PhantomPowering,
            HwCtlFlag::default(),
        ]
        .iter()
        .for_each(|flag| {
            let val = serialize_hw_ctl_flag(&flag);
            let mut f = HwCtlFlag::default();
            deserialize_hw_ctl_flag(&mut f, val);
            assert_eq!(*flag, f);
        });
    }
}
