// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, alsaseq::*};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventConverter<T: MachineStateOperation> {
    map: Vec<MachineItem>,
    _phantom: PhantomData<T>,
}

impl<T: MachineStateOperation> Default for EventConverter<T> {
    fn default() -> Self {
        let mut map = Vec::new();

        T::BOOL_ITEMS.iter().chain(T::U16_ITEMS).for_each(|&item| {
            assert!(
                map.iter().find(|i| item.eq(i)).is_none(),
                "Programming error for list of machine item: {}",
                item,
            );
            map.push(item);
        });

        if T::HAS_TRANSPORT {
            map.extend_from_slice(&T::TRANSPORT_ITEMS);
        }

        if T::HAS_BANK {
            map.push(MachineItem::Bank);
        }

        Self {
            map,
            _phantom: Default::default(),
        }
    }
}

impl<T: MachineStateOperation> EventConverter<T> {
    pub(crate) fn seq_event_from_machine_event(
        &self,
        machine_value: &(MachineItem, ItemValue),
    ) -> Result<Event, Error> {
        self.map
            .iter()
            .position(|item| machine_value.0.eq(item))
            .ok_or_else(|| {
                let msg = format!("Unsupported machine item: {}", machine_value.0);
                Error::new(FileError::Inval, &msg)
            })
            .and_then(|index| {
                let param = index as u32;

                let value = match machine_value.1 {
                    ItemValue::Bool(val) => {
                        if val {
                            BOOL_TRUE
                        } else {
                            0
                        }
                    }
                    ItemValue::U16(val) => val as i32,
                };

                let mut ev = Event::new(EventType::Controller);

                let mut data = ev.ctl_data()?;
                data.set_channel(0);
                data.set_param(param);
                data.set_value(value);

                ev.set_ctl_data(&data)?;

                Ok(ev)
            })
    }

    pub(crate) fn seq_event_to_machine_event(
        &self,
        ev: &Event,
    ) -> Result<(MachineItem, ItemValue), Error> {
        // NOTE: At present, controller event is handled for my convenience.
        ev.ctl_data().and_then(|data| {
            if data.channel() != 0 {
                let msg = format!("Channel {} is not supported yet.", data.channel());
                Err(Error::new(FileError::Inval, &msg))?;
            }

            let index = data.param();
            let &machine_item = self.map.iter().nth(index as usize).ok_or_else(|| {
                let msg = format!("Unsupported control number: {}", index);
                Error::new(FileError::Inval, &msg)
            })?;

            let value = data.value();
            let item_value = if T::BOOL_ITEMS.iter().find(|i| machine_item.eq(i)).is_some() {
                ItemValue::Bool(value == BOOL_TRUE)
            } else if T::TRANSPORT_ITEMS
                .iter()
                .find(|i| machine_item.eq(i))
                .is_some()
            {
                ItemValue::Bool(value == BOOL_TRUE)
            } else if T::U16_ITEMS.iter().find(|i| machine_item.eq(i)).is_some() {
                ItemValue::U16(value as u16)
            } else if machine_item.eq(&MachineItem::Bank) {
                ItemValue::U16(value as u16)
            } else {
                // Programming error.
                unreachable!();
            };

            Ok((machine_item, item_value))
        })
    }
}

pub struct SeqCntr {
    pub client: UserClient,
    port_id: u8,
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

        Ok(SeqCntr { client, port_id: 0 })
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

    pub fn schedule_event(&mut self, mut event: Event) -> Result<(), Error> {
        event.set_queue_id(SpecificAddress::Subscribers.into());
        self.client.schedule_event(&event)
    }
}
