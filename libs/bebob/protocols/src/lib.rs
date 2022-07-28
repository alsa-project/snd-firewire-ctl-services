// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

//! Protocols defined for BridgeCo. Enhanced Break Out Box (BeBoB) solution.
//!
//! The crate includes various kind of protocols defined by BridgeCo. AG and application vendors
//! for DM1000, DM1100, and DM1500 ASICs with its BridgeCo. Enhanced Break Out Box (BeBoB) solution.

pub mod bridgeco;

pub mod apogee;
pub mod behringer;
pub mod digidesign;
pub mod esi;
pub mod focusrite;
pub mod icon;
pub mod maudio;
pub mod presonus;
pub mod roland;
pub mod stanton;
pub mod terratec;
pub mod yamaha_terratec;

use {
    self::bridgeco::{ExtendedStreamFormatSingle, *},
    glib::{Error, FileError, IsA},
    hinawa::{
        prelude::{FwFcpExtManual, FwFcpExt, FwReqExtManual},
        FwFcp, FwNode, FwReq, FwTcode,
    },
    ta1394::{amdtp::*, audio::*, ccm::*, general::*, *},
};

/// The offset for specific purposes in DM1000/DM1100/DM1500 ASICs.
pub const DM_APPL_OFFSET: u64 = 0xffc700000000;
pub const DM_APPL_METER_OFFSET: u64 = DM_APPL_OFFSET + 0x00600000;
pub const DM_APPL_PARAM_OFFSET: u64 = DM_APPL_OFFSET + 0x00700000;
pub const DM_BCO_OFFSET: u64 = 0xffffc8000000;
pub const DM_BCO_BOOTLOADER_INFO_OFFSET: u64 = DM_BCO_OFFSET + 0x00020000;

/// The structure for AV/C transaction helper with quirks specific to BeBoB solution.
#[derive(Default, Debug)]
pub struct BebobAvc(FwFcp);

impl Ta1394Avc for BebobAvc {
    fn transaction(
        &self,
        ctype: AvcCmdType,
        addr: &AvcAddr,
        opcode: u8,
        operands: &[u8],
        timeout_ms: u32,
    ) -> Result<(AvcRespCode, Vec<u8>), Error> {
        let mut cmd = Vec::new();
        cmd.push(ctype.into());
        cmd.push(addr.into());
        cmd.push(opcode);
        cmd.extend_from_slice(operands);

        let mut resp = vec![0; Self::FRAME_SIZE];
        self.0
            .avc_transaction(&cmd, &mut resp, timeout_ms)
            .and_then(|len| {
                if resp[1] != addr.into() {
                    let label = format!("Unexpected address in response: {}", resp[1]);
                    Err(Error::new(Ta1394AvcError::RespParse(AvcRespParseError::UnexpectedStatus), &label))
                } else if resp[2] != opcode {
                    let label =
                        format!("Unexpected opcode in response: {} but {}", opcode, resp[2]);
                    Err(Error::new(Ta1394AvcError::RespParse(AvcRespParseError::UnexpectedStatus), &label))
                } else {
                    let rcode = AvcRespCode::from(resp[0] & Self::RESP_CODE_MASK);

                    resp.truncate(len);
                    let operands = resp.split_off(3);

                    Ok((rcode, operands))
                }
            })
    }

    fn control<O: AvcOp + AvcControl>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut operands = Vec::new();
        AvcControl::build_operands(op, addr, &mut operands)
            .map_err(|err| Error::new(Ta1394AvcError::CmdBuild(err), ""))?;
        self.transaction(
            AvcCmdType::Control,
            addr,
            O::OPCODE,
            &mut operands,
            timeout_ms,
        )
        .and_then(|(rcode, operands)| {
            let expected = match O::OPCODE {
                InputPlugSignalFormat::OPCODE
                | OutputPlugSignalFormat::OPCODE
                | SignalSource::OPCODE => {
                    // NOTE: quirk.
                    rcode == AvcRespCode::Accepted || rcode == AvcRespCode::Reserved(0x00)
                }
                _ => rcode == AvcRespCode::Accepted,
            };
            if !expected {
                let label = format!(
                    "Unexpected response code for control opcode {}: {:?}",
                    O::OPCODE,
                    rcode
                );
                Err(Error::new(Ta1394AvcError::RespParse(AvcRespParseError::UnexpectedStatus), &label))
            } else {
                AvcControl::parse_operands(op, addr, &operands)
                    .map_err(|err| Error::new(Ta1394AvcError::RespParse(err), ""))
            }
        })
    }
}

impl BebobAvc {
    pub fn bind(&self, node: &impl IsA<FwNode>) -> Result<(), Error> {
        self.0.bind(node)
    }
}

/// The trait of frequency operation for media clock.
pub trait MediaClockFrequencyOperation {
    const FREQ_LIST: &'static [u32];

    fn read_clk_freq(avc: &BebobAvc, timeout_ms: u32) -> Result<usize, Error> {
        let plug_addr =
            BcoPlugAddr::new_for_unit(BcoPlugDirection::Output, BcoPlugAddrUnitType::Isoc, 0);
        let mut op = ExtendedStreamFormatSingle::new(&plug_addr);

        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;

        op.stream_format
            .as_bco_compound_am824_stream()
            .and_then(|format| {
                Self::FREQ_LIST
                    .iter()
                    .position(|&r| r == format.freq)
                    .ok_or_else(|| {
                        let msg = format!("Unexpected entry for source of clock: {}", format.freq);
                        Error::new(FileError::Io, &msg)
                    })
            })
    }

    /// Change frequency of media clock. This operation can involve INTERIM AV/C response to expand
    /// response time of AV/C transaction.
    fn write_clk_freq(avc: &BebobAvc, idx: usize, timeout_ms: u32) -> Result<(), Error> {
        let fdf = Self::FREQ_LIST
            .iter()
            .nth(idx)
            .ok_or_else(|| {
                let msg = format!("Invalid argument for index of frequency: {}", idx);
                Error::new(FileError::Inval, &msg)
            })
            .map(|&freq| AmdtpFdf::new(AmdtpEventType::Am824, false, freq))?;

        let mut op = InputPlugSignalFormat {
            plug_id: 0,
            fmt: FMT_IS_AMDTP,
            fdf: fdf.into(),
        };
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = OutputPlugSignalFormat {
            plug_id: 0,
            fmt: FMT_IS_AMDTP,
            fdf: fdf.into(),
        };
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }
}

/// The trait of source operation for sampling clock.
pub trait SamplingClockSourceOperation {
    // NOTE: all of bebob models support "SignalAddr::Unit(SignalUnitAddr::Isoc(0x00))" named as
    // "PCR Compound Input" and "SignalAddr::Unit(SignalUnitAddr::Isoc(0x01))" named as
    // "PCR Sync Input" for source of sampling clock. They are available to be synchronized to the
    // series of syt field in incoming packets from the other unit on IEEE 1394 bus. However, the
    // most of models doesn't work with it actually even if configured, therefore useless.
    const DST: SignalAddr;
    const SRC_LIST: &'static [SignalAddr];

    fn read_clk_src(avc: &BebobAvc, timeout_ms: u32) -> Result<usize, Error> {
        let mut op = SignalSource::new(&Self::DST);

        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;

        Self::SRC_LIST
            .iter()
            .position(|&s| s == op.src)
            .ok_or_else(|| {
                let label = "Unexpected entry for source of clock";
                Error::new(FileError::Io, &label)
            })
    }

    /// Change source of sampling clock. This operation can involve INTERIM AV/C response to expand
    /// response time of AV/C transaction.
    fn write_clk_src(avc: &BebobAvc, idx: usize, timeout_ms: u32) -> Result<(), Error> {
        let src = Self::SRC_LIST.iter().nth(idx).map(|s| *s).ok_or_else(|| {
            let label = "Invalid value for source of clock";
            Error::new(FileError::Inval, &label)
        })?;

        let mut op = SignalSource::new(&Self::DST);
        op.src = src;

        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
    }
}

/// The trait of level operation for audio function blocks by AV/C transaction.
pub trait AvcLevelOperation {
    const ENTRIES: &'static [(u8, AudioCh)];

    const LEVEL_MIN: i16 = FeatureCtl::NEG_INFINITY;
    const LEVEL_MAX: i16 = 0;
    const LEVEL_STEP: i16 = 0x100;

    fn read_level(avc: &BebobAvc, idx: usize, timeout_ms: u32) -> Result<i16, Error> {
        let &(func_block_id, audio_ch) = Self::ENTRIES.iter().nth(idx).ok_or_else(|| {
            let msg = format!("Invalid index of function block list: {}", idx);
            Error::new(FileError::Inval, &msg)
        })?;

        let mut op = AudioFeature::new(
            func_block_id,
            CtlAttr::Current,
            audio_ch,
            FeatureCtl::Volume(vec![-1]),
        );
        avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)?;

        if let FeatureCtl::Volume(data) = op.ctl {
            Ok(data[0])
        } else {
            unreachable!();
        }
    }

    fn write_level(avc: &BebobAvc, idx: usize, vol: i16, timeout_ms: u32) -> Result<(), Error> {
        let &(func_block_id, audio_ch) = Self::ENTRIES.iter().nth(idx).ok_or_else(|| {
            let msg = format!("Invalid index of function block list: {}", idx);
            Error::new(FileError::Inval, &msg)
        })?;

        let mut op = AudioFeature::new(
            func_block_id,
            CtlAttr::Current,
            audio_ch,
            FeatureCtl::Volume(vec![vol]),
        );
        avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
    }
}

/// The trait of LR balance operation for audio function blocks.
pub trait AvcLrBalanceOperation: AvcLevelOperation {
    const BALANCE_MIN: i16 = FeatureCtl::NEG_INFINITY;
    const BALANCE_MAX: i16 = FeatureCtl::INFINITY;
    const BALANCE_STEP: i16 = 0x80;

    fn read_lr_balance(avc: &BebobAvc, idx: usize, timeout_ms: u32) -> Result<i16, Error> {
        let &(func_block_id, audio_ch) = Self::ENTRIES.iter().nth(idx).ok_or_else(|| {
            let msg = format!("Invalid index of function block list: {}", idx);
            Error::new(FileError::Inval, &msg)
        })?;

        let mut op = AudioFeature::new(
            func_block_id,
            CtlAttr::Current,
            audio_ch,
            FeatureCtl::LrBalance(-1),
        );
        avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)?;

        if let FeatureCtl::LrBalance(balance) = op.ctl {
            Ok(balance)
        } else {
            unreachable!();
        }
    }

    fn write_lr_balance(
        avc: &BebobAvc,
        idx: usize,
        balance: i16,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let &(func_block_id, audio_ch) = Self::ENTRIES.iter().nth(idx).ok_or_else(|| {
            let msg = format!("Invalid index of function block list: {}", idx);
            Error::new(FileError::Inval, &msg)
        })?;

        let mut op = AudioFeature::new(
            func_block_id,
            CtlAttr::Current,
            audio_ch,
            FeatureCtl::LrBalance(balance),
        );
        avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
    }
}

/// The trait of mute operation for audio function blocks.
pub trait AvcMuteOperation: AvcLevelOperation {
    fn read_mute(avc: &BebobAvc, idx: usize, timeout_ms: u32) -> Result<bool, Error> {
        let &(func_block_id, audio_ch) = Self::ENTRIES.iter().nth(idx).ok_or_else(|| {
            let msg = format!("Invalid index of function block list: {}", idx);
            Error::new(FileError::Inval, &msg)
        })?;

        let mut op = AudioFeature::new(
            func_block_id,
            CtlAttr::Current,
            audio_ch,
            FeatureCtl::Mute(vec![false]),
        );
        avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)?;

        if let FeatureCtl::Mute(data) = op.ctl {
            Ok(data[0])
        } else {
            unreachable!();
        }
    }

    fn write_mute(avc: &BebobAvc, idx: usize, mute: bool, timeout_ms: u32) -> Result<(), Error> {
        let &(func_block_id, audio_ch) = Self::ENTRIES.iter().nth(idx).ok_or_else(|| {
            let msg = format!("Invalid index of function block list: {}", idx);
            Error::new(FileError::Inval, &msg)
        })?;

        let mut op = AudioFeature::new(
            func_block_id,
            CtlAttr::Current,
            audio_ch,
            FeatureCtl::Mute(vec![mute]),
        );
        avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
    }
}

/// The trait of select operation for audio function block.
pub trait AvcSelectorOperation {
    const FUNC_BLOCK_ID_LIST: &'static [u8];
    const INPUT_PLUG_ID_LIST: &'static [u8];

    fn read_selector(avc: &BebobAvc, idx: usize, timeout_ms: u32) -> Result<usize, Error> {
        let &func_block_id = Self::FUNC_BLOCK_ID_LIST.iter().nth(idx).ok_or_else(|| {
            let msg = format!("Invalid index of selector: {}", idx);
            Error::new(FileError::Inval, &msg)
        })?;

        let mut op = AudioSelector::new(func_block_id, CtlAttr::Current, 0xff);
        avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)?;

        Self::INPUT_PLUG_ID_LIST
            .iter()
            .position(|&input_plug_id| input_plug_id == op.input_plug_id)
            .ok_or_else(|| {
                let msg = format!(
                    "Unexpected index of input plug number: {}",
                    op.input_plug_id
                );
                Error::new(FileError::Io, &msg)
            })
    }

    fn write_selector(
        avc: &BebobAvc,
        idx: usize,
        val: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let &func_block_id = Self::FUNC_BLOCK_ID_LIST.iter().nth(idx).ok_or_else(|| {
            let msg = format!("Invalid index of selector: {}", idx);
            Error::new(FileError::Inval, &msg)
        })?;

        let input_plug_id = Self::INPUT_PLUG_ID_LIST
            .iter()
            .nth(val)
            .ok_or_else(|| {
                let msg = format!("Invalid index of input plug number: {}", val);
                Error::new(FileError::Inval, &msg)
            })
            .map(|input_plug_id| *input_plug_id)?;

        let mut op = AudioSelector::new(func_block_id, CtlAttr::Current, input_plug_id);
        avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
    }
}
