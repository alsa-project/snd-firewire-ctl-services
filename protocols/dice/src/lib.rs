// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

#![doc = include_str!("../README.md")]

pub mod alesis;
pub mod avid;
pub mod focusrite;
pub mod lexicon;
pub mod loud;
pub mod maudio;
pub mod presonus;
pub mod tcat;
pub mod tcelectronic;

use {
    glib::{Error, FileError},
    hinawa::{FwNode, FwReq},
};

fn serialize_bool(val: &bool, raw: &mut [u8]) {
    assert!(raw.len() >= 4);

    raw[..4].copy_from_slice(&(*val as u32).to_be_bytes())
}

fn deserialize_bool(val: &mut bool, raw: &[u8]) {
    assert!(raw.len() >= 4);

    let mut quadlet = [0; 4];
    quadlet.copy_from_slice(&raw[..4]);
    *val = u32::from_be_bytes(quadlet) > 0;
}

fn serialize_i32(val: &i32, raw: &mut [u8]) {
    assert!(raw.len() >= 4);

    raw[..4].copy_from_slice(&val.to_be_bytes())
}

fn deserialize_i32(val: &mut i32, raw: &[u8]) {
    assert!(raw.len() >= 4);

    let mut quadlet = [0; 4];
    quadlet.copy_from_slice(&raw[..4]);
    *val = i32::from_be_bytes(quadlet);
}

fn serialize_i8(val: &i8, raw: &mut [u8]) {
    serialize_i32(&(*val as i32), raw)
}

fn deserialize_i8(val: &mut i8, raw: &[u8]) {
    let mut v = 0i32;
    deserialize_i32(&mut v, raw);
    *val = v as i8;
}

fn serialize_u32(val: &u32, raw: &mut [u8]) {
    assert!(raw.len() >= 4);

    raw[..4].copy_from_slice(&val.to_be_bytes())
}

fn deserialize_u32(val: &mut u32, raw: &[u8]) {
    assert!(raw.len() >= 4);

    let mut quadlet = [0; 4];
    quadlet.copy_from_slice(&raw[..4]);
    *val = u32::from_be_bytes(quadlet);
}

fn serialize_u8(val: &u8, raw: &mut [u8]) {
    assert!(raw.len() >= 4);

    serialize_u32(&(*val as u32), raw);
}

fn deserialize_u8(val: &mut u8, raw: &[u8]) {
    assert!(raw.len() >= 4);

    let mut v = 0u32;
    deserialize_u32(&mut v, raw);
    *val = v as u8;
}

fn serialize_usize(val: &usize, raw: &mut [u8]) {
    assert!(raw.len() >= 4);

    serialize_u32(&(*val as u32), raw);
}

fn deserialize_usize(val: &mut usize, raw: &[u8]) {
    assert!(raw.len() >= 4);

    let mut v = 0u32;
    deserialize_u32(&mut v, raw);
    *val = v as usize;
}

const QUADLET_SIZE: usize = 4;

/// For conversion between quadlet-aligned byte array and computed value.
trait QuadletConvert<T>: From<T> {
    fn build_quadlet(&self, raw: &mut [u8]);
    fn parse_quadlet(&mut self, raw: &[u8]);
}

/// For primitive u32 type and enumeration which has implementation to convert between u32.
impl<O> QuadletConvert<u32> for O
where
    u32: From<O>,
    O: From<u32> + Copy,
{
    fn build_quadlet(&self, raw: &mut [u8]) {
        assert_eq!(
            raw.len(),
            QUADLET_SIZE,
            "Programming error for length of quadlet data"
        );
        raw[..4].copy_from_slice(&u32::from(*self).to_be_bytes());
    }

    fn parse_quadlet(&mut self, raw: &[u8]) {
        assert_eq!(
            raw.len(),
            QUADLET_SIZE,
            "Programming error for length of quadlet data"
        );
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&raw[..4]);
        *self = Self::from(u32::from_be_bytes(quadlet))
    }
}

/// For primitive i32 type.
impl QuadletConvert<i32> for i32 {
    fn build_quadlet(&self, raw: &mut [u8]) {
        assert_eq!(
            raw.len(),
            QUADLET_SIZE,
            "Programming error for length of quadlet data"
        );
        raw[..4].copy_from_slice(&i32::from(*self).to_be_bytes());
    }

    fn parse_quadlet(&mut self, raw: &[u8]) {
        assert_eq!(
            raw.len(),
            QUADLET_SIZE,
            "Programming error for length of quadlet data"
        );
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&raw[..4]);
        *self = Self::from(i32::from_be_bytes(quadlet))
    }
}

/// For primitive bool type.
impl QuadletConvert<bool> for bool {
    fn build_quadlet(&self, raw: &mut [u8]) {
        assert_eq!(
            raw.len(),
            QUADLET_SIZE,
            "Programming error for length of quadlet data"
        );
        raw[..4].copy_from_slice(&u32::from(*self).to_be_bytes());
    }

    fn parse_quadlet(&mut self, raw: &[u8]) {
        assert_eq!(
            raw.len(),
            QUADLET_SIZE,
            "Programming error for length of quadlet data"
        );
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&raw[..4]);
        *self = u32::from_be_bytes(quadlet) > 0;
    }
}

/// For primitive u8 type.
impl QuadletConvert<u8> for u8 {
    fn build_quadlet(&self, raw: &mut [u8]) {
        assert_eq!(
            raw.len(),
            QUADLET_SIZE,
            "Programming error for length of quadlet data"
        );
        raw[..4].copy_from_slice(&u32::from(*self).to_be_bytes());
    }

    fn parse_quadlet(&mut self, raw: &[u8]) {
        assert_eq!(
            raw.len(),
            QUADLET_SIZE,
            "Programming error for length of quadlet data"
        );
        let mut quadlet = [0; 4];
        quadlet.copy_from_slice(&raw[..4]);
        *self = u32::from_be_bytes(quadlet) as u8;
    }
}

/// For conversion between quadlet-aligned byte array and array of computed value.
trait QuadletBlockConvert<T> {
    fn build_quadlet_block(&self, raw: &mut [u8]);
    fn parse_quadlet_block(&mut self, raw: &[u8]);
}

impl<T, U> QuadletBlockConvert<T> for [U]
where
    U: QuadletConvert<T>,
{
    fn build_quadlet_block(&self, raw: &mut [u8]) {
        assert_eq!(
            raw.len(),
            self.len() * QUADLET_SIZE,
            "Programming error for length of block data."
        );

        self.iter().enumerate().for_each(|(i, v)| {
            let pos = i * 4;
            v.build_quadlet(&mut raw[pos..(pos + 4)]);
        });
    }

    fn parse_quadlet_block(&mut self, raw: &[u8]) {
        assert_eq!(
            raw.len(),
            self.len() * QUADLET_SIZE,
            "Programming error for length of block data."
        );

        self.iter_mut().enumerate().for_each(|(i, v)| {
            let pos = i * 4;
            v.parse_quadlet(&raw[pos..(pos + 4)]);
        });
    }
}
