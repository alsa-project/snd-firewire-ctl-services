// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto
use glib::Error;

use hinawa::{FwNode, FwReq};

use crate::tcat::*;

#[derive(Default, Debug)]
pub struct FStudioProto(FwReq);

impl AsRef<FwReq> for FStudioProto {
    fn as_ref(&self) -> &FwReq {
        &self.0
    }
}

/// The trait to represent protocol specific to FireStudio.
pub trait PresonusFStudioProto<T> : GeneralProtocol<T>
    where T: AsRef<FwNode>,
{
    const OFFSET: usize = 0x00700000;

    fn read(&self, node: &T, offset: usize, raw: &mut [u8], timeout_ms: u32) -> Result<(), Error> {
        GeneralProtocol::read(self, node, Self::OFFSET + offset, raw, timeout_ms)
    }

    fn write(&self, node: &T, offset: usize, raw: &mut [u8], timeout_ms: u32) -> Result<(), Error> {
        GeneralProtocol::write(self, node, Self::OFFSET + offset, raw, timeout_ms)
    }
}

impl<T: AsRef<FwNode>> PresonusFStudioProto<T> for FStudioProto {}
