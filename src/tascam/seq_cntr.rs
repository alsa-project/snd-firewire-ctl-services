// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use alsaseq::*;

pub struct SeqCntr {
    pub client: alsaseq::UserClient,
    port_id: u8,
}

impl Drop for SeqCntr {
    fn drop(&mut self) {
        let _ = self.client.delete_port(self.port_id);
    }
}

impl<'a> SeqCntr {
    const SEQ_PORT_NAME: &'a str = "Control Surface";

    pub fn new(name: String) -> Result<Self, Error> {
        let client = alsaseq::UserClient::new();
        client.open(0)?;

        let info = alsaseq::ClientInfo::new();
        info.set_property_name(Some(&name));
        client.set_info(&info)?;

        Ok(SeqCntr {
            client,
            port_id: 0,
        })
    }

    pub fn open_port(&mut self) -> Result<(), Error> {
        let mut info = alsaseq::PortInfo::new();
        let attr_flags = alsaseq::PortAttrFlag::MIDI_GENERIC | alsaseq::PortAttrFlag::HARDWARE;
        info.set_property_attrs(attr_flags);
        let cap_flags = alsaseq::PortCapFlag::READ | alsaseq::PortCapFlag::SUBS_READ;
        info.set_property_caps(cap_flags);
        info.set_property_name(Some(&Self::SEQ_PORT_NAME));
        self.client.create_port(&mut info)?;
        self.port_id = match info.get_property_addr() {
            Some(addr) => addr.get_port_id(),
            None => {
                let label = "Fail to get address for added port.";
                return Err(Error::new(FileError::Io, &label));
            }
        };

        Ok(())
    }
}
