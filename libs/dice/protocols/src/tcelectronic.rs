// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Protocol specific to TC Electronic Konnekt series.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by TC Electronic for Konnekt series.
//!
//! In Konnekt series, the accessible memory space is separated to several segments.

pub mod shell;

use glib::Error;

use hinawa::FwNode;

use super::tcat::*;

pub trait TcKonnektSegmentData : Default {
    fn build(&self, raw: &mut [u8]);
    fn parse(&mut self, raw: &[u8]);
}

/// The trait to represent specification of segment.
pub trait TcKonnektSegmentSpec {
    const OFFSET: usize;
    const SIZE: usize;
}

/// The structure to represent segment.
#[derive(Debug)]
pub struct TcKonnektSegment<U>
    where U: TcKonnektSegmentData,
{
    pub data: U,
    raw: Vec<u8>,
}

impl<U> Default for TcKonnektSegment<U>
    where U: TcKonnektSegmentData,
          TcKonnektSegment<U>: TcKonnektSegmentSpec,
{
    fn default() -> Self {
        TcKonnektSegment{
            data: Default::default(),
            raw: vec![0;Self::SIZE],
        }
    }
}

/// The trait to represent protocol for segment.
pub trait TcKonnektSegmentProtocol<T, U> : GeneralProtocol<T>
    where T: AsRef<FwNode>,
          U: TcKonnektSegmentData,
          TcKonnektSegment<U>: TcKonnektSegmentSpec,
{
    const BASE_OFFSET: usize = 0x00a01000;

    fn read_segment(&self, node: &T, segment: &mut TcKonnektSegment<U>, timeout_ms: u32)
        -> Result<(), Error>
    {
        assert_eq!(segment.raw.len(), TcKonnektSegment::<U>::SIZE, "Programming error...");

        self.read(node, Self::BASE_OFFSET + TcKonnektSegment::<U>::OFFSET, &mut segment.raw, timeout_ms)
            .map(|_| segment.data.parse(&segment.raw))
    }

    fn write_segment(&self, node: &T, segment: &mut TcKonnektSegment<U>, timeout_ms: u32)
        -> Result<(), Error>
    {
        assert_eq!(segment.raw.len(), TcKonnektSegment::<U>::SIZE, "Programming error...");

        let mut raw = segment.raw.clone();
        segment.data.build(&mut raw);

        (0..(raw.len() / 4))
            .map(|i| i * 4)
            .try_for_each(|pos| {
                if raw[pos..(pos + 4)] != segment.raw[pos..(pos + 4)] {
                    self.write(node, Self::BASE_OFFSET + TcKonnektSegment::<U>::OFFSET + pos,
                               &mut raw[pos..(pos + 4)], timeout_ms)
                        .map(|_| segment.raw[pos..(pos + 4)].copy_from_slice(&raw[pos..(pos + 4)]))
                } else {
                    Ok(())
                }
            })
    }
}

impl<O, T, U> TcKonnektSegmentProtocol<T, U> for O
    where O: GeneralProtocol<T>,
          T: AsRef<FwNode>,
          U: TcKonnektSegmentData,
          TcKonnektSegment<U>: TcKonnektSegmentSpec,
{}

/// The trait to represent specification for segment in which any change is notified to controller.
pub trait TcKonnektNotifiedSegmentSpec {
    const NOTIFY_FLAG: u32;

    fn get_flag(&self) -> u32 {
        Self::NOTIFY_FLAG
    }
}

/// The trait to represent notification for segment in which any change is notified to controller.
pub trait TcKonnektSegmentNotification<U> : TcatNotification
    where U: TcKonnektSegmentData,
          TcKonnektSegment<U>: TcKonnektNotifiedSegmentSpec,
{
    fn has_segment_change(self, segment: &TcKonnektSegment<U>) -> bool {
        self.bitand(segment.get_flag()) > 0
    }
}

impl<O, U> TcKonnektSegmentNotification<U> for O
    where O: TcatNotification,
          U: TcKonnektSegmentData,
          TcKonnektSegment<U>: TcKonnektNotifiedSegmentSpec,
{}


/// The trait to parse notification.
pub trait TcKonnektNotifiedSegmentProtocol<T, U, V> : TcKonnektSegmentProtocol<T, U>
    where T: AsRef<FwNode>,
          U: TcKonnektSegmentData,
          TcKonnektSegment<U>: TcKonnektSegmentSpec + TcKonnektNotifiedSegmentSpec,
          V: TcKonnektSegmentNotification<U>,
{
    fn parse_notification(&self, node: &T, segment: &mut TcKonnektSegment<U>, timeout_ms: u32, msg: V)
        -> Result<(), Error>
    {
        if msg.has_segment_change(segment) {
            self.read_segment(node, segment, timeout_ms)
        } else {
            Ok(())
        }
    }
}

impl<O, T, U, V> TcKonnektNotifiedSegmentProtocol<T, U, V> for O
    where O: TcKonnektSegmentProtocol<T, U>,
          T: AsRef<FwNode>,
          U: TcKonnektSegmentData,
          TcKonnektSegment<U>: TcKonnektSegmentSpec + TcKonnektNotifiedSegmentSpec,
          V: TcKonnektSegmentNotification<U>,
{}
