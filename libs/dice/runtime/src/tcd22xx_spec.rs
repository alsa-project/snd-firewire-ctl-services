// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use dice_protocols::tcat::extension::{*, caps_section::*, cmd_section::*};

use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Input<'a> {
    pub id: SrcBlkId,
    pub offset: u8,
    pub count: u8,
    pub label: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Output<'a> {
    pub id: DstBlkId,
    pub offset: u8,
    pub count: u8,
    pub label: Option<&'a str>,
}

pub trait Tcd22xxSpec<'a> {
    // For each model.
    const INPUTS: &'a [Input<'a>];
    const OUTPUTS: &'a [Output<'a>];
    const FIXED: &'a [SrcBlk];

    // From specification of TCD22xx.
    const MIXER_OUT_PORTS: [u8;3] = [16, 16, 8];
    const MIXER_IN_PORTS: [(DstBlkId, u8);2] = [(DstBlkId::MixerTx0, 16), (DstBlkId::MixerTx1, 2)];

    // From specification of ADAT/SMUX.
    const ADAT_CHANNELS: [u8;3] = [8, 4, 2];

    fn get_adat_channel_count(rate_mode: RateMode) -> u8 {
        let index = match rate_mode {
            RateMode::Low => 0,
            RateMode::Middle => 1,
            RateMode::High => 2,
        };
        Self::ADAT_CHANNELS[index]
    }

    fn get_mixer_out_port_count(rate_mode: RateMode) -> u8 {
        let index = match rate_mode {
            RateMode::Low => 0,
            RateMode::Middle => 1,
            RateMode::High => 2,
        };
        Self::MIXER_OUT_PORTS[index]
    }

    fn get_mixer_in_port_count() -> u8 {
        Self::MIXER_IN_PORTS.iter().fold(0, |accum, (_, count)| accum + count)
    }

    fn compute_avail_real_blk_pair(&self, rate_mode: RateMode) -> (Vec<u8>, Vec<u8>)
    {
        let mut srcs = Vec::<u8>::new();
        Self::INPUTS.iter().for_each(|entry| {
            let offset = match entry.id {
                SrcBlkId::Adat => srcs.iter().filter(|&d| SrcBlk::from(*d).id == entry.id).count() as u8,
                _ => entry.offset,
            };
            let count = match entry.id {
                SrcBlkId::Adat => Self::get_adat_channel_count(rate_mode),
                _ => entry.count,
            };
            (offset..(offset + count)).for_each(|ch| {
                srcs.push(u8::from(SrcBlk{id: entry.id, ch}));
            });
        });

        let mut dsts = Vec::<u8>::new();
        Self::OUTPUTS.iter().for_each(|entry| {
            let offset = match entry.id {
                DstBlkId::Adat => dsts.iter().filter(|&d| DstBlk::from(*d).id == entry.id).count() as u8,
                _ => entry.offset,
            };
            let count = match entry.id {
                DstBlkId::Adat => Self::get_adat_channel_count(rate_mode),
                _ => entry.count,
            };
            (offset..(offset + count)).for_each(|ch| {
                dsts.push(u8::from(DstBlk{id: entry.id, ch}));
            });
        });

        (srcs, dsts)
    }

    fn compute_avail_stream_blk_pair(&self, tx_entries: &[FormatEntryData], rx_entries: &[FormatEntryData])
        -> (Vec<u8>, Vec<u8>)
    {
        let dst_blk_list = tx_entries.iter()
            .zip([DstBlkId::Avs0, DstBlkId::Avs1].iter())
            .map(|(&data, &id)| {
                let entry = FormatEntry::try_from(data).unwrap();
                (0..entry.pcm_count).map(move |ch| u8::from(DstBlk{id, ch}))
            }).flatten()
            .collect();

        let src_blk_list = rx_entries.iter()
            .zip([SrcBlkId::Avs0, SrcBlkId::Avs1].iter())
            .map(|(&data, &id)| {
                let entry = FormatEntry::try_from(data).unwrap();
                (0..entry.pcm_count).map(move |ch| u8::from(SrcBlk{id, ch}))
            }).flatten()
            .collect();

        (src_blk_list, dst_blk_list)
    }

    fn compute_avail_mixer_blk_pair(&self, caps: &ExtensionCaps, rate_mode: RateMode)
        -> (Vec<u8>, Vec<u8>)
    {
        let port_count = std::cmp::min(caps.mixer.output_count,
                                       Self::get_mixer_out_port_count(rate_mode));

        let id = SrcBlkId::Mixer;
        let src_blk_list = (0..port_count).map(move |ch| u8::from(SrcBlk{id, ch})).collect();

        let dst_blk_list = Self::MIXER_IN_PORTS.iter()
            .flat_map(|&(id, count)| (0..count).map(move |ch| u8::from(DstBlk{id, ch})))
            .take(caps.mixer.input_count as usize)
            .collect();

        (src_blk_list, dst_blk_list)
    }

    fn get_src_blk_label(&self, src_blk_data: u8) -> String {
        let src_blk = SrcBlk::from(src_blk_data);
        Self::INPUTS.iter()
            .find(|entry| {
                entry.id == src_blk.id &&
                src_blk.ch >= entry.offset && src_blk.ch < entry.offset + entry.count &&
                entry.label.is_some()
            })
            .map(|entry| format!("{}-{}", entry.label.unwrap(), src_blk.ch - entry.offset))
            .unwrap_or_else(|| {
                let name = match src_blk.id {
                    SrcBlkId::Aes => "S/PDIF",
                    SrcBlkId::Adat => "ADAT",
                    SrcBlkId::Mixer => "Mixer",
                    SrcBlkId::Ins0 => "Analog-A",
                    SrcBlkId::Ins1 => "Analog-B",
                    SrcBlkId::Avs0 => "Stream-A",
                    SrcBlkId::Avs1 => "Stream-B",
                    _ => "Unknown",
                };
                format!("{}-{}", name, src_blk.ch)
            })
    }

    fn get_dst_blk_label(&self, dst_blk_data: u8) -> String {
        let dst_blk = DstBlk::from(dst_blk_data);
        Self::OUTPUTS.iter()
            .find(|entry| {
                entry.id == dst_blk.id &&
                dst_blk.ch >= entry.offset && dst_blk.ch < entry.offset + entry.count &&
                entry.label.is_some()
            })
            .map(|entry| format!("{}-{}", entry.label.unwrap(), dst_blk.ch - entry.offset))
            .unwrap_or_else(|| {
                let name = match dst_blk.id {
                    DstBlkId::Aes => "S/PDIF",
                    DstBlkId::Adat => "ADAT",
                    DstBlkId::MixerTx0 => "Mixer-A",
                    DstBlkId::MixerTx1 => "Mixer-B",
                    DstBlkId::Ins0 => "Analog-A",
                    DstBlkId::Ins1 => "Analog-B",
                    DstBlkId::Avs0 => "Stream-A",
                    DstBlkId::Avs1 => "Stream-B",
                    _ => "Unknown",
                };
                format!("{}-{}", name, dst_blk.ch)
            })
    }
}
