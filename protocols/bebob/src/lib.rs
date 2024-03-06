// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

#![doc = include_str!("../README.md")]

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
    glib::{prelude::IsA, Error, FileError},
    hinawa::{
        prelude::{FwFcpExt, FwFcpExtManual, FwReqExtManual},
        FwFcp, FwNode, FwReq, FwTcode,
    },
    ta1394_avc_audio::{amdtp::*, *},
    ta1394_avc_ccm::*,
    ta1394_avc_general::{general::*, *},
    ta1394_avc_stream_format::*,
};

/// The offset for specific purposes in DM1000/DM1100/DM1500 ASICs.
const DM_APPL_OFFSET: u64 = 0xffc700000000;
const DM_APPL_METER_OFFSET: u64 = DM_APPL_OFFSET + 0x00600000;
const DM_APPL_PARAM_OFFSET: u64 = DM_APPL_OFFSET + 0x00700000;
const DM_BCO_OFFSET: u64 = 0xffffc8000000;
const DM_BCO_BOOTLOADER_INFO_OFFSET: u64 = DM_BCO_OFFSET + 0x00020000;

/// The implementation of AV/C transaction with quirks specific to BeBoB solution.
///
/// It seems a unique quirk that the status code in response frame for some AV/C commands is
/// against AV/C general specification in control operation.
#[derive(Default, Debug)]
pub struct BebobAvc(FwFcp);

impl Ta1394Avc<Error> for BebobAvc {
    fn transaction(&self, command_frame: &[u8], timeout_ms: u32) -> Result<Vec<u8>, Error> {
        let mut resp = vec![0; Self::FRAME_SIZE];
        self.0
            .avc_transaction(&command_frame, &mut resp, timeout_ms)
            .map(|_| resp)
    }

    fn control<O: AvcOp + AvcControl>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Ta1394AvcError<Error>> {
        let operands =
            AvcControl::build_operands(op, addr).map_err(|err| Ta1394AvcError::CmdBuild(err))?;
        let command_frame =
            Self::compose_command_frame(AvcCmdType::Control, addr, O::OPCODE, &operands)?;
        let response_frame = self
            .transaction(&command_frame, timeout_ms)
            .map_err(|cause| Ta1394AvcError::CommunicationFailure(cause))?;
        Self::detect_response_operands(&response_frame, addr, O::OPCODE)
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
                    Err(AvcRespParseError::UnexpectedStatus)
                } else {
                    AvcControl::parse_operands(op, addr, &operands)
                }
            })
            .map_err(|err| Ta1394AvcError::RespParse(err))
    }
}

impl BebobAvc {
    pub fn bind(&self, node: &impl IsA<FwNode>) -> Result<(), Error> {
        self.0.bind(node)
    }

    pub fn control<O: AvcOp + AvcControl>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Ta1394Avc::<Error>::control(self, addr, op, timeout_ms).map_err(|err| from_avc_err(err))
    }

    pub fn status<O: AvcOp + AvcStatus>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Ta1394Avc::<Error>::status(self, addr, op, timeout_ms).map_err(|err| from_avc_err(err))
    }
}

fn from_avc_err(err: Ta1394AvcError<Error>) -> Error {
    match err {
        Ta1394AvcError::CmdBuild(cause) => Error::new(FileError::Inval, &cause.to_string()),
        Ta1394AvcError::CommunicationFailure(cause) => cause,
        Ta1394AvcError::RespParse(cause) => Error::new(FileError::Io, &cause.to_string()),
    }
}

/// The parameters of media clock.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct MediaClockParameters {
    /// The index for entry in frequency list.
    pub freq_idx: usize,
}

/// The trait of frequency operation for media clock.
pub trait MediaClockFrequencyOperation {
    /// The list of supported frequencies.
    const FREQ_LIST: &'static [u32];

    /// Cache the state of media clock to the parameters.
    fn cache_freq(
        avc: &BebobAvc,
        params: &mut MediaClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let plug_addr =
            BcoPlugAddr::new_for_unit(BcoPlugDirection::Output, BcoPlugAddrUnitType::Isoc, 0);
        let mut op = ExtendedStreamFormatSingle::new(&plug_addr);

        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;

        op.stream_format
            .as_bco_compound_am824_stream()
            .ok_or_else(|| {
                let label = "Bco Compound AM824 stream is not available for the unit";
                Error::new(FileError::Nxio, &label)
            })
            .and_then(|format| {
                Self::FREQ_LIST
                    .iter()
                    .position(|&r| r == format.freq)
                    .ok_or_else(|| {
                        let msg = format!("Unexpected entry for source of clock: {}", format.freq);
                        Error::new(FileError::Io, &msg)
                    })
            })
            .map(|freq_idx| params.freq_idx = freq_idx)
    }

    /// Update the hardware by the given parameter. This operation can involve INTERIM AV/C
    /// response to expand response time of AV/C transaction.
    fn update_freq(
        avc: &BebobAvc,
        params: &MediaClockParameters,
        old: &mut MediaClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let fdf = Self::FREQ_LIST
            .iter()
            .nth(params.freq_idx)
            .ok_or_else(|| {
                let msg = format!(
                    "Invalid argument for index of frequency: {}",
                    params.freq_idx
                );
                Error::new(FileError::Inval, &msg)
            })
            .map(|&freq| AmdtpFdf::new(AmdtpEventType::Am824, false, freq))?;

        let mut op = InputPlugSignalFormat(PlugSignalFormat {
            plug_id: 0,
            fmt: FMT_IS_AMDTP,
            fdf: fdf.into(),
        });
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        let mut op = OutputPlugSignalFormat(PlugSignalFormat {
            plug_id: 0,
            fmt: FMT_IS_AMDTP,
            fdf: fdf.into(),
        });
        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)?;

        *old = *params;

        Ok(())
    }
}

/// The parameters of sampling clock.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SamplingClockParameters {
    /// The index for entry in source list.
    pub src_idx: usize,
}

/// The trait of source operation for sampling clock.
pub trait SamplingClockSourceOperation {
    // NOTE: all of bebob models support "SignalAddr::Unit(SignalUnitAddr::Isoc(0x00))" named as
    // "PCR Compound Input" and "SignalAddr::Unit(SignalUnitAddr::Isoc(0x01))" named as
    // "PCR Sync Input" for source of sampling clock. They are available to be synchronized to the
    // series of syt field in incoming packets from the other unit on IEEE 1394 bus. However, the
    // most of models doesn't work with it actually even if configured, therefore useless.
    /// The destination plug address for source signal.
    const DST: SignalAddr;
    /// The list of supported sources expressed by plug address.
    const SRC_LIST: &'static [SignalAddr];

    /// Cache the state of sampling clock to the parameters.
    fn cache_src(
        avc: &BebobAvc,
        params: &mut SamplingClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut op = SignalSource::new(&Self::DST);

        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)?;

        Self::SRC_LIST
            .iter()
            .position(|&s| s == op.src)
            .ok_or_else(|| {
                let label = "Unexpected entry for source of clock";
                Error::new(FileError::Io, &label)
            })
            .map(|src_idx| params.src_idx = src_idx)
    }

    /// Update the hardware by the given parameter. This operation can involve INTERIM AV/C
    /// response to expand response time of AV/C transaction.
    fn update_src(
        avc: &BebobAvc,
        params: &SamplingClockParameters,
        old: &mut SamplingClockParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let src = Self::SRC_LIST
            .iter()
            .nth(params.src_idx)
            .ok_or_else(|| {
                let label = "Invalid value for source of clock";
                Error::new(FileError::Inval, &label)
            })
            .copied()?;

        let mut op = SignalSource::new(&Self::DST);
        op.src = src;

        avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
            .map(|_| *old = *params)
    }
}

/// The specification of Feature Function Blocks of AV/C Audio subunit.
pub trait AvcAudioFeatureSpecification {
    /// The entries of pair of function block identifier and audio channel.
    const ENTRIES: &'static [(u8, AudioCh)];
}

/// The parameters of signal level. The `Default` trait should be implemented to call
/// `AvcLevelOperation::create_level_parameters()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvcLevelParameters {
    /// The signal levels.
    pub levels: Vec<i16>,
}

/// The trait of level operation for audio function blocks by AV/C transaction.
pub trait AvcLevelOperation: AvcAudioFeatureSpecification {
    /// The minimum value of signal level.
    const LEVEL_MIN: i16 = VolumeData::VALUE_NEG_INFINITY;
    /// The maximum value of signal level.
    const LEVEL_MAX: i16 = VolumeData::VALUE_ZERO;
    /// The step value of signal level.
    const LEVEL_STEP: i16 = 0x100;

    /// Instantiate parameters.
    fn create_level_parameters() -> AvcLevelParameters {
        AvcLevelParameters {
            levels: vec![Default::default(); Self::ENTRIES.len()],
        }
    }

    /// Cache state of hardware to the parameters.
    fn cache_levels(
        avc: &BebobAvc,
        params: &mut AvcLevelParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(params.levels.len(), Self::ENTRIES.len());

        params
            .levels
            .iter_mut()
            .zip(Self::ENTRIES)
            .try_for_each(|(level, entry)| {
                let &(func_block_id, audio_ch) = entry;
                let mut op = AudioFeature::new(
                    func_block_id,
                    CtlAttr::Current,
                    audio_ch,
                    FeatureCtl::Volume(VolumeData::new(1)),
                );
                avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
                    .map(|_| {
                        if let FeatureCtl::Volume(data) = op.ctl {
                            *level = data.0[0]
                        }
                    })
            })
    }

    /// Update the hardware when detecting any changes in the parameters.
    fn update_levels(
        avc: &BebobAvc,
        params: &AvcLevelParameters,
        old: &mut AvcLevelParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(params.levels.len(), Self::ENTRIES.len());
        assert_eq!(old.levels.len(), Self::ENTRIES.len());

        old.levels
            .iter_mut()
            .zip(params.levels.iter())
            .zip(Self::ENTRIES)
            .filter(|((old, new), _)| !new.eq(old))
            .try_for_each(|((old, new), entry)| {
                let &(func_block_id, audio_ch) = entry;
                let mut op = AudioFeature::new(
                    func_block_id,
                    CtlAttr::Current,
                    audio_ch,
                    FeatureCtl::Volume(VolumeData(vec![*new])),
                );
                avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
                    .map(|_| *old = *new)
            })
    }
}

/// The parameters of L/R balance. The `Default` trait should be implemented to call
/// `AvcLrBalanceOperation::create_lr_balance_parameters()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvcLrBalanceParameters {
    /// The L/R balances.
    pub balances: Vec<i16>,
}

/// The trait of LR balance operation for audio function blocks.
pub trait AvcLrBalanceOperation: AvcAudioFeatureSpecification {
    /// The minimum value of L/R balance.
    const BALANCE_MIN: i16 = LrBalanceData::VALUE_LEFT_NEG_INFINITY;
    /// The maximum value of L/R balance.
    const BALANCE_MAX: i16 = LrBalanceData::VALUE_LEFT_MAX;
    /// The step value of L/R balance.
    const BALANCE_STEP: i16 = 0x80;

    /// Instantiate parameters.
    fn create_lr_balance_parameters() -> AvcLrBalanceParameters {
        AvcLrBalanceParameters {
            balances: vec![Default::default(); Self::ENTRIES.len()],
        }
    }

    /// Cache state of hardware to the parameters.
    fn cache_lr_balances(
        avc: &BebobAvc,
        params: &mut AvcLrBalanceParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(params.balances.len(), Self::ENTRIES.len());

        params
            .balances
            .iter_mut()
            .zip(Self::ENTRIES)
            .try_for_each(|(balance, entry)| {
                let &(func_block_id, audio_ch) = entry;
                let mut op = AudioFeature::new(
                    func_block_id,
                    CtlAttr::Current,
                    audio_ch,
                    FeatureCtl::LrBalance(Default::default()),
                );
                avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
                    .map(|_| {
                        if let FeatureCtl::LrBalance(data) = op.ctl {
                            *balance = data.0;
                        }
                    })
            })
    }

    /// Update the hardware when detecting any changes in the parameters.
    fn update_lr_balances(
        avc: &BebobAvc,
        params: &AvcLrBalanceParameters,
        old: &mut AvcLrBalanceParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(params.balances.len(), Self::ENTRIES.len());
        assert_eq!(old.balances.len(), Self::ENTRIES.len());

        old.balances
            .iter_mut()
            .zip(params.balances.iter())
            .zip(Self::ENTRIES)
            .filter(|((o, n), _)| !o.eq(n))
            .try_for_each(|((old, &new), entry)| {
                let &(func_block_id, audio_ch) = entry;
                let mut op = AudioFeature::new(
                    func_block_id,
                    CtlAttr::Current,
                    audio_ch,
                    FeatureCtl::LrBalance(LrBalanceData(new)),
                );
                avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
                    .map(|_| *old = new)
            })
    }
}

/// The parameters of mute. The `Default` trait should be implemented to call
/// `AvcMuteOperation::create_mute_parameters()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvcMuteParameters {
    /// Muted or not.
    pub mutes: Vec<bool>,
}

/// The trait of mute operation for audio function blocks.
pub trait AvcMuteOperation: AvcAudioFeatureSpecification {
    /// Instantiate parameters.
    fn create_mute_parameters() -> AvcMuteParameters {
        AvcMuteParameters {
            mutes: vec![Default::default(); Self::ENTRIES.len()],
        }
    }

    /// Cache state of hardware to the parameters.
    fn cache_mutes(
        avc: &BebobAvc,
        params: &mut AvcMuteParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(params.mutes.len(), Self::ENTRIES.len());

        params
            .mutes
            .iter_mut()
            .zip(Self::ENTRIES)
            .try_for_each(|(mute, entry)| {
                let &(func_block_id, audio_ch) = entry;

                let mut op = AudioFeature::new(
                    func_block_id,
                    CtlAttr::Current,
                    audio_ch,
                    FeatureCtl::Mute(vec![false]),
                );
                avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
                    .map(|_| {
                        if let FeatureCtl::Mute(data) = op.ctl {
                            *mute = data[0];
                        }
                    })
            })
    }

    /// Update the hardware when detecting any changes in the parameters.
    fn update_mutes(
        avc: &BebobAvc,
        params: &AvcMuteParameters,
        old: &mut AvcMuteParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(params.mutes.len(), Self::ENTRIES.len());
        assert_eq!(old.mutes.len(), Self::ENTRIES.len());

        old.mutes
            .iter_mut()
            .zip(params.mutes.iter())
            .zip(Self::ENTRIES)
            .filter(|((o, n), _)| !n.eq(o))
            .try_for_each(|((old, &new), entry)| {
                let &(func_block_id, audio_ch) = entry;

                let mut op = AudioFeature::new(
                    func_block_id,
                    CtlAttr::Current,
                    audio_ch,
                    FeatureCtl::Mute(vec![new]),
                );
                avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
                    .map(|_| *old = new)
            })
    }
}

/// The parameter of selectors. The `Default` trait should be implemented to call
/// `AvcSelectorOperation::create_selector_parameters()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvcSelectorParameters {
    /// The index for entry in the list of function block.
    pub selectors: Vec<usize>,
}

/// The trait of select operation for audio function block.
pub trait AvcSelectorOperation {
    /// The list of function block identifier.
    const FUNC_BLOCK_ID_LIST: &'static [u8];
    /// The list of plug identifier.
    const INPUT_PLUG_ID_LIST: &'static [u8];

    /// Instantiate parameters.
    fn create_selector_parameters() -> AvcSelectorParameters {
        AvcSelectorParameters {
            selectors: vec![Default::default(); Self::FUNC_BLOCK_ID_LIST.len()],
        }
    }

    /// Cache state of hardware to the parameters.
    fn cache_selectors(
        avc: &BebobAvc,
        params: &mut AvcSelectorParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(params.selectors.len(), Self::FUNC_BLOCK_ID_LIST.len());

        params
            .selectors
            .iter_mut()
            .zip(Self::FUNC_BLOCK_ID_LIST)
            .try_for_each(|(selector, &func_block_id)| {
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
                    .map(|pos| *selector = pos)
            })
    }

    /// Update the hardware when detecting any changes in the parameters.
    fn update_selectors(
        avc: &BebobAvc,
        params: &AvcSelectorParameters,
        old: &mut AvcSelectorParameters,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert_eq!(params.selectors.len(), Self::FUNC_BLOCK_ID_LIST.len());
        assert_eq!(old.selectors.len(), Self::FUNC_BLOCK_ID_LIST.len());

        old.selectors
            .iter_mut()
            .zip(params.selectors.iter())
            .zip(Self::FUNC_BLOCK_ID_LIST)
            .filter(|((o, n), _)| !o.eq(n))
            .try_for_each(|((old, &new), &func_block_id)| {
                let mut op = AudioSelector::new(func_block_id, CtlAttr::Current, new as u8);
                avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
                    .map(|_| *old = new)
            })
    }
}
