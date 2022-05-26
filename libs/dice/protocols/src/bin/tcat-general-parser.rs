// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::{Error, FileError, MainContext, MainLoop};

use hinawa::FwReq;
use hinawa::{FwNode, FwNodeError, FwNodeExt};

use dice_protocols::tcat::{
    ext_sync_section::*, global_section::*, rx_stream_format_section::*,
    tx_stream_format_section::*, *,
};

use std::sync::Arc;
use std::thread;

const TIMEOUT_MS: u32 = 20;

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
    req: &mut FwReq,
    node: &mut FwNode,
    sections: &GeneralSections,
) -> Result<(), Error> {
    let owner = GlobalSectionProtocol::read_owner_addr(req, node, sections, TIMEOUT_MS)?;
    println!("Owner:");
    println!("  node ID:        0x{:04x}", owner >> 48);
    println!("  offset:         0x{:012x}", owner & 0xffffffffu64);

    let latest_notification =
        GlobalSectionProtocol::read_latest_notification(req, node, sections, TIMEOUT_MS)?;
    println!("Last Notification:0x{:08x}", latest_notification);

    let nickname = GlobalSectionProtocol::read_nickname(req, node, sections, TIMEOUT_MS)?;
    println!("Nickname:         '{}'", nickname);

    let config = GlobalSectionProtocol::read_clock_config(req, node, sections, TIMEOUT_MS)?;
    println!("Clock configureation:");
    println!("  rate:           {}", config.rate);
    println!("  src:            {}", config.src);

    let enabled = GlobalSectionProtocol::read_enabled(req, node, sections, TIMEOUT_MS)?;
    println!("Enabled:          {}", enabled);

    let status = GlobalSectionProtocol::read_clock_status(req, node, sections, TIMEOUT_MS)?;
    println!("Status:");
    println!("  rate:           {}", status.rate);
    println!("  source is locked:  {}", status.src_is_locked);

    let curr_rate = GlobalSectionProtocol::read_current_rate(req, node, sections, TIMEOUT_MS)?;
    println!("Sampling rate:    {}", curr_rate);

    let version = GlobalSectionProtocol::read_version(req, node, sections, TIMEOUT_MS)?;
    println!("Version:          0x{:08x}", version);

    let labels = GlobalSectionProtocol::read_clock_source_labels(req, node, sections, TIMEOUT_MS)?;
    let caps = GlobalSectionProtocol::read_clock_caps(req, node, sections, TIMEOUT_MS)?;

    let rates = caps
        .get_rate_entries()
        .iter()
        .map(|r| r.to_string())
        .collect::<Vec<_>>();
    let srcs = caps
        .get_src_entries(&labels)
        .iter()
        .map(|s| s.get_label(&labels, false).unwrap())
        .collect::<Vec<_>>();
    println!("Clock capabilities:");
    println!("  rate:           {}", rates.join(", "));
    println!("  src:            {}", srcs.join(", "));

    let ext_srcs = ExtSourceStates::get_entries(&caps, &labels);
    let states = GlobalSectionProtocol::read_clock_source_states(req, node, sections, TIMEOUT_MS)?;
    println!("Clock states:");
    let locked = ext_srcs
        .iter()
        .filter(|s| s.is_locked(&states))
        .map(|s| s.get_label(&labels, true).unwrap())
        .collect::<Vec<_>>();
    let slipped = ext_srcs
        .iter()
        .filter(|s| s.is_slipped(&states))
        .map(|s| s.get_label(&labels, true).unwrap())
        .collect::<Vec<_>>();
    println!("  locked:         {}", locked.join(", "));
    println!("  slipped:        {}", slipped.join(", "));

    Ok(())
}

fn print_tx_stream_formats(
    req: &mut FwReq,
    node: &mut FwNode,
    sections: &GeneralSections,
) -> Result<(), Error> {
    let entries = TxStreamFormatSectionProtocol::read_entries(req, node, sections, TIMEOUT_MS)?;
    println!("Tx stream format entries:");
    entries.iter().enumerate().for_each(|(i, entry)| {
        println!("  Stream {}:", i);
        println!("    iso channel:  {}", entry.iso_channel);
        println!("    pcm:          {}", entry.pcm);
        println!("    midi:         {}", entry.midi);
        println!("    speed:        {}", entry.speed);
        println!("    channel name:");
        entry.labels.iter().enumerate().for_each(|(i, label)| {
            println!("      ch {}:       {}", i, label);
        });
        println!("    IEC 60958 parameters:");
        entry
            .iec60958
            .iter()
            .enumerate()
            .filter(|(_, param)| param.enable)
            .for_each(|(i, param)| {
                println!(
                    "      ch {}: cap: {}, enable: {}",
                    i, param.cap, param.enable
                );
            });
    });
    Ok(())
}

fn print_rx_stream_formats(
    req: &mut FwReq,
    node: &mut FwNode,
    sections: &GeneralSections,
) -> Result<(), Error> {
    let entries = RxStreamFormatSectionProtocol::read_entries(req, node, sections, TIMEOUT_MS)?;
    println!("Rx stream format entries:");
    entries.iter().enumerate().for_each(|(i, entry)| {
        println!("  Stream {}:", i);
        println!("    iso channel:  {}", entry.iso_channel);
        println!("    start:        {}", entry.start);
        println!("    pcm:          {}", entry.pcm);
        println!("    midi:         {}", entry.midi);
        println!("    channel name:");
        entry.labels.iter().enumerate().for_each(|(i, label)| {
            println!("      ch {}:       {}", i, label);
        });
        println!("    IEC 60958 parameters:");
        entry
            .iec60958
            .iter()
            .enumerate()
            .filter(|(_, param)| param.enable)
            .for_each(|(i, param)| {
                println!(
                    "      ch {}: cap: {}, enable: {}",
                    i, param.cap, param.enable
                );
            });
    });
    Ok(())
}

fn print_ext_sync(req: &mut FwReq, node: &mut FwNode, sections: &GeneralSections) {
    let _ = ExtSyncSectionProtocol::read_block(req, node, sections, TIMEOUT_MS).map(|blk| {
        println!("External sync:");
        println!("  source:         {}", blk.get_sync_src());
        println!("  locked:         {}", blk.get_sync_src_locked());
        println!("  rate:           {}", blk.get_sync_src_rate());
        print!("  ADAT user data: ");
        if let Some(bits) = blk.get_sync_src_adat_user_data() {
            println!("{:02x}", bits);
        } else {
            println!("N/A");
        }
    });
}

fn main() {
    let code = std::env::args().nth(1)
        .ok_or("At least one argument is required for path to special file of FireWire character device".to_string())
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

            let mut req = FwReq::new();
            let result = GeneralProtocol::read_general_sections(&mut req, &mut node, TIMEOUT_MS)
                .and_then(|sections| {
                    print_sections(&sections);
                    print_global_section(&mut req, &mut node, &sections)?;
                    print_tx_stream_formats(&mut req, &mut node, &sections)?;
                    print_rx_stream_formats(&mut req, &mut node, &sections)?;
                    print_ext_sync(&mut req, &mut node, &sections);
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
