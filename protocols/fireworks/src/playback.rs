// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol about stream playback.
//!
//! The module includes protocol about stream playback defined by Echo Audio Digital Corporation for
//! Fireworks board module.

use super::*;

const CATEGORY_PLAYBACK: u32 = 6;

const CMD_SET_VOL: u32 = 0;
const CMD_GET_VOL: u32 = 1;
const CMD_SET_MUTE: u32 = 2;
const CMD_GET_MUTE: u32 = 3;
const CMD_SET_SOLO: u32 = 4;
const CMD_GET_SOLO: u32 = 5;

/// The parameters of playback.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct EfwPlaybackParameters {
    /// The volume of playback. The value is unsigned fixed-point number of 8.24 format; i.e. Q24.
    /// It is between 0x00000000..0x02000000 for -144.0..+6.0 dB.
    pub volumes: Vec<i32>,
    /// Whether to mute the playback.
    pub mutes: Vec<bool>,
}

impl<O, P> EfwWhollyCachableParamsOperation<P, EfwPlaybackParameters> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn cache_wholly(
        proto: &mut P,
        states: &mut EfwPlaybackParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(states.volumes.len(), Self::RX_CHANNEL_COUNTS[0]);
        assert_eq!(states.mutes.len(), Self::RX_CHANNEL_COUNTS[0]);

        states
            .volumes
            .iter_mut()
            .enumerate()
            .try_for_each(|(ch, vol)| {
                let args = [ch as u32, 0];
                let mut resps = vec![0; 2];
                proto
                    .transaction(
                        CATEGORY_PLAYBACK,
                        CMD_GET_VOL,
                        &args,
                        &mut resps,
                        timeout_ms,
                    )
                    .map(|_| *vol = resps[1] as i32)
            })?;

        states
            .mutes
            .iter_mut()
            .enumerate()
            .try_for_each(|(ch, mute)| {
                let args = [ch as u32, 0];
                let mut resps = vec![0; 2];
                proto
                    .transaction(
                        CATEGORY_PLAYBACK,
                        CMD_GET_MUTE,
                        &args,
                        &mut resps,
                        timeout_ms,
                    )
                    .map(|_| *mute = resps[1] > 0)
            })?;

        Ok(())
    }
}

impl<O, P> EfwPartiallyUpdatableParamsOperation<P, EfwPlaybackParameters> for O
where
    O: EfwHardwareSpecification,
    P: EfwProtocolExtManual,
{
    fn update_partially(
        proto: &mut P,
        states: &mut EfwPlaybackParameters,
        updates: EfwPlaybackParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(states.volumes.len(), Self::RX_CHANNEL_COUNTS[0]);
        assert_eq!(states.mutes.len(), Self::RX_CHANNEL_COUNTS[0]);
        assert_eq!(updates.volumes.len(), Self::RX_CHANNEL_COUNTS[0]);
        assert_eq!(updates.mutes.len(), Self::RX_CHANNEL_COUNTS[0]);

        states
            .volumes
            .iter_mut()
            .zip(updates.volumes.iter())
            .enumerate()
            .filter(|(_, (o, n))| !o.eq(n))
            .try_for_each(|(ch, (curr, &vol))| {
                let args = [ch as u32, vol as u32];
                let mut params = vec![0; 2];
                proto
                    .transaction(
                        CATEGORY_PLAYBACK,
                        CMD_SET_VOL,
                        &args,
                        &mut params,
                        timeout_ms,
                    )
                    .map(|_| *curr = vol)
            })?;

        states
            .mutes
            .iter_mut()
            .zip(updates.mutes.iter())
            .enumerate()
            .filter(|(_, (o, n))| !o.eq(n))
            .try_for_each(|(ch, (curr, &mute))| {
                let args = [ch as u32, mute as u32];
                let mut params = vec![0; 2];
                proto
                    .transaction(
                        CATEGORY_PLAYBACK,
                        CMD_SET_MUTE,
                        &args,
                        &mut params,
                        timeout_ms,
                    )
                    .map(|_| *curr = mute)
            })?;

        Ok(())
    }
}

/// The parameters of playback.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct EfwPlaybackSoloParameters {
    /// Whether to mute the other channels.
    pub solos: Vec<bool>,
}

/// The specification for solo of playback.
pub trait EfwPlaybackSoloSpecification: EfwHardwareSpecification {
    fn create_playback_solo_parameters() -> EfwPlaybackSoloParameters {
        EfwPlaybackSoloParameters {
            solos: vec![Default::default(); Self::RX_CHANNEL_COUNTS[0]],
        }
    }
}

impl<O, P> EfwWhollyCachableParamsOperation<P, EfwPlaybackSoloParameters> for O
where
    O: EfwPlaybackSoloSpecification,
    P: EfwProtocolExtManual,
{
    fn cache_wholly(
        proto: &mut P,
        states: &mut EfwPlaybackSoloParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(states.solos.len(), Self::RX_CHANNEL_COUNTS[0]);

        states
            .solos
            .iter_mut()
            .enumerate()
            .try_for_each(|(ch, solo)| {
                let args = [ch as u32, 0];
                let mut resps = vec![0; 2];
                proto
                    .transaction(
                        CATEGORY_PLAYBACK,
                        CMD_GET_SOLO,
                        &args,
                        &mut resps,
                        timeout_ms,
                    )
                    .map(|_| *solo = resps[1] > 0)
            })?;

        Ok(())
    }
}

impl<O, P> EfwPartiallyUpdatableParamsOperation<P, EfwPlaybackSoloParameters> for O
where
    O: EfwPlaybackSoloSpecification,
    P: EfwProtocolExtManual,
{
    fn update_partially(
        proto: &mut P,
        states: &mut EfwPlaybackSoloParameters,
        updates: EfwPlaybackSoloParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(states.solos.len(), Self::RX_CHANNEL_COUNTS[0]);
        assert_eq!(updates.solos.len(), Self::RX_CHANNEL_COUNTS[0]);

        states
            .solos
            .iter_mut()
            .zip(updates.solos.iter())
            .enumerate()
            .filter(|(_, (o, n))| !o.eq(n))
            .try_for_each(|(ch, (curr, &solo))| {
                let args = [ch as u32, solo as u32];
                let mut params = vec![0; 2];
                proto
                    .transaction(
                        CATEGORY_PLAYBACK,
                        CMD_SET_SOLO,
                        &args,
                        &mut params,
                        timeout_ms,
                    )
                    .map(|_| *curr = solo)
            })?;

        Ok(())
    }
}

/// Protocol about stream playback for Fireworks board module.
pub trait PlaybackProtocol: EfwProtocolExtManual {
    /// Set volume of stream playback. The value of vol is unsigned fixed-point number of 8.24
    /// format; i.e. Q24. (0x00000000..0x02000000, -144.0..+6.0 dB)
    fn set_playback_vol(&mut self, ch: usize, vol: i32, timeout_ms: u32) -> Result<(), Error> {
        let args = [ch as u32, vol as u32];
        let mut params = vec![0; 2];
        self.transaction(
            CATEGORY_PLAYBACK,
            CMD_SET_VOL,
            &args,
            &mut params,
            timeout_ms,
        )
    }

    /// Get volume of stream playback. The value of vol is unsigned fixed-point number of 8.24
    /// format; e.g. Q24. (0x00000000..0x02000000, -144.0..+6.0 dB)
    fn get_playback_vol(&mut self, ch: usize, timeout_ms: u32) -> Result<i32, Error> {
        let args = [ch as u32, 0];
        let mut params = vec![0; 2];
        self.transaction(
            CATEGORY_PLAYBACK,
            CMD_GET_VOL,
            &args,
            &mut params,
            timeout_ms,
        )
        .map(|_| params[1] as i32)
    }

    fn set_playback_mute(&mut self, ch: usize, mute: bool, timeout_ms: u32) -> Result<(), Error> {
        let args = [ch as u32, mute as u32];
        let mut params = vec![0; 2];
        self.transaction(
            CATEGORY_PLAYBACK,
            CMD_SET_MUTE,
            &args,
            &mut params,
            timeout_ms,
        )
    }

    fn get_playback_mute(&mut self, ch: usize, timeout_ms: u32) -> Result<bool, Error> {
        let args = [ch as u32, 0];
        let mut params = vec![0; 2];
        self.transaction(
            CATEGORY_PLAYBACK,
            CMD_GET_MUTE,
            &args,
            &mut params,
            timeout_ms,
        )
        .map(|_| params[1] > 0)
    }

    fn set_playback_solo(&mut self, ch: usize, solo: bool, timeout_ms: u32) -> Result<(), Error> {
        let args = [ch as u32, solo as u32];
        let mut params = vec![0; 2];
        self.transaction(
            CATEGORY_PLAYBACK,
            CMD_SET_SOLO,
            &args,
            &mut params,
            timeout_ms,
        )
    }

    fn get_playback_solo(&mut self, ch: usize, timeout_ms: u32) -> Result<bool, Error> {
        let args = [ch as u32, 0];
        let mut params = vec![0; 2];
        self.transaction(
            CATEGORY_PLAYBACK,
            CMD_GET_SOLO,
            &args,
            &mut params,
            timeout_ms,
        )
        .map(|_| params[1] > 0)
    }
}

impl<O: EfwProtocolExtManual> PlaybackProtocol for O {}
