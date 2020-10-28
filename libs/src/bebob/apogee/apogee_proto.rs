// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use crate::ta1394::AvcAddr;
use crate::ta1394::{AvcOp, AvcControl};
use crate::ta1394::general::VendorDependent;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum VendorCmd {
    InputLimit(u8),     // index, state
    MicPower(u8),       // index, state
    IoAttr(u8, u8),     // index, direction, state
    IoRouting(u8),      // destination, source
    Hw(HwCmdOp),
    HpSrc(u8),          // destination, source
    MixerSrc0(u8),      // mixer_pair, [u8;36]
    MixerSrc1(u8),
    MixerSrc2(u8),
    MixerSrc3(u8),
    MicGain(u8),        // 1/2/3/4, dB(10-75), also available as knob control
    OptIfaceMode(u8),   // direction, mode
    Downgrade,          // on/off
    SpdifResample,      // on/off, iface, direction, rate
    MicPolarity(u8),    // index, state
    OutVol(u8),         // main/hp0/hp1, dB(127-0), also available as knob control
    HwStatus(bool),     // [u8;17] or [u8;56]
    Reserved(Vec<u8>),
}

impl VendorCmd {
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

impl From<&VendorCmd> for Vec<u8> {
    fn from(cmd: &VendorCmd) -> Self {
        let mut raw = Vec::new();

        match cmd {
            VendorCmd::InputLimit(ch) => {
                raw.push(VendorCmd::INPUT_LIMIT);
                raw.push(*ch);
            }
            VendorCmd::MicPower(ch) => {
                raw.push(VendorCmd::MIC_POWER);
                raw.push(*ch);
            }
            VendorCmd::IoAttr(ch, direction) => {
                raw.push(VendorCmd::IO_ATTR);
                raw.push(*ch);
                raw.push(*direction);
            }
            VendorCmd::IoRouting(dst) => {
                raw.push(VendorCmd::IO_ROUTING);
                raw.push(*dst);
            }
            VendorCmd::Hw(op) => {
                raw.push(VendorCmd::HW);
                raw.push(u8::from(*op));
            }
            VendorCmd::HpSrc(dst) => {
                raw.push(VendorCmd::HP_SRC);
                raw.push((*dst + 1) % 2);
            }
            VendorCmd::MixerSrc0(pair) => {
                raw.push(VendorCmd::MIXER_SRC0);
                raw.push(*pair);
            }
            VendorCmd::MixerSrc1(pair) => {
                raw.push(VendorCmd::MIXER_SRC1);
                raw.push(*pair);
            }
            VendorCmd::MixerSrc2(pair) => {
                raw.push(VendorCmd::MIXER_SRC2);
                raw.push(*pair);
            }
            VendorCmd::MixerSrc3(pair) => {
                raw.push(VendorCmd::MIXER_SRC3);
                raw.push(*pair);
            }
            VendorCmd::MicGain(target) => {
                raw.push(VendorCmd::IN_VOL);
                raw.push(*target);
            }
            VendorCmd::OptIfaceMode(direction) => {
                raw.push(VendorCmd::OPT_IFACE_MODE);
                raw.push(*direction);
            }
            VendorCmd::Downgrade => {
                raw.push(VendorCmd::DOWNGRADE);
            }
            VendorCmd::SpdifResample => {
                raw.push(VendorCmd::SPDIF_RESAMPLE);
            }
            VendorCmd::MicPolarity(ch) => {
                raw.push(VendorCmd::MIC_POLARITY);
                raw.push(*ch);
            }
            VendorCmd::OutVol(target) => {
                raw.push(VendorCmd::OUT_VOL);
                raw.push(*target);
            }
            VendorCmd::HwStatus(is_long) => {
                raw.push(VendorCmd::HW_STATUS);
                raw.push(*is_long as u8);
            }
            VendorCmd::Reserved(r) => raw.extend_from_slice(&r),
        }

        raw
    }
}

impl From<&[u8]> for VendorCmd {
    fn from(raw: &[u8]) -> VendorCmd {
        match raw[0] {
            VendorCmd::INPUT_LIMIT => VendorCmd::InputLimit(raw[1]),
            VendorCmd::MIC_POWER => VendorCmd::MicPower(raw[1]),
            VendorCmd::IO_ATTR => VendorCmd::IoAttr(raw[1], raw[2]),
            VendorCmd::IO_ROUTING => VendorCmd::IoRouting(raw[1]),
            VendorCmd::HW => VendorCmd::Hw(HwCmdOp::from(raw[1])),
            VendorCmd::HP_SRC => VendorCmd::HpSrc((raw[1] + 1) % 2),
            VendorCmd::MIXER_SRC0 => VendorCmd::MixerSrc0(raw[1]),
            VendorCmd::MIXER_SRC1 => VendorCmd::MixerSrc1(raw[1]),
            VendorCmd::MIXER_SRC2 => VendorCmd::MixerSrc2(raw[1]),
            VendorCmd::MIXER_SRC3 => VendorCmd::MixerSrc3(raw[1]),
            VendorCmd::OPT_IFACE_MODE => VendorCmd::OptIfaceMode(raw[1]),
            VendorCmd::DOWNGRADE => VendorCmd::Downgrade,
            VendorCmd::SPDIF_RESAMPLE => VendorCmd::SpdifResample,
            VendorCmd::MIC_POLARITY => VendorCmd::MicPolarity(raw[1]),
            VendorCmd::OUT_VOL => VendorCmd::OutVol(raw[1]),
            VendorCmd::HW_STATUS => VendorCmd::HwStatus(raw[1] > 0),
            _ => VendorCmd::Reserved(raw.to_vec()),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HwCmdOp {
    StreamMode,
    DisplayIlluminate,
    DisplayMode,
    DisplayTarget,
    DisplayOverhold,
    MeterReset,
    CdMode,
    Reserved(u8),
}

impl HwCmdOp {
    // NOTE: STREAM_MODE command generates bus reset to change available stream formats.
    const STREAM_MODE: u8 = 0x06;
    const DISPLAY_ILLUMINATE: u8 = 0x08;
    const DISPLAY_MODE: u8 = 0x09;
    const DISPLAY_TARGET: u8 = 0x0a;
    const DISPLAY_OVERHOLD: u8 = 0x0e;
    const METER_RESET: u8 = 0x0f;
    const CD_MODE: u8 = 0xf5;
}

impl From<HwCmdOp> for u8 {
    fn from(op: HwCmdOp) -> Self {
        match op {
            HwCmdOp::StreamMode => HwCmdOp::STREAM_MODE,
            HwCmdOp::DisplayIlluminate => HwCmdOp::DISPLAY_ILLUMINATE,
            HwCmdOp::DisplayMode => HwCmdOp::DISPLAY_MODE,
            HwCmdOp::DisplayTarget => HwCmdOp::DISPLAY_TARGET,
            HwCmdOp::DisplayOverhold => HwCmdOp::DISPLAY_OVERHOLD,
            HwCmdOp::MeterReset => HwCmdOp::METER_RESET,
            HwCmdOp::CdMode => HwCmdOp::CD_MODE,
            HwCmdOp::Reserved(val) => val,
        }
    }
}

impl From<u8> for HwCmdOp {
    fn from(val: u8) -> HwCmdOp {
        match val {
            HwCmdOp::STREAM_MODE => HwCmdOp::StreamMode,
            HwCmdOp::DISPLAY_ILLUMINATE => HwCmdOp::DisplayIlluminate,
            HwCmdOp::DISPLAY_MODE => HwCmdOp::DisplayMode,
            HwCmdOp::DISPLAY_TARGET => HwCmdOp::DisplayTarget,
            HwCmdOp::DISPLAY_OVERHOLD => HwCmdOp::DisplayOverhold,
            HwCmdOp::METER_RESET => HwCmdOp::MeterReset,
            HwCmdOp::CD_MODE => HwCmdOp::CdMode,
            _ => HwCmdOp::Reserved(val),
        }
    }
}

#[derive(Debug)]
pub struct ApogeeCmd{
    pub cmd: VendorCmd,
    pub params: Vec<u8>,
    op: VendorDependent,
}

impl ApogeeCmd {
    pub fn new(company_id: &[u8;3], cmd: VendorCmd, params: &[u8]) -> Self {
        ApogeeCmd{
            cmd,
            params: params.to_vec(),
            op: VendorDependent::new(company_id),
        }
    }
}

impl AvcOp for ApogeeCmd {
    const OPCODE: u8 = VendorDependent::OPCODE;
}

impl AvcControl for ApogeeCmd {
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
        let cmd = VendorCmd::from(self.op.data.as_slice());
        if let VendorCmd::HwStatus(_) = cmd {
            self.params = self.op.data[Into::<Vec<u8>>::into(&cmd).len()..].to_vec();
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{VendorCmd, HwCmdOp, ApogeeCmd};
    use crate::ta1394::AvcAddr;
    use crate::ta1394::AvcControl;

    #[test]
    fn vendorcmd_from() {
        let cmd = VendorCmd::InputLimit(1);
        assert_eq!(cmd, VendorCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice()));

        let cmd = VendorCmd::MicPower(1);
        assert_eq!(cmd, VendorCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice()));

        let cmd = VendorCmd::IoAttr(1, 0);
        assert_eq!(cmd, VendorCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice()));

        let cmd = VendorCmd::IoRouting(1);
        assert_eq!(cmd, VendorCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice()));

        let cmd = VendorCmd::Hw(HwCmdOp::StreamMode);
        assert_eq!(cmd, VendorCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice()));

        let cmd = VendorCmd::HpSrc(1);
        assert_eq!(cmd, VendorCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice()));

        let cmd = VendorCmd::MixerSrc0(3);
        assert_eq!(cmd, VendorCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice()));

        let cmd = VendorCmd::MixerSrc1(2);
        assert_eq!(cmd, VendorCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice()));

        let cmd = VendorCmd::MixerSrc2(1);
        assert_eq!(cmd, VendorCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice()));

        let cmd = VendorCmd::MixerSrc3(0);
        assert_eq!(cmd, VendorCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice()));

        let cmd = VendorCmd::OptIfaceMode(0);
        assert_eq!(cmd, VendorCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice()));

        let cmd = VendorCmd::Downgrade;
        assert_eq!(cmd, VendorCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice()));

        let cmd = VendorCmd::SpdifResample;
        assert_eq!(cmd, VendorCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice()));

        let cmd = VendorCmd::MicPolarity(0);
        assert_eq!(cmd, VendorCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice()));

        let cmd = VendorCmd::OutVol(0);
        assert_eq!(cmd, VendorCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice()));

        let cmd = VendorCmd::HwStatus(true);
        assert_eq!(cmd, VendorCmd::from(Into::<Vec<u8>>::into(&cmd).as_slice()));
    }

    #[test]
    fn apogeecmd_operands() {
        let operands = vec![0xde, 0xad, 0xbe, 0xef, 0x03, 0x02];
        let mut op = ApogeeCmd::new(&[0xc0, 0xfe, 0xe1], VendorCmd::IoRouting(0x03), &[0x02]);
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.params, vec![0x02]);
        let mut o = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(&o[..6], operands.as_slice());

        let operands = vec![0xde, 0xad, 0xbe, 0xf3, 0x01, 0x02, 0x03, 0x04];
        let mut op = ApogeeCmd::new(&[0xde, 0xad, 0xbe], VendorCmd::SpdifResample, &[0x01, 0x02, 0x03, 0x04]);
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.params, vec![0x01, 0x02, 0x03, 0x04]);
        let mut o = Vec::new();
        AvcControl::build_operands(&mut op, &AvcAddr::Unit, &mut o).unwrap();
        assert_eq!(&o[..8], operands.as_slice());
    }
}
