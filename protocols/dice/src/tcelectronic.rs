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
pub mod fw_led;
pub mod midi_send;
pub mod prog;
pub mod reverb;
pub mod standalone;

use super::{tcat::*, *};

const BASE_OFFSET: usize = 0x00a01000;

/// Data of segment.
pub trait TcKonnektSegmentData: Default {
    fn build(&self, raw: &mut [u8]);
    fn parse(&mut self, raw: &[u8]);
}

/// Specification of segment.
pub trait TcKonnektSegmentSpec {
    const OFFSET: usize;
    const SIZE: usize;
}

/// The generic structure for segment.
#[derive(Debug)]
pub struct TcKonnektSegment<U>
where
    U: TcKonnektSegmentData,
{
    pub data: U,
    raw: Vec<u8>,
}

impl<U> Default for TcKonnektSegment<U>
where
    U: TcKonnektSegmentData,
    TcKonnektSegment<U>: TcKonnektSegmentSpec,
{
    fn default() -> Self {
        TcKonnektSegment {
            data: Default::default(),
            raw: vec![0; Self::SIZE],
        }
    }
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
pub trait TcKonnektSegmentOperation<T>: TcatOperation + TcKonnektSegmentSerdes<T>
where
    T: TcKonnektSegmentData,
{
    /// Cache whole segment and deserialize for parameters.
    fn cache_whole_segment(
        req: &FwReq,
        node: &FwNode,
        segment: &mut TcKonnektSegment<T>,
        timeout_ms: u32,
    ) -> Result<(), Error> {
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

impl<O: TcatOperation + TcKonnektSegmentSerdes<T>, T: TcKonnektSegmentData>
    TcKonnektSegmentOperation<T> for O
{
}

/// Operation to update content of segment in TC Electronic Konnekt series.
pub trait TcKonnektMutableSegmentOperation<T>: TcatOperation + TcKonnektSegmentSerdes<T>
where
    T: TcKonnektSegmentData,
{
    /// Update part of segment for any change at the parameters.
    fn update_partial_segment(
        req: &FwReq,
        node: &FwNode,
        params: &T,
        segment: &mut TcKonnektSegment<T>,
        timeout_ms: u32,
    ) -> Result<(), Error> {
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
pub trait TcKonnektNotifiedSegmentOperation<T: TcKonnektSegmentData> {
    const NOTIFY_FLAG: u32;

    /// Check message to be notified or not.
    fn is_notified_segment(_: &TcKonnektSegment<T>, msg: u32) -> bool {
        msg & Self::NOTIFY_FLAG > 0
    }
}

/// Operation of segment in TC Electronics Konnekt series.
pub trait SegmentOperation<T>
where
    T: TcKonnektSegmentData,
    TcKonnektSegment<T>: TcKonnektSegmentSpec,
{
    fn read_segment(
        req: &mut FwReq,
        node: &mut FwNode,
        segment: &mut TcKonnektSegment<T>,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(segment.raw.len(), TcKonnektSegment::<T>::SIZE);

        GeneralProtocol::read(
            req,
            node,
            BASE_OFFSET + TcKonnektSegment::<T>::OFFSET,
            &mut segment.raw,
            timeout_ms,
        )
        .map(|_| segment.data.parse(&segment.raw))
    }

    fn write_segment(
        req: &mut FwReq,
        node: &mut FwNode,
        segment: &mut TcKonnektSegment<T>,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(segment.raw.len(), TcKonnektSegment::<T>::SIZE);

        let mut raw = segment.raw.clone();
        segment.data.build(&mut raw);

        (0..TcKonnektSegment::<T>::SIZE)
            .step_by(4)
            .try_for_each(|pos| {
                if raw[pos..(pos + 4)] != segment.raw[pos..(pos + 4)] {
                    GeneralProtocol::write(
                        req,
                        node,
                        BASE_OFFSET + TcKonnektSegment::<T>::OFFSET + pos,
                        &mut raw[pos..(pos + 4)],
                        timeout_ms,
                    )
                    .map(|_| segment.raw[pos..(pos + 4)].copy_from_slice(&raw[pos..(pos + 4)]))
                } else {
                    Ok(())
                }
            })
    }
}

/// Specification for segment in which any change is notified to controller.
pub trait TcKonnektNotifiedSegmentSpec {
    const NOTIFY_FLAG: u32;

    fn has_segment_change(&self, msg: u32) -> bool {
        msg & Self::NOTIFY_FLAG > 0
    }
}
