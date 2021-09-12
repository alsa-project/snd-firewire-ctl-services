// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::{Error, FileError, MainContext, MainLoop};

use hinawa::{FwNode, FwNodeExt, FwNodeError};
use hinawa::FwReq;

use dice_protocols::tcat::extension::{*, caps_section::*, cmd_section::*, mixer_section::*};
use dice_protocols::tcat::extension::peak_section::*;
use dice_protocols::tcat::extension::current_config_section::*;
use dice_protocols::tcat::extension::standalone_section::*;

use std::sync::Arc;
use std::thread;

const TIMEOUT_MS: u32 = 20;

fn print_sections(sections: &ExtensionSections) {
    println!("Extension sections:");
    println!("  caps:           offset 0x{:x}, size: 0x{:x}", sections.caps.offset, sections.caps.size);
    println!("  cmd:            offset 0x{:x}, size: 0x{:x}", sections.cmd.offset, sections.cmd.size);
    println!("  mixer:          offset 0x{:x}, size: 0x{:x}", sections.mixer.offset, sections.mixer.size);
    println!("  peak:           offset 0x{:x}, size: 0x{:x}", sections.peak.offset, sections.peak.size);
    println!("  router:         offset 0x{:x}, size: 0x{:x}", sections.router.offset, sections.router.size);
    println!("  stream_format:  offset 0x{:x}, size: 0x{:x}", sections.stream_format.offset, sections.stream_format.size);
    println!("  config:         offset 0x{:x}, size: 0x{:x}", sections.current_config.offset, sections.current_config.size);
    println!("  standalone:     offset 0x{:x}, size: 0x{:x}", sections.standalone.offset, sections.standalone.size);
    println!("  application:    offset 0x{:x}, size: 0x{:x}", sections.application.offset, sections.application.size);
}

fn print_caps(caps: &ExtensionCaps) {
    println!("Caps:");
    println!("  Router:");
    println!("    is_exposed:       {}", caps.router.is_exposed);
    println!("    is_readonly:      {}", caps.router.is_readonly);
    println!("    is_storable:      {}", caps.router.is_storable);
    println!("    maximum_entry_count: {}", caps.router.maximum_entry_count);
    println!("  Mixer:");
    println!("    is_exposed:       {}", caps.mixer.is_exposed);
    println!("    is_readonly:      {}", caps.mixer.is_readonly);
    println!("    is_storable:      {}", caps.mixer.is_storable);
    println!("    input_device_id:  {}", caps.mixer.input_device_id);
    println!("    output_device_id: {}", caps.mixer.output_device_id);
    println!("    input_count:      {}", caps.mixer.input_count);
    println!("    output_count:     {}", caps.mixer.output_count);
    println!("  General:");
    println!("    dynamic_stream_format: {}", caps.general.dynamic_stream_format);
    println!("    storage_avail:    {}", caps.general.storage_avail);
    println!("    peak_avail:       {}", caps.general.peak_avail);
    println!("    max_tx_streams:   {}", caps.general.max_tx_streams);
    println!("    max_rx_streams:   {}", caps.general.max_rx_streams);
    println!("    stream_format_is_storable: {}", caps.general.stream_format_is_storable);

    let label = match caps.general.asic_type {
        AsicType::DiceII => "DiceII".to_string(),
        AsicType::Tcd2210 => "TCD2210".to_string(),
        AsicType::Tcd2220 => "TCD2220".to_string(),
        AsicType::Reserved(val) => format!("Reserved({})", val),
    };
    println!("    asic_type:        {}", label);
}

fn print_mixer(proto: &FwReq, node: &mut FwNode, sections: &ExtensionSections, caps: &ExtensionCaps)
    -> Result<(), Error>
{
    println!("Mixer:");
    println!("  Saturation:");
    let entries = proto.read_saturation(node, sections, caps, TIMEOUT_MS)?;
    entries.iter().enumerate().for_each(|(i, saturation)| {
        println!("    dst {}: {}", i, saturation);
    });

    println!("  Coefficiency:");
    (0..(caps.mixer.output_count as usize)).try_for_each(|dst| {
        (0..(caps.mixer.input_count as usize)).try_for_each(|src| {
            proto.read_coef(node, sections, caps, dst, src, TIMEOUT_MS)
                .map(|coef| println!("    dst {} <- src {}: {}", dst, src, coef))
        })
    })
}

fn print_peak(req: &mut FwReq, node: &mut FwNode, sections: &ExtensionSections, caps: &ExtensionCaps)
    -> Result<(), Error>
{
    PeakSectionProtocol::read_peak_entries(req, node, sections, caps, TIMEOUT_MS)
        .map(|entries| {
            println!("Peak:");
            entries.iter()
                .enumerate()
                .for_each(|(i, entry)| {
                println!("  entry {}: 0x{:04x}", i, entry.peak);
            })
        })
}

const RATE_MODES: [RateMode;3] = [RateMode::Low, RateMode::Middle, RateMode::High];

fn print_current_router_entries(req: &mut FwReq, node: &mut FwNode, sections: &ExtensionSections,
                                caps: &ExtensionCaps)
    -> Result<(), Error>
{
    println!("Current router entries:");
    RATE_MODES.iter().try_for_each(|&mode| {
        CurrentConfigSectionProtocol::read_current_router_entries(
            req,
            node,
            sections,
            caps,
            mode,
            TIMEOUT_MS
        )
            .map(|entries| {
                println!("  {}:", mode);
                entries.iter().enumerate().for_each(|(i, entry)| {
                    println!("    entry {}: {:?} <- {:?}", i, entry.dst, entry.src);
                });
            })
    })
}

fn print_stream_format_entry(entry: &FormatEntry) {
    println!("      pcm:      {}", entry.pcm_count);
    println!("      midi:     {}", entry.midi_count);
    println!("      channel names:");
    entry.labels.iter().enumerate().for_each(|(i, label)| {
        println!("        ch {}:   {}", i, label);
    });
    println!("      AC3 capabilities:");
    entry.enable_ac3.iter().enumerate().filter(|&(_, enabled)| *enabled).for_each(|(i, enabled)| {
        println!("        ch {}: {}", i, enabled);
    });
}

fn print_current_stream_format_entries(req: &mut FwReq, node: &mut FwNode, sections: &ExtensionSections,
                                       caps: &ExtensionCaps)
    -> Result<(), Error>
{
    println!("Current stream format entries:");
    RATE_MODES.iter().try_for_each(|&mode| {
        CurrentConfigSectionProtocol::read_current_stream_format_entries(
            req,
            node,
            sections,
            caps,
            mode,
            TIMEOUT_MS
        )
            .map(|(tx_entries, rx_entries)| {
                println!("  {}:", mode);
                tx_entries.iter()
                    .enumerate()
                    .for_each(|(i, entry)| {
                        println!("    Tx stream {}:", i);
                        print_stream_format_entry(entry);
                    });
                rx_entries.iter()
                    .enumerate()
                    .for_each(|(i, entry)| {
                        println!("    Rx stream {}:", i);
                        print_stream_format_entry(entry);
                    });
            })
    })
}

fn print_standalone_config(proto: &FwReq, node: &mut FwNode, sections: &ExtensionSections) -> Result<(), Error> {
    println!("Standalone configurations:");
    let src = proto.read_standalone_clock_source(node, sections, TIMEOUT_MS)?;
    println!("  clock source: {}", src);
    let mode = proto.read_standalone_aes_high_rate(node, sections, TIMEOUT_MS)?;
    println!("  AES high rate: {}", mode);
    let mode = proto.read_standalone_adat_mode(node, sections, TIMEOUT_MS)?;
    println!("  ADAT mode: {}", mode);
    let params = proto.read_standalone_word_clock_param(node, sections, TIMEOUT_MS)?;
    println!("  Word clock params: {}, {} / {}", params.mode, params.rate.numerator, params.rate.denominator);
    let rate = proto.read_standalone_internal_rate(node, sections, TIMEOUT_MS)?;
    println!("  Internal rate: {}", rate);
    Ok(())
}

fn main() {
    let code = std::env::args().nth(1)
        .ok_or("At least one argument is required for path to special file of firewire character device".to_string())
        .and_then(|path| {
            let node = FwNode::new();
            node.open(&path)
                .map_err(|e| {
                    let cause = if let Some(error) = e.kind::<FileError>() {
                        match error {
                            FileError::Isdir => "is directory",
                            FileError::Acces => "access permission",
                            FileError::Noent => "not exists",
                            _ => "unknown",
                        }.to_string()
                    } else if let Some(error) = e.kind::<FwNodeError>() {
                        match error {
                            FwNodeError::Disconnected => "disconnected",
                            FwNodeError::Failed => "ioctl error",
                            _ => "unknown",
                        }.to_string()
                    } else {
                        e.to_string()
                    };
                    format!("Fail to open firewire character device {}: {} {}", path, cause, e)
                })
                .and_then(|_| {
                    node.create_source()
                        .map_err(|e| e.to_string())
                        .map(|src| (node, src))
                })
        })
        .and_then(|(mut node, src)| {
            let ctx = MainContext::new();
            let _ = src.attach(Some(&ctx));
            let dispatcher = Arc::new(MainLoop::new(Some(&ctx), false));
            let d = dispatcher.clone();
            let th = thread::spawn(move || d.run());

            let mut proto = FwReq::new();
            let result = proto.read_extension_sections(&mut node, TIMEOUT_MS)
                .and_then(|sections| {
                    print_sections(&sections);
                    let caps = CapsSectionProtocol::read_caps(
                        &mut proto,
                        &mut node,
                        &sections,
                        TIMEOUT_MS
                    )?;
                    print_caps(&caps);
                    print_mixer(&proto, &mut node, &sections, &caps)?;
                    print_peak(&mut proto, &mut node, &sections, &caps)?;
                    print_current_router_entries(&mut proto, &mut node, &sections, &caps)?;
                    print_current_stream_format_entries(&mut proto, &mut node, &sections, &caps)?;
                    print_standalone_config(&proto, &mut node, &sections)?;
                    Ok(())
                })
                .map_err(|e| e.to_string());

            dispatcher.quit();
            th.join().unwrap();
            result
        })
        .map(|_| 0)
        .unwrap_or_else(|msg| {
            eprintln!("{}", msg);
            print_help();
            1
        });

    std::process::exit(code)
}

fn print_help() {
    print!(
r###"
Usage:
  tcat-extension-parser CDEV

  where:
    CDEV:       The path to special file of firewire character device, typically '/dev/fw1'.
"###);
}
