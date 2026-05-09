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
        FwRcode, FwResp, FwTcode,
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
}

pub const MIX_COUNT: usize = 16;
pub const OUTPUT_COUNT: usize = 22;
pub const CHANNEL_COUNT: usize = 18;
pub const MASTER_METER_COUNT: usize = 2;

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

fn serialize_set_volume(channel: usize, mix: usize, volume: i32, frame: &mut [u8]) {
    frame[..4].copy_from_slice(&[0x80, 0x00, 0x20, 0x02]);
    frame[16] = (volume >> 8) as u8;
    frame[17] = volume as u8;
    frame[18] = channel as u8;
    frame[19] = mix as u8;
}

fn serialize_get_meter(pos: u8, len: u8, frame: &mut [u8]) {
    frame[..4].copy_from_slice(&[0x80, 0x00, 0x10, 0x01]);
    frame[17] = len;
    frame[19] = pos;
}

fn deserialize_meter(meter: &mut [i32], frame: &[u8]) {
    meter.iter_mut().enumerate().for_each(|(i, m)| {
        let pos = 16 + i * 4;
        deserialize_i32(m, &frame[pos..(pos + 4)]);
    });
}

fn serialize_get_control(pos: u32, len: u32, frame: &mut [u8]) {
    frame[..4].copy_from_slice(&[0x80, 0x80, 0x00, 0x00]);
    serialize_u32(&pos, &mut frame[16..20]);
    serialize_u32(&len, &mut frame[20..24]);
}

fn serialize_set_control(pos: u32, len: u32, value: &[u8], frame: &mut [u8]) {
    frame[..4].copy_from_slice(&[0x80, 0x80, 0x00, 0x01]);
    serialize_u32(&pos, &mut frame[16..20]);
    serialize_u32(&len, &mut frame[20..24]);
    frame[24..].copy_from_slice(value);
}

fn serialize_write_sw_notice(frame: &mut [u8], notice: u32) {
    frame[..4].copy_from_slice(&[0x80, 0x80, 0x00, 0x02]);
    serialize_u32(&notice, &mut frame[16..20]);
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

fn serialize_routing_low_rate(
    router_out_src: &[u32],
    router_mixer_src: &[u32],
    router_meter_src: &[u32],
    frame: &mut [u8],
) {
    frame[..4].copy_from_slice(&[0x80, 0x00, 0x30, 0x02]);

    // to stream, fixed
    for i in 0..18 {
        serialize_route(
            1 + i as u32,
            STREAM + i as u32,
            &mut frame[20 + i * 4..][..4],
        );
    }

    // to outputs
    for i in 0..5 {
        serialize_route(
            router_out_src[i * 2],
            ANALOG + 8 - i as u32 * 2,
            &mut frame[92 + i * 8..][..4],
        );
        serialize_route(
            router_out_src[1 + i * 2],
            ANALOG + 9 - i as u32 * 2,
            &mut frame[96 + i * 8..][..4],
        );
    }
    serialize_route(router_out_src[10], SPDIF, &mut frame[132..136]);
    serialize_route(router_out_src[11], SPDIF + 1, &mut frame[136..140]);

    for i in 0..8 {
        serialize_route(
            router_out_src[12 + i],
            ADAT + i as u32,
            &mut frame[140 + i * 4..][..4],
        );
    }
    // loopback
    serialize_route(router_out_src[20], STREAM + 18, &mut frame[172..176]);
    serialize_route(router_out_src[21], STREAM + 19, &mut frame[176..180]);

    // to mixer
    for i in 0..18 {
        serialize_route(
            router_mixer_src[i],
            MIXER + i as u32,
            &mut frame[180 + i * 4..][..4],
        );
    }

    // master meter
    serialize_route(router_meter_src[0], 0x0, &mut frame[252..256]);
    serialize_route(router_meter_src[1], 0x0, &mut frame[256..260]);
}

fn serialize_routing_high_rate(
    router_out_src: &[u32],
    router_mixer_src: &[u32],
    router_meter_src: &[u32],
    frame: &mut [u8],
) {
    frame[..4].copy_from_slice(&[0x80, 0x00, 0x30, 0x02]);
    frame[17] = 1;

    // to stream, fixed
    for i in 0..14 {
        serialize_route(
            1 + i as u32,
            STREAM + i as u32,
            &mut frame[20 + i * 4..][..4],
        );
    }

    // to outputs
    for i in 0..5 {
        serialize_route(
            router_out_src[i * 2],
            ANALOG + 8 - i as u32 * 2,
            &mut frame[76 + i * 8..][..4],
        );
        serialize_route(
            router_out_src[1 + i * 2],
            ANALOG + 9 - i as u32 * 2,
            &mut frame[80 + i * 8..][..4],
        );
    }
    serialize_route(router_out_src[10], SPDIF, &mut frame[116..120]);
    serialize_route(router_out_src[11], SPDIF + 1, &mut frame[120..124]);

    for i in 0..4 {
        serialize_route(
            router_out_src[12 + i],
            ADAT + i as u32,
            &mut frame[124 + i * 4..][..4],
        );
    }
    // loopback
    serialize_route(router_out_src[20], STREAM + 14, &mut frame[140..144]);
    serialize_route(router_out_src[21], STREAM + 15, &mut frame[144..148]);

    // to mixer
    for i in 0..18 {
        serialize_route(
            router_mixer_src[i],
            MIXER + i as u32,
            &mut frame[148 + i * 4..][..4],
        );
    }

    // master meter
    serialize_route(router_meter_src[0], 0x0, &mut frame[220..224]);
    serialize_route(router_meter_src[1], 0x0, &mut frame[224..228]);
}

impl SPro40D3Protocol {
    pub fn init_communication(&mut self, node: &FwNode, timeout_ms: u32) -> Result<(), Error> {
        let base_address = 0xffffe0400000u64;
        let mut frame = vec![0u8; 48];

        self.req.transaction(
            node,
            FwTcode::ReadBlockRequest,
            base_address,
            48,
            &mut frame,
            timeout_ms,
        )?;
        deserialize_address(&mut self.write_address, &frame[32..40]);
        deserialize_address(&mut self.read_address, &frame[40..48]);
        let mut notification_address = 0u64;
        deserialize_address(&mut notification_address, &frame[8..16]);

        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        self.resp
            .connect_requested(move |_, _tcode, _, _src, _, _, _, _, frame| {
                let rlen = ((frame[6] as u16) << 8) | (frame[7] as u16);
                let mut rstatus = 0u32;
                deserialize_u32(&mut rstatus, &frame[8..12]);
                let _ = tx.send((rstatus, rlen));

                FwRcode::Complete
            });

        self.resp
            .reserve_within_region(node, 0, 0x1000000000000, 16)?;
        let new_notification_address = self.resp.offset();

        if new_notification_address != notification_address {
            let mut data = vec![0u8; 16];

            serialize_u32(&(notification_address as u32), &mut data[0..]);
            serialize_u32(&((notification_address >> 32) as u32), &mut data[4..]);
            serialize_u32(&(new_notification_address as u32), &mut data[8..]);
            serialize_u32(&((new_notification_address >> 32) as u32), &mut data[12..]);

            self.req.transaction(
                node,
                FwTcode::LockCompareSwap,
                base_address + 8,
                8,
                &mut data,
                timeout_ms,
            )?;
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

        self.req.transaction(
            node,
            FwTcode::WriteBlockRequest,
            self.write_address,
            len,
            message,
            timeout_ms,
        )?;

        let (rstatus, rlen) = self
            .rx
            .as_ref()
            .unwrap()
            .recv_timeout(Duration::from_millis(timeout_ms.into()))
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause.to_string()))?;

        self.req.transaction(
            node,
            FwTcode::ReadBlockRequest,
            self.read_address,
            (rlen + 16).into(),
            message,
            timeout_ms,
        )?;
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
        let mut frame = vec![0; 20];
        serialize_set_volume(channel, mix, volume, &mut frame);
        self.send_command(node, frame.len(), &mut frame, timeout_ms)
    }

    pub fn get_master_meter(
        &mut self,
        node: &FwNode,
        current_rate: u32,
        master_meter: &mut [i32],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut frame = vec![0; 16 + MASTER_METER_COUNT * 4];
        serialize_get_meter(
            if current_rate > 48000 { 0x32 } else { 0x3a },
            MASTER_METER_COUNT as u8,
            &mut frame,
        );
        self.send_command(node, 24, &mut frame, timeout_ms)?;
        deserialize_meter(master_meter, &frame);
        Ok(())
    }

    pub fn get_mixer_meter(
        &mut self,
        node: &FwNode,
        current_rate: u32,
        mixer_meter: &mut [i32],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut frame = vec![0; 16 + CHANNEL_COUNT * 4];
        serialize_get_meter(
            if current_rate > 48000 { 0x20 } else { 0x28 },
            CHANNEL_COUNT as u8,
            &mut frame,
        );
        self.send_command(node, 24, &mut frame, timeout_ms)?;
        deserialize_meter(mixer_meter, &frame);
        Ok(())
    }

    fn get_control_u32(&mut self, node: &FwNode, pos: u32, timeout_ms: u32) -> Result<u32, Error> {
        let mut frame = vec![0; 24];
        serialize_get_control(pos, 4, &mut frame);
        self.send_command(node, 24, &mut frame, timeout_ms)?;
        let mut val = 0;
        deserialize_u32(&mut val, &frame[16..20]);
        Ok(val)
    }

    fn set_control_u32(
        &mut self,
        node: &FwNode,
        pos: u32,
        value: u32,
        notice: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut frame = vec![0; 28];
        let mut raw = [0; 4];
        serialize_u32(&value, &mut raw);
        serialize_set_control(pos, 4, &raw, &mut frame);
        self.send_command(node, 28, &mut frame, timeout_ms)?;

        frame[0..20].fill(0);
        serialize_write_sw_notice(&mut frame, notice);
        self.send_command(node, 20, &mut frame, timeout_ms)?;
        Ok(())
    }

    pub fn get_use_adat_as_spdif(&mut self, node: &FwNode, timeout_ms: u32) -> Result<u32, Error> {
        self.get_control_u32(node, 0x5c, timeout_ms)
    }

    pub fn set_use_adat_as_spdif(
        &mut self,
        node: &FwNode,
        value: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.set_control_u32(node, 0x5c, value, 4, timeout_ms)
    }

    pub fn get_active_monitor_pad(&mut self, node: &FwNode, timeout_ms: u32) -> Result<u32, Error> {
        self.get_control_u32(node, 0x40, timeout_ms)
    }

    pub fn set_active_monitor_pad(
        &mut self,
        node: &FwNode,
        value: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        self.set_control_u32(node, 0x40, value, 3, timeout_ms)
    }

    pub fn cache_dim_mute(
        &mut self,
        state: &mut OutGroupState,
        node: &FwNode,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        state.dim_enabled = self.get_dim(node, timeout_ms)? != 0;
        state.mute_enabled = self.get_mute(node, timeout_ms)? != 0;
        Ok(())
    }

    pub fn cache_output_group(
        &mut self,
        node: &FwNode,
        state: &mut OutGroupState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        state.vols = vec![0; 10];
        state.vol_mutes = vec![false; 10];
        state.vol_hwctls = vec![false; 10];
        state.mute_hwctls = vec![false; 10];
        state.dim_hwctls = vec![false; 10];

        state.mute_enabled = self.get_mute(node, timeout_ms)? != 0;
        state.dim_enabled = self.get_dim(node, timeout_ms)? != 0;

        for i in 0..(state.vols.len() / 2) {
            let pos = 0x0c + 0x08 + i as u32 * 4;
            let val = self.get_control_u32(node, pos, timeout_ms)?;
            state.vols[(i * 2)..(i * 2 + 2)]
                .iter_mut()
                .enumerate()
                .for_each(|(j, vol)| {
                    let v = ((val >> (j * 8)) & 0xff) as i8;
                    // NOTE: inverted.
                    *vol = VOL_MAX - v;
                });
        }

        for i in 0..(state.vol_hwctls.len() / 2) {
            let pos = 0x0c + 0x1c + i as u32 * 4;
            let val = self.get_control_u32(node, pos, timeout_ms)?;
            let idx = i * 2;

            state.vol_hwctls[idx..(idx + 2)]
                .iter_mut()
                .enumerate()
                .for_each(|(i, vol_hwctl)| *vol_hwctl = val & (1 << i) > 0);

            state.vol_mutes[idx..(idx + 2)]
                .iter_mut()
                .enumerate()
                .for_each(|(i, vol_mute)| *vol_mute = val & (1 << (i + 2)) > 0);
        }

        let val = self.get_control_u32(node, 0x3c, timeout_ms)?;
        state
            .dim_hwctls
            .iter_mut()
            .enumerate()
            .for_each(|(i, assign)| *assign = val & (1 << (i + 10)) > 0);
        state
            .mute_hwctls
            .iter_mut()
            .enumerate()
            .for_each(|(i, assign)| *assign = val & (1 << i) > 0);

        state.hw_knob_value = self.get_monitor_volume(node, timeout_ms)? as i8;
        Ok(())
    }

    pub fn get_dim(&mut self, node: &FwNode, timeout_ms: u32) -> Result<u32, Error> {
        self.get_control_u32(node, 0x10, timeout_ms)
    }

    pub fn set_dim(&mut self, node: &FwNode, value: u32, timeout_ms: u32) -> Result<(), Error> {
        self.set_control_u32(node, 0x10, value, 2, timeout_ms)
    }

    pub fn get_mute(&mut self, node: &FwNode, timeout_ms: u32) -> Result<u32, Error> {
        self.get_control_u32(node, 0x0c, timeout_ms)
    }

    pub fn set_mute(&mut self, node: &FwNode, value: u32, timeout_ms: u32) -> Result<(), Error> {
        self.set_control_u32(node, 0x0c, value, 2, timeout_ms)
    }

    pub fn set_output_vol(
        &mut self,
        node: &FwNode,
        params: &OutGroupState,
        prev: &mut OutGroupState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        for i in 0..(params.vols.len() / 2) {
            let idx = i * 2;
            if params.vols[idx..(idx + 2)] != prev.vols[idx..(idx + 2)] {
                let mut val = 0u32;
                params.vols[idx..(idx + 2)]
                    .iter()
                    .enumerate()
                    .for_each(|(j, &vol)| {
                        // NOTE: inverted.
                        let v = VOL_MAX - vol;
                        val |= (v as u32) << (8 * j);
                    });
                let pos = 0x0c + 0x08 + i as u32 * 4;
                self.set_control_u32(node, pos, val, 1, timeout_ms)?;

                prev.vols[idx..(idx + 2)].copy_from_slice(&params.vols[idx..(idx + 2)]);
            }
        }
        Ok(())
    }

    pub fn set_vol_hwctl_mute(
        &mut self,
        node: &FwNode,
        params: &OutGroupState,
        prev: &mut OutGroupState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        for i in 0..(params.vol_hwctls.len() / 2) {
            let idx = i * 2;
            if params.vol_hwctls[idx..(idx + 2)] != prev.vol_hwctls[idx..(idx + 2)]
                || params.vol_mutes[idx..(idx + 2)] != prev.vol_mutes[idx..(idx + 2)]
            {
                let mut val = 0u32;

                params.vol_hwctls[idx..(idx + 2)]
                    .iter()
                    .enumerate()
                    .filter(|(_, &vol_hwctl)| vol_hwctl)
                    .for_each(|(i, _)| val |= 1 << i);

                params.vol_mutes[idx..(idx + 2)]
                    .iter()
                    .enumerate()
                    .filter(|(_, &vol_mute)| vol_mute)
                    .for_each(|(i, _)| val |= 1 << (i + 2));

                let pos = 0x0c + 0x1c + i as u32 * 4;
                self.set_control_u32(node, pos, val, 1, timeout_ms)?;

                prev.vol_hwctls[idx..(idx + 2)].copy_from_slice(&params.vol_hwctls[idx..(idx + 2)]);
                prev.vol_mutes[idx..(idx + 2)].copy_from_slice(&params.vol_mutes[idx..(idx + 2)]);
            }
        }
        Ok(())
    }

    pub fn set_dim_mute_hwctl(
        &mut self,
        node: &FwNode,
        state: &OutGroupState,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut val = 0u32;
        state
            .dim_hwctls
            .iter()
            .enumerate()
            .filter(|(_, &assigned)| assigned)
            .for_each(|(i, _)| val |= 1 << (i + 10));
        state
            .mute_hwctls
            .iter()
            .enumerate()
            .filter(|(_, &assigned)| assigned)
            .for_each(|(i, _)| val |= 1 << i);
        self.set_control_u32(node, 0x3c, val, 3, timeout_ms)
    }

    pub fn get_monitor_volume(&mut self, node: &FwNode, timeout_ms: u32) -> Result<u32, Error> {
        self.get_control_u32(node, 0x54, timeout_ms)
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
        self.set_routing_low_rate(
            node,
            router_out_src,
            router_mixer_src,
            router_meter_src,
            timeout_ms,
        )?;
        self.set_routing_high_rate(
            node,
            router_out_src,
            router_mixer_src,
            router_meter_src,
            timeout_ms,
        )
    }

    fn set_routing_low_rate(
        &mut self,
        node: &FwNode,
        router_out_src: &[u32],
        router_mixer_src: &[u32],
        router_meter_src: &[u32],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut frame = vec![0; 316];
        serialize_routing_low_rate(
            router_out_src,
            router_mixer_src,
            router_meter_src,
            &mut frame,
        );
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
        let mut frame = vec![0; 284];
        serialize_routing_high_rate(
            router_out_src,
            router_mixer_src,
            router_meter_src,
            &mut frame,
        );
        self.send_command(node, frame.len(), &mut frame, timeout_ms)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn set_volume_serialize() {
        let channel = 1;
        let mix = 2;
        let volume = 0x1abc;

        #[rustfmt::skip]
        let target = &[
            0x80, 0x00, 0x20, 0x02,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x1a, 0xbc, 0x01, 0x02,
        ];

        let mut raw = vec![0; 20];
        serialize_set_volume(channel, mix, volume, &mut raw);
        println!("{:02x?}", raw);
        assert!(target.iter().eq(raw.iter()))
    }

    #[test]
    fn get_meter_serialize() {
        let pos = 0x3a;
        let len = 2;

        #[rustfmt::skip]
        let target = &[
            0x80, 0x00, 0x10, 0x01,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,

            0x00, 0x02, 0x00, 0x3a,
            0x00, 0x00, 0x00, 0x00,
        ];

        let mut raw = vec![0; 24];
        serialize_get_meter(pos, len, &mut raw);
        println!("{:02x?}", raw);
        assert!(target.iter().eq(raw.iter()))
    }

    #[test]
    fn meter_deserialize() {
        #[rustfmt::skip]
        let frame = &[
            0x80, 0x00, 0x10, 0x01,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,

            0x00, 0x00, 0x01, 0x23,
            0x00, 0x00, 0x04, 0x56,
        ];

        let target = &[0x123, 0x456];

        let mut meter = vec![0; 2];
        deserialize_meter(&mut meter, frame);
        println!("{:x?}", meter);
        assert!(target.iter().eq(meter.iter()))
    }

    #[test]
    fn routing_low_rate_serialize() {
        #[rustfmt::skip]
        let router_out_src = &[
            39, 40, 39, 40, 39, 40, 39, 40, 39, 40,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        #[rustfmt::skip]
        let router_mixer_src = &[
            1, 2, 3, 4, 5, 6, 7, 8,
            0, 0, 0, 0, 0, 0, 0, 0,
            19, 20,
        ];
        #[rustfmt::skip]
        let router_meter_src = &[
            39, 40,
        ];

        #[rustfmt::skip]
        let target = &[
            // header
            0x80, 0x00, 0x30, 0x02,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x08, 0x04, 0x00,

            // to stream
            0x00, 0x08, 0x14, 0x01,
            0x00, 0x08, 0x24, 0x02,
            0x00, 0x08, 0x34, 0x03,
            0x00, 0x08, 0x44, 0x04,
            0x00, 0x08, 0x54, 0x05,
            0x00, 0x08, 0x64, 0x06,
            0x00, 0x08, 0x74, 0x07,
            0x00, 0x18, 0x04, 0x08,
            0x00, 0x18, 0x14, 0x09,
            0x00, 0x20, 0x04, 0x0a,
            0x00, 0x20, 0x14, 0x0b,
            0x00, 0x20, 0x24, 0x0c,
            0x00, 0x20, 0x34, 0x0d,
            0x00, 0x20, 0x44, 0x0e,
            0x00, 0x20, 0x54, 0x0f,
            0x00, 0x20, 0x64, 0x10,
            0x00, 0x20, 0x74, 0x11,

            // to outputs
            0x00, 0x30, 0x00, 0x88,
            0x00, 0x30, 0x10, 0x89,
            0x00, 0x30, 0x00, 0x86,
            0x00, 0x30, 0x10, 0x87,
            0x00, 0x30, 0x00, 0x84,
            0x00, 0x30, 0x10, 0x85,
            0x00, 0x30, 0x00, 0x82,
            0x00, 0x30, 0x10, 0x83,
            0x00, 0x30, 0x00, 0x80,
            0x00, 0x30, 0x10, 0x81,
            // SPDIF out
            0x00, 0x00, 0x01, 0x80,
            0x00, 0x00, 0x01, 0x81,
            // ADAT out
            0x00, 0x00, 0x02, 0x00,
            0x00, 0x00, 0x02, 0x01,
            0x00, 0x00, 0x02, 0x02,
            0x00, 0x00, 0x02, 0x03,
            0x00, 0x00, 0x02, 0x04,
            0x00, 0x00, 0x02, 0x05,
            0x00, 0x00, 0x02, 0x06,
            0x00, 0x00, 0x02, 0x07,
            // loop back
            0x00, 0x00, 0x04, 0x12,
            0x00, 0x00, 0x04, 0x13,

            // mixer inputs
            0x00, 0x08, 0x03, 0x00,
            0x00, 0x08, 0x13, 0x01,
            0x00, 0x08, 0x23, 0x02,
            0x00, 0x08, 0x33, 0x03,
            0x00, 0x08, 0x43, 0x04,
            0x00, 0x08, 0x53, 0x05,
            0x00, 0x08, 0x63, 0x06,
            0x00, 0x08, 0x73, 0x07,
            0x00, 0x00, 0x03, 0x08,
            0x00, 0x00, 0x03, 0x09,
            0x00, 0x00, 0x03, 0x0a,
            0x00, 0x00, 0x03, 0x0b,
            0x00, 0x00, 0x03, 0x0c,
            0x00, 0x00, 0x03, 0x0d,
            0x00, 0x00, 0x03, 0x0e,
            0x00, 0x00, 0x03, 0x0f,
            0x00, 0x40, 0x03, 0x10,
            0x00, 0x40, 0x13, 0x11,

            // meters
            0x00, 0x30, 0x00, 0x00,
            0x00, 0x30, 0x10, 0x00,

            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let mut raw = vec![0; 316];
        serialize_routing_low_rate(router_out_src, router_mixer_src, router_meter_src, &mut raw);
        println!("{:02x?}", raw);
        assert!(target.iter().eq(raw.iter()))
    }

    #[test]
    fn routing_high_rate_serialize() {
        #[rustfmt::skip]
        let router_out_src = &[
            39, 40, 39, 40, 39, 40, 39, 40, 39, 40,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        #[rustfmt::skip]
        let router_mixer_src = &[
            1, 2, 3, 4, 5, 6, 7, 8,
            0, 0, 0, 0, 0, 0, 0, 0,
            19, 20,
        ];
        #[rustfmt::skip]
        let router_meter_src = &[
            39, 40,
        ];

        #[rustfmt::skip]
        let target = &[
            // header
            0x80, 0x00, 0x30, 0x02,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00,
            0x00, 0x08, 0x04, 0x00,

            // to stream
            0x00, 0x08, 0x14, 0x01,
            0x00, 0x08, 0x24, 0x02,
            0x00, 0x08, 0x34, 0x03,
            0x00, 0x08, 0x44, 0x04,
            0x00, 0x08, 0x54, 0x05,
            0x00, 0x08, 0x64, 0x06,
            0x00, 0x08, 0x74, 0x07,
            0x00, 0x18, 0x04, 0x08,
            0x00, 0x18, 0x14, 0x09,
            0x00, 0x20, 0x04, 0x0a,
            0x00, 0x20, 0x14, 0x0b,
            0x00, 0x20, 0x24, 0x0c,
            0x00, 0x20, 0x34, 0x0d,

            // to outputs
            0x00, 0x30, 0x00, 0x88,
            0x00, 0x30, 0x10, 0x89,
            0x00, 0x30, 0x00, 0x86,
            0x00, 0x30, 0x10, 0x87,
            0x00, 0x30, 0x00, 0x84,
            0x00, 0x30, 0x10, 0x85,
            0x00, 0x30, 0x00, 0x82,
            0x00, 0x30, 0x10, 0x83,
            0x00, 0x30, 0x00, 0x80,
            0x00, 0x30, 0x10, 0x81,
            // SPDIF out
            0x00, 0x00, 0x01, 0x80,
            0x00, 0x00, 0x01, 0x81,
            // ADAT out
            0x00, 0x00, 0x02, 0x00,
            0x00, 0x00, 0x02, 0x01,
            0x00, 0x00, 0x02, 0x02,
            0x00, 0x00, 0x02, 0x03,
            // loop back
            0x00, 0x00, 0x04, 0x0e,
            0x00, 0x00, 0x04, 0x0f,

            // mixer inputs
            0x00, 0x08, 0x03, 0x00,
            0x00, 0x08, 0x13, 0x01,
            0x00, 0x08, 0x23, 0x02,
            0x00, 0x08, 0x33, 0x03,
            0x00, 0x08, 0x43, 0x04,
            0x00, 0x08, 0x53, 0x05,
            0x00, 0x08, 0x63, 0x06,
            0x00, 0x08, 0x73, 0x07,

            0x00, 0x00, 0x03, 0x08,
            0x00, 0x00, 0x03, 0x09,
            0x00, 0x00, 0x03, 0x0a,
            0x00, 0x00, 0x03, 0x0b,
            0x00, 0x00, 0x03, 0x0c,
            0x00, 0x00, 0x03, 0x0d,
            0x00, 0x00, 0x03, 0x0e,
            0x00, 0x00, 0x03, 0x0f,
            0x00, 0x40, 0x03, 0x10,
            0x00, 0x40, 0x13, 0x11,

            // meters
            0x00, 0x30, 0x00, 0x00,
            0x00, 0x30, 0x10, 0x00,

            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let mut raw = vec![0; 284];
        serialize_routing_high_rate(router_out_src, router_mixer_src, router_meter_src, &mut raw);
        println!("{:02x?}", raw);
        assert!(target.iter().eq(raw.iter()))
    }
}
