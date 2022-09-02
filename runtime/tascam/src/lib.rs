// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
mod asynch_runtime;
mod isoch_console_runtime;
mod isoch_rack_runtime;

mod fe8_model;
mod fw1082_model;
mod fw1804_model;
mod fw1884_model;

mod isoch_ctls;

mod seq_cntr;

use {
    alsaseq::{prelude::*, *},
    asynch_runtime::*,
    core::{card_cntr::*, RuntimeOperation},
    firewire_tascam_protocols as protocols,
    glib::{source, Error, FileError, IsA},
    hinawa::{
        prelude::{FwNodeExt, FwNodeExtManual},
        FwNode, FwReq,
    },
    hitaki::{prelude::*, *},
    ieee1212_config_rom::*,
    isoch_console_runtime::*,
    isoch_rack_runtime::*,
    protocols::{config_rom::*, *},
    seq_cntr::*,
    std::convert::TryFrom,
};

pub enum TascamRuntime {
    Fw1884(Fw1884Runtime),
    Fw1082(Fw1082Runtime),
    Fw1804(Fw1804Runtime),
    Fe8(Fe8Runtime),
}

const TASCAM_OUI: u32 = 0x00022e;
const FW1884_SW_VERSION: u32 = 0x800000;
const FE8_SW_VERSION: u32 = 0x800001;
const FW1082_SW_VERSION: u32 = 0x800003;
const FW1804_SW_VERSION: u32 = 0x800004;

impl RuntimeOperation<(String, u32)> for TascamRuntime {
    fn new((subsystem, sysnum): (String, u32)) -> Result<Self, Error> {
        match subsystem.as_str() {
            "snd" => {
                let unit = SndTascam::new();
                let devnode = format!("/dev/snd/hwC{}D0", sysnum);
                unit.open(&devnode, 0)?;

                let devnode = format!("/dev/{}", unit.node_device().unwrap());
                let node = FwNode::new();
                node.open(&devnode)?;

                let data = node.config_rom()?;
                let config_rom = ConfigRom::try_from(data).map_err(|e| {
                    let label = format!("Malformed configuration ROM detected: {}", e.to_string());
                    Error::new(FileError::Nxio, &label)
                })?;
                let unit_data = config_rom.get_unit_data()?;
                let name = unit_data.model_name.to_owned();
                match (unit_data.specifier_id, unit_data.version) {
                    (TASCAM_OUI, FW1884_SW_VERSION) => {
                        let runtime = Fw1884Runtime::new(unit, node, &name, sysnum)?;
                        Ok(Self::Fw1884(runtime))
                    }
                    (TASCAM_OUI, FW1082_SW_VERSION) => {
                        let runtime = Fw1082Runtime::new(unit, node, &name, sysnum)?;
                        Ok(Self::Fw1082(runtime))
                    }
                    (TASCAM_OUI, FW1804_SW_VERSION) => {
                        let runtime = Fw1804Runtime::new(unit, node, &name, sysnum)?;
                        Ok(Self::Fw1804(runtime))
                    }
                    _ => Err(Error::new(FileError::Noent, "Not supported")),
                }
            }
            "fw" => {
                let node = FwNode::new();
                let devnode = format!("/dev/fw{}", sysnum);
                node.open(&devnode)?;

                let data = node.config_rom()?;
                let config_rom = ConfigRom::try_from(data).map_err(|e| {
                    let label = format!("Malformed configuration ROM detected: {}", e.to_string());
                    Error::new(FileError::Nxio, &label)
                })?;
                let unit_data = config_rom.get_unit_data()?;
                match (unit_data.specifier_id, unit_data.version) {
                    (TASCAM_OUI, FE8_SW_VERSION) => {
                        let name = unit_data.model_name.to_string();
                        let runtime = Fe8Runtime::new(node, name)?;
                        Ok(Self::Fe8(runtime))
                    }
                    _ => Err(Error::new(FileError::Noent, "Not supported")),
                }
            }
            _ => {
                let label = "Invalid name of subsystem";
                Err(Error::new(FileError::Nodev, &label))
            }
        }
    }

    fn listen(&mut self) -> Result<(), Error> {
        match self {
            Self::Fw1884(runtime) => runtime.listen(),
            Self::Fw1082(runtime) => runtime.listen(),
            Self::Fw1804(runtime) => runtime.listen(),
            Self::Fe8(runtime) => runtime.listen(),
        }
    }

    fn run(&mut self) -> Result<(), Error> {
        match self {
            Self::Fw1884(runtime) => runtime.run(),
            Self::Fw1082(runtime) => runtime.run(),
            Self::Fw1804(runtime) => runtime.run(),
            Self::Fe8(runtime) => runtime.run(),
        }
    }
}

#[derive(Default)]
pub struct SequencerState<U> {
    map: Vec<MachineItem>,
    machine_state: MachineState,
    surface_state: U,
}

const BOOL_TRUE: i32 = 0x7f;

pub trait SequencerCtlOperation<
    S: IsA<TascamProtocol>,
    T: MachineStateOperation + SurfaceImageOperation<U>,
    U,
>
{
    fn state(&self) -> &SequencerState<U>;
    fn state_mut(&mut self) -> &mut SequencerState<U>;

    fn image(&self) -> &[u32];
    fn image_mut(&mut self) -> &mut Vec<u32>;

    fn initialize_surface(
        &mut self,
        node: &mut FwNode,
        machine_values: &[(MachineItem, ItemValue)],
    ) -> Result<(), Error>;
    fn finalize_surface(&mut self, node: &mut FwNode) -> Result<(), Error>;

    fn feedback_to_surface(
        &mut self,
        node: &mut FwNode,
        event: &(MachineItem, ItemValue),
    ) -> Result<(), Error>;

    fn initialize_sequencer(&mut self, node: &mut FwNode) -> Result<(), Error> {
        self.initialize_message_map();
        T::initialize_surface_state(&mut self.state_mut().surface_state);
        T::initialize_machine(&mut self.state_mut().machine_state);
        let machine_values = T::get_machine_current_values(&self.state().machine_state);
        self.initialize_surface(node, &machine_values)
    }

    fn finalize_sequencer(&mut self, node: &mut FwNode) -> Result<(), Error> {
        self.finalize_surface(node)
    }

    fn initialize_message_map(&mut self) {
        let map = &mut self.state_mut().map;
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
    }

    fn dispatch_surface_event(
        &mut self,
        unit: &mut S,
        node: &mut FwNode,
        seq_cntr: &mut SeqCntr,
        index: u32,
        before: u32,
        after: u32,
    ) -> Result<(), Error> {
        unit.read_state(self.image_mut())?;
        let inputs = T::decode_surface_image(
            &self.state().surface_state,
            self.image(),
            index,
            before,
            after,
        );
        inputs.iter().try_for_each(|input| {
            let outputs = self.dispatch_machine_event(input);
            outputs.iter().try_for_each(|output| {
                self.feedback_to_appl(seq_cntr, output)?;
                self.feedback_to_surface(node, output)
            })
        })
    }

    fn dispatch_appl_events(
        &mut self,
        unit: &mut FwNode,
        seq_cntr: &mut SeqCntr,
        events: &[Event],
    ) -> Result<(), Error> {
        // NOTE: At present, controller event is handled for my convenience.
        events
            .iter()
            .filter(|ev| EventType::Controller == ev.event_type())
            .filter_map(|ev| ev.ctl_data().ok())
            .try_for_each(|ctl_data| {
                let input = self.parse_appl_event(&ctl_data)?;
                let outputs = self.dispatch_machine_event(&input);
                outputs.iter().try_for_each(|output| {
                    if !output.eq(&input) {
                        self.feedback_to_appl(seq_cntr, output)?;
                    }
                    self.feedback_to_surface(unit, output)
                })
            })
    }

    fn parse_appl_event(&self, data: &EventDataCtl) -> Result<(MachineItem, ItemValue), Error> {
        if data.channel() != 0 {
            let msg = format!("Channel {} is not supported yet.", data.channel());
            Err(Error::new(FileError::Inval, &msg))?;
        }

        let index = data.param();
        let &machine_item = self.state().map.iter().nth(index as usize).ok_or_else(|| {
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
    }

    fn dispatch_machine_event(
        &mut self,
        input: &(MachineItem, ItemValue),
    ) -> Vec<(MachineItem, ItemValue)> {
        T::change_machine_value(&mut self.state_mut().machine_state, input)
    }

    fn feedback_to_appl(
        &mut self,
        cntr: &mut SeqCntr,
        event: &(MachineItem, ItemValue),
    ) -> Result<(), Error> {
        let index = self
            .state()
            .map
            .iter()
            .position(|item| event.0.eq(item))
            .ok_or_else(|| {
                let msg = format!("Unsupported machine item: {}", event.0);
                Error::new(FileError::Inval, &msg)
            })?;

        let value = match event.1 {
            ItemValue::Bool(val) => {
                if val {
                    BOOL_TRUE
                } else {
                    0
                }
            }
            ItemValue::U16(val) => val as i32,
        };

        cntr.schedule_event(index as u32, value)
    }
}
