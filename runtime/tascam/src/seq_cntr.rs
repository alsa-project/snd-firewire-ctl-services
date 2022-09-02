// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, alsaseq::*};

pub struct SeqCntr {
    pub client: UserClient,
    port_id: u8,
    event: Event,
}

impl Drop for SeqCntr {
    fn drop(&mut self) {
        let _ = self.client.delete_port(self.port_id);
    }
}

impl SeqCntr {
    const SEQ_PORT_NAME: &'static str = "Control Surface";

    pub fn new(name: &str) -> Result<Self, Error> {
        let client = UserClient::new();
        client.open(0)?;

        let info = ClientInfo::new();
        info.set_name(Some(name));
        client.set_info(&info)?;

        let mut event = Event::new(EventType::Controller);
        event.set_queue_id(SpecificAddress::Subscribers.into());

        Ok(SeqCntr {
            client,
            port_id: 0,
            event,
        })
    }

    pub fn open_port(&mut self) -> Result<(), Error> {
        let mut info = PortInfo::new();
        let attr_flags = PortAttrFlag::MIDI_GENERIC | PortAttrFlag::HARDWARE;
        info.set_attrs(attr_flags);
        let cap_flags = PortCapFlag::READ
            | PortCapFlag::SUBS_READ
            | PortCapFlag::WRITE
            | PortCapFlag::SUBS_WRITE;
        info.set_caps(cap_flags);
        info.set_name(Some(&Self::SEQ_PORT_NAME));
        self.client.create_port(&mut info)?;
        self.port_id = match info.addr() {
            Some(addr) => addr.port_id(),
            None => {
                let label = "Fail to get address for added port.";
                return Err(Error::new(FileError::Io, &label));
            }
        };

        Ok(())
    }

    pub fn schedule_event(&mut self, param: u32, val: i32) -> Result<(), Error> {
        let mut data = self.event.ctl_data()?;
        data.set_channel(0);
        data.set_param(param);
        data.set_value(val);
        self.event.set_ctl_data(&data)?;

        self.client.schedule_event(&self.event)
    }
}
