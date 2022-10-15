// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Output protocol specific to Alesis iO FireWire series.
//!
//! The module includes structure, enumeration, and trait and its implementation for output
//! protocol defined by Alesis for iO FireWire series.

use super::*;

/// Protocol of output for iO FireWire series.
pub trait IofwOutputOperation {
    const ANALOG_OUTPUT_COUNT: usize;
    const HAS_OPT_IFACE_B: bool;

    fn read_out_levels(
        req: &mut FwReq,
        node: &mut FwNode,
        levels: &mut [NominalSignalLevel],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        alesis_read_block(req, node, OUT_LEVEL_OFFSET, &mut raw, timeout_ms).map(|_| {
            let _ = deserialize_nominal_signal_levels(levels, &raw);
        })
    }

    fn write_out_levels(
        req: &mut FwReq,
        node: &mut FwNode,
        levels: &[NominalSignalLevel],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        let _ = serialize_nominal_signal_levels(levels, &mut raw);
        alesis_read_block(req, node, OUT_LEVEL_OFFSET, &mut raw, timeout_ms)
    }

    fn read_mixer_digital_b_67_src(
        req: &mut FwReq,
        node: &mut FwNode,
        src: &mut DigitalB67Src,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        alesis_read_block(
            req,
            node,
            MIXER_DIGITAL_B_67_SRC_OFFSET,
            &mut raw,
            timeout_ms,
        )
        .map(|_| {
            let _ = deserialize_digital_b67_src(src, &raw);
        })
    }

    fn write_mixer_digital_b_67_src(
        req: &mut FwReq,
        node: &mut FwNode,
        src: &DigitalB67Src,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        let _ = serialize_digital_b67_src(src, &mut raw);
        alesis_write_block(
            req,
            node,
            MIXER_DIGITAL_B_67_SRC_OFFSET,
            &mut raw,
            timeout_ms,
        )
    }

    fn read_spdif_out_src(
        req: &mut FwReq,
        node: &mut FwNode,
        src: &mut MixerOutPair,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        alesis_read_block(req, node, SPDIF_OUT_SRC_OFFSET, &mut raw, timeout_ms).map(|_| {
            let _ = deserialize_mixer_out_pair(src, &raw);
        })
    }

    fn write_spdif_out_src(
        req: &mut FwReq,
        node: &mut FwNode,
        src: &MixerOutPair,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        let _ = serialize_mixer_out_pair(src, &mut raw);
        alesis_write_block(req, node, SPDIF_OUT_SRC_OFFSET, &mut raw, timeout_ms)
    }

    fn read_hp23_out_src(
        req: &mut FwReq,
        node: &mut FwNode,
        src: &mut MixerOutPair,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        alesis_read_block(req, node, HP34_SRC_OFFSET, &mut raw, timeout_ms).map(|_| {
            let _ = deserialize_mixer_out_pair(src, &raw);
        })
    }

    fn write_hp23_out_src(
        req: &mut FwReq,
        node: &mut FwNode,
        src: &MixerOutPair,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = [0; 4];
        let _ = serialize_mixer_out_pair(src, &mut raw);
        alesis_write_block(req, node, HP34_SRC_OFFSET, &mut raw, timeout_ms)
    }
}
