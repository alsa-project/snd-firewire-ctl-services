// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
#[derive(Debug)]
pub struct Entry<'a> {
    pub key: u8,
    pub data: EntryData<'a>,
}

#[derive(Debug)]
pub enum EntryData<'a> {
    Immediate(u32),
    CsrOffset(usize),
    Leaf(&'a [u8]),
    Directory(Vec<Entry<'a>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum KeyType {
    Root = 0x00, // For my convenience.
    Descriptor = 0x01,
    BusDependentInfo = 0x02,
    Vendor = 0x03,
    HardwareVersion = 0x04,
    Module = 0x07,
    NodeCapabilities = 0x0c,
    Eui64 = 0x0d,
    Unit = 0x11,
    SpecifierId = 0x12,
    Version = 0x13,
    DependentInfo = 0x14,
    UnitLocation = 0x15,
    Model = 0x17,
    Instance = 0x18,
    Keyword = 0x19,
    Feature = 0x1a,
    ModifiableDescriptor = 0x1f,
    DirectoryId = 0x20,
}

pub fn get_root_entry_list<'a>(mut data: &'a [u8]) -> Vec<Entry<'a>> {
    // Get block of bus information.
    let bus_info_length = 4 * data[0] as usize;
    let bus_info = &data[..(4 + bus_info_length)];
    data = &data[(4 + bus_info_length)..];

    // Get block of root directory.
    let doublet = [data[0], data[1]];
    let directory_length = 4 * u16::from_be_bytes(doublet) as usize;
    let root = &data[..(4 + directory_length)];
    data = &data[(4 + directory_length)..];

    get_directory_entry_list(root, data)
}

fn get_directory_entry_list<'a>(mut directory: &'a [u8], data: &'a [u8]) -> Vec<Entry<'a>> {
    let mut entries = Vec::new();

    directory = &directory[4..];
    while directory.len() > 0 {
        let entry_type = directory[0] >> 6;
        let key = directory[0] & 0x3f;
        let quadlet = [0, directory[1], directory[2], directory[3]];
        let value = u32::from_be_bytes(quadlet);

        let entry_data = match entry_type {
            0 => EntryData::Immediate(value),
            1 => {
                let offset = 0xfffff0000000 + (value as usize);
                EntryData::CsrOffset(offset)
            }
            2 => {
                let offset = (value as usize) * 4;
                if offset < directory.len() {
                    break;
                }
                let start_offset = offset - directory.len();
                if start_offset > data.len() {
                    break;
                }
                let doublet = [data[start_offset], data[start_offset + 1]];
                let length = (u16::from_be_bytes(doublet) as usize) * 4;
                if length < 8 {
                    break;
                }
                let end_offset = start_offset + 4 + length;
                if end_offset > data.len() {
                    break;
                }
                let leaf = &data[start_offset..end_offset];
                EntryData::Leaf(leaf)
            }
            3 => {
                let offset = (value as usize) * 4;
                if offset < directory.len() {
                    break;
                }
                let start_offset = offset - directory.len();
                if start_offset > data.len() {
                    break;
                }
                let doublet = [data[start_offset], data[start_offset + 1]];
                let length = (u16::from_be_bytes(doublet) as usize) * 4;
                if length == 0 {
                    break;
                }
                let end_offset = start_offset + 4 + length;
                if end_offset > data.len() {
                    break;
                }
                let directory = &data[start_offset..end_offset];
                let entries = get_directory_entry_list(directory, &data[end_offset..]);
                EntryData::Directory(entries)
            }
            _ => break,
        };
        let entry = Entry {
            key: key,
            data: entry_data,
        };
        entries.push(entry);

        directory = &directory[4..];
    }

    entries
}

pub fn parse_leaf_entry_as_text<'a>(leaf: &'a [u8]) -> Option<&'a str> {
    Some(leaf)
        .filter(|leaf| {
            // The type of descriptor should be 'textual descriptor'.
            leaf[4] == 0
        })
        .filter(|leaf| {
            // The specifier_ID should be 0.
            let mut quadlet = [0;4];
            quadlet.copy_from_slice(&leaf[4..8]);
            let spec_id = u32::from_be_bytes(quadlet);
            spec_id == 0
        })
        // The width/character_set/language fields are just ignored since being useless.
        .and_then(|leaf| {
            // Text string.
            let literal = &leaf[12..];
            literal.iter().position(|&c| c == 0x00)
                .and_then(|pos| {
                    std::str::from_utf8(&literal[..pos])
                        .ok()
                })
                .or_else(|| {
                    std::str::from_utf8(&literal)
                        .ok()
                })
        })
}

#[cfg(test)]
mod test {
    #[test]
    fn texual_descriptor_leaf_entry_deser() {
        let raw = [
            0x00, 0x06, 0x4c, 0xb7, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4c, 0x69,
            0x6e, 0x75, 0x78, 0x20, 0x46, 0x69, 0x72, 0x65, 0x77, 0x69, 0x72, 0x65, 0x00, 0x00,
        ];
        assert_eq!(
            Some("Linux Firewire"),
            super::parse_leaf_entry_as_text(&raw)
        );
    }
}
