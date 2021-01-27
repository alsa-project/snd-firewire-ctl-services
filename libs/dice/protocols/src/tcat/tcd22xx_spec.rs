// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::FwNode;

use super::extension::{*, caps_section::*, cmd_section::*, mixer_section::*, router_section::*,
                       current_config_section::*};

use std::convert::TryFrom;

#[derive(Default, Debug)]
pub struct Tcd22xxState {
    pub router_entries: Vec<RouterEntryData>,
    pub mixer_cache: Vec<Vec<i32>>,

    rate_mode: RateMode,
    real_blk_pair: (Vec<SrcBlk>, Vec<DstBlk>),
    stream_blk_pair: (Vec<SrcBlk>, Vec<DstBlk>),
    mixer_blk_pair: (Vec<SrcBlk>, Vec<DstBlk>),
}

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

    fn compute_avail_real_blk_pair(&self, rate_mode: RateMode) -> (Vec<SrcBlk>, Vec<DstBlk>)
    {
        let mut srcs = Vec::<SrcBlk>::new();
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
                srcs.push(SrcBlk{id: entry.id, ch});
            });
        });

        let mut dsts = Vec::<DstBlk>::new();
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
                dsts.push(DstBlk{id: entry.id, ch});
            });
        });

        (srcs, dsts)
    }

    fn compute_avail_stream_blk_pair(&self, tx_entries: &[FormatEntryData], rx_entries: &[FormatEntryData])
        -> (Vec<SrcBlk>, Vec<DstBlk>)
    {
        let dst_blk_list = tx_entries.iter()
            .zip([DstBlkId::Avs0, DstBlkId::Avs1].iter())
            .map(|(&data, &id)| {
                let entry = FormatEntry::try_from(data).unwrap();
                (0..entry.pcm_count).map(move |ch| DstBlk{id, ch})
            }).flatten()
            .collect();

        let src_blk_list = rx_entries.iter()
            .zip([SrcBlkId::Avs0, SrcBlkId::Avs1].iter())
            .map(|(&data, &id)| {
                let entry = FormatEntry::try_from(data).unwrap();
                (0..entry.pcm_count).map(move |ch| SrcBlk{id, ch})
            }).flatten()
            .collect();

        (src_blk_list, dst_blk_list)
    }

    fn compute_avail_mixer_blk_pair(&self, caps: &ExtensionCaps, rate_mode: RateMode)
        -> (Vec<SrcBlk>, Vec<DstBlk>)
    {
        let port_count = std::cmp::min(caps.mixer.output_count,
                                       Self::get_mixer_out_port_count(rate_mode));

        let id = SrcBlkId::Mixer;
        let src_blk_list = (0..port_count).map(move |ch| SrcBlk{id, ch}).collect();

        let dst_blk_list = Self::MIXER_IN_PORTS.iter()
            .flat_map(|&(id, count)| (0..count).map(move |ch| DstBlk{id, ch}))
            .take(caps.mixer.input_count as usize)
            .collect();

        (src_blk_list, dst_blk_list)
    }

    fn get_src_blk_label(&self, src_blk: &SrcBlk) -> String {
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

    fn get_dst_blk_label(&self, dst_blk: DstBlk) -> String {
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

pub trait Tcd22xxRouterOperation<'a, T, U> : Tcd22xxSpec<'a> + AsRef<Tcd22xxState> + AsMut<Tcd22xxState>
    where T: AsRef<FwNode>,
          U: CmdSectionProtocol<T> + RouterSectionProtocol<T> + CurrentConfigSectionProtocol<T>,
{
    fn update_router_entries(&mut self, node: &T, proto: &U, sections: &ExtensionSections,
                             caps: &ExtensionCaps, entries: Vec<RouterEntryData>, timeout_ms: u32)
        -> Result<(), Error>
    {
        if entries.len() > caps.router.maximum_entry_count as usize {
            let msg = format!("The number of entries for router section should be less than {} but {}",
                              caps.router.maximum_entry_count, entries.len());
            Err(Error::new(FileError::Inval, &msg))?
        }

        let state = self.as_mut();
        if entries != state.router_entries {
            let rate_mode = state.rate_mode;
            proto.write_router_entries(node, sections, caps, &entries, timeout_ms)?;
            proto.initiate(node, sections, caps, Opcode::LoadRouter(rate_mode), timeout_ms)?;
            state.router_entries = entries;
        }

        Ok(())
    }

    fn cache_router_entries(&mut self, node: &T, proto: &U, sections: &ExtensionSections,
                            caps: &ExtensionCaps, timeout_ms: u32)
        -> Result<(), Error>
    {
        let rate_mode = self.as_ref().rate_mode;

        let real_blk_pair = self.compute_avail_real_blk_pair(rate_mode);

        let (tx_entries, rx_entries) = proto.read_current_stream_format_entries(node, sections, caps,
                                                                                rate_mode, timeout_ms)?;
        let stream_blk_pair = self.compute_avail_stream_blk_pair(&tx_entries, &rx_entries);

        let mixer_blk_pair = self.compute_avail_mixer_blk_pair(caps, rate_mode);

        let state = self.as_mut();
        state.real_blk_pair = real_blk_pair;
        state.stream_blk_pair = stream_blk_pair;
        state.mixer_blk_pair = mixer_blk_pair;

        let entries = proto.read_current_router_entries(node, sections, caps, rate_mode, timeout_ms)?;
        self.update_router_entries(node, proto, sections, caps, entries, timeout_ms)
    }
}

pub trait Tcd22xxMixerOperation<'a, T, U> : Tcd22xxSpec<'a> + AsRef<Tcd22xxState> + AsMut<Tcd22xxState>
    where T: AsRef<FwNode>,
          U: MixerSectionProtocol<T>,
{
    fn update_mixer_coef(&mut self, node: &T, proto: &U, sections: &ExtensionSections,
                         caps: &ExtensionCaps, entries: &[Vec<i32>], timeout_ms: u32)
        -> Result<(), Error>
    {
        let cache = &mut self.as_mut().mixer_cache;

        (0..cache.len()).take(entries.len()).try_for_each(|dst_ch| {
            (0..cache[dst_ch].len()).take(entries[dst_ch].len()).try_for_each(|src_ch| {
                let coef = entries[dst_ch][src_ch];
                if cache[dst_ch][src_ch] != coef {
                    proto.write_coef(node, sections, caps, dst_ch, src_ch, coef as u32, timeout_ms)?;
                    cache[dst_ch][src_ch] = coef;
                }
                Ok(())
            })
        })
    }

    fn cache_mixer_coefs(&mut self, node: &T, proto: &U, sections: &ExtensionSections,
                         caps: &ExtensionCaps, timeout_ms: u32)
        -> Result<(), Error>
    {
        let rate_mode = self.as_ref().rate_mode;

        let output_count = Self::get_mixer_out_port_count(rate_mode);
        let input_count = Self::get_mixer_in_port_count();

        self.as_mut().mixer_cache = Vec::new();
        (0..output_count as usize).try_for_each(|dst_ch| {
            let mut entry = Vec::new();
            (0..input_count as usize).try_for_each(|src_ch| {
                let coef = proto.read_coef(node, sections, caps, dst_ch, src_ch, timeout_ms)?;
                entry.push(coef as i32);
                Ok(())
            })?;
            self.as_mut().mixer_cache.push(entry);
            Ok(())
        })
    }
}

pub trait Tcd22xxStateOperation<'a, T, U> : Tcd22xxSpec<'a> + AsRef<Tcd22xxState> + AsMut<Tcd22xxState> +
                                            Tcd22xxRouterOperation<'a, T, U> +  Tcd22xxMixerOperation<'a, T, U>
    where T: AsRef<FwNode>,
          U: CmdSectionProtocol<T> + MixerSectionProtocol<T> + RouterSectionProtocol<T> +
             CurrentConfigSectionProtocol<T>,
{
    fn cache(&mut self, node: &T, proto: &U, sections: &ExtensionSections, caps: &ExtensionCaps,
             rate_mode: RateMode, timeout_ms: u32)
        -> Result<(), Error>
    {
        self.as_mut().rate_mode = rate_mode;
        self.cache_router_entries(node, proto, sections, caps, timeout_ms)?;
        self.cache_mixer_coefs(node, proto, sections, caps, timeout_ms)?;
        Ok(())
    }
}

impl<'a, O, T, U> Tcd22xxRouterOperation<'a, T, U> for O
    where O: Tcd22xxSpec<'a> + AsRef<Tcd22xxState> + AsMut<Tcd22xxState>,
          T: AsRef<FwNode>,
          U: CmdSectionProtocol<T> + RouterSectionProtocol<T> + CurrentConfigSectionProtocol<T>,
{}

impl<'a, O, T, U> Tcd22xxMixerOperation<'a, T, U> for O
    where O: Tcd22xxSpec<'a> + AsRef<Tcd22xxState> + AsMut<Tcd22xxState>,
          T: AsRef<FwNode>,
          U: MixerSectionProtocol<T>,
{}

impl<'a, O, T, U> Tcd22xxStateOperation<'a, T, U> for O
    where O: Tcd22xxSpec<'a> + AsRef<Tcd22xxState> + AsMut<Tcd22xxState> +
             Tcd22xxRouterOperation<'a, T, U> +  Tcd22xxMixerOperation<'a, T, U>,
          T: AsRef<FwNode>,
          U: CmdSectionProtocol<T> + MixerSectionProtocol<T> + RouterSectionProtocol<T> +
             CurrentConfigSectionProtocol<T>,
{}
