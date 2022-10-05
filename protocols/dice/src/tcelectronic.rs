// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol specific to TC Electronic Konnekt series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Konnekt series.
//!
//! In Konnekt series, the accessible memory space is separated to several segments.

pub mod desktop;
pub mod shell;
pub mod studio;

pub mod ch_strip;
pub mod midi_send;
pub mod prog;
pub mod reverb;
pub mod standalone;

use super::{tcat::*, *};

const BASE_OFFSET: usize = 0x00a01000;

/// The generic structure for segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcKonnektSegment<U> {
    /// Intermediate structured data for parameters.
    pub data: U,
    /// Raw byte data for memory layout in hardware.
    raw: Vec<u8>,
}

/// Serialize and deserialize for segment in TC Konnekt protocol.
pub trait TcKonnektSegmentSerdes<T> {
    /// The name of segment.
    const NAME: &'static str;

    /// The offset of segment.
    const OFFSET: usize;

    /// The size of segment.
    const SIZE: usize;

    /// Serialize for parameter.
    fn serialize(params: &T, raw: &mut [u8]) -> Result<(), String>;

    /// Deserialize for parameter.
    fn deserialize(params: &mut T, raw: &[u8]) -> Result<(), String>;
}

fn generate_error(segment_name: &str, cause: &str, raw: &[u8]) -> Error {
    let msg = format!(
        "segment: {}, cause: '{}', raw: {:02x?}",
        segment_name, cause, raw
    );
    Error::new(GeneralProtocolError::VendorDependent, &msg)
}

/// Operation to cache content of segment in TC Electronic Konnekt series.
pub trait TcKonnektSegmentOperation<T>: TcatOperation + TcKonnektSegmentSerdes<T> {
    /// Cache whole segment and deserialize for parameters.
    fn cache_whole_segment(
        req: &FwReq,
        node: &FwNode,
        segment: &mut TcKonnektSegment<T>,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        // NOTE: Something wrong to implement Default trait.
        assert_eq!(segment.raw.len(), Self::SIZE);

        Self::read(
            req,
            node,
            BASE_OFFSET + Self::OFFSET,
            &mut segment.raw,
            timeout_ms,
        )?;

        Self::deserialize(&mut segment.data, &segment.raw).map_err(|cause| {
            generate_error(Self::NAME, &cause, &segment.raw)
        })
    }
}

impl<O: TcatOperation + TcKonnektSegmentSerdes<T>, T> TcKonnektSegmentOperation<T> for O {}

/// Operation to update content of segment in TC Electronic Konnekt series.
pub trait TcKonnektMutableSegmentOperation<T>: TcatOperation + TcKonnektSegmentSerdes<T> {
    /// Update part of segment for any change at the parameters.
    fn update_partial_segment(
        req: &FwReq,
        node: &FwNode,
        params: &T,
        segment: &mut TcKonnektSegment<T>,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        // NOTE: Something wrong to implement Default trait.
        assert_eq!(segment.raw.len(), Self::SIZE);

        let mut raw = segment.raw.clone();
        Self::serialize(params, &mut raw).map_err(|cause| {
            generate_error(Self::NAME, &cause, &segment.raw)
        })?;

        (0..Self::SIZE).step_by(4).try_for_each(|pos| {
            let new = &mut raw[pos..(pos + 4)];
            if new != &segment.raw[pos..(pos + 4)] {
                Self::write(req, node, BASE_OFFSET + Self::OFFSET + pos, new, timeout_ms)
                    .map(|_| segment.raw[pos..(pos + 4)].copy_from_slice(new))
            } else {
                Ok(())
            }
        })?;

        Self::deserialize(&mut segment.data, &raw).map_err(|cause| {
            generate_error(Self::NAME, &cause, &segment.raw)
        })
    }

    /// Update whole segment by the parameters.
    fn update_whole_segment(
        req: &FwReq,
        node: &FwNode,
        params: &T,
        segment: &mut TcKonnektSegment<T>,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        // NOTE: Something wrong to implement Default trait.
        assert_eq!(segment.raw.len(), Self::SIZE);

        let mut raw = segment.raw.clone();
        Self::serialize(&params, &mut raw).map_err(|cause| {
            generate_error(Self::NAME, &cause, &segment.raw)
        })?;

        Self::write(req, node, BASE_OFFSET + Self::OFFSET, &mut raw, timeout_ms)?;

        segment.raw.copy_from_slice(&raw);
        Self::deserialize(&mut segment.data, &segment.raw).map_err(|cause| {
            generate_error(Self::NAME, &cause, &segment.raw)
        })
    }
}

/// Operation for segment in which any change is notified in TC Electronic Konnekt series.
pub trait TcKonnektNotifiedSegmentOperation<T> {
    const NOTIFY_FLAG: u32;

    /// Check message to be notified or not.
    fn is_notified_segment(_: &TcKonnektSegment<T>, msg: u32) -> bool {
        msg & Self::NOTIFY_FLAG > 0
    }
}

fn serialize_position<T: Eq + std::fmt::Debug>(
    entries: &[T],
    entry: &T,
    raw: &mut [u8],
    label: &str,
) -> Result<(), String> {
    assert!(raw.len() >= 4);

    entries
        .iter()
        .position(|t| entry.eq(t))
        .ok_or_else(|| format!("{} {:?} is not supported", label, entry))
        .map(|pos| {
            (pos as u32).build_quadlet(raw);
        })
}

fn deserialize_position<T: Copy + Eq + std::fmt::Debug>(
    entries: &[T],
    entry: &mut T,
    raw: &[u8],
    label: &str,
) -> Result<(), String> {
    assert!(raw.len() >= 4);

    let mut val = 0u32;
    val.parse_quadlet(raw);

    entries
        .iter()
        .nth(val as usize)
        .ok_or_else(|| format!("{} not found for index {}", label, val))
        .map(|&e| *entry = e)
}

/// The state of FireWire LED.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FireWireLedState {
    /// Off.
    Off,
    /// On.
    On,
    /// Blinking fastly.
    BlinkFast,
    /// Blinking slowly.
    BlinkSlow,
}

impl Default for FireWireLedState {
    fn default() -> Self {
        Self::Off
    }
}

const FW_LED_STATES: &[FireWireLedState] = &[
    FireWireLedState::Off,
    FireWireLedState::On,
    FireWireLedState::BlinkSlow,
    FireWireLedState::BlinkFast,
];

const FW_LED_STATE_LABEL: &str = "FireWire LED state";

fn serialize_fw_led_state(state: &FireWireLedState, raw: &mut [u8]) -> Result<(), String> {
    serialize_position(FW_LED_STATES, state, raw, FW_LED_STATE_LABEL)
}

fn deserialize_fw_led_state(state: &mut FireWireLedState, raw: &[u8]) -> Result<(), String> {
    deserialize_position(FW_LED_STATES, state, raw, FW_LED_STATE_LABEL)
}
