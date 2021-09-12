// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Meter protocol specific to Lexicon I-ONIX FW810s.
//!
//! The module includes structure, enumeration, and trait and its implementation for meter protocol
//! defined by Lexicon for I-ONIX FW810s.

use super::*;

use crate::*;
use crate::tcat::extension::*;

#[derive(Default, Debug)]
/// The structure to represent hardware meter.
pub struct IonixMeter{
    pub analog_inputs: [i32;8],
    pub spdif_inputs: [i32;8],
    pub stream_inputs: [i32;10],
    pub bus_outputs: [i32;8],
    pub main_outputs: [i32;2],
}

/// The structure to represent entry of hardware meter.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
struct IonixMeterEntry{
    /// The level of audio signal from the source to the destination.
    level: i16,
    /// The source of audio signal.
    src: SrcBlk,
    /// The destination of audio signal.
    dst: DstBlk,
}

impl From<u32> for IonixMeterEntry {
    fn from(val: u32) -> Self {
        IonixMeterEntry{
            level: ((val & 0xffff0000) >> 16) as i16,
            src: SrcBlk::from(((val & 0x0000ff00) >> 8) as u8),
            dst: DstBlk::from(((val & 0x000000ff) >> 0) as u8),
        }
    }
}

impl From<IonixMeterEntry> for u32 {
    fn from(entry: IonixMeterEntry) -> Self {
        ((entry.level as u32) << 16) | ((u8::from(entry.src) as u32) << 8) | (u8::from(entry.dst) as u32)
    }
}

/// The trait to represent protocol for hardware meter.
pub trait IonixMeterProtocol: IonixProtocol {
    const METER_OFFSET: usize = 0x0500;

    // NOTE: 90 entries are valid at all of supported sampling rate.
    const ENTRY_COUNT: usize = 90;

    fn read_meters(
        &self,
        node: &mut FwNode,
        meters: &mut IonixMeter,
        timeout_ms: u32
    ) -> Result<(), Error> {
        let mut raw = vec![0;Self::ENTRY_COUNT * 4];
        IonixProtocol::read(self, node, Self::METER_OFFSET, &mut raw, timeout_ms)
            .map(|_| {
                let mut entries = vec![IonixMeterEntry::default();Self::ENTRY_COUNT];
                entries.parse_quadlet_block(&raw);

                entries.iter()
                    .filter(|entry| entry.src.id == SrcBlkId::Avs0 && entry.dst.id == DstBlkId::MixerTx0)
                    .take(meters.stream_inputs.len())
                    .enumerate()
                    .for_each(|(i, entry)| meters.stream_inputs[i] = entry.level as i32);

                entries.iter()
                    .filter(|entry| entry.src.id == SrcBlkId::Ins0 && entry.dst.id == DstBlkId::MixerTx0)
                    .take(meters.analog_inputs.len())
                    .enumerate()
                    .for_each(|(i, entry)| meters.analog_inputs[i] = entry.level as i32);

                entries.iter()
                    .filter(|entry| entry.src.id == SrcBlkId::Aes && entry.dst.id == DstBlkId::MixerTx0)
                    .take(meters.spdif_inputs.len())
                    .enumerate()
                    .for_each(|(i, entry)| meters.spdif_inputs[i] = entry.level as i32);

                entries.iter()
                    .filter(|entry| entry.src.id == SrcBlkId::Mixer && entry.dst.id == DstBlkId::Ins0)
                    .take(meters.bus_outputs.len())
                    .enumerate()
                    .for_each(|(i, entry)| meters.bus_outputs[i] = entry.level as i32);

                entries.iter()
                    .filter(|entry| {
                        entry.src.id == SrcBlkId::Mixer &&
                            entry.dst.id == DstBlkId::Ins1 && entry.dst.ch < 2
                    })
                    .take(meters.main_outputs.len())
                    .enumerate()
                    .for_each(|(i, entry)| meters.main_outputs[i] = entry.level as i32);
            })
    }
}

impl<O> IonixMeterProtocol for O
    where O: IonixProtocol,
{}
