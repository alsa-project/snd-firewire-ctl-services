// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::*;

impl DstBlk {
    const ID_MASK: u8 = 0xf0;
    const ID_SHIFT: usize = 4;
    const CH_MASK: u8 = 0x0f;
    const CH_SHIFT: usize = 0;

    const AES_VALUE: u8 = 0;
    const ADAT_VALUE: u8 = 1;
    const MIXER_TX0_VALUE: u8 = 2;
    const MIXER_TX1_VALUE: u8 = 3;
    const INS0_VALUE: u8 = 4;
    const INS1_VALUE: u8 = 5;
    const ARM_APB_AUDIO_VALUE: u8 = 10;
    const AVS0_VALUE: u8 = 11;
    const AVS1_VALUE: u8 = 12;
}

fn serialize_dst_blk(dst: &DstBlk, val: &mut u8) -> Result<(), String> {
    let id = match dst.id {
        DstBlkId::Aes => DstBlk::AES_VALUE,
        DstBlkId::Adat => DstBlk::ADAT_VALUE,
        DstBlkId::MixerTx0 => DstBlk::MIXER_TX0_VALUE,
        DstBlkId::MixerTx1 => DstBlk::MIXER_TX1_VALUE,
        DstBlkId::Ins0 => DstBlk::INS0_VALUE,
        DstBlkId::Ins1 => DstBlk::INS1_VALUE,
        DstBlkId::ArmApbAudio => DstBlk::ARM_APB_AUDIO_VALUE,
        DstBlkId::Avs0 => DstBlk::AVS0_VALUE,
        DstBlkId::Avs1 => DstBlk::AVS1_VALUE,
        DstBlkId::Reserved(id) => id,
    };

    let ch = dst.ch;

    *val =
        ((id << DstBlk::ID_SHIFT) & DstBlk::ID_MASK) | ((ch << DstBlk::CH_SHIFT) & DstBlk::CH_MASK);

    Ok(())
}

fn deserialize_dst_blk(dst: &mut DstBlk, val: &u8) -> Result<(), String> {
    let id = (*val & DstBlk::ID_MASK) >> DstBlk::ID_SHIFT;
    dst.id = match id {
        DstBlk::AES_VALUE => DstBlkId::Aes,
        DstBlk::ADAT_VALUE => DstBlkId::Adat,
        DstBlk::MIXER_TX0_VALUE => DstBlkId::MixerTx0,
        DstBlk::MIXER_TX1_VALUE => DstBlkId::MixerTx1,
        DstBlk::INS0_VALUE => DstBlkId::Ins0,
        DstBlk::INS1_VALUE => DstBlkId::Ins1,
        DstBlk::ARM_APB_AUDIO_VALUE => DstBlkId::ArmApbAudio,
        DstBlk::AVS0_VALUE => DstBlkId::Avs0,
        DstBlk::AVS1_VALUE => DstBlkId::Avs1,
        _ => DstBlkId::Reserved(id),
    };

    dst.ch = (*val & DstBlk::CH_MASK) >> DstBlk::CH_SHIFT;

    Ok(())
}

impl From<u8> for DstBlk {
    fn from(val: u8) -> Self {
        let mut dst = DstBlk::default();
        deserialize_dst_blk(&mut dst, &val).unwrap();
        dst
    }
}

impl From<DstBlk> for u8 {
    fn from(blk: DstBlk) -> Self {
        let mut val = 0u8;
        serialize_dst_blk(&blk, &mut val).unwrap();
        val
    }
}

impl Ord for DstBlk {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut lval = 0u8;
        serialize_dst_blk(self, &mut lval).unwrap();

        let mut rval = 0u8;
        serialize_dst_blk(other, &mut rval).unwrap();

        lval.cmp(&rval)
    }
}

impl PartialOrd for DstBlk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl SrcBlk {
    const ID_MASK: u8 = 0xf0;
    const ID_SHIFT: usize = 4;
    const CH_MASK: u8 = 0x0f;
    const CH_SHIFT: usize = 0;

    const AES_VALUE: u8 = 0;
    const ADAT_VALUE: u8 = 1;
    const MIXER_VALUE: u8 = 2;
    const INS0_VALUE: u8 = 4;
    const INS1_VALUE: u8 = 5;
    const ARMAPRAUDIO_VALUE: u8 = 10;
    const AVS0_VALUE: u8 = 11;
    const AVS1_VALUE: u8 = 12;
    const MUTE_VALUE: u8 = 15;
}

fn serialize_src_blk(src: &SrcBlk, val: &mut u8) -> Result<(), String> {
    let id = match src.id {
        SrcBlkId::Aes => SrcBlk::AES_VALUE,
        SrcBlkId::Adat => SrcBlk::ADAT_VALUE,
        SrcBlkId::Mixer => SrcBlk::MIXER_VALUE,
        SrcBlkId::Ins0 => SrcBlk::INS0_VALUE,
        SrcBlkId::Ins1 => SrcBlk::INS1_VALUE,
        SrcBlkId::ArmAprAudio => SrcBlk::ARMAPRAUDIO_VALUE,
        SrcBlkId::Avs0 => SrcBlk::AVS0_VALUE,
        SrcBlkId::Avs1 => SrcBlk::AVS1_VALUE,
        SrcBlkId::Mute => SrcBlk::MUTE_VALUE,
        SrcBlkId::Reserved(id) => id,
    };

    let ch = src.ch;

    *val =
        ((id << SrcBlk::ID_SHIFT) & SrcBlk::ID_MASK) | ((ch << SrcBlk::CH_SHIFT) & SrcBlk::CH_MASK);

    Ok(())
}

fn deserialize_src_blk(src: &mut SrcBlk, val: &u8) -> Result<(), String> {
    let id = (*val & SrcBlk::ID_MASK) >> SrcBlk::ID_SHIFT;
    src.id = match id {
        SrcBlk::AES_VALUE => SrcBlkId::Aes,
        SrcBlk::ADAT_VALUE => SrcBlkId::Adat,
        SrcBlk::MIXER_VALUE => SrcBlkId::Mixer,
        SrcBlk::INS0_VALUE => SrcBlkId::Ins0,
        SrcBlk::INS1_VALUE => SrcBlkId::Ins1,
        SrcBlk::ARMAPRAUDIO_VALUE => SrcBlkId::ArmAprAudio,
        SrcBlk::AVS0_VALUE => SrcBlkId::Avs0,
        SrcBlk::AVS1_VALUE => SrcBlkId::Avs1,
        SrcBlk::MUTE_VALUE => SrcBlkId::Mute,
        _ => SrcBlkId::Reserved(id),
    };

    src.ch = (*val & SrcBlk::CH_MASK) >> SrcBlk::CH_SHIFT;

    Ok(())
}

impl From<u8> for SrcBlk {
    fn from(val: u8) -> Self {
        let mut src = SrcBlk::default();
        deserialize_src_blk(&mut src, &val).unwrap();
        src
    }
}

impl From<SrcBlk> for u8 {
    fn from(src: SrcBlk) -> Self {
        let mut val = 0u8;
        serialize_src_blk(&src, &mut val).unwrap();
        val
    }
}

impl Ord for SrcBlk {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut lval = 0u8;
        serialize_src_blk(self, &mut lval).unwrap();

        let mut rval = 0u8;
        serialize_src_blk(other, &mut rval).unwrap();

        lval.cmp(&rval)
    }
}

impl PartialOrd for SrcBlk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl RouterEntry {
    const SIZE: usize = 4;

    const DST_MASK: u32 = 0x000000ff;
    const DST_SHIFT: usize = 0;

    const SRC_MASK: u32 = 0x0000ff00;
    const SRC_SHIFT: usize = 8;

    const PEAK_MASK: u32 = 0xffff0000;
    const PEAK_SHIFT: usize = 16;
}

fn serialize_router_entry(entry: &RouterEntry, raw: &mut [u8]) -> Result<(), String> {
    assert!(raw.len() >= RouterEntry::SIZE);

    let mut dst_val = 0u8;
    serialize_dst_blk(&entry.dst, &mut dst_val)?;

    let mut src_val = 0u8;
    serialize_src_blk(&entry.src, &mut src_val)?;

    let val = (((entry.peak as u32) << RouterEntry::PEAK_SHIFT) & RouterEntry::PEAK_MASK)
        | (((src_val as u32) << RouterEntry::SRC_SHIFT) & RouterEntry::SRC_MASK)
        | (((dst_val as u32) << RouterEntry::DST_SHIFT) & RouterEntry::DST_MASK);
    val.build_quadlet(raw);

    Ok(())
}

fn deserialize_router_entry(entry: &mut RouterEntry, raw: &[u8]) -> Result<(), String> {
    assert!(raw.len() >= RouterEntry::SIZE);

    let mut val = 0u32;
    val.parse_quadlet(raw);

    let dst_val = ((val & RouterEntry::DST_MASK) >> RouterEntry::DST_SHIFT) as u8;
    deserialize_dst_blk(&mut entry.dst, &dst_val)?;

    let src_val = ((val & RouterEntry::SRC_MASK) >> RouterEntry::SRC_SHIFT) as u8;
    deserialize_src_blk(&mut entry.src, &src_val)?;

    entry.peak = ((val & RouterEntry::PEAK_MASK) >> RouterEntry::PEAK_SHIFT) as u16;

    Ok(())
}

pub(crate) fn calculate_router_entries_size(entry_count: usize) -> usize {
    entry_count * RouterEntry::SIZE
}

pub(crate) fn serialize_router_entries(
    entries: &[RouterEntry],
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= calculate_router_entries_size(entries.len()));

    entries.iter().enumerate().try_for_each(|(i, entry)| {
        let pos = i * RouterEntry::SIZE;
        serialize_router_entry(entry, &mut raw[pos..(pos + RouterEntry::SIZE)])
    })
}

pub(crate) fn deserialize_router_entries(
    entries: &mut [RouterEntry],
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= calculate_router_entries_size(entries.len()));

    entries.iter_mut().enumerate().try_for_each(|(i, entry)| {
        let pos = i * RouterEntry::SIZE;
        deserialize_router_entry(entry, &raw[pos..(pos + RouterEntry::SIZE)])
    })
}

#[cfg(test)]
mod test {
    use super::{DstBlk, DstBlkId, SrcBlk, SrcBlkId};

    #[test]
    fn dst_blk_from() {
        let blk = DstBlk {
            id: DstBlkId::ArmApbAudio,
            ch: 0x04,
        };
        assert_eq!(blk, DstBlk::from(u8::from(blk)));
    }

    #[test]
    fn src_blk_from() {
        let blk = SrcBlk {
            id: SrcBlkId::ArmAprAudio,
            ch: 0x04,
        };
        assert_eq!(blk, SrcBlk::from(u8::from(blk)));
    }
}
