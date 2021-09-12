// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use super::{*, caps_section::*};

impl From<u8> for DstBlkId {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::Aes,
            1 => Self::Adat,
            2 => Self::MixerTx0,
            3 => Self::MixerTx1,
            4 => Self::Ins0,
            5 => Self::Ins1,
            10 => Self::ArmApbAudio,
            11 => Self::Avs0,
            12 => Self::Avs1,
            _ => Self::Reserved(val),
        }
    }
}

impl From<DstBlkId> for u8 {
    fn from(id: DstBlkId) -> Self {
        match id {
            DstBlkId::Aes => 0,
            DstBlkId::Adat => 1,
            DstBlkId::MixerTx0 => 2,
            DstBlkId::MixerTx1 => 3,
            DstBlkId::Ins0 => 4,
            DstBlkId::Ins1 => 5,
            DstBlkId::ArmApbAudio => 10,
            DstBlkId::Avs0 => 11,
            DstBlkId::Avs1 => 12,
            DstBlkId::Reserved(val) => val,
        }
    }
}

impl DstBlk {
    const ID_MASK: u8 = 0xf0;
    const ID_SHIFT: usize = 4;
    const CH_MASK: u8 = 0x0f;
    const CH_SHIFT: usize = 0;
}

impl From<u8> for DstBlk {
    fn from(val: u8) -> Self {
        DstBlk {
            id: DstBlkId::from((val & DstBlk::ID_MASK) >> DstBlk::ID_SHIFT),
            ch: (val & DstBlk::CH_MASK) >> DstBlk::CH_SHIFT,
        }
    }
}

impl From<DstBlk> for u8 {
    fn from(blk: DstBlk) -> Self {
        (u8::from(blk.id) << DstBlk::ID_SHIFT) | blk.ch
    }
}

impl Ord for DstBlk {
    fn cmp(&self, other: &Self) -> Ordering {
        u8::from(*self).cmp(&u8::from(*other))
    }
}

impl PartialOrd for DstBlk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(u8::from(*self).cmp(&u8::from(*other)))
    }
}

impl From<u8> for SrcBlkId {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::Aes,
            1 => Self::Adat,
            2 => Self::Mixer,
            4 => Self::Ins0,
            5 => Self::Ins1,
            10 => Self::ArmAprAudio,
            11 => Self::Avs0,
            12 => Self::Avs1,
            15 => Self::Mute,
            _ => Self::Reserved(val),
        }
    }
}

impl From<SrcBlkId> for u8 {
    fn from(id: SrcBlkId) -> Self {
        match id {
            SrcBlkId::Aes => 0,
            SrcBlkId::Adat => 1,
            SrcBlkId::Mixer => 2,
            SrcBlkId::Ins0 => 4,
            SrcBlkId::Ins1 => 5,
            SrcBlkId::ArmAprAudio => 10,
            SrcBlkId::Avs0 => 11,
            SrcBlkId::Avs1 => 12,
            SrcBlkId::Mute => 15,
            SrcBlkId::Reserved(val) => val,
        }
    }
}

impl SrcBlk {
    const ID_MASK: u8 = 0xf0;
    const ID_SHIFT: usize = 4;
    const CH_MASK: u8 = 0x0f;
    const CH_SHIFT: usize = 0;
}

impl From<u8> for SrcBlk {
    fn from(val: u8) -> Self {
        SrcBlk {
            id: SrcBlkId::from((val & SrcBlk::ID_MASK) >> SrcBlk::ID_SHIFT),
            ch: (val & SrcBlk::CH_MASK) >> SrcBlk::CH_SHIFT,
        }
    }
}

impl From<SrcBlk> for u8 {
    fn from(blk: SrcBlk) -> Self {
        (u8::from(blk.id) << SrcBlk::ID_SHIFT) | blk.ch
    }
}

impl Ord for SrcBlk {
    fn cmp(&self, other: &Self) -> Ordering {
        u8::from(*self).cmp(&u8::from(*other))
    }
}

impl PartialOrd for SrcBlk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(u8::from(*self).cmp(&u8::from(*other)))
    }
}

impl RouterEntry {
    const SIZE: usize = 4;

    const SRC_OFFSET: usize = 2;
    const DST_OFFSET: usize = 3;

    fn build(&self, raw: &mut [u8]) {
        raw[Self::DST_OFFSET] = u8::from(self.dst);
        raw[Self::SRC_OFFSET] = u8::from(self.src);
        raw[..Self::SRC_OFFSET].copy_from_slice(&self.peak.to_be_bytes());
    }

    fn parse(&mut self, raw: &[u8]) {
        self.dst = raw[Self::DST_OFFSET].into();
        self.src = raw[Self::SRC_OFFSET].into();
        let mut doublet = [0;2];
        doublet.copy_from_slice(&raw[..Self::SRC_OFFSET]);
        self.peak = u16::from_be_bytes(doublet);
    }
}

pub trait RouterEntryProtocol: ProtocolExtension {
    fn read_router_entries(
        &self,
        node: &mut FwNode,
        caps: &ExtensionCaps,
        offset: usize,
        entry_count: usize,
        timeout_ms: u32
    ) -> Result<Vec<RouterEntry>, Error> {
        if entry_count > caps.router.maximum_entry_count as usize {
            let msg = format!("Invalid entries to read: {} but greater than {}",
                              entry_count, caps.router.maximum_entry_count);
            Err(Error::new(ProtocolExtensionError::RouterEntry, &msg))?
        }

        let mut raw = vec![0;entry_count * RouterEntry::SIZE];
        ProtocolExtension::read(self, node, offset, &mut raw, timeout_ms)
            .map(|_| {
                let mut entries = vec![RouterEntry::default();entry_count];
                entries.iter_mut()
                    .enumerate()
                    .for_each(|(i, entry)| {
                        let pos = i * 4;
                        entry.parse(&raw[pos..(pos + 4)]);
                    });
                entries
            })
    }

    fn write_router_entries(
        &self,
        node: &mut FwNode,
        caps: &ExtensionCaps,
        offset: usize,
        entries: &[RouterEntry],
        timeout_ms: u32
    ) -> Result<(), Error> {
        if entries.len() > caps.router.maximum_entry_count as usize {
            let msg = format!("Invalid number of entries to read: {} but greater than {}",
                              entries.len(), caps.router.maximum_entry_count * 4);
            Err(Error::new(ProtocolExtensionError::RouterEntry, &msg))?
        }

        let mut data = [0;4];
        data.copy_from_slice(&(entries.len() as u32).to_be_bytes());
        ProtocolExtension::write(self, node, offset, &mut data, timeout_ms)?;

        let mut raw = vec![0;entries.len() * RouterEntry::SIZE];
        entries.iter()
            .enumerate()
            .for_each(|(i, entry)| {
                let pos = i * 4;
                entry.build(&mut raw[pos..(pos + 4)]);
            });
        ProtocolExtension::write(self, node, offset + 4, &mut raw, timeout_ms)
    }
}

impl<O: AsRef<FwReq>> RouterEntryProtocol for O {}

#[cfg(test)]
mod test {
    use super::{DstBlk, SrcBlk, DstBlkId, SrcBlkId};

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
