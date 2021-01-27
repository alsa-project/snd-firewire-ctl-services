// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use super::{*, caps_section::*};

pub trait RouterEntryProtocol<T> : ProtocolExtension<T>
    where T: AsRef<FwNode>,
{
    fn read_router_entries(&self, node: &T, caps: &ExtensionCaps, offset: usize,
                           entry_count: usize, timeout_ms: u32)
        -> Result<Vec<RouterEntry>, Error>
    {
        if entry_count > caps.router.maximum_entry_count as usize {
            let msg = format!("Invalid entries to read: {} but greater than {}",
                              entry_count, caps.router.maximum_entry_count);
            Err(Error::new(ProtocolExtensionError::RouterEntry, &msg))?
        }

        let mut data = vec![0;entry_count * RouterEntry::SIZE];
        ProtocolExtension::read(self, node, offset, &mut data, timeout_ms)?;

        let entries = (0..entry_count)
            .map(|i| {
                let mut raw = RouterEntryData::default();
                let pos = i * RouterEntry::SIZE;
                raw.copy_from_slice(&data[pos..(pos + RouterEntry::SIZE)]);
                RouterEntry::from(&raw)
            })
            .collect::<Vec<_>>();

        Ok(entries)
    }

    fn write_router_entries(&self, node: &T, caps: &ExtensionCaps, offset: usize,
                            entries: &[RouterEntry], timeout_ms: u32)
        -> Result<(), Error>
    {
        if entries.len() > caps.router.maximum_entry_count as usize {
            let msg = format!("Invalid number of entries to read: {} but greater than {}",
                              entries.len(), caps.router.maximum_entry_count * 4);
            Err(Error::new(ProtocolExtensionError::RouterEntry, &msg))?
        }

        let mut data = [0;4];
        data.copy_from_slice(&(entries.len() as u32).to_be_bytes());
        ProtocolExtension::write(self, node, offset, &mut data, timeout_ms)?;

        let mut data = Vec::new();
        entries.iter().for_each(|entry| {
            let raw = RouterEntryData::from(entry);
            data.extend_from_slice(&raw);
        });
        ProtocolExtension::write(self, node, offset + 4, &mut data, timeout_ms)
    }
}

impl<O: AsRef<FwReq>, T: AsRef<FwNode>> RouterEntryProtocol<T> for O {}
