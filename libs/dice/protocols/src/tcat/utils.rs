// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

trait EndianConvert {
    fn from_ne(&mut self);
    fn to_ne(&mut self);
}

impl EndianConvert for &mut [u8] {
    fn from_ne(&mut self) {
        let mut quadlet = [0;4];
        (0..(self.len() / 4)).for_each(|i| {
            let pos = i * 4;
            quadlet.copy_from_slice(&self[pos..(pos + 4)]);
            self[pos..(pos + 4)].copy_from_slice(&u32::from_ne_bytes(quadlet).to_be_bytes());
        });
    }

    fn to_ne(&mut self) {
        let mut quadlet = [0;4];
        (0..(self.len() / 4)).for_each(|i| {
            let pos = i * 4;
            quadlet.copy_from_slice(&self[pos..(pos + 4)]);
            self[pos..(pos + 4)].copy_from_slice(&u32::from_be_bytes(quadlet).to_be_bytes());
        });
    }
}

pub fn build_label<T: AsRef<str>>(name: T, len: usize) -> Vec<u8>
{
    let mut raw = name.as_ref().as_bytes().to_vec();
    raw.resize(len, 0x00);
    raw.as_mut_slice().from_ne();
    raw
}

pub fn parse_label(raw: &[u8]) -> Result<String, std::str::Utf8Error> {
    let mut raw = raw.to_vec();
    raw.as_mut_slice().to_ne();

    raw.push(0x00);
    std::str::from_utf8(&raw)
        .map(|text| {
            if let Some(pos) = text.find('\0') {
                text[..pos].to_string()
            } else {
                String::new()
            }
        })
}

pub fn parse_labels(raw: &[u8]) -> Result<Vec<String>, std::str::Utf8Error> {
    let mut raw = raw.to_vec();
    raw.as_mut_slice().to_ne();

    raw.push(0x00);
    std::str::from_utf8(&raw)
        .map(|text| {
            if let Some(pos) = text.find("\\\\") {
                text[..pos].split("\\").map(|l| l.to_string()).collect()
            } else {
                Vec::new()
            }
        })
}
