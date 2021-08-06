// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocol implementation for Apogee Electronics Ensemble FireWire.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Apogee Electronics Ensemble FireWire.
//!
//! DM1500 ASIC is used for Apogee Ensemble FireWire.
//!
//! ## Diagram of internal signal flow for Apogee Ensemble FireWire
//!
//! ```text
//!                                ++==========++
//! analog-inputs (8 channels) --> ||  18x18   ||
//! spdif-inputs (2 channels) ---> ||  capture || --> stream-outputs (18 channels)
//! adat-inputs (8 channels) ----> ||  router  ||
//!                                ++==========++
//!
//!                                ++==========++
//! analog-inputs (8 channels) --> ||  36x4    ||
//! spdif-inputs (2 channels) ---> ||          || --> mixer-outputs (4 channels)
//! adat-inputs (8 channels) ----> ||  mixer   ||
//! stream-inputs (18 channels) -> ||          ||
//!                                ++==========++
//!
//!                                ++==========++
//! analog-inputs (8 channels) --> ||  40x18   ||
//! spdif-inputs (2 channels) ---> ||          || --> analog-outputs (8 channels)
//! adat-inputs (8 channels) ----> || playback || --> spdif-outputs (2 channels)
//! stream-inputs (18 channels) -> ||          || --> adat-outputs (8 channels)
//! mixer-outputs (4 channels) --> ||  router  ||
//!                                ++==========++
//!
//! (source) ----------------------------> spdif-output-1/2
//!                           ^
//!                           |
//!                 ++================++
//!                 || rate converter || (optional)
//!                 ++================++
//!                           |
//!                           v
//! spdif-input-1/2 ------------------------> (destination)
//!
//! analog-input-1/2 ------------------------> (destination)
//! analog-input-3/4 ------------------------> (destination)
//! analog-input-5/6 ------------------------> (destination)
//! analog-input-7/8 ------------------------> (destination)
//! spdif-input-1/2 -------------------------> (destination)
//!                           ^
//!                           |
//!                ++==================++
//!                || format converter || (optional)
//!                ++==================++
//!                           |
//!                           v
//! (source) ------------------------------> spdif-output-1/2
//! ```
//!
//! The protocol implementation for Apogee Ensemble FireWire was written with firmware version
//! below:
//!
//! ```sh
//! $ cargo run --bin bco-bootloader-info -- /dev/fw1
//! protocol:
//!   version: 3
//! bootloader:
//!   timestamp: 2006-04-07T11:13:17+0000
//!   version: 0.0.0
//! hardware:
//!   GUID: 0x0000f1a50003db05
//!   model ID: 0x000000
//!   revision: 0.0.0
//! software:
//!   timestamp: 2008-11-08T12:36:10+0000
//!   ID: 0x0001eeee
//!   revision: 0.0.5297
//! image:
//!   base address: 0x400c0080
//!   maximum size: 0x156aa8
//! ```

use crate::*;

use super::*;

use ta1394::ccm::{SignalAddr, SignalSubunitAddr, SignalUnitAddr};
use ta1394::MUSIC_SUBUNIT_0;

/// The protocol implementation for media and sampling clock of Ensemble FireWire.
#[derive(Default)]
pub struct EnsembleClkProtocol;

impl MediaClockFrequencyOperation for EnsembleClkProtocol {
    const FREQ_LIST: &'static [u32] = &[44100, 48000, 88200, 96000, 176400, 192000];
}

impl SamplingClockSourceOperation for EnsembleClkProtocol {
    const DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr {
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 7,
    });

    const SRC_LIST: &'static [SignalAddr] = &[
        // Internal
        SignalAddr::Subunit(SignalSubunitAddr {
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 7,
        }),
        // S/PDIF-coax
        SignalAddr::Unit(SignalUnitAddr::Ext(4)),
        // Optical
        SignalAddr::Unit(SignalUnitAddr::Ext(5)),
        // Word clock
        SignalAddr::Unit(SignalUnitAddr::Ext(6)),
    ];
}

/// The enumeration of command specific to Apogee Ensemble.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EnsembleCmd {
    InputLimit(u8), // index, state
    MicPower(u8),   // index, state
    IoAttr(u8, u8), // index, direction, state
    IoRouting(u8),  // destination, source
    Hw(HwCmd),
    HpSrc(u8),     // destination, source
    MixerSrc0(u8), // mixer_pair, [u8;36]
    MixerSrc1(u8),
    MixerSrc2(u8),
    MixerSrc3(u8),
    MicGain(u8),      // 1/2/3/4, dB(10-75), also available as knob control
    OptIfaceMode(u8), // direction, mode
    Downgrade,        // on/off
    SpdifResample,    // on/off, iface, direction, rate
    MicPolarity(u8),  // index, state
    OutVol(u8),       // main/hp0/hp1, dB(127-0), also available as knob control
    HwStatusShort([u8; 17]),
    HwStatusLong([u8; 56]),
    Reserved(Vec<u8>),
}

impl Default for EnsembleCmd {
    fn default() -> Self {
        Self::Reserved(Vec::new())
    }
}

impl EnsembleCmd {
    const INPUT_LIMIT: u8 = 0xe4;
    const MIC_POWER: u8 = 0xe5;
    const IO_ATTR: u8 = 0xe8;
    const IO_ROUTING: u8 = 0xef;
    const HW: u8 = 0xeb;
    const HP_SRC: u8 = 0xab;
    const MIXER_SRC0: u8 = 0xb0;
    const MIXER_SRC1: u8 = 0xb1;
    const MIXER_SRC2: u8 = 0xb2;
    const MIXER_SRC3: u8 = 0xb3;
    const IN_VOL: u8 = 0xe6;
    const OPT_IFACE_MODE: u8 = 0xf1;
    const DOWNGRADE: u8 = 0xf2;
    const SPDIF_RESAMPLE: u8 = 0xf3;
    const MIC_POLARITY: u8 = 0xf5;
    const OUT_VOL: u8 = 0xf6;
    const HW_STATUS: u8 = 0xff;
}

impl From<&EnsembleCmd> for Vec<u8> {
    fn from(cmd: &EnsembleCmd) -> Self {
        match cmd {
            EnsembleCmd::InputLimit(ch) => {
                vec![EnsembleCmd::INPUT_LIMIT, *ch]
            }
            EnsembleCmd::MicPower(ch) => {
                vec![EnsembleCmd::MIC_POWER, *ch]
            }
            EnsembleCmd::IoAttr(ch, direction) => {
                vec![EnsembleCmd::IO_ATTR, *ch, *direction]
            }
            EnsembleCmd::IoRouting(dst) => {
                vec![EnsembleCmd::IO_ROUTING, *dst]
            }
            EnsembleCmd::Hw(op) => {
                vec![EnsembleCmd::HW, u8::from(*op)]
            }
            EnsembleCmd::HpSrc(dst) => {
                vec![EnsembleCmd::HP_SRC, (*dst + 1) % 2]
            }
            EnsembleCmd::MixerSrc0(pair) => {
                vec![EnsembleCmd::MIXER_SRC0, *pair]
            }
            EnsembleCmd::MixerSrc1(pair) => {
                vec![EnsembleCmd::MIXER_SRC1, *pair]
            }
            EnsembleCmd::MixerSrc2(pair) => {
                vec![EnsembleCmd::MIXER_SRC2, *pair]
            }
            EnsembleCmd::MixerSrc3(pair) => {
                vec![EnsembleCmd::MIXER_SRC3, *pair]
            }
            EnsembleCmd::MicGain(target) => {
                vec![EnsembleCmd::IN_VOL, *target]
            }
            EnsembleCmd::OptIfaceMode(direction) => {
                vec![EnsembleCmd::OPT_IFACE_MODE, *direction]
            }
            EnsembleCmd::Downgrade => {
                vec![EnsembleCmd::DOWNGRADE]
            }
            EnsembleCmd::SpdifResample => {
                vec![EnsembleCmd::SPDIF_RESAMPLE]
            }
            EnsembleCmd::MicPolarity(ch) => {
                vec![EnsembleCmd::MIC_POLARITY, *ch]
            }
            EnsembleCmd::OutVol(target) => {
                vec![EnsembleCmd::OUT_VOL, *target]
            }
            EnsembleCmd::HwStatusShort(_) => vec![EnsembleCmd::HW_STATUS, 0],
            EnsembleCmd::HwStatusLong(_) => vec![EnsembleCmd::HW_STATUS, 1],
            EnsembleCmd::Reserved(r) => r.to_vec(),
        }
    }
}

impl From<&[u8]> for EnsembleCmd {
    fn from(raw: &[u8]) -> Self {
        match raw[0] {
            Self::INPUT_LIMIT => Self::InputLimit(raw[1]),
            Self::MIC_POWER => Self::MicPower(raw[1]),
            Self::IO_ATTR => Self::IoAttr(raw[1], raw[2]),
            Self::IO_ROUTING => Self::IoRouting(raw[1]),
            Self::HW => Self::Hw(HwCmd::from(raw[1])),
            Self::HP_SRC => Self::HpSrc((raw[1] + 1) % 2),
            Self::MIXER_SRC0 => Self::MixerSrc0(raw[1]),
            Self::MIXER_SRC1 => Self::MixerSrc1(raw[1]),
            Self::MIXER_SRC2 => Self::MixerSrc2(raw[1]),
            Self::MIXER_SRC3 => Self::MixerSrc3(raw[1]),
            Self::OPT_IFACE_MODE => Self::OptIfaceMode(raw[1]),
            Self::DOWNGRADE => Self::Downgrade,
            Self::SPDIF_RESAMPLE => Self::SpdifResample,
            Self::MIC_POLARITY => Self::MicPolarity(raw[1]),
            Self::OUT_VOL => Self::OutVol(raw[1]),
            Self::HW_STATUS => {
                if raw[1] > 0 {
                    let mut params = [0; 56];
                    params.copy_from_slice(&raw[1..]);
                    Self::HwStatusLong(params)
                } else {
                    let mut params = [0; 17];
                    params.copy_from_slice(&raw[1..]);
                    Self::HwStatusShort(params)
                }
            }
            _ => Self::Reserved(raw.to_vec()),
        }
    }
}

/// The enumeration of command for hardware operation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HwCmd {
    /// STREAM_MODE command generates bus reset to change available stream formats.
    StreamMode,
    DisplayIlluminate,
    DisplayMode,
    DisplayTarget,
    DisplayOverhold,
    MeterReset,
    CdMode,
    Reserved(u8),
}

impl HwCmd {
    const STREAM_MODE: u8 = 0x06;
    const DISPLAY_ILLUMINATE: u8 = 0x08;
    const DISPLAY_MODE: u8 = 0x09;
    const DISPLAY_TARGET: u8 = 0x0a;
    const DISPLAY_OVERHOLD: u8 = 0x0e;
    const METER_RESET: u8 = 0x0f;
    const CD_MODE: u8 = 0xf5;
}

impl From<HwCmd> for u8 {
    fn from(op: HwCmd) -> Self {
        match op {
            HwCmd::StreamMode => HwCmd::STREAM_MODE,
            HwCmd::DisplayIlluminate => HwCmd::DISPLAY_ILLUMINATE,
            HwCmd::DisplayMode => HwCmd::DISPLAY_MODE,
            HwCmd::DisplayTarget => HwCmd::DISPLAY_TARGET,
            HwCmd::DisplayOverhold => HwCmd::DISPLAY_OVERHOLD,
            HwCmd::MeterReset => HwCmd::METER_RESET,
            HwCmd::CdMode => HwCmd::CD_MODE,
            HwCmd::Reserved(val) => val,
        }
    }
}

impl From<u8> for HwCmd {
    fn from(val: u8) -> HwCmd {
        match val {
            HwCmd::STREAM_MODE => HwCmd::StreamMode,
            HwCmd::DISPLAY_ILLUMINATE => HwCmd::DisplayIlluminate,
            HwCmd::DISPLAY_MODE => HwCmd::DisplayMode,
            HwCmd::DISPLAY_TARGET => HwCmd::DisplayTarget,
            HwCmd::DISPLAY_OVERHOLD => HwCmd::DisplayOverhold,
            HwCmd::METER_RESET => HwCmd::MeterReset,
            HwCmd::CD_MODE => HwCmd::CdMode,
            _ => HwCmd::Reserved(val),
        }
    }
}

/// The protocol implementation of AV/C vendor-dependent command specific to Apogee Ensemble.
#[derive(Debug)]
pub struct EnsembleOperation {
    pub cmd: EnsembleCmd,
    pub params: Vec<u8>,
    op: VendorDependent,
}

impl Default for EnsembleOperation {
    fn default() -> Self {
        Self {
            cmd: Default::default(),
            params: Default::default(),
            op: VendorDependent {
                company_id: APOGEE_OUI,
                data: Default::default(),
            },
        }
    }
}

impl EnsembleOperation {
    pub fn new(cmd: EnsembleCmd, params: &[u8]) -> Self {
        let mut op = EnsembleOperation::default();
        op.cmd = cmd;
        op.params.extend_from_slice(params);
        op
    }
}

impl AvcOp for EnsembleOperation {
    const OPCODE: u8 = VendorDependent::OPCODE;
}

impl AvcControl for EnsembleOperation {
    fn build_operands(&mut self, addr: &AvcAddr, operands: &mut Vec<u8>) -> Result<(), Error> {
        self.op.data.clear();
        self.op.data.append(&mut Into::<Vec<u8>>::into(&self.cmd));
        self.op.data.append(&mut self.params.clone());

        // At least, 6 bytes should be required to align to 3 quadlets. Unless, the target unit is freezed.
        while self.op.data.len() < 6 {
            self.op.data.push(0xff);
        }

        AvcControl::build_operands(&mut self.op, addr, operands)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), Error> {
        AvcControl::parse_operands(&mut self.op, addr, operands)?;

        // NOTE: parameters are retrieved by HwStatus command only.
        match &mut self.cmd {
            EnsembleCmd::HwStatusShort(buf) => buf.copy_from_slice(&self.op.data[2..]),
            EnsembleCmd::HwStatusLong(buf) => buf.copy_from_slice(&self.op.data[2..]),
            _ => (),
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{EnsembleCmd, EnsembleOperation, HwCmd};
    use ta1394::AvcAddr;
    use ta1394::AvcControl;

    #[test]
    fn vendorcmd_from() {
        let cmd = EnsembleCmd::InputLimit(1);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::MicPower(1);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::IoAttr(1, 0);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::IoRouting(1);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::Hw(HwCmd::StreamMode);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::HpSrc(1);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::MixerSrc0(3);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::MixerSrc1(2);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::MixerSrc2(1);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::MixerSrc3(0);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::OptIfaceMode(0);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::Downgrade;
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::SpdifResample;
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::MicPolarity(0);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );

        let cmd = EnsembleCmd::OutVol(0);
        assert_eq!(
            cmd,
            EnsembleCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice())
        );
    }

    #[test]
    fn apogeecmd_operands() {
        let operands = vec![0xde, 0xad, 0xbe, 0xef, 0x03, 0x02];
        let mut op = EnsembleOperation::new(EnsembleCmd::IoRouting(0x03), &[0x02]);
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.params, vec![0x02]);
        let mut o = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(&o[..6], operands.as_slice());

        let operands = vec![0xde, 0xad, 0xbe, 0xf3, 0x01, 0x02, 0x03, 0x04];
        let mut op = EnsembleOperation::new(EnsembleCmd::SpdifResample, &[0x01, 0x02, 0x03, 0x04]);
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.params, vec![0x01, 0x02, 0x03, 0x04]);
        let mut o = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(&o[..8], operands.as_slice());
    }
}
