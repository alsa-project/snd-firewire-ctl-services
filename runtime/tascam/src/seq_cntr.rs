// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, alsaseq::*, tracing::enabled};

#[derive(Debug)]
enum Tstamp {
    RealTime((u32, u32)),
    TickTime(u32),
}

#[allow(dead_code)]
#[derive(Debug)]
struct Address {
    client_id: u8,
    port_id: u8,
}

fn dump_event_info(event: &Event) {
    let event_type = event.event_type();

    let length_mode = event.length_mode();
    let priority_mode = event.priority_mode();

    let time_mode = event.time_mode();
    let tstamp = match event.tstamp_mode() {
        EventTstampMode::Real => {
            let real = event.real_time().unwrap();
            Tstamp::RealTime((real[0], real[1]))
        }
        EventTstampMode::Tick => {
            let tick = event.tick_time().unwrap();
            Tstamp::TickTime(tick)
        }
        _ => unreachable!(),
    };

    let tag = event.tag();
    let queue_id = event.queue_id();

    let addr = event.source();
    let src = Address {
        client_id: addr.client_id(),
        port_id: addr.port_id(),
    };
    let addr = event.destination();
    let dst = Address {
        client_id: addr.client_id(),
        port_id: addr.port_id(),
    };

    match event_type {
        EventType::System | EventType::Result => {
            let result = event.result_data().unwrap();
            let result_event = result.event();
            let result_value = result.result();
            debug!(
                ?event_type,
                ?length_mode,
                ?priority_mode,
                ?time_mode,
                ?tstamp,
                ?tag,
                ?queue_id,
                ?src,
                ?dst,
                ?result_event,
                ?result_value,
            );
        }
        EventType::Note | EventType::Noteon | EventType::Noteoff | EventType::Keypress => {
            let data = event.note_data().unwrap();

            let channel = data.channel();
            let duration = data.duration();
            let note = data.note();
            let off_velocity = data.off_velocity();
            let velocity = data.velocity();

            debug!(
                ?event_type,
                ?length_mode,
                ?priority_mode,
                ?time_mode,
                ?tstamp,
                ?tag,
                ?queue_id,
                ?src,
                ?dst,
                ?channel,
                ?duration,
                ?note,
                ?off_velocity,
                ?velocity,
            );
        }
        EventType::Controller
        | EventType::Pgmchange
        | EventType::Chanpress
        | EventType::Pitchbend
        | EventType::Control14
        | EventType::Nonregparam
        | EventType::Regparam
        | EventType::Songpos
        | EventType::Songsel
        | EventType::Qframe
        | EventType::Timesign
        | EventType::Keysign => {
            let data = event.ctl_data().unwrap();
            let channel = data.channel();
            let param = data.param();
            let value = data.value();
            debug!(
                ?event_type,
                ?length_mode,
                ?priority_mode,
                ?time_mode,
                ?tstamp,
                ?tag,
                ?queue_id,
                ?src,
                ?dst,
                ?channel,
                ?param,
                ?value,
            );
        }
        EventType::Start
        | EventType::Continue
        | EventType::Stop
        | EventType::SetposTick
        | EventType::SetposTime
        | EventType::Tempo
        | EventType::Clock
        | EventType::Tick
        | EventType::QueueSkew => {
            let data = event.queue_data().unwrap();
            let queue_id = data.queue_id();
            debug!(
                ?event_type,
                ?length_mode,
                ?priority_mode,
                ?time_mode,
                ?tstamp,
                ?tag,
                ?queue_id,
                ?src,
                ?dst,
                ?queue_id,
            );
        }
        EventType::ClientStart
        | EventType::ClientExit
        | EventType::ClientChange
        | EventType::PortStart
        | EventType::PortExit
        | EventType::PortChange => {
            let data = event.addr_data().unwrap();
            let addr = Address {
                client_id: data.client_id(),
                port_id: data.port_id(),
            };
            debug!(
                ?event_type,
                ?length_mode,
                ?priority_mode,
                ?time_mode,
                ?tstamp,
                ?tag,
                ?queue_id,
                ?src,
                ?dst,
                ?addr,
            );
        }
        EventType::PortSubscribed | EventType::PortUnsubscribed => {
            let mut data = event.connect_data().unwrap();
            let addr = data.src();
            let s = Address {
                client_id: addr.client_id(),
                port_id: addr.port_id(),
            };
            let addr = data.dst();
            let d = Address {
                client_id: addr.client_id(),
                port_id: addr.port_id(),
            };
            debug!(
                ?event_type,
                ?length_mode,
                ?priority_mode,
                ?time_mode,
                ?tstamp,
                ?tag,
                ?queue_id,
                ?src,
                ?dst,
                ?s,
                ?d,
            );
        }
        EventType::Sysex
        | EventType::Bounce
        | EventType::UsrVar0
        | EventType::UsrVar1
        | EventType::UsrVar2
        | EventType::UsrVar3
        | EventType::UsrVar4 => {
            let data = event.blob_data().unwrap();
            debug!(
                ?event_type,
                ?length_mode,
                ?priority_mode,
                ?time_mode,
                ?tstamp,
                ?tag,
                ?queue_id,
                ?src,
                ?dst,
                ?data,
            );
        }
        _ => {
            debug!(
                ?event_type,
                ?length_mode,
                ?priority_mode,
                ?time_mode,
                ?tstamp,
                ?tag,
                ?queue_id,
                ?src,
                ?dst,
            );
        }
    }
}

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
        if enabled!(Level::DEBUG) {
            dump_event_info(ev);
        }

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
        if enabled!(Level::DEBUG) {
            dump_event_info(&event);
        }
        self.client.schedule_event(&event)
    }
}
