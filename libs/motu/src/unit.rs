// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};
use glib::source;
use nix::sys::signal;
use std::sync::mpsc;
use std::convert::TryFrom;

use hinawa::{FwNodeExt, FwNodeExtManual, SndUnitExt, SndMotuExt};
use alsactl::CardExt;

use core::dispatcher;
use core::card_cntr;

use ieee1212_config_rom::{*, entry::*};

use super::model::MotuModel;

const OUI_MOTU: u32 = 0x0001f2;

enum Event {
    Shutdown,
    Disconnected,
    BusReset(u32),
    Elem((alsactl::ElemId, alsactl::ElemEventMask)),
    Notify(u32),
}

pub struct MotuUnit<'a> {
    unit: hinawa::SndMotu,
    model: MotuModel<'a>,
    card_cntr: card_cntr::CardCntr,
    rx: mpsc::Receiver<Event>,
    tx: mpsc::SyncSender<Event>,
    dispatchers: Vec<dispatcher::Dispatcher>,
}

impl<'a> Drop for MotuUnit<'a> {
    fn drop(&mut self) {
        // Finish I/O threads.
        self.dispatchers.clear();
    }
}

impl<'a> MotuUnit<'a> {
    const NODE_DISPATCHER_NAME: &'a str = "node event dispatcher";
    const SYSTEM_DISPATCHER_NAME: &'a str = "system event dispatcher";

    pub fn new(card_id: u32) -> Result<Self, Error> {
        let unit = hinawa::SndMotu::new();
        unit.open(&format!("/dev/snd/hwC{}D0", card_id))?;

        let node = unit.get_node();
        let (model_id, version) = detect_model(&node)?;
        let model = MotuModel::new(model_id, version)?;

        let card_cntr = card_cntr::CardCntr::new();
        card_cntr.card.open(card_id, 0)?;

        // Use uni-directional channel for communication to child threads.
        let (tx, rx) = mpsc::sync_channel(32);

        let dispatchers = Vec::new();

        Ok(MotuUnit {
            unit,
            model,
            card_cntr,
            rx,
            tx,
            dispatchers,
        })
    }

    fn launch_node_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = Self::NODE_DISPATCHER_NAME.to_string();
        let mut dispatcher = dispatcher::Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_snd_unit(&self.unit, move |_| {
            let _ = tx.send(Event::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.connect_notified(move |_, msg| {
            let t = tx.clone();
            let _ = std::thread::spawn(move || {
                // Just after notification, the target device tends to return RCODE_BUSY against
                // read request. Here, wait for 100 msec to avoid it.
                std::thread::sleep(std::time::Duration::from_millis(100));
                let _ = t.send(Event::Notify(msg));
            });
        });

        let tx = self.tx.clone();
        dispatcher.attach_fw_node(&self.unit.get_node(), move |_| {
            let _ = tx.send(Event::Disconnected);
        })?;

        let tx = self.tx.clone();
        self.unit.get_node().connect_bus_update(move |node| {
            let _ = tx.send(Event::BusReset(node.get_property_generation()));
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    fn launch_system_event_dispatcher(&mut self) -> Result<(), Error> {
        let name = Self::SYSTEM_DISPATCHER_NAME.to_string();
        let mut dispatcher = dispatcher::Dispatcher::run(name)?;

        let tx = self.tx.clone();
        dispatcher.attach_signal_handler(signal::Signal::SIGINT, move || {
            let _ = tx.send(Event::Shutdown);
            source::Continue(false)
        });

        let tx = self.tx.clone();
        dispatcher.attach_snd_card(&self.card_cntr.card, |_| {})?;
        self.card_cntr.card.connect_handle_elem_event(move |_, elem_id, events| {
            let _ = tx.send(Event::Elem((elem_id.clone(), events)));
        });

        self.dispatchers.push(dispatcher);

        Ok(())
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        self.launch_node_event_dispatcher()?;
        self.launch_system_event_dispatcher()?;

        self.model.load(&self.unit, &mut self.card_cntr)?;

        Ok(())
    }

    pub fn run(&mut self) {
        loop {
            let ev = match self.rx.recv() {
                Ok(ev) => ev,
                Err(_) => continue,
            };

            match ev {
                Event::Shutdown | Event::Disconnected => break,
                Event::BusReset(generation) => {
                    println!("IEEE 1394 bus is updated: {}", generation);
                }
                Event::Elem((elem_id, events)) => {
                    let _ = self.model.dispatch_elem_event(&self.unit, &mut self.card_cntr,
                                                           &elem_id, &events);
                }
                Event::Notify(msg) => {
                    let _ = self.model.dispatch_notification(&self.unit, &msg, &mut self.card_cntr);
                }
            }
        }
    }
}

fn read_directory<'a>(entries: &'a [Entry], key_type: KeyType, field_name: &str)
    -> Result<&'a [Entry<'a>], Error>
{
    entries.iter().find_map(|entry| {
        EntryDataAccess::<&[Entry]>::get(entry, key_type)
    })
    .ok_or_else(|| {
        let label = format!("Fail to detect {} directory in configuration ROM", field_name);
        Error::new(FileError::Nxio, &label)
    })
}

fn read_immediate(entries: &[Entry], key_type: KeyType, field_name: &str)
    -> Result<u32, Error>
{
    entries.iter().find_map(|entry| {
        EntryDataAccess::<u32>::get(entry, key_type)
    })
    .ok_or_else(|| {
        let label = format!("Fail to detect {} in configuration ROM", field_name);
        Error::new(FileError::Nxio, &label)
    })
}

fn detect_model(node: &hinawa::FwNode) -> Result<(u32, u32), Error> {
    let data = node.get_config_rom()?;
    let config_rom = ConfigRom::try_from(data)
        .map_err(|e| {
            let label = format!("Malformed configuration ROM detected: {}", e.to_string());
            Error::new(FileError::Nxio, &label)
        })?;

    let vendor = read_immediate(&config_rom.root, KeyType::Vendor, "Vendor ID")?;
    if vendor != OUI_MOTU {
        let label = format!("Vendor Id is not OUI of Mark of the Unicorn: {:08x}", vendor);
        return Err(Error::new(FileError::Nxio, &label));
    }

    let unit_entries = read_directory(&config_rom.root, KeyType::Unit, "Unit")?;

    let spec_id = read_immediate(&unit_entries, KeyType::SpecifierId, "Specifier ID")?;
    if spec_id != OUI_MOTU {
        let label = format!("Specifier ID is not OUI of Mark of the Unicorn: {:08x} ", spec_id);
        return Err(Error::new(FileError::Nxio, &label));
    }

    // NOTE: It's odd but version field is used for model ID and model field is used for version
    // in MOTU case.
    let model_id = read_immediate(&unit_entries, KeyType::Version, "Version")?;
    let version = read_immediate(&unit_entries, KeyType::Model, "Model ID")?;

    Ok((model_id, version))
}
