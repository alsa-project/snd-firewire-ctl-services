// SPDX-License-Identifier: LGPL-3.0-or-later
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
#[derive(Default, Debug)]
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
#[derive(Default, Debug)]
pub struct Inspire1394PhysInputProtocol;

impl AvcAudioFeatureSpecification for Inspire1394PhysInputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x1, AudioCh::Each(0)),
        (0x1, AudioCh::Each(1)),
        (0x2, AudioCh::Each(0)),
        (0x2, AudioCh::Each(1)),
    ];
}

impl AvcLevelOperation for Inspire1394PhysInputProtocol {}

impl AvcMuteOperation for Inspire1394PhysInputProtocol {}

/// The protocol implementation of physical output.
#[derive(Default, Debug)]
pub struct Inspire1394PhysOutputProtocol;

impl AvcAudioFeatureSpecification for Inspire1394PhysOutputProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[(0x06, AudioCh::Each(0)), (0x06, AudioCh::Each(1))];
}

impl AvcLevelOperation for Inspire1394PhysOutputProtocol {}

impl AvcMuteOperation for Inspire1394PhysOutputProtocol {}

impl AvcSelectorOperation for Inspire1394PhysOutputProtocol {
    const FUNC_BLOCK_ID_LIST: &'static [u8] = &[0x01];
    // NOTE: "mixer-output-1/2", "stream-input-1/2"
    const INPUT_PLUG_ID_LIST: &'static [u8] = &[0x00, 0x01];
}

/// The protocol implementation of headphone.
#[derive(Default, Debug)]
pub struct Inspire1394HeadphoneProtocol;

impl AvcAudioFeatureSpecification for Inspire1394HeadphoneProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[(0x07, AudioCh::Each(0)), (0x07, AudioCh::Each(1))];
}

impl AvcLevelOperation for Inspire1394HeadphoneProtocol {}

impl AvcMuteOperation for Inspire1394HeadphoneProtocol {}

/// The protocol implementation of analog source for mixer.
#[derive(Default, Debug)]
pub struct Inspire1394MixerAnalogSourceProtocol;

impl AvcAudioFeatureSpecification for Inspire1394MixerAnalogSourceProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[
        (0x03, AudioCh::Each(0)),
        (0x03, AudioCh::Each(1)),
        (0x04, AudioCh::Each(0)),
        (0x04, AudioCh::Each(1)),
    ];
}

impl AvcLevelOperation for Inspire1394MixerAnalogSourceProtocol {}

impl AvcLrBalanceOperation for Inspire1394MixerAnalogSourceProtocol {}

impl AvcMuteOperation for Inspire1394MixerAnalogSourceProtocol {}

/// The protocol implementation of stream source for mixer.
#[derive(Default, Debug)]
pub struct Inspire1394MixerStreamSourceProtocol;

impl AvcAudioFeatureSpecification for Inspire1394MixerStreamSourceProtocol {
    const ENTRIES: &'static [(u8, AudioCh)] = &[(0x05, AudioCh::Master)];
}

impl AvcLevelOperation for Inspire1394MixerStreamSourceProtocol {}

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

fn read_input_switch(
    avc: &BebobAvc,
    switch: &mut InputSwitch,
    timeout_ms: u32,
) -> Result<(), Error> {
    let mut op = InputSwitchOperation::new(switch);
    avc.status(&AvcAddr::Subunit(MUSIC_SUBUNIT_0), &mut op, timeout_ms)?;
    *switch = op.switch;
    Ok(())
}

fn write_input_switch(avc: &BebobAvc, switch: &InputSwitch, timeout_ms: u32) -> Result<(), Error> {
    let mut op = InputSwitchOperation::new(switch);
    avc.control(&AvcAddr::Subunit(MUSIC_SUBUNIT_0), &mut op, timeout_ms)
}

/// The protocol implementation of phono for physical input 3/4.
#[derive(Default)]
pub struct Inspire1394PhonoProtocol;

impl PresonusSwitchOperation for Inspire1394PhonoProtocol {
    const CH_COUNT: usize = 1;

    fn read_switch(avc: &BebobAvc, _: usize, timeout_ms: u32) -> Result<bool, Error> {
        let mut param = InputSwitch::Analog34Phono(false);
        read_input_switch(avc, &mut param, timeout_ms)?;
        if let InputSwitch::Analog34Phono(state) = param {
            Ok(state)
        } else {
            unreachable!();
        }
    }

    fn write_switch(avc: &BebobAvc, _: usize, val: bool, timeout_ms: u32) -> Result<(), Error> {
        let param = InputSwitch::Analog34Phono(val);
        write_input_switch(avc, &param, timeout_ms)
    }
}

/// The protocol implementation of phantom powering for physical input 1/2.
#[derive(Default)]
pub struct Inspire1394MicPhantomProtocol;

impl PresonusSwitchOperation for Inspire1394MicPhantomProtocol {
    const CH_COUNT: usize = 2;

    fn read_switch(avc: &BebobAvc, idx: usize, timeout_ms: u32) -> Result<bool, Error> {
        let mut param = InputSwitch::Analog12Phantom(idx, false);
        read_input_switch(avc, &mut param, timeout_ms)?;
        if let InputSwitch::Analog12Phantom(_, state) = param {
            Ok(state)
        } else {
            unreachable!();
        }
    }

    fn write_switch(avc: &BebobAvc, idx: usize, val: bool, timeout_ms: u32) -> Result<(), Error> {
        let param = InputSwitch::Analog12Phantom(idx, val);
        write_input_switch(avc, &param, timeout_ms)
    }
}

/// The protocol implementation of phantom powering for physical input 1/2.
#[derive(Default)]
pub struct Inspire1394MicBoostProtocol;

impl PresonusSwitchOperation for Inspire1394MicBoostProtocol {
    const CH_COUNT: usize = 2;

    fn read_switch(avc: &BebobAvc, idx: usize, timeout_ms: u32) -> Result<bool, Error> {
        let mut param = InputSwitch::Analog12Boost(idx, false);
        read_input_switch(avc, &mut param, timeout_ms)?;
        if let InputSwitch::Analog12Boost(_, state) = param {
            Ok(state)
        } else {
            unreachable!();
        }
    }

    fn write_switch(avc: &BebobAvc, idx: usize, val: bool, timeout_ms: u32) -> Result<(), Error> {
        let param = InputSwitch::Analog12Boost(idx, val);
        write_input_switch(avc, &param, timeout_ms)
    }
}

/// The protocol implementation of phantom powering for physical input 1/2.
#[derive(Default)]
pub struct Inspire1394MicLimitProtocol;

impl PresonusSwitchOperation for Inspire1394MicLimitProtocol {
    const CH_COUNT: usize = 2;

    fn read_switch(avc: &BebobAvc, idx: usize, timeout_ms: u32) -> Result<bool, Error> {
        let mut param = InputSwitch::Analog12Limit(idx, false);
        read_input_switch(avc, &mut param, timeout_ms)?;
        if let InputSwitch::Analog12Limit(_, state) = param {
            Ok(state)
        } else {
            unreachable!();
        }
    }

    fn write_switch(avc: &BebobAvc, idx: usize, val: bool, timeout_ms: u32) -> Result<(), Error> {
        let param = InputSwitch::Analog12Limit(idx, val);
        write_input_switch(avc, &param, timeout_ms)
    }
}

/// The trait for switch operation specific to Inspire 1394.
pub trait PresonusSwitchOperation {
    const CH_COUNT: usize;

    fn read_switch(avc: &BebobAvc, idx: usize, timeout_ms: u32) -> Result<bool, Error>;
    fn write_switch(avc: &BebobAvc, idx: usize, val: bool, timeout_ms: u32) -> Result<(), Error>;
}

/// The switch related to inputs.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InputSwitch {
    /// Phono mode for 2nd pair of inputs (line).
    Analog34Phono(bool),
    /// Phantom powering for 1st pair of inputs (microphones).
    Analog12Phantom(usize, bool),
    /// Boost for 1st pair of inputs (microphones).
    Analog12Boost(usize, bool),
    /// Limitter for 1st pair of inputs (microphones).
    Analog12Limit(usize, bool),
    /// Link stereo pairs.
    #[allow(dead_code)]
    AnalogStereoLink(usize, bool),
}

const CMD_PHONO: u8 = 0x00;
const CMD_MIC_PHANTOM: u8 = 0x01;
const CMD_MIC_BOOST: u8 = 0x02;
const CMD_MIC_LIMIT: u8 = 0x03;
const CMD_STEREO_LINK: u8 = 0x05;

impl Default for InputSwitch {
    fn default() -> Self {
        Self::Analog34Phono(false)
    }
}

/// The AV/C operation for input switch.
#[derive(Debug)]
pub struct InputSwitchOperation {
    /// The switch for input.
    pub switch: InputSwitch,
    op: VendorDependent,
}

impl Default for InputSwitchOperation {
    fn default() -> Self {
        Self {
            switch: Default::default(),
            op: VendorDependent {
                company_id: PRESONUS_OUI,
                data: vec![0; 3],
            },
        }
    }
}

impl InputSwitchOperation {
    fn new(switch: &InputSwitch) -> Self {
        let mut op = Self::default();
        op.switch = *switch;
        op
    }
}

impl AvcOp for InputSwitchOperation {
    const OPCODE: u8 = VendorDependent::OPCODE;
}

impl AvcControl for InputSwitchOperation {
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        match self.switch {
            InputSwitch::Analog34Phono(state) => {
                self.op.data[0] = CMD_PHONO;
                self.op.data[1] = 0x00;
                self.op.data[2] = state as u8;
            }
            InputSwitch::Analog12Phantom(ch, state) => {
                self.op.data[0] = CMD_MIC_PHANTOM;
                self.op.data[1] = 1 + ch as u8;
                self.op.data[2] = state as u8;
            }
            InputSwitch::Analog12Boost(ch, state) => {
                self.op.data[0] = CMD_MIC_BOOST;
                self.op.data[1] = 1 + ch as u8;
                self.op.data[2] = state as u8;
            }
            InputSwitch::Analog12Limit(ch, state) => {
                self.op.data[0] = CMD_MIC_LIMIT;
                self.op.data[1] = 1 + ch as u8;
                self.op.data[2] = state as u8;
            }
            InputSwitch::AnalogStereoLink(ch, state) => {
                self.op.data[0] = CMD_STEREO_LINK;
                self.op.data[1] = 1 + ch as u8;
                self.op.data[2] = state as u8;
            }
        }
        AvcControl::build_operands(&mut self.op, addr)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AvcControl::parse_operands(&mut self.op, addr, operands)
    }
}

impl AvcStatus for InputSwitchOperation {
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        match self.switch {
            InputSwitch::Analog34Phono(_) => {
                self.op.data[0] = CMD_PHONO;
                self.op.data[1] = 0x00;
            }
            InputSwitch::Analog12Phantom(ch, _) => {
                self.op.data[0] = CMD_PHONO;
                self.op.data[1] = 1 + ch as u8;
            }
            InputSwitch::Analog12Boost(ch, _) => {
                self.op.data[0] = CMD_PHONO;
                self.op.data[1] = 1 + ch as u8;
            }
            InputSwitch::Analog12Limit(ch, _) => {
                self.op.data[0] = CMD_PHONO;
                self.op.data[1] = 1 + ch as u8;
            }
            InputSwitch::AnalogStereoLink(ch, _) => {
                self.op.data[0] = CMD_STEREO_LINK;
                self.op.data[1] = 1 + ch as u8;
            }
        }
        self.op.data[2] = 0xff;
        AvcControl::build_operands(&mut self.op, addr)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AvcControl::parse_operands(&mut self.op, addr, operands).map(|_| match &mut self.switch {
            InputSwitch::Analog34Phono(state) => {
                *state = self.op.data[2] > 0;
            }
            InputSwitch::Analog12Phantom(_, state) => {
                *state = self.op.data[2] > 0;
            }
            InputSwitch::Analog12Boost(_, state) => {
                *state = self.op.data[2] > 0;
            }
            InputSwitch::Analog12Limit(_, state) => {
                *state = self.op.data[2] > 0;
            }
            InputSwitch::AnalogStereoLink(_, state) => {
                *state = self.op.data[2] > 0;
            }
        })
    }
}
