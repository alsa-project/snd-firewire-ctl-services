// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    firewire_dice_protocols as protocols,
    glib::{Error, FileError, MainContext, MainLoop},
    hinawa::{prelude::FwNodeExt, FwNode, FwNodeError, FwReq},
    protocols::tcat::{
        extension::{
            caps_section::*, cmd_section::*, current_config_section::*, mixer_section::*,
            peak_section::*, standalone_section::*, *,
        },
        global_section::*,
        *,
    },
    std::{sync::Arc, thread},
};

const TIMEOUT_MS: u32 = 20;

struct Protocol;

impl TcatOperation for Protocol {}

impl TcatGlobalSectionSpecification for Protocol {}

impl TcatExtensionOperation for Protocol {}

fn clock_rate_to_string(rate: &ClockRate) -> String {
    match rate {
        ClockRate::R32000 => "32000".to_string(),
        ClockRate::R44100 => "44100".to_string(),
        ClockRate::R48000 => "48000".to_string(),
        ClockRate::R88200 => "88200".to_string(),
        ClockRate::R96000 => "96000".to_string(),
        ClockRate::R176400 => "176400".to_string(),
        ClockRate::R192000 => "192000".to_string(),
        ClockRate::AnyLow => "Any-low".to_string(),
        ClockRate::AnyMid => "Any-mid".to_string(),
        ClockRate::AnyHigh => "Any-high".to_string(),
        ClockRate::None => "None".to_string(),
        ClockRate::Reserved(val) => format!("Reserved({})", val),
    }
}

fn clock_source_to_string(source: &ClockSource) -> String {
    match source {
        ClockSource::Aes1 => "AES1".to_string(),
        ClockSource::Aes2 => "AES2".to_string(),
        ClockSource::Aes3 => "AES3".to_string(),
        ClockSource::Aes4 => "AES4".to_string(),
        ClockSource::AesAny => "AES-ANY".to_string(),
        ClockSource::Adat => "ADAT".to_string(),
        ClockSource::Tdif => "TDIF".to_string(),
        ClockSource::WordClock => "Word-Clock".to_string(),
        ClockSource::Arx1 => "AVS-Audio-RX1".to_string(),
        ClockSource::Arx2 => "AVS-Audio-RX2".to_string(),
        ClockSource::Arx3 => "AVS-Audio-RX3".to_string(),
        ClockSource::Arx4 => "AVS-Audio-RX4".to_string(),
        ClockSource::Internal => "Internal".to_string(),
        ClockSource::Reserved(val) => format!("Reserved({})", val),
    }
}

fn print_sections(sections: &ExtensionSections) {
    println!("Extension sections:");
    println!(
        "  caps:           offset 0x{:x}, size: 0x{:x}",
        sections.caps.offset, sections.caps.size
    );
    println!(
        "  cmd:            offset 0x{:x}, size: 0x{:x}",
        sections.cmd.offset, sections.cmd.size
    );
    println!(
        "  mixer:          offset 0x{:x}, size: 0x{:x}",
        sections.mixer.offset, sections.mixer.size
    );
    println!(
        "  peak:           offset 0x{:x}, size: 0x{:x}",
        sections.peak.offset, sections.peak.size
    );
    println!(
        "  router:         offset 0x{:x}, size: 0x{:x}",
        sections.router.offset, sections.router.size
    );
    println!(
        "  stream_format:  offset 0x{:x}, size: 0x{:x}",
        sections.stream_format.offset, sections.stream_format.size
    );
    println!(
        "  config:         offset 0x{:x}, size: 0x{:x}",
        sections.current_config.offset, sections.current_config.size
    );
    println!(
        "  standalone:     offset 0x{:x}, size: 0x{:x}",
        sections.standalone.offset, sections.standalone.size
    );
    println!(
        "  application:    offset 0x{:x}, size: 0x{:x}",
        sections.application.offset, sections.application.size
    );
}

fn print_caps(caps: &ExtensionCaps) {
    println!("Caps:");
    println!("  Router:");
    println!("    is_exposed:       {}", caps.router.is_exposed);
    println!("    is_readonly:      {}", caps.router.is_readonly);
    println!("    is_storable:      {}", caps.router.is_storable);
    println!(
        "    maximum_entry_count: {}",
        caps.router.maximum_entry_count
    );
    println!("  Mixer:");
    println!("    is_exposed:       {}", caps.mixer.is_exposed);
    println!("    is_readonly:      {}", caps.mixer.is_readonly);
    println!("    is_storable:      {}", caps.mixer.is_storable);
    println!("    input_device_id:  {}", caps.mixer.input_device_id);
    println!("    output_device_id: {}", caps.mixer.output_device_id);
    println!("    input_count:      {}", caps.mixer.input_count);
    println!("    output_count:     {}", caps.mixer.output_count);
    println!("  General:");
    println!(
        "    dynamic_stream_format: {}",
        caps.general.dynamic_stream_format
    );
    println!("    storage_avail:    {}", caps.general.storage_avail);
    println!("    peak_avail:       {}", caps.general.peak_avail);
    println!("    max_tx_streams:   {}", caps.general.max_tx_streams);
    println!("    max_rx_streams:   {}", caps.general.max_rx_streams);
    println!(
        "    stream_format_is_storable: {}",
        caps.general.stream_format_is_storable
    );

    let label = match caps.general.asic_type {
        AsicType::DiceII => "DiceII".to_string(),
        AsicType::Tcd2210 => "TCD2210".to_string(),
        AsicType::Tcd2220 => "TCD2220".to_string(),
    };
    println!("    asic_type:        {}", label);
}

fn print_mixer(
    req: &FwReq,
    node: &FwNode,
    sections: &ExtensionSections,
    caps: &ExtensionCaps,
) -> Result<(), Error> {
    let entries = vec![false; caps.mixer.output_count as usize];
    let mut saturation_params = MixerSaturationParams(entries);
    Protocol::cache_extension_whole_params(
        req,
        node,
        sections,
        caps,
        &mut saturation_params,
        TIMEOUT_MS,
    )?;

    let entries =
        vec![vec![0u16; caps.mixer.input_count as usize]; caps.mixer.output_count as usize];
    let mut coefficient_params = MixerCoefficientParams(entries);
    Protocol::cache_extension_whole_params(
        req,
        node,
        sections,
        caps,
        &mut coefficient_params,
        TIMEOUT_MS,
    )?;

    println!("Mixer:");

    println!("  Saturation:");
    saturation_params
        .0
        .iter()
        .enumerate()
        .for_each(|(i, saturation)| {
            println!("    dst {}: {}", i, saturation);
        });

    println!("  Coefficiency:");
    coefficient_params
        .0
        .iter()
        .enumerate()
        .for_each(|(dst, coefs)| {
            coefs
                .iter()
                .enumerate()
                .for_each(|(src, &coef)| println!("    dst {} <- src {}: {}", dst, src, coef));
        });

    Ok(())
}

fn print_peak(
    req: &FwReq,
    node: &FwNode,
    sections: &ExtensionSections,
    caps: &ExtensionCaps,
) -> Result<(), Error> {
    let entries = vec![RouterEntry::default(); caps.router.maximum_entry_count as usize];
    let mut params = PeakParams(RouterParams(entries));
    Protocol::cache_extension_whole_params(req, node, sections, caps, &mut params, TIMEOUT_MS)?;
    println!("Peak:");
    params.0 .0.iter().enumerate().for_each(|(i, entry)| {
        println!("  entry {}: 0x{:04x}", i, entry.peak);
    });

    Ok(())
}

const RATE_MODES: [RateMode; 3] = [RateMode::Low, RateMode::Middle, RateMode::High];

fn print_current_router_entries(
    req: &FwReq,
    node: &FwNode,
    sections: &ExtensionSections,
    caps: &ExtensionCaps,
) -> Result<(), Error> {
    println!("Current router entries:");
    RATE_MODES.iter().try_for_each(|&rate_mode| {
        let entries = RouterParams(vec![
            Default::default();
            caps.router.maximum_entry_count as usize
        ]);
        let mut params = CurrentRouterParams { entries, rate_mode };
        Protocol::cache_extension_whole_params(req, node, sections, caps, &mut params, TIMEOUT_MS)
            .map(|_| {
                println!("  {:?}:", rate_mode);
                params.entries.0.iter().enumerate().for_each(|(i, entry)| {
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
    entry
        .enable_ac3
        .iter()
        .enumerate()
        .filter(|&(_, enabled)| *enabled)
        .for_each(|(i, enabled)| {
            println!("        ch {}: {}", i, enabled);
        });
}

fn print_current_stream_format_entries(
    req: &FwReq,
    node: &FwNode,
    sections: &ExtensionSections,
    caps: &ExtensionCaps,
) -> Result<(), Error> {
    println!("Current stream format entries:");
    RATE_MODES.iter().try_for_each(|&rate_mode| {
        let pair = StreamFormatParams {
            tx_entries: Vec::with_capacity(caps.general.max_tx_streams as usize),
            rx_entries: Vec::with_capacity(caps.general.max_rx_streams as usize),
        };
        let mut params = CurrentStreamFormatParams { pair, rate_mode };
        Protocol::cache_extension_whole_params(req, node, sections, caps, &mut params, TIMEOUT_MS)
            .map(|_| {
                println!("  {:?}:", rate_mode);
                params
                    .pair
                    .tx_entries
                    .iter()
                    .enumerate()
                    .for_each(|(i, entry)| {
                        println!("    Tx stream {}:", i);
                        print_stream_format_entry(entry);
                    });
                params
                    .pair
                    .rx_entries
                    .iter()
                    .enumerate()
                    .for_each(|(i, entry)| {
                        println!("    Rx stream {}:", i);
                        print_stream_format_entry(entry);
                    });
            })
    })
}

fn print_standalone_config(
    req: &FwReq,
    node: &FwNode,
    sections: &ExtensionSections,
    caps: &ExtensionCaps,
) -> Result<(), Error> {
    println!("Standalone configurations:");
    let mut params = StandaloneParameters::default();
    Protocol::cache_extension_whole_params(req, node, sections, caps, &mut params, TIMEOUT_MS)?;
    println!(
        "  clock source: {}",
        clock_source_to_string(&params.clock_source)
    );
    println!("  AES high rate: {}", params.aes_high_rate);
    println!("  ADAT mode: {:?}", params.adat_mode);
    println!(
        "  Word clock params: {:?}, {} / {}",
        params.word_clock_param.mode,
        params.word_clock_param.rate.numerator,
        params.word_clock_param.rate.denominator
    );
    println!(
        "  Internal rate: {}",
        clock_rate_to_string(&params.internal_rate)
    );
    Ok(())
}

fn main() {
    let code = std::env::args().nth(1)
        .ok_or("At least one argument is required for path to special file of firewire character device".to_string())
        .and_then(|path| {
            let node = FwNode::new();
            node.open(&path, 0)
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

            let mut req = FwReq::new();
            let mut sections = ExtensionSections::default();
            let result = Protocol::read_extension_sections(&req, &node, &mut sections, TIMEOUT_MS)
                .and_then(|_| {
                    print_sections(&sections);
                    let mut caps = ExtensionCaps::default();
                    Protocol::read_extension_caps(
                        &mut req,
                        &mut node,
                        &sections,
                        &mut caps,
                        TIMEOUT_MS
                    )?;
                    print_caps(&caps);
                    print_mixer(&mut req, &mut node, &sections, &caps)?;
                    print_peak(&mut req, &mut node, &sections, &caps)?;
                    print_current_router_entries(&mut req, &mut node, &sections, &caps)?;
                    print_current_stream_format_entries(&mut req, &mut node, &sections, &caps)?;
                    print_standalone_config(&mut req, &mut node, &sections, &caps)?;
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
"###
    );
}
