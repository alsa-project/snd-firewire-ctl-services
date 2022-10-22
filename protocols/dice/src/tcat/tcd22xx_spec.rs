// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::{
    extension::{caps_section::*, cmd_section::*, current_config_section::*, router_section::*, *},
    *,
};

#[derive(Default, Debug)]
pub struct Tcd22xxState {
    pub router_entries: Vec<RouterEntry>,

    real_blk_pair: (Vec<SrcBlk>, Vec<DstBlk>),
    stream_blk_pair: (Vec<SrcBlk>, Vec<DstBlk>),
    mixer_blk_pair: (Vec<SrcBlk>, Vec<DstBlk>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Input {
    pub id: SrcBlkId,
    pub offset: u8,
    pub count: u8,
    pub label: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Output {
    pub id: DstBlkId,
    pub offset: u8,
    pub count: u8,
    pub label: Option<&'static str>,
}

pub trait Tcd22xxSpecification {
    // For each model.
    const INPUTS: &'static [Input];
    const OUTPUTS: &'static [Output];
    const FIXED: &'static [SrcBlk];

    // From specification of TCD22xx.
    const MIXER_OUT_PORTS: [u8; 3] = [16, 16, 8];
    const MIXER_IN_PORTS: [(DstBlkId, u8); 2] = [(DstBlkId::MixerTx0, 16), (DstBlkId::MixerTx1, 2)];

    // From specification of ADAT/SMUX.
    const ADAT_CHANNELS: [u8; 3] = [8, 4, 2];

    fn adat_channel_count(rate_mode: RateMode) -> u8 {
        let index = match rate_mode {
            RateMode::Low => 0,
            RateMode::Middle => 1,
            RateMode::High => 2,
        };
        Self::ADAT_CHANNELS[index]
    }

    fn mixer_out_port_count(rate_mode: RateMode) -> u8 {
        let index = match rate_mode {
            RateMode::Low => 0,
            RateMode::Middle => 1,
            RateMode::High => 2,
        };
        Self::MIXER_OUT_PORTS[index]
    }

    fn mixer_in_port_count() -> u8 {
        Self::MIXER_IN_PORTS
            .iter()
            .fold(0, |accum, (_, count)| accum + count)
    }

    fn compute_avail_real_blk_pair(rate_mode: RateMode) -> (Vec<SrcBlk>, Vec<DstBlk>) {
        let mut srcs = Vec::<SrcBlk>::new();
        Self::INPUTS.iter().for_each(|entry| {
            let offset = match entry.id {
                SrcBlkId::Adat => srcs.iter().filter(|&s| s.id.eq(&entry.id)).count() as u8,
                _ => entry.offset,
            };
            let count = match entry.id {
                SrcBlkId::Adat => Self::adat_channel_count(rate_mode),
                _ => entry.count,
            };
            (offset..(offset + count)).for_each(|ch| {
                srcs.push(SrcBlk { id: entry.id, ch });
            });
        });

        let mut dsts = Vec::<DstBlk>::new();
        Self::OUTPUTS.iter().for_each(|entry| {
            let offset = match entry.id {
                DstBlkId::Adat => dsts.iter().filter(|d| d.id.eq(&entry.id)).count() as u8,
                _ => entry.offset,
            };
            let count = match entry.id {
                DstBlkId::Adat => Self::adat_channel_count(rate_mode),
                _ => entry.count,
            };
            (offset..(offset + count)).for_each(|ch| {
                dsts.push(DstBlk { id: entry.id, ch });
            });
        });

        (srcs, dsts)
    }

    fn compute_avail_stream_blk_pair(
        tx_entries: &[FormatEntry],
        rx_entries: &[FormatEntry],
    ) -> (Vec<SrcBlk>, Vec<DstBlk>) {
        let dst_blk_list = tx_entries
            .iter()
            .zip([DstBlkId::Avs0, DstBlkId::Avs1])
            .map(|(entry, id)| (0..entry.pcm_count).map(move |ch| DstBlk { id, ch }))
            .flatten()
            .collect();

        let src_blk_list = rx_entries
            .iter()
            .zip([SrcBlkId::Avs0, SrcBlkId::Avs1])
            .map(|(entry, id)| (0..entry.pcm_count).map(move |ch| SrcBlk { id, ch }))
            .flatten()
            .collect();

        (src_blk_list, dst_blk_list)
    }

    fn compute_avail_mixer_blk_pair(
        caps: &ExtensionCaps,
        rate_mode: RateMode,
    ) -> (Vec<SrcBlk>, Vec<DstBlk>) {
        let port_count = std::cmp::min(
            caps.mixer.output_count,
            Self::mixer_out_port_count(rate_mode),
        );

        let id = SrcBlkId::Mixer;
        let src_blk_list = (0..port_count).map(move |ch| SrcBlk { id, ch }).collect();

        let dst_blk_list = Self::MIXER_IN_PORTS
            .iter()
            .flat_map(|&(id, count)| (0..count).map(move |ch| DstBlk { id, ch }))
            .take(caps.mixer.input_count as usize)
            .collect();

        (src_blk_list, dst_blk_list)
    }

    fn src_blk_label(src_blk: &SrcBlk) -> String {
        Self::INPUTS
            .iter()
            .find(|entry| {
                entry.id == src_blk.id
                    && src_blk.ch >= entry.offset
                    && src_blk.ch < entry.offset + entry.count
                    && entry.label.is_some()
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

    fn dst_blk_label(dst_blk: DstBlk) -> String {
        Self::OUTPUTS
            .iter()
            .find(|entry| {
                entry.id == dst_blk.id
                    && dst_blk.ch >= entry.offset
                    && dst_blk.ch < entry.offset + entry.count
                    && entry.label.is_some()
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

    fn refine_router_entries(
        mut entries: Vec<RouterEntry>,
        srcs: &[&SrcBlk],
        dsts: &[&DstBlk],
    ) -> Vec<RouterEntry> {
        entries.retain(|entry| srcs.iter().find(|src| entry.src.eq(src)).is_some());
        entries.retain(|entry| dsts.iter().find(|dst| entry.dst.eq(dst)).is_some());
        Self::FIXED.iter().enumerate().for_each(|(i, &src)| {
            match entries.iter().position(|entry| entry.src.eq(&src)) {
                Some(pos) => entries.swap(i, pos),
                None => {
                    let dst = DstBlk {
                        id: DstBlkId::Reserved(0xff),
                        ch: 0xff,
                    };
                    entries.insert(
                        i,
                        RouterEntry {
                            dst,
                            src,
                            ..Default::default()
                        },
                    )
                }
            }
        });
        entries
    }
}

pub trait Tcd22xxOperation: Tcd22xxSpecification {
    fn update_router_entries(
        node: &mut FwNode,
        req: &mut FwReq,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        rate_mode: RateMode,
        state: &mut Tcd22xxState,
        entries: Vec<RouterEntry>,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let srcs: Vec<_> = state
            .real_blk_pair
            .0
            .iter()
            .chain(&state.stream_blk_pair.0)
            .chain(&state.mixer_blk_pair.0)
            .collect();
        let dsts: Vec<_> = state
            .real_blk_pair
            .1
            .iter()
            .chain(&state.stream_blk_pair.1)
            .chain(&state.mixer_blk_pair.1)
            .collect();

        let entries = Self::refine_router_entries(entries, &srcs, &dsts);
        if entries.len() > caps.router.maximum_entry_count as usize {
            let msg = format!(
                "The number of entries for router section should be less than {} but {}",
                caps.router.maximum_entry_count,
                entries.len()
            );
            Err(Error::new(FileError::Inval, &msg))?
        }

        if entries != state.router_entries {
            RouterSectionProtocol::write_router_whole_entries(
                req, node, sections, caps, &entries, timeout_ms,
            )?;
            CmdSectionProtocol::initiate(
                req,
                node,
                sections,
                caps,
                Opcode::LoadRouter(rate_mode),
                timeout_ms,
            )?;
            state.router_entries = entries;
        }

        Ok(())
    }

    fn cache_router_entries(
        node: &mut FwNode,
        req: &mut FwReq,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        rate_mode: RateMode,
        state: &mut Tcd22xxState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        state.real_blk_pair = Self::compute_avail_real_blk_pair(rate_mode);

        let mut tx_entries = Vec::with_capacity(caps.general.max_tx_streams as usize);
        let mut rx_entries = Vec::with_capacity(caps.general.max_rx_streams as usize);
        CurrentConfigSectionProtocol::cache_current_config_stream_format_entries(
            req,
            node,
            sections,
            caps,
            rate_mode,
            (&mut tx_entries, &mut rx_entries),
            timeout_ms,
        )?;
        state.stream_blk_pair = Self::compute_avail_stream_blk_pair(&tx_entries, &rx_entries);

        state.mixer_blk_pair = Self::compute_avail_mixer_blk_pair(caps, rate_mode);

        let mut entries = Vec::with_capacity(caps.router.maximum_entry_count as usize);
        CurrentConfigSectionProtocol::cache_current_config_router_entries(
            req,
            node,
            sections,
            caps,
            rate_mode,
            &mut entries,
            timeout_ms,
        )?;
        Self::update_router_entries(
            node, req, sections, caps, rate_mode, state, entries, timeout_ms,
        )
    }
}

impl<O: Tcd22xxSpecification> Tcd22xxOperation for O {}

pub trait Tcd22xxStateOperation: Tcd22xxOperation {
    fn cache(
        node: &mut FwNode,
        req: &mut FwReq,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        state: &mut Tcd22xxState,
        rate_mode: RateMode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::cache_router_entries(node, req, sections, caps, rate_mode, state, timeout_ms)?;
        Ok(())
    }
}

impl<O: Tcd22xxOperation> Tcd22xxStateOperation for O {}
