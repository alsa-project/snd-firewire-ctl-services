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

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct FStudioMeter{
    pub analog_inputs: [u8;8],
    pub stream_inputs: [u8;18],
    pub mixer_outputs: [u8;18],
}

impl FStudioMeter {
    const SIZE: usize = 0x40;
}

pub trait PresonusFStudioMeterProtocol<T> : PresonusFStudioProto<T>
    where T: AsRef<FwNode>,
{
    const METER_OFFSET: usize = 0x13e8;

    fn read_meter(&self, node: &T, meter: &mut FStudioMeter, timeout_ms: u32) -> Result<(), Error> {
        let mut raw = vec![0;FStudioMeter::SIZE];
        PresonusFStudioProto::read(self, node, Self::METER_OFFSET, &mut raw, timeout_ms)
            .map(|_| {
                let mut quadlet = [0;4];
                (0..(FStudioMeter::SIZE / 4))
                    .for_each(|i| {
                        let pos = i * 4;
                        quadlet.copy_from_slice(&raw[pos..(pos + 4)]);
                        let val = u32::from_be_bytes(quadlet);
                        raw[pos..(pos + 4)].copy_from_slice(&val.to_le_bytes());
                    });
                meter.analog_inputs.copy_from_slice(&raw[8..16]);
                meter.stream_inputs.copy_from_slice(&raw[16..34]);
                meter.mixer_outputs.copy_from_slice(&raw[40..58]);
            })
    }
}

impl<T: AsRef<FwNode>> PresonusFStudioMeterProtocol<T> for FStudioProto {}
