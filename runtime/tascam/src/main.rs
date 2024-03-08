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
    clap::Parser,
    firewire_tascam_protocols as protocols,
    glib::{prelude::IsA, Error, FileError},
    hinawa::{
        prelude::{FwNodeExt, FwNodeExtManual},
        FwNode, FwReq,
    },
    hitaki::{prelude::*, *},
    ieee1212_config_rom::*,
    isoch_console_runtime::*,
    isoch_rack_runtime::*,
    protocols::{config_rom::*, *},
    runtime_core::{card_cntr::*, cmdline::*, LogLevel, *},
    seq_cntr::*,
    std::{convert::TryFrom, marker::PhantomData},
    tracing::{debug, debug_span, Level},
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
    fn new((subsystem, sysnum): (String, u32), log_level: Option<LogLevel>) -> Result<Self, Error> {
        if let Some(level) = log_level {
            let fmt_level = match level {
                LogLevel::Debug => Level::DEBUG,
            };
            tracing_subscriber::fmt().with_max_level(fmt_level).init();
        }

        match subsystem.as_str() {
            "snd" => {
                let unit = SndTascam::new();
                let devnode = format!("/dev/snd/hwC{}D0", sysnum);
                unit.open(&devnode, 0)?;

                let devnode = format!("/dev/{}", unit.node_device().unwrap());
                let node = FwNode::new();
                node.open(&devnode, 0)?;

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
                node.open(&devnode, 0)?;

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

pub trait SurfaceCtlOperation<T: IsA<TascamProtocol>> {
    fn init(&mut self, node: &mut FwNode) -> Result<(), Error>;

    fn peek(
        &mut self,
        unit: &mut T,
        index: u32,
        before: u32,
        after: u32,
    ) -> Result<Vec<(MachineItem, ItemValue)>, Error>;

    fn ack(
        &mut self,
        machine_value: &(MachineItem, ItemValue),
        node: &mut FwNode,
    ) -> Result<(), Error>;

    fn fin(&mut self, node: &mut FwNode) -> Result<(), Error>;
}

const BOOL_TRUE: i32 = 0x7f;

pub trait SequencerCtlOperation<S, T>: SurfaceCtlOperation<S>
where
    S: IsA<TascamProtocol>,
    T: MachineStateOperation,
{
    fn state(&self) -> &MachineState;
    fn state_mut(&mut self) -> &mut MachineState;

    fn initialize_sequencer(&mut self, node: &mut FwNode) -> Result<(), Error> {
        self.init(node)?;
        T::get_machine_current_values(self.state())
            .iter()
            .try_for_each(|machine_value| {
                debug!(?machine_value);
                self.ack(machine_value, node)
            })
    }

    fn dispatch_surface_event(
        &mut self,
        unit: &mut S,
        node: &mut FwNode,
        seq_cntr: &mut SeqCntr,
        converter: &EventConverter<T>,
        index: u32,
        before: u32,
        after: u32,
    ) -> Result<(), Error> {
        let inputs = self.peek(unit, index, before, after)?;
        inputs.iter().try_for_each(|input| {
            let outputs = T::change_machine_value(self.state_mut(), input);
            debug!(?outputs, ?input);
            outputs.iter().try_for_each(|output| {
                let event = converter.seq_event_from_machine_event(output)?;
                seq_cntr.schedule_event(event)?;
                self.ack(output, node)
            })
        })
    }

    fn dispatch_appl_events(
        &mut self,
        node: &mut FwNode,
        seq_cntr: &mut SeqCntr,
        converter: &EventConverter<T>,
        events: &[Event],
    ) -> Result<(), Error> {
        events.iter().try_for_each(|event| {
            let input = converter.seq_event_to_machine_event(event)?;
            let outputs = T::change_machine_value(self.state_mut(), &input);
            debug!(?outputs, ?input);
            outputs.iter().try_for_each(|output| {
                if !output.eq(&input) {
                    let event = converter.seq_event_from_machine_event(output)?;
                    seq_cntr.schedule_event(event)?;
                }
                self.ack(output, node)
            })
        })
    }
}

struct TascamServiceCmd;

#[derive(Parser, Default)]
#[clap(name = "snd-firewire-tascam-ctl-service")]
struct Arguments {
    /// The name of subsystem; 'snd' or 'fw'.
    subsystem: String,
    /// The numeric identifier of sound card or firewire character device.
    sysnum: u32,

    /// The level to debug runtime, disabled as a default.
    #[clap(long, short, value_enum)]
    log_level: Option<LogLevel>,
}

impl ServiceCmd<Arguments, (String, u32), TascamRuntime> for TascamServiceCmd {
    fn params(args: &Arguments) -> ((String, u32), Option<LogLevel>) {
        ((args.subsystem.clone(), args.sysnum), args.log_level)
    }
}

fn main() {
    TascamServiceCmd::run()
}
