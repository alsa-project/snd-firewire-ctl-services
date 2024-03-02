// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    firewire_dice_protocols as protocols,
    glib::{Error, FileError, MainContext, MainLoop},
    hinawa::{prelude::FwNodeExt, FwNode, FwNodeError, FwReq},
    protocols::tcat::{global_section::*, *},
    std::sync::Arc,
    std::thread,
};

const TIMEOUT_MS: u32 = 20;

struct Protocol;

impl TcatOperation for Protocol {}

impl TcatGlobalSectionSpecification for Protocol {}

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

fn print_sections(sections: &GeneralSections) {
    println!("General sections:");
    println!(
        "  global:             offset 0x{:x}, size: 0x{:x}",
        sections.global.offset, sections.global.size
    );
    println!(
        "  tx-stream-format:   offset 0x{:x}, size: 0x{:x}",
        sections.tx_stream_format.offset, sections.tx_stream_format.size
    );
    println!(
        "  rx-stream-format:   offset 0x{:x}, size: 0x{:x}",
        sections.rx_stream_format.offset, sections.rx_stream_format.size
    );
    println!(
        "  ext_sync:           offset 0x{:x}, size: 0x{:x}",
        sections.ext_sync.offset, sections.ext_sync.size
    );
}

fn print_global_section(
    req: &FwReq,
    node: &FwNode,
    sections: &mut GeneralSections,
) -> Result<(), Error> {
    let mut params = GlobalParameters::default();
    Protocol::whole_cache(req, node, &sections.global, &mut params, TIMEOUT_MS)?;

    println!("Parameters in global section:");
    println!("  Owner:");
    println!("    node ID:        0x{:04x}", params.owner >> 48);
    println!(
        "    offset:         0x{:012x}",
        params.owner & 0xffffffffu64
    );

    println!("  Last Notification:0x{:08x}", params.latest_notification);

    println!("  Nickname:         '{}'", params.nickname);

    println!("  Clock configureation:");
    println!(
        "    rate:           {}",
        clock_rate_to_string(&params.clock_config.rate)
    );
    println!(
        "    src:            {}",
        clock_source_to_string(&params.clock_config.src)
    );

    println!("  Enabled:          {}", params.enable);

    println!("  Status:");
    println!(
        "    rate:           {}",
        clock_rate_to_string(&params.clock_status.rate)
    );
    println!(
        "    source is locked:  {}",
        params.clock_status.src_is_locked
    );

    println!("  Sampling rate:    {}", params.current_rate);

    println!("  Version:          0x{:08x}", params.version);

    let rates: Vec<String> = params
        .avail_rates
        .iter()
        .map(|r| clock_rate_to_string(r))
        .collect();
    let srcs: Vec<String> = params
        .avail_sources
        .iter()
        .map(|s| clock_source_to_string(s))
        .collect();
    println!("  Clock capabilities:");
    println!("    rate:           {}", rates.join(", "));
    println!("    src:            {}", srcs.join(", "));

    let locked: Vec<String> = params
        .external_source_states
        .sources
        .iter()
        .zip(params.external_source_states.locked.iter())
        .filter_map(|(src, state)| {
            params
                .clock_source_labels
                .iter()
                .find(|(s, _)| s.eq(src))
                .map(|(_, label)| (label, state))
        })
        .map(|(label, state)| format!("\"{}\"({})", label, state))
        .collect();
    let slipped: Vec<String> = params
        .external_source_states
        .sources
        .iter()
        .zip(params.external_source_states.slipped.iter())
        .filter_map(|(src, state)| {
            params
                .clock_source_labels
                .iter()
                .find(|(s, _)| s.eq(src))
                .map(|(_, label)| (label, state))
        })
        .map(|(label, state)| format!("\"{}\"({})", label, state))
        .collect();
    println!("  External clock states:");
    println!("    locked: {}", locked.join(", "));
    println!("    slipped: {}", slipped.join(", "));

    Ok(())
}

fn print_tx_stream_formats(
    req: &FwReq,
    node: &FwNode,
    sections: &mut GeneralSections,
) -> Result<(), Error> {
    let mut params = TxStreamFormatParameters::default();
    Protocol::whole_cache(
        req,
        node,
        &sections.tx_stream_format,
        &mut params,
        TIMEOUT_MS,
    )?;

    let entries = &params.0;

    println!("Parameters in tx stream format section");
    println!("  Tx stream format entries:");
    entries.iter().enumerate().for_each(|(i, entry)| {
        println!("    Stream {}:", i);
        println!("      iso channel:  {}", entry.iso_channel);
        println!("      pcm:          {}", entry.pcm);
        println!("      midi:         {}", entry.midi);
        println!("      speed:        {}", entry.speed);
        println!("      channel name:");
        entry.labels.iter().enumerate().for_each(|(i, label)| {
            println!("      ch {}:       {}", i, label);
        });
        println!("      IEC 60958 parameters:");
        entry
            .iec60958
            .iter()
            .enumerate()
            .filter(|(_, param)| param.enable)
            .for_each(|(i, param)| {
                println!(
                    "        ch {}: cap: {}, enable: {}",
                    i, param.cap, param.enable
                );
            });
    });
    Ok(())
}

fn print_rx_stream_formats(
    req: &FwReq,
    node: &FwNode,
    sections: &mut GeneralSections,
) -> Result<(), Error> {
    let mut params = RxStreamFormatParameters::default();
    Protocol::whole_cache(
        req,
        node,
        &sections.rx_stream_format,
        &mut params,
        TIMEOUT_MS,
    )?;

    let entries = &params.0;
    println!("Parameters in tx stream format section");
    println!("  Rx stream format entries:");
    entries.iter().enumerate().for_each(|(i, entry)| {
        println!("    Stream {}:", i);
        println!("      iso channel:  {}", entry.iso_channel);
        println!("      start:        {}", entry.start);
        println!("      pcm:          {}", entry.pcm);
        println!("      midi:         {}", entry.midi);
        println!("      channel name:");
        entry.labels.iter().enumerate().for_each(|(i, label)| {
            println!("        ch {}:       {}", i, label);
        });
        println!("      IEC 60958 parameters:");
        entry
            .iec60958
            .iter()
            .enumerate()
            .filter(|(_, param)| param.enable)
            .for_each(|(i, param)| {
                println!(
                    "        ch {}: cap: {}, enable: {}",
                    i, param.cap, param.enable
                );
            });
    });
    Ok(())
}

fn print_ext_sync(req: &FwReq, node: &FwNode, sections: &mut GeneralSections) -> Result<(), Error> {
    let mut params = ExtendedSyncParameters::default();
    Protocol::whole_cache(req, node, &sections.ext_sync, &mut params, TIMEOUT_MS)?;

    println!("Parameters in external synchronization section");
    println!("  External sync:");
    println!(
        "    source:         {}",
        clock_source_to_string(&params.clk_src)
    );
    println!("    locked:         {}", params.clk_src_locked);
    println!(
        "    rate:           {}",
        clock_rate_to_string(&params.clk_rate)
    );
    print!("    ADAT user data: ");
    if let Some(bits) = params.adat_user_data {
        println!("{:02x}", bits);
    } else {
        println!("N/A");
    }

    Ok(())
}

fn main() {
    let code = std::env::args().nth(1)
        .ok_or("At least one argument is required for path to special file of FireWire character device".to_string())
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
        .and_then(|(node, src)| {
            let ctx = MainContext::new();
            let _ = src.attach(Some(&ctx));
            let dispatcher = Arc::new(MainLoop::new(Some(&ctx), false));
            let d = dispatcher.clone();
            let th = thread::spawn(move || d.run());

            let mut sections = GeneralSections::default();

            let req = FwReq::new();
            let result = Protocol::read_general_sections(&req, &node, &mut sections, TIMEOUT_MS)
                .and_then(|_| {
                    print_sections(&sections);
                    print_global_section(&req, &node, &mut sections)?;
                    print_tx_stream_formats(&req, &node, &mut sections)?;
                    print_rx_stream_formats(&req, &node, &mut sections)?;
                    print_ext_sync(&req, &node, &mut sections)?;
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
  tcat-general-parser CDEV

  where:
    CDEV:       The path to special file of firewire character device, typically '/dev/fw1'.
"###
    );
}
