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
