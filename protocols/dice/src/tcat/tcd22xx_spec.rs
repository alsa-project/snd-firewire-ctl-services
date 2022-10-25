// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Specification of TCD2210 and TCD2220 ASICs and firmware.

use super::{
    extension::{caps_section::*, cmd_section::*, current_config_section::*, router_section::*, *},
    *,
};

/// Available source and destination blocks of TCD22xx.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Tcd22xxAvailableBlocks(pub Vec<SrcBlk>, pub Vec<DstBlk>);

/// Descriptor for input port.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Input {
    /// Identifier of source block.
    pub id: SrcBlkId,
    /// Offset of channel number.
    pub offset: u8,
    /// Count of channel number.
    pub count: u8,
    /// String expression.
    pub label: Option<&'static str>,
}

/// Descriptor for output port.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Output {
    /// Identifier of destination block.
    pub id: DstBlkId,
    /// Offset of channel number.
    pub offset: u8,
    /// Count of channel number.
    pub count: u8,
    /// String expression.
    pub label: Option<&'static str>,
}

/// Specification of TCD22xx.
pub trait Tcd22xxSpecification {
    /// Physical input ports.
    const INPUTS: &'static [Input];

    /// Physical output ports.
    const OUTPUTS: &'static [Output];

    /// Ports with fixed position in router entries; e.g. target ports for meter display.
    const FIXED: &'static [SrcBlk];

    /// The number of mixer outputs at specification of TCD22xx.
    const MIXER_OUT_PORTS: [u8; 3] = [16, 16, 8];

    /// The number of mixer inputs at specification of TCD22xx.
    const MIXER_IN_PORTS: [(DstBlkId, u8); 2] = [(DstBlkId::MixerTx0, 16), (DstBlkId::MixerTx1, 2)];

    /// The number of ADAT channels at specification of ADAT/SMUX.
    const ADAT_CHANNELS: [u8; 3] = [8, 4, 2];

    /// Compute the number of ADAT channels.
    fn adat_channel_count(rate_mode: RateMode) -> u8 {
        let index = match rate_mode {
            RateMode::Low => 0,
            RateMode::Middle => 1,
            RateMode::High => 2,
        };
        Self::ADAT_CHANNELS[index]
    }

    /// Compute the number of mixer outputs.
    fn mixer_out_port_count(rate_mode: RateMode) -> u8 {
        let index = match rate_mode {
            RateMode::Low => 0,
            RateMode::Middle => 1,
            RateMode::High => 2,
        };
        Self::MIXER_OUT_PORTS[index]
    }

    /// Compute the number of mixer inputs.
    fn mixer_in_port_count() -> u8 {
        Self::MIXER_IN_PORTS
            .iter()
            .fold(0, |accum, (_, count)| accum + count)
    }

    /// Compute available destination and source blocks for physical ports.
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

    /// Compute available destination and source blocks for Tx/Rx streams.
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

    /// Compute available destination and source blocks for mixer inputs and outputs.
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

    /// Refine router entries by defined descriptors.
    fn refine_router_entries(
        entries: &mut Vec<RouterEntry>,
        avail_blocks: &Tcd22xxAvailableBlocks,
    ) {
        entries.retain(|entry| {
            avail_blocks
                .0
                .iter()
                .find(|src| entry.src.eq(src))
                .is_some()
        });
        entries.retain(|entry| {
            avail_blocks
                .1
                .iter()
                .find(|dst| entry.dst.eq(dst))
                .is_some()
        });
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
    }
}

/// Operation specific to TCD22xx.
pub trait Tcd22xxOperation: Tcd22xxSpecification {
    /// Detect available source and destination blocks at given rate mode.
    fn detect_available_blocks(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        rate_mode: RateMode,
        avail_blocks: &mut Tcd22xxAvailableBlocks,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let real_blk_pair = Self::compute_avail_real_blk_pair(rate_mode);

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
        let stream_blk_pair = Self::compute_avail_stream_blk_pair(&tx_entries, &rx_entries);

        let mixer_blk_pair = Self::compute_avail_mixer_blk_pair(caps, rate_mode);

        avail_blocks.0 = real_blk_pair
            .0
            .iter()
            .chain(&stream_blk_pair.0)
            .chain(&mixer_blk_pair.0)
            .copied()
            .collect();

        avail_blocks.1 = real_blk_pair
            .1
            .iter()
            .chain(&stream_blk_pair.1)
            .chain(&mixer_blk_pair.1)
            .copied()
            .collect();

        Ok(())
    }

    /// Update router entries.
    fn update_router_entries(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        rate_mode: RateMode,
        avail_blocks: &Tcd22xxAvailableBlocks,
        entries: &mut Vec<RouterEntry>,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Self::refine_router_entries(entries, avail_blocks);
        if entries.len() > caps.router.maximum_entry_count as usize {
            let msg = format!(
                "The number of entries for router section should be less than {} but {}",
                caps.router.maximum_entry_count,
                entries.len()
            );
            Err(Error::new(FileError::Inval, &msg))?
        }

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

        Ok(())
    }

    /// Load configuration from on-board flash memory, including parameters in application section.
    fn load_configuration(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        CmdSectionProtocol::initiate(
            req,
            node,
            sections,
            caps,
            Opcode::LoadConfigFromFlash,
            timeout_ms,
        )
        .map(|_| ())
    }

    /// Store configuration to on-board flash memory, including parameters in application section.
    fn store_configuration(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        CmdSectionProtocol::initiate(
            req,
            node,
            sections,
            caps,
            Opcode::StoreConfigToFlash,
            timeout_ms,
        )
        .map(|_| ())
    }
}

impl<O: Tcd22xxSpecification> Tcd22xxOperation for O {}
