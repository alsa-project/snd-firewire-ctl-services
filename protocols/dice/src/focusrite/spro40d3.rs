// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2025 Andreas Persson

//! Protocol specific to Focusrite Saffire Pro 40 D3.
//!
//! The module includes structure and its implementation for protocol
//! defined by Focusrite for Saffire Pro 40 D3 (TCD3070 ASIC).

use {
    super::*,
    hinawa::{
        prelude::{FwReqExtManual, FwRespExt, FwRespExtManual},
        FwTcode, FwResp, FwRcode,
    },
    std::{sync::mpsc, time::Duration},
};

/// Protocol implementation specific to Saffire Pro 40 D3.
#[derive(Default, Debug)]
pub struct SPro40D3Protocol {
    req: FwReq,
    resp: FwResp,
    write_address: u64,
    read_address: u64,
    rx: Option<mpsc::Receiver<(u32, u16)>>,
    counter: u16,
    pub master_meter: [i32; 2],
    pub mixer_meter: [i32; 18],
}

const NONE: u32 = 0;
const ANALOG: u32 = 0x80;
const SPDIF: u32 = 0x180;
const ADAT: u32 = 0x200;
const MIXER: u32 = 0x300;
const STREAM: u32 = 0x400;

impl TcatOperation for SPro40D3Protocol {}

impl TcatGlobalSectionSpecification for SPro40D3Protocol {}

fn deserialize_address(val: &mut u64, raw: &[u8]) {
    let mut a1 = 0u32;
    deserialize_u32(&mut a1, &raw[..4]);
    let mut a2 = 0u32;
    deserialize_u32(&mut a2, &raw[4..8]);
    *val = (a1 as u64) | ((a2 as u64) << 32);
}


fn serialize_route(from: u32, to: u32, raw: &mut [u8]) {
    let source = if from < 1 {
        NONE
    } else if from < 9 {
        ANALOG + from - 1
    } else if from < 11 {
        SPDIF + from - 9
    } else if from < 19 {
        ADAT + from - 11
    } else if from < 39 {
        STREAM + from - 19
    } else {
        MIXER + from - 39
    };
    let val = (source << 12) | to;
    serialize_u32(&val, raw)
}

impl SPro40D3Protocol {
    pub fn init_communication(&mut self, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let base_address = 0xffffe0400000u64;
        let mut frame = vec![0u8; 48];

        self.req.transaction(node, FwTcode::ReadBlockRequest, base_address, 48,
                             &mut frame, timeout_ms)?;
        deserialize_address(&mut self.write_address, &frame[32..40]);
        deserialize_address(&mut self.read_address, &frame[40..48]);
        let mut notification_address = 0u64;
        deserialize_address(&mut notification_address, &frame[8..16]);

        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        self.resp.connect_requested(
            move |_, _tcode, _, _src, _, _, _, _, frame| {
                let rlen = ((frame[6] as u16) << 8) | (frame[7] as u16);
                let mut rstatus = 0u32;
                deserialize_u32(&mut rstatus, &frame[8..12]);
                let _ = tx.send((rstatus, rlen));

                FwRcode::Complete
            });
        
        self.resp.reserve_within_region(node, 0, 0x1000000000000, 16)?;
        let new_notification_address = self.resp.offset();

        if new_notification_address != notification_address {
            let mut data = vec![0u8; 16];

            serialize_u32(&(notification_address as u32), &mut data[0..]);
            serialize_u32(&((notification_address >> 32) as u32),
                          &mut data[4..]);
            serialize_u32(&(new_notification_address as u32), &mut data[8..]);
            serialize_u32(&((new_notification_address >> 32) as u32),
                          &mut data[12..]);

            self.req.transaction(node, FwTcode::LockCompareSwap,
                                 base_address + 8, 8, &mut data, timeout_ms)?;
        }
        
        Ok(())
    }

    fn send_message(
        &mut self,
        node: &FwNode,
        len: usize,
        message: &mut [u8],
        timeout_ms: u32,
    ) -> Result<u32, Error> {
        message[4] = (self.counter >> 8) as u8;
        message[5] = self.counter as u8;
        message[6] = ((len - 16) >> 8) as u8;
        message[7] = (len - 16) as u8;

        self.counter = self.counter.wrapping_add(1);
        
        self.req.transaction(node, FwTcode::WriteBlockRequest,
                             self.write_address, len,
                             message, timeout_ms)?;

        let (rstatus,rlen) = self.rx.as_ref().unwrap()
            .recv_timeout(Duration::from_millis(timeout_ms.into()))
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause.to_string()))?;

        self.req.transaction(node, FwTcode::ReadBlockRequest,
                             self.read_address, (rlen + 16).into(),
                             message, timeout_ms)?;
        Ok(rstatus)
    }

    fn send_command(
        &mut self,
        node: &FwNode,
        len: usize,
        message: &mut [u8],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let rstatus = self.send_message(node, len, message, timeout_ms)?;
        if rstatus != 0 {
            self.counter = 0;

            let mut reset = vec![0u8; 16];
            reset[0] = 0x80;
            self.send_message(node, reset.len(), &mut reset, timeout_ms)?;

            self.send_message(node, len, message, timeout_ms)?;
        }
        Ok(())
    }
    
    pub fn set_volume(
        &mut self,
        node: &FwNode,
        channel: usize,
        mix: usize,
        volume: i32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut frame: Vec<u8> = vec![
            0x80, 0x00, 0x20, 0x02,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            (volume >> 8) as u8, volume as u8,
            channel as u8, mix as u8
        ];
        self.send_command(node, frame.len(), &mut frame, timeout_ms)
    }

    pub fn get_master_meter(
        &mut self,
        node: &FwNode,
        current_rate: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut frame: Vec<u8> = vec![
            0x80, 0x00, 0x10, 0x01,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 2, 0, if current_rate > 48000 {0x32} else {0x3a},
            0, 0, 0, 0
        ];
        self.send_command(node, frame.len(), &mut frame, timeout_ms)?;

        deserialize_i32(&mut self.master_meter[0], &frame[16..20]);
        deserialize_i32(&mut self.master_meter[1], &frame[20..24]);
        Ok(())
    }

    pub fn get_mixer_meter(
        &mut self,
        node: &FwNode,
        current_rate: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let cmd: [u8; 24] = [
            0x80, 0x00, 0x10, 0x01,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 18, 0, if current_rate > 48000 {0x20} else {0x28},
            0, 0, 0, 0
        ];
        let mut frame: Vec<u8> = vec![0; 88];
        frame[..cmd.len()].copy_from_slice(&cmd);
        self.send_command(node, cmd.len(), &mut frame, timeout_ms)?;

        self
            .mixer_meter
            .iter_mut()
            .enumerate()
            .for_each(|(i, m)| {
                let pos = 16 + i * 4;
                deserialize_i32(m, &frame[pos..(pos + 4)]);
            });
        Ok(())
    }

    pub fn set_routing(
        &mut self,
        node: &FwNode,
        router_out_src: &[u32],
        router_mixer_src: &[u32],
        router_meter_src: &[u32],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        // There are two separate routing tables, one for sample rates
        // 44100 and 48000 with eight ADAT channels, and one for 88200
        // and 96000 with four ADAT. Both tables are updated here, to
        // be ready if the sample rate is changed.
        self.set_routing_low_rate(node, router_out_src, router_mixer_src, router_meter_src, timeout_ms)?;
        self.set_routing_high_rate(node, router_out_src, router_mixer_src, router_meter_src, timeout_ms)
    }
    
    fn set_routing_low_rate(
        &mut self,
        node: &FwNode,
        router_out_src: &[u32],
        router_mixer_src: &[u32],
        router_meter_src: &[u32],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut frame: Vec<u8> = vec![0; 316];
        frame[..4].copy_from_slice(&[0x80, 0x00, 0x30, 0x02]);

        // to stream, fixed
        for i in 0..18 {
            serialize_route(1 + i as u32, STREAM + i as u32,
                            &mut frame[20 + i * 4..][..4]);
        }

        // to outputs
        for i in 0..5 {
            serialize_route(router_out_src[i * 2],
                            ANALOG + 8 - i as u32 * 2,
                            &mut frame[92 + i * 8..][..4]); // korta: 76 osv
            serialize_route(router_out_src[1 + i * 2],
                            ANALOG + 9 - i as u32 * 2,
                            &mut frame[96 + i * 8..][..4]);
        }
        serialize_route(router_out_src[10],
                        SPDIF, &mut frame[132..136]);
        serialize_route(router_out_src[11],
                        SPDIF + 1, &mut frame[136..140]);

        for i in 0..8 {
            serialize_route(router_out_src[12 + i],
                            ADAT + i as u32,
                            &mut frame[140 + i * 4..][..4]);
        }
        // loopback
        serialize_route(router_out_src[20],
                        STREAM + 18, &mut frame[172..176]);
        serialize_route(router_out_src[21],
                        STREAM + 19, &mut frame[176..180]);

        // to mixer
        for i in 0..8 {
            serialize_route(router_mixer_src[i],
                            MIXER + i as u32,
                            &mut frame[180 + i * 4..][..4]);
        }
        for i in 0..8 {
            serialize_route(router_mixer_src[8 + i],
                            MIXER + 8 + i as u32,
                            &mut frame[212 + i * 4..][..4]);
        }
        serialize_route(router_mixer_src[16],
                        MIXER + 16, &mut frame[244..248]);
        serialize_route(router_mixer_src[17],
                        MIXER + 17, &mut frame[248..252]);

        // master meter
        serialize_route(router_meter_src[0], 0x0, &mut frame[252..256]);
        serialize_route(router_meter_src[1], 0x0, &mut frame[256..260]);

        self.send_command(node, frame.len(), &mut frame, timeout_ms)
    }

    fn set_routing_high_rate(
        &mut self,
        node: &FwNode,
        router_out_src: &[u32],
        router_mixer_src: &[u32],
        router_meter_src: &[u32],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut frame: Vec<u8> = vec![0; 284];
        frame[..4].copy_from_slice(&[0x80, 0x00, 0x30, 0x02]);
        frame[17] = 1;

        // to stream, fixed
        for i in 0..14 {
            serialize_route(1 + i as u32, STREAM + i as u32,
                            &mut frame[20 + i * 4..][..4]);
        }

        // to outputs
        for i in 0..5 {
            serialize_route(router_out_src[i * 2],
                            ANALOG + 8 - i as u32 * 2,
                            &mut frame[76 + i * 8..][..4]);
            serialize_route(router_out_src[1 + i * 2],
                            ANALOG + 9 - i as u32 * 2,
                            &mut frame[80 + i * 8..][..4]);
        }
        serialize_route(router_out_src[10],
                        SPDIF, &mut frame[116..120]);
        serialize_route(router_out_src[11],
                        SPDIF + 1, &mut frame[120..124]);

        for i in 0..4 {
            serialize_route(router_out_src[12 + i],
                            ADAT + i as u32,
                            &mut frame[124 + i * 4..][..4]);
        }
        // loopback
        serialize_route(router_out_src[20],
                        STREAM + 14, &mut frame[140..144]);
        serialize_route(router_out_src[21],
                        STREAM + 15, &mut frame[144..148]);

        // to mixer
        for i in 0..8 {
            serialize_route(router_mixer_src[i],
                            MIXER + i as u32,
                            &mut frame[148 + i * 4..][..4]);
        }
        for i in 0..8 {
            serialize_route(router_mixer_src[8 + i],
                            MIXER + 8 + i as u32,
                            &mut frame[180 + i * 4..][..4]);
        }
        serialize_route(router_mixer_src[16],
                        MIXER + 16, &mut frame[212..216]);
        serialize_route(router_mixer_src[17],
                        MIXER + 17, &mut frame[216..220]);

        // master meter
        serialize_route(router_meter_src[0], 0x0, &mut frame[220..224]);
        serialize_route(router_meter_src[1], 0x0, &mut frame[224..228]);

        self.send_command(node, frame.len(), &mut frame, timeout_ms)
    }
}
