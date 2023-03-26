// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol about input monitor.
//!
//! The module includes protocol about input monitor defined by Echo Audio Digital Corporation for
//! Fireworks board module.

use super::*;

const CATEGORY_MONITOR: u32 = 8;

const CMD_SET_VOL: u32 = 0;
const CMD_GET_VOL: u32 = 1;
const CMD_SET_MUTE: u32 = 2;
const CMD_GET_MUTE: u32 = 3;
const CMD_SET_SOLO: u32 = 4;
const CMD_GET_SOLO: u32 = 5;
const CMD_SET_PAN: u32 = 6;
const CMD_GET_PAN: u32 = 7;

/// The parameters of input monitor.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct EfwMonitorSourceParameters {
    /// The gain of monitor input. The value is unsigned fixed-point number of 8.24 format; i.e.
    /// Q24. It is 0x00000000..0x02000000 for -144.0..+6.0 dB.
    pub gains: Vec<i32>,
    /// Whether to mute the monitor input.
    pub mutes: Vec<bool>,
    /// Whether to mute the other monitor sources.
    pub solos: Vec<bool>,
    /// L/R balance of monitor input. It is 0..255 from left to right.
    pub pans: Vec<u8>,
}

/// The parameters of input monitor.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct EfwMonitorParameters(pub Vec<EfwMonitorSourceParameters>);

impl<O, P> EfwWhollyCachableParamsOperation<P, EfwMonitorParameters> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn cache_wholly(
        proto: &mut P,
        states: &mut EfwMonitorParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(states.0.len(), Self::MONITOR_DESTINATION_COUNT);

        states
            .0
            .iter_mut()
            .enumerate()
            .try_for_each(|(dst_ch, sources)| {
                assert_eq!(sources.gains.len(), Self::MONITOR_SOURCE_COUNT);
                assert_eq!(sources.mutes.len(), Self::MONITOR_SOURCE_COUNT);
                assert_eq!(sources.solos.len(), Self::MONITOR_SOURCE_COUNT);
                assert_eq!(sources.pans.len(), Self::MONITOR_SOURCE_COUNT);

                sources
                    .gains
                    .iter_mut()
                    .enumerate()
                    .try_for_each(|(src_ch, gain)| {
                        let args = [src_ch as u32, dst_ch as u32, 0];
                        let mut params = vec![0; 3];
                        proto
                            .transaction(
                                CATEGORY_MONITOR,
                                CMD_GET_VOL,
                                &args,
                                &mut params,
                                timeout_ms,
                            )
                            .map(|_| *gain = params[2] as i32)
                    })?;

                sources
                    .mutes
                    .iter_mut()
                    .enumerate()
                    .try_for_each(|(src_ch, mute)| {
                        let args = [src_ch as u32, dst_ch as u32, 0];
                        let mut params = vec![0; 3];
                        proto
                            .transaction(
                                CATEGORY_MONITOR,
                                CMD_GET_MUTE,
                                &args,
                                &mut params,
                                timeout_ms,
                            )
                            .map(|_| *mute = params[2] > 0)
                    })?;

                sources
                    .solos
                    .iter_mut()
                    .enumerate()
                    .try_for_each(|(src_ch, solo)| {
                        let args = [src_ch as u32, dst_ch as u32, 0];
                        let mut params = vec![0; 3];
                        proto
                            .transaction(
                                CATEGORY_MONITOR,
                                CMD_GET_SOLO,
                                &args,
                                &mut params,
                                timeout_ms,
                            )
                            .map(|_| *solo = params[2] > 0)
                    })?;

                sources
                    .pans
                    .iter_mut()
                    .enumerate()
                    .try_for_each(|(src_ch, pan)| {
                        let args = [src_ch as u32, dst_ch as u32, 0];
                        let mut params = vec![0; 3];
                        proto
                            .transaction(
                                CATEGORY_MONITOR,
                                CMD_GET_PAN,
                                &args,
                                &mut params,
                                timeout_ms,
                            )
                            .map(|_| *pan = params[2] as u8)
                    })
            })
    }
}

impl<O, P> EfwPartiallyUpdatableParamsOperation<P, EfwMonitorParameters> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn update_partially(
        proto: &mut P,
        states: &mut EfwMonitorParameters,
        updates: EfwMonitorParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(states.0.len(), Self::MONITOR_DESTINATION_COUNT);

        states
            .0
            .iter_mut()
            .zip(updates.0.iter())
            .enumerate()
            .try_for_each(|(dst_ch, (input, update))| {
                assert_eq!(input.gains.len(), Self::MONITOR_SOURCE_COUNT);
                assert_eq!(input.mutes.len(), Self::MONITOR_SOURCE_COUNT);
                assert_eq!(input.solos.len(), Self::MONITOR_SOURCE_COUNT);
                assert_eq!(input.pans.len(), Self::MONITOR_SOURCE_COUNT);
                assert_eq!(update.gains.len(), Self::MONITOR_SOURCE_COUNT);
                assert_eq!(update.mutes.len(), Self::MONITOR_SOURCE_COUNT);
                assert_eq!(update.solos.len(), Self::MONITOR_SOURCE_COUNT);
                assert_eq!(update.pans.len(), Self::MONITOR_SOURCE_COUNT);

                input
                    .gains
                    .iter_mut()
                    .zip(update.gains.iter())
                    .enumerate()
                    .filter(|(_, (o, n))| !o.eq(n))
                    .try_for_each(|(src_ch, (curr, &gain))| {
                        let args = [src_ch as u32, dst_ch as u32, gain as u32];
                        proto
                            .transaction(
                                CATEGORY_MONITOR,
                                CMD_SET_VOL,
                                &args,
                                &mut vec![0; 3],
                                timeout_ms,
                            )
                            .map(|_| *curr = gain)
                    })?;

                input
                    .mutes
                    .iter_mut()
                    .zip(update.mutes.iter())
                    .enumerate()
                    .filter(|(_, (o, n))| !o.eq(n))
                    .try_for_each(|(src_ch, (curr, &mute))| {
                        let args = [src_ch as u32, dst_ch as u32, mute as u32];
                        let mut params = vec![0; 3];
                        proto
                            .transaction(
                                CATEGORY_MONITOR,
                                CMD_SET_MUTE,
                                &args,
                                &mut params,
                                timeout_ms,
                            )
                            .map(|_| *curr = mute)
                    })?;

                input
                    .solos
                    .iter_mut()
                    .zip(update.solos.iter())
                    .enumerate()
                    .filter(|(_, (o, n))| !o.eq(n))
                    .try_for_each(|(src_ch, (curr, &solo))| {
                        let args = [src_ch as u32, dst_ch as u32, solo as u32];
                        proto
                            .transaction(
                                CATEGORY_MONITOR,
                                CMD_SET_SOLO,
                                &args,
                                &mut vec![0; 3],
                                timeout_ms,
                            )
                            .map(|_| *curr = solo)
                    })?;

                input
                    .pans
                    .iter_mut()
                    .zip(update.pans.iter())
                    .enumerate()
                    .filter(|(_, (o, n))| !o.eq(n))
                    .try_for_each(|(src_ch, (curr, &pan))| {
                        let args = [src_ch as u32, dst_ch as u32, pan as u32];
                        proto
                            .transaction(
                                CATEGORY_MONITOR,
                                CMD_SET_PAN,
                                &args,
                                &mut vec![0; 3],
                                timeout_ms,
                            )
                            .map(|_| *curr = pan)
                    })
            })
    }
}

/// Protocol about input monitor for Fireworks board module.
pub trait MonitorProtocol: EfwProtocolExtManual {
    /// Set volume of monitor. The value of vol is unsigned fixed-point number of 8.24 format; i.e. Q24.
    /// (0x00000000..0x02000000, -144.0..+6.0 dB)
    fn set_monitor_vol(
        &mut self,
        dst: usize,
        src: usize,
        vol: i32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = [src as u32, dst as u32, vol as u32];
        self.transaction(
            CATEGORY_MONITOR,
            CMD_SET_VOL,
            &args,
            &mut vec![0; 3],
            timeout_ms,
        )
    }

    /// Get volume of monitor. The value of vol is unsigned fixed-point number of 8.24 format; i.e. Q24.
    /// (0x00000000..0x02000000, -144.0..+6.0 dB)
    fn get_monitor_vol(&mut self, dst: usize, src: usize, timeout_ms: u32) -> Result<i32, Error> {
        let args = [src as u32, dst as u32, 0];
        let mut params = vec![0; 3];
        self.transaction(
            CATEGORY_MONITOR,
            CMD_GET_VOL,
            &args,
            &mut params,
            timeout_ms,
        )
        .map(|_| params[2] as i32)
    }

    fn set_monitor_mute(
        &mut self,
        dst: usize,
        src: usize,
        mute: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = [src as u32, dst as u32, mute as u32];
        let mut params = vec![0; 3];
        self.transaction(
            CATEGORY_MONITOR,
            CMD_SET_MUTE,
            &args,
            &mut params,
            timeout_ms,
        )
    }

    fn get_monitor_mute(&mut self, dst: usize, src: usize, timeout_ms: u32) -> Result<bool, Error> {
        let args = [src as u32, dst as u32, 0];
        let mut params = vec![0; 3];
        self.transaction(
            CATEGORY_MONITOR,
            CMD_GET_MUTE,
            &args,
            &mut params,
            timeout_ms,
        )
        .map(|_| params[2] > 0)
    }

    fn set_monitor_solo(
        &mut self,
        dst: usize,
        src: usize,
        solo: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = [src as u32, dst as u32, solo as u32];
        self.transaction(
            CATEGORY_MONITOR,
            CMD_SET_SOLO,
            &args,
            &mut vec![0; 3],
            timeout_ms,
        )
    }

    fn get_monitor_solo(&mut self, dst: usize, src: usize, timeout_ms: u32) -> Result<bool, Error> {
        let args = [src as u32, dst as u32, 0];
        let mut params = vec![0; 3];
        self.transaction(
            CATEGORY_MONITOR,
            CMD_GET_SOLO,
            &args,
            &mut params,
            timeout_ms,
        )
        .map(|_| params[2] > 0)
    }

    /// Set L/R balance of monitor. (0..255, left to right)
    fn set_monitor_pan(
        &mut self,
        dst: usize,
        src: usize,
        pan: u8,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let args = [src as u32, dst as u32, pan as u32];
        self.transaction(
            CATEGORY_MONITOR,
            CMD_SET_PAN,
            &args,
            &mut vec![0; 3],
            timeout_ms,
        )
    }

    /// Get L/R balance of monitor. (0..255, left to right)
    fn get_monitor_pan(&mut self, dst: usize, src: usize, timeout_ms: u32) -> Result<u8, Error> {
        let args = [src as u32, dst as u32, 0];
        let mut params = vec![0; 3];
        self.transaction(
            CATEGORY_MONITOR,
            CMD_GET_PAN,
            &args,
            &mut params,
            timeout_ms,
        )
        .map(|_| params[2] as u8)
    }
}

impl<O: EfwProtocolExtManual> MonitorProtocol for O {}
