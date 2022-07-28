// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Inspire 1394.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by PreSonus for Inspire 1394.
//!
//! ## Diagram of internal signal flow
//!
//! ```text
//! analog-input-1/2 ---------+--------> stream-output-1/2
//! analog-input-3/4 ---------|-+------> stream-output-3/4
//!                           | |
//!                           v v
//!                       ++=======++
//! stream-input-1/2 -+-> ||  6x2  ||
//!                   |   || mixer ||
//!                   |   ++=======++
//!                   |        |
//!                   v        v
//!               (one source only)
//!               analog-output-1/2
//!                 headphone-1/2
//! ```
//!
//! The protocol implementation for PreSonus Inspire 1394 was written with firmware version below:
//!
//! ```sh
//! $ cargo run --bin bco-bootloader-info -- /dev/fw1
//! protocol:
//!   version: 1
//! bootloader:
//!   timestamp: 2005-09-02T05:43:21+0000
//!   version: 0.0.0
//! hardware:
//!   GUID: 0xe6120493000a9200
//!   model ID: 0x00009c
//!   revision: 0.0.1
//! software:
//!   timestamp: 2005-09-02T06:23:23+0000
//!   ID: 0x00010001
//!   revision: 0.255.65535
//! image:
//!   base address: 0x20080000
//!   maximum size: 0x180000
//! ```

use super::*;

/// The protocol implementation of clock operation.
#[derive(Default)]
pub struct Inspire1394ClkProtocol;

impl MediaClockFrequencyOperation for Inspire1394ClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000];
}

impl SamplingClockSourceOperation for Inspire1394ClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 0x03,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal.
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 0x02,
        }),
    ];
}

/// The protocol implementation of meter information.
#[derive(Default)]
pub struct Inspire1394MeterProtocol;

impl Inspire1394MeterOperation for Inspire1394MeterProtocol {}

/// The protocol implementation of physical input.
#[derive(Default)]
pub struct Inspire1394PhysInputProtocol;

impl AvcLevelOperation for Inspire1394PhysInputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x1, AudioCh::Each(0)),
        (0x1, AudioCh::Each(1)),
        (0x2, AudioCh::Each(0)),
        (0x2, AudioCh::Each(1)),
    ];
}

impl AvcMuteOperation for Inspire1394PhysInputProtocol {}

/// The protocol implementation of physical output.
#[derive(Default)]
pub struct Inspire1394PhysOutputProtocol;

impl AvcLevelOperation for Inspire1394PhysOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[(0x06, AudioCh::Each(0)), (0x06, AudioCh::Each(1))];
}

impl AvcMuteOperation for Inspire1394PhysOutputProtocol {}

impl AvcSelectorOperation for Inspire1394PhysOutputProtocol {
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x01];
    // NOTE: "mixer-output-1/2", "stream-input-1/2"
    const INPUT_PLUG_ID_LIST: &'static [u8] = &[0x00, 0x01];
}

/// The protocol implementation of headphone.
#[derive(Default)]
pub struct Inspire1394HeadphoneProtocol;

impl AvcLevelOperation for Inspire1394HeadphoneProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[(0x07, AudioCh::Each(0)), (0x07, AudioCh::Each(1))];
}

impl AvcMuteOperation for Inspire1394HeadphoneProtocol {}

/// The protocol implementation of analog source for mixer.
#[derive(Default)]
pub struct Inspire1394MixerAnalogSourceProtocol;

impl AvcLevelOperation for Inspire1394MixerAnalogSourceProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x03, AudioCh::Each(0)),
        (0x03, AudioCh::Each(1)),
        (0x04, AudioCh::Each(0)),
        (0x04, AudioCh::Each(1)),
    ];
}

impl AvcLrBalanceOperation for Inspire1394MixerAnalogSourceProtocol {}

impl AvcMuteOperation for Inspire1394MixerAnalogSourceProtocol {}

/// The protocol implementation of stream source for mixer.
#[derive(Default)]
pub struct Inspire1394MixerStreamSourceProtocol;

impl AvcLevelOperation for Inspire1394MixerStreamSourceProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[(0x05, AudioCh::All)];
}

impl AvcMuteOperation for Inspire1394MixerStreamSourceProtocol {}

const METER_FRAME_SIZE: usize = 32;

/// The structure of meter information for Inspire 1394.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Inspire1394Meter {
    pub phys_inputs: [i32; 4],
    pub stream_inputs: [i32; 2],
    pub phys_outputs: [i32; 2],
    frame: [u8; METER_FRAME_SIZE],
}

/// The trait for meter information operation.
pub trait Inspire1394MeterOperation {
    const LEVEL_MIN: i32 = 0;
    const LEVEL_MAX: i32 = 0x07ffffff;
    const LEVEL_STEP: i32 = 0x100;

    fn read_meter(
        req: &FwReq,
        node: &FwNode,
        meter: &mut Inspire1394Meter,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let frame = &mut meter.frame;
        req.transaction_sync(
            node,
            FwTcode::ReadBlockRequest,
            DM_APPL_METER_OFFSET,
            METER_FRAME_SIZE,
            frame,
            timeout_ms,
        )?;

        let mut quadlet = [0u8; 4];
        meter
            .phys_inputs
            .iter_mut()
            .chain(&mut meter.stream_inputs)
            .chain(&mut meter.phys_outputs)
            .enumerate()
            .for_each(|(i, m)| {
                let pos = i * 4;
                quadlet.copy_from_slice(&frame[pos..(pos + 4)]);
                *m = i32::from_be_bytes(quadlet);
            });

        Ok(())
    }
}

fn read_input_param(
    avc: &BebobAvc,
    param: &mut InputParameter,
    timeout_ms: u32,
) -> Result<(), Error> {
    let mut op = InputParameterOperation::new(param);
    avc.status(&AvcAddr::Subunit(MUSIC_SUBUNIT_0), &mut op, timeout_ms)?;
    *param = op.param;
    Ok(())
}

fn write_input_param(avc: &BebobAvc, param: &InputParameter, timeout_ms: u32) -> Result<(), Error> {
    let mut op = InputParameterOperation::new(param);
    avc.control(&AvcAddr::Subunit(MUSIC_SUBUNIT_0), &mut op, timeout_ms)
}

/// The protocol implementation of phono for physical input 3/4.
#[derive(Default)]
pub struct Inspire1394PhonoProtocol;

impl PresonusSwitchOperation for Inspire1394PhonoProtocol {
    const CH_COUNT: usize = 1;

    fn read_switch(avc: &BebobAvc, _: usize, timeout_ms: u32) -> Result<bool, Error> {
        let mut param = InputParameter::Analog34Phono(false);
        read_input_param(avc, &mut param, timeout_ms)?;
        if let InputParameter::Analog34Phono(state) = param {
            Ok(state)
        } else {
            unreachable!();
        }
    }

    fn write_switch(avc: &BebobAvc, _: usize, val: bool, timeout_ms: u32) -> Result<(), Error> {
        let param = InputParameter::Analog34Phono(val);
        write_input_param(avc, &param, timeout_ms)
    }
}

/// The protocol implementation of phantom powering for physical input 1/2.
#[derive(Default)]
pub struct Inspire1394MicPhantomProtocol;

impl PresonusSwitchOperation for Inspire1394MicPhantomProtocol {
    const CH_COUNT: usize = 2;

    fn read_switch(avc: &BebobAvc, idx: usize, timeout_ms: u32) -> Result<bool, Error> {
        let mut param = InputParameter::Analog12Phantom(idx, false);
        read_input_param(avc, &mut param, timeout_ms)?;
        if let InputParameter::Analog12Phantom(_, state) = param {
            Ok(state)
        } else {
            unreachable!();
        }
    }

    fn write_switch(avc: &BebobAvc, idx: usize, val: bool, timeout_ms: u32) -> Result<(), Error> {
        let param = InputParameter::Analog12Phantom(idx, val);
        write_input_param(avc, &param, timeout_ms)
    }
}

/// The protocol implementation of phantom powering for physical input 1/2.
#[derive(Default)]
pub struct Inspire1394MicBoostProtocol;

impl PresonusSwitchOperation for Inspire1394MicBoostProtocol {
    const CH_COUNT: usize = 2;

    fn read_switch(avc: &BebobAvc, idx: usize, timeout_ms: u32) -> Result<bool, Error> {
        let mut param = InputParameter::Analog12Boost(idx, false);
        read_input_param(avc, &mut param, timeout_ms)?;
        if let InputParameter::Analog12Boost(_, state) = param {
            Ok(state)
        } else {
            unreachable!();
        }
    }

    fn write_switch(avc: &BebobAvc, idx: usize, val: bool, timeout_ms: u32) -> Result<(), Error> {
        let param = InputParameter::Analog12Boost(idx, val);
        write_input_param(avc, &param, timeout_ms)
    }
}

/// The protocol implementation of phantom powering for physical input 1/2.
#[derive(Default)]
pub struct Inspire1394MicLimitProtocol;

impl PresonusSwitchOperation for Inspire1394MicLimitProtocol {
    const CH_COUNT: usize = 2;

    fn read_switch(avc: &BebobAvc, idx: usize, timeout_ms: u32) -> Result<bool, Error> {
        let mut param = InputParameter::Analog12Limit(idx, false);
        read_input_param(avc, &mut param, timeout_ms)?;
        if let InputParameter::Analog12Limit(_, state) = param {
            Ok(state)
        } else {
            unreachable!();
        }
    }

    fn write_switch(avc: &BebobAvc, idx: usize, val: bool, timeout_ms: u32) -> Result<(), Error> {
        let param = InputParameter::Analog12Limit(idx, val);
        write_input_param(avc, &param, timeout_ms)
    }
}

/// The trait for switch operation specific to Inspire 1394.
pub trait PresonusSwitchOperation {
    const CH_COUNT: usize;

    fn read_switch(avc: &BebobAvc, idx: usize, timeout_ms: u32) -> Result<bool, Error>;
    fn write_switch(avc: &BebobAvc, idx: usize, val: bool, timeout_ms: u32) -> Result<(), Error>;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum InputParameter {
    Analog34Phono(bool),
    Analog12Phantom(usize, bool),
    Analog12Boost(usize, bool),
    Analog12Limit(usize, bool),
    #[allow(dead_code)]
    AnalogStereoLink(usize, bool),
}

const CMD_PHONO: u8 = 0x00;
const CMD_MIC_PHANTOM: u8 = 0x01;
const CMD_MIC_BOOST: u8 = 0x02;
const CMD_MIC_LIMIT: u8 = 0x03;
const CMD_STEREO_LINK: u8 = 0x05;

impl Default for InputParameter {
    fn default() -> Self {
        Self::Analog34Phono(false)
    }
}

#[derive(Debug)]
struct InputParameterOperation {
    param: InputParameter,
    op: VendorDependent,
}

impl Default for InputParameterOperation {
    fn default() -> Self {
        Self {
            param: Default::default(),
            op: VendorDependent {
                company_id: PRESONUS_OUI,
                data: vec![0; 3],
            },
        }
    }
}

impl InputParameterOperation {
    fn new(param: &InputParameter) -> Self {
        let mut op = Self::default();
        op.param = *param;
        op
    }
}

impl AvcOp for InputParameterOperation {
    const OPCODE: u8 = VendorDependent::OPCODE;
}

impl AvcControl for InputParameterOperation {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        match self.param {
            InputParameter::Analog34Phono(state) => {
                self.op.data[0] = CMD_PHONO;
                self.op.data[1] = 0x00;
                self.op.data[2] = state as u8;
            }
            InputParameter::Analog12Phantom(ch, state) => {
                self.op.data[0] = CMD_MIC_PHANTOM;
                self.op.data[1] = 1 + ch as u8;
                self.op.data[2] = state as u8;
            }
            InputParameter::Analog12Boost(ch, state) => {
                self.op.data[0] = CMD_MIC_BOOST;
                self.op.data[1] = 1 + ch as u8;
                self.op.data[2] = state as u8;
            }
            InputParameter::Analog12Limit(ch, state) => {
                self.op.data[0] = CMD_MIC_LIMIT;
                self.op.data[1] = 1 + ch as u8;
                self.op.data[2] = state as u8;
            }
            InputParameter::AnalogStereoLink(ch, state) => {
                self.op.data[0] = CMD_STEREO_LINK;
                self.op.data[1] = 1 + ch as u8;
                self.op.data[2] = state as u8;
            }
        }
        AvcControl::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcControl::parse_operands(&mut self.op, addr, operands)
    }
}

impl AvcStatus for InputParameterOperation {
    fn build_operands(
        &mut self,
        addr: &AvcAddr,
        operands: &mut Vec<u8>,
    ) -> Result<(), AvcCmdBuildError> {
        match self.param {
            InputParameter::Analog34Phono(_) => {
                self.op.data[0] = CMD_PHONO;
                self.op.data[1] = 0x00;
            }
            InputParameter::Analog12Phantom(ch, _) => {
                self.op.data[0] = CMD_PHONO;
                self.op.data[1] = 1 + ch as u8;
            }
            InputParameter::Analog12Boost(ch, _) => {
                self.op.data[0] = CMD_PHONO;
                self.op.data[1] = 1 + ch as u8;
            }
            InputParameter::Analog12Limit(ch, _) => {
                self.op.data[0] = CMD_PHONO;
                self.op.data[1] = 1 + ch as u8;
            }
            InputParameter::AnalogStereoLink(ch, _) => {
                self.op.data[0] = CMD_STEREO_LINK;
                self.op.data[1] = 1 + ch as u8;
            }
        }
        self.op.data[2] = 0xff;
        AvcControl::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcControl::parse_operands(&mut self.op, addr, operands)?;
        match &mut self.param {
            InputParameter::Analog34Phono(state) => {
                *state = self.op.data[2] > 0;
            }
            InputParameter::Analog12Phantom(_, state) => {
                *state = self.op.data[2] > 0;
            }
            InputParameter::Analog12Boost(_, state) => {
                *state = self.op.data[2] > 0;
            }
            InputParameter::Analog12Limit(_, state) => {
                *state = self.op.data[2] > 0;
            }
            InputParameter::AnalogStereoLink(_, state) => {
                *state = self.op.data[2] > 0;
            }
        }
        Ok(())
    }
}
