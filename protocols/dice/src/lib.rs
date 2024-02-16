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
pub mod weiss;

use {
    glib::Error,
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

fn serialize_i16(val: &i16, raw: &mut [u8]) {
    serialize_i32(&(*val as i32), raw)
}

fn deserialize_i16(val: &mut i16, raw: &[u8]) {
    let mut v = 0i32;
    deserialize_i32(&mut v, raw);
    *val = v as i16;
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
