// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2024 Takashi Sakamoto

//! Protocol specific to Weiss Engineering AV/C models.
//!
//! The module includes structure, enumeration, and trait and its implementation for protocol
//! defined by Weiss Engineering.
//!
//! MAN301 includes two units in the root directory of its configuration ROM. The first unit
//! expresses AV/C protocol, and the second unit expresses TCAT protocol.
//!
//! ```text
//! spdif-opt-input-1/2  ---+
//! spdif-coax-input-1/2 --(or)--> digital-input-1/2 -----------------> stream-output-1/2
//! aesebu-xlr-input-1/2 ---+
//!
//! stream-input-1/2 --------------------------+----------------------> spdif-coax-output-1/2
//!                                            +----------------------> aesebu-xlr-output-1/2
//!                                            +--analog-output-1/2 --> analog-xlr-output-1/2
//!                                                       +-----------> analog-coax-output-1/2
//! ```

use {
    super::*,
    crate::{tcelectronic::*, *},
    glib::{prelude::IsA, Error, FileError},
    hinawa::{
        prelude::{FwFcpExt, FwFcpExtManual},
        FwFcp, FwNode,
    },
    ta1394_avc_general::{general::*, *},
};

/// Protocol implementation specific to MAN301.
#[derive(Default, Debug)]
pub struct WeissMan301Protocol;

impl TcatOperation for WeissMan301Protocol {}

// clock caps: 44100 48000 88200 96000 176400 192000
// clock source names: AES/EBU (XLR)\S/PDIF (RCA)\S/PDIF (TOS)\Unused\Unused\Unused\Unused\Word Clock (BNC)\Unused\Unused\Unused\Unused\Internal\\
impl TcatGlobalSectionSpecification for WeissMan301Protocol {}

/// The implementation of AV/C transaction.
#[derive(Default, Debug)]
pub struct WeissAvc(FwFcp);

impl Ta1394Avc<Error> for WeissAvc {
    fn transaction(&self, command_frame: &[u8], timeout_ms: u32) -> Result<Vec<u8>, Error> {
        let mut resp = vec![0; Self::FRAME_SIZE];
        self.0
            .avc_transaction(&command_frame, &mut resp, timeout_ms)
            .map(|_| resp)
    }
}

fn from_avc_err(err: Ta1394AvcError<Error>) -> Error {
    match err {
        Ta1394AvcError::CmdBuild(cause) => Error::new(FileError::Inval, &cause.to_string()),
        Ta1394AvcError::CommunicationFailure(cause) => cause,
        Ta1394AvcError::RespParse(cause) => Error::new(FileError::Io, &cause.to_string()),
    }
}

impl WeissAvc {
    /// Bind FCP protocol to the given node for AV/C operation.
    pub fn bind(&self, node: &impl IsA<FwNode>) -> Result<(), Error> {
        self.0.bind(node)
    }

    /// Request AV/C control operation and wait for response.
    pub fn control<O: AvcOp + AvcControl>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Ta1394Avc::<Error>::control(self, addr, op, timeout_ms).map_err(|err| from_avc_err(err))
    }

    /// Request AV/C status operation and wait for response.
    pub fn status<O: AvcOp + AvcStatus>(
        &self,
        addr: &AvcAddr,
        op: &mut O,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        Ta1394Avc::<Error>::status(self, addr, op, timeout_ms).map_err(|err| from_avc_err(err))
    }
}

/// The content of command to operate parameter.
#[derive(Debug)]
pub struct WeissAvcParamCmd {
    /// The numeric identifier of parameter.
    pub numeric_id: u32,
    /// The value of parameter.
    pub value: u32,
    /// For future use.
    pub reserved: [u32; 4],
    op: TcAvcCmd,
}

impl Default for WeissAvcParamCmd {
    fn default() -> Self {
        let mut op = TcAvcCmd::new(&WEISS_OUI);
        op.class_id = 1;
        op.sequence_id = u8::MAX;
        op.command_id = 0x8002;
        Self {
            numeric_id: u32::MAX,
            value: u32::MAX,
            reserved: [u32::MAX; 4],
            op: TcAvcCmd::new(&WEISS_OUI),
        }
    }
}

impl AvcOp for WeissAvcParamCmd {
    const OPCODE: u8 = VendorDependent::OPCODE;
}

fn build_param_command_status_data(cmd: &mut WeissAvcParamCmd) -> Result<(), AvcCmdBuildError> {
    cmd.op.arguments.resize(24, u8::MAX);
    serialize_u32(&cmd.numeric_id, &mut cmd.op.arguments[..4]);
    (0..4).for_each(|i| {
        let pos = 8 + i * 4;
        serialize_u32(&cmd.reserved[i], &mut cmd.op.arguments[pos..(pos + 4)]);
    });
    Ok(())
}

fn build_param_command_control_data(cmd: &mut WeissAvcParamCmd) -> Result<(), AvcCmdBuildError> {
    cmd.op.arguments.resize(24, u8::MIN);
    serialize_u32(&cmd.numeric_id, &mut cmd.op.arguments[..4]);
    serialize_u32(&cmd.value, &mut cmd.op.arguments[4..8]);
    (0..4).for_each(|i| {
        let pos = 8 + i * 4;
        serialize_u32(&cmd.reserved[i], &mut cmd.op.arguments[pos..(pos + 4)]);
    });
    Ok(())
}

fn parse_param_command_response_data(cmd: &mut WeissAvcParamCmd) -> Result<(), AvcRespParseError> {
    if cmd.op.arguments.len() < 24 {
        Err(AvcRespParseError::TooShortResp(24))?
    }

    deserialize_u32(&mut cmd.numeric_id, &cmd.op.arguments[..4]);
    deserialize_u32(&mut cmd.value, &cmd.op.arguments[4..8]);
    (0..4).for_each(|i| {
        let pos = 8 + i * 4;
        deserialize_u32(&mut cmd.reserved[i], &cmd.op.arguments[pos..(pos + 4)]);
    });

    Ok(())
}

impl AvcStatus for WeissAvcParamCmd {
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        build_param_command_status_data(self)?;
        AvcStatus::build_operands(&mut self.op, addr)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AvcStatus::parse_operands(&mut self.op, addr, operands)?;
        parse_param_command_response_data(self)
    }
}

impl AvcControl for WeissAvcParamCmd {
    fn build_operands(&mut self, addr: &AvcAddr) -> Result<Vec<u8>, AvcCmdBuildError> {
        build_param_command_control_data(self)?;
        AvcControl::build_operands(&mut self.op, addr)
    }

    fn parse_operands(&mut self, addr: &AvcAddr, operands: &[u8]) -> Result<(), AvcRespParseError> {
        AvcControl::parse_operands(&mut self.op, addr, operands)?;
        parse_param_command_response_data(self)
    }
}

impl WeissMan301Protocol {
    const PARAM_ID_DIGITAL_CAPTURE_SOURCE: u32 = 0;
    const PARAM_ID_DIGITAL_OUTPUT_MODE: u32 = 1;
    const PARAM_ID_WORD_CLOCK_OUTPUT_HALF_RATE: u32 = 2;
    const PARAM_ID_AESEBU_XLR_OUTPUT_MUTE: u32 = 3;
    const PARAM_ID_SPDIF_COAXIAL_OUTPUT_MUTE: u32 = 4;
    const PARAM_ID_ANALOG_OUTPUT_POLARITY_INVERSION: u32 = 5;
    const PARAM_ID_ANALOG_OUTPUT_FILTER_TYPE: u32 = 6;
    const PARAM_ID_ANALOG_OUTPUT_MUTE: u32 = 7;
    const PARAM_ID_ANALOG_OUTPUT_LEVEL: u32 = 8;
}

/// Convert between parameter and command.
pub trait WeissAvcParamConvert<T> {
    /// Build data for status command.
    fn build_status_data(cmd: &mut WeissAvcParamCmd) -> Result<(), Error>;
    /// Build data for control command.
    fn build_control_data(param: &T, cmd: &mut WeissAvcParamCmd) -> Result<(), Error>;
    /// Parse data for response.
    fn parse_response_data(param: &mut T, cmd: &WeissAvcParamCmd) -> Result<(), Error>;
}

/// The source of digital input captured for output packet stream.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WeissAvcDigitalCaptureSource {
    /// AES/EBU signal in XLR input interface.
    AesebuXlr,
    /// S/PDIF signal in coaxial input interface.
    SpdifCoaxial,
    /// S/PDIF signal in optical input interface.
    SpdifOptical,
}

impl Default for WeissAvcDigitalCaptureSource {
    fn default() -> Self {
        Self::AesebuXlr
    }
}

impl WeissAvcParamConvert<WeissAvcDigitalCaptureSource> for WeissMan301Protocol {
    fn build_status_data(cmd: &mut WeissAvcParamCmd) -> Result<(), Error> {
        cmd.numeric_id = Self::PARAM_ID_DIGITAL_CAPTURE_SOURCE;
        cmd.value = u32::MAX;
        Ok(())
    }

    fn build_control_data(
        param: &WeissAvcDigitalCaptureSource,
        cmd: &mut WeissAvcParamCmd,
    ) -> Result<(), Error> {
        cmd.numeric_id = Self::PARAM_ID_DIGITAL_CAPTURE_SOURCE;
        cmd.value = match param {
            WeissAvcDigitalCaptureSource::SpdifOptical => 2,
            WeissAvcDigitalCaptureSource::SpdifCoaxial => 1,
            WeissAvcDigitalCaptureSource::AesebuXlr => 0,
        };
        Ok(())
    }

    fn parse_response_data(
        param: &mut WeissAvcDigitalCaptureSource,
        cmd: &WeissAvcParamCmd,
    ) -> Result<(), Error> {
        assert_eq!(cmd.numeric_id, Self::PARAM_ID_DIGITAL_CAPTURE_SOURCE);
        *param = match cmd.value {
            2 => WeissAvcDigitalCaptureSource::SpdifOptical,
            1 => WeissAvcDigitalCaptureSource::SpdifCoaxial,
            _ => WeissAvcDigitalCaptureSource::AesebuXlr,
        };
        Ok(())
    }
}

/// The mode of AES/EBU XLR output and S/PDIF coaxial output. When enabled, it is in "dual wire"
/// mode at 176.4/192.0 kHz; i.e. the former is for the left audio channel, and the latter is for
/// right audio channel.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct WeissAvcDigitalOutputMode(pub bool);

impl WeissAvcParamConvert<WeissAvcDigitalOutputMode> for WeissMan301Protocol {
    fn build_status_data(cmd: &mut WeissAvcParamCmd) -> Result<(), Error> {
        cmd.numeric_id = Self::PARAM_ID_DIGITAL_OUTPUT_MODE;
        cmd.value = u32::MAX;
        Ok(())
    }

    fn build_control_data(
        param: &WeissAvcDigitalOutputMode,
        cmd: &mut WeissAvcParamCmd,
    ) -> Result<(), Error> {
        cmd.numeric_id = Self::PARAM_ID_DIGITAL_OUTPUT_MODE;
        cmd.value = if param.0 { 1 } else { 0 };
        Ok(())
    }

    fn parse_response_data(
        param: &mut WeissAvcDigitalOutputMode,
        cmd: &WeissAvcParamCmd,
    ) -> Result<(), Error> {
        assert_eq!(cmd.numeric_id, Self::PARAM_ID_DIGITAL_OUTPUT_MODE);
        param.0 = cmd.value > 0;
        Ok(())
    }
}

/// The mode of word clock at dual wire mode. When enabled, the output of BNC for word clock is at
/// half of sampling rate.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct WeissAvcWordClockOutputHalfRate(pub bool);

impl WeissAvcParamConvert<WeissAvcWordClockOutputHalfRate> for WeissMan301Protocol {
    fn build_status_data(cmd: &mut WeissAvcParamCmd) -> Result<(), Error> {
        cmd.numeric_id = Self::PARAM_ID_WORD_CLOCK_OUTPUT_HALF_RATE;
        cmd.value = u32::MAX;
        Ok(())
    }

    fn build_control_data(
        param: &WeissAvcWordClockOutputHalfRate,
        cmd: &mut WeissAvcParamCmd,
    ) -> Result<(), Error> {
        cmd.numeric_id = Self::PARAM_ID_WORD_CLOCK_OUTPUT_HALF_RATE;
        cmd.value = if param.0 { 1 } else { 0 };
        Ok(())
    }

    fn parse_response_data(
        param: &mut WeissAvcWordClockOutputHalfRate,
        cmd: &WeissAvcParamCmd,
    ) -> Result<(), Error> {
        assert_eq!(cmd.numeric_id, Self::PARAM_ID_WORD_CLOCK_OUTPUT_HALF_RATE);
        param.0 = cmd.value > 0;
        Ok(())
    }
}

/// Mute XLR output.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct WeissAvcAesebuXlrOutputMute(pub bool);

impl WeissAvcParamConvert<WeissAvcAesebuXlrOutputMute> for WeissMan301Protocol {
    fn build_status_data(cmd: &mut WeissAvcParamCmd) -> Result<(), Error> {
        cmd.numeric_id = Self::PARAM_ID_AESEBU_XLR_OUTPUT_MUTE;
        cmd.value = u32::MAX;
        Ok(())
    }

    fn build_control_data(
        param: &WeissAvcAesebuXlrOutputMute,
        cmd: &mut WeissAvcParamCmd,
    ) -> Result<(), Error> {
        cmd.numeric_id = Self::PARAM_ID_AESEBU_XLR_OUTPUT_MUTE;
        cmd.value = if param.0 { 0 } else { 1 };
        Ok(())
    }

    fn parse_response_data(
        param: &mut WeissAvcAesebuXlrOutputMute,
        cmd: &WeissAvcParamCmd,
    ) -> Result<(), Error> {
        assert_eq!(cmd.numeric_id, Self::PARAM_ID_AESEBU_XLR_OUTPUT_MUTE);
        param.0 = cmd.value == 0;
        Ok(())
    }
}

/// Mute coaxial output.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct WeissAvcSpdifCoaxialOutputMute(pub bool);

impl WeissAvcParamConvert<WeissAvcSpdifCoaxialOutputMute> for WeissMan301Protocol {
    fn build_status_data(cmd: &mut WeissAvcParamCmd) -> Result<(), Error> {
        cmd.numeric_id = Self::PARAM_ID_SPDIF_COAXIAL_OUTPUT_MUTE;
        cmd.value = u32::MAX;
        Ok(())
    }

    fn build_control_data(
        param: &WeissAvcSpdifCoaxialOutputMute,
        cmd: &mut WeissAvcParamCmd,
    ) -> Result<(), Error> {
        cmd.numeric_id = Self::PARAM_ID_SPDIF_COAXIAL_OUTPUT_MUTE;
        cmd.value = if param.0 { 0 } else { 1 };
        Ok(())
    }

    fn parse_response_data(
        param: &mut WeissAvcSpdifCoaxialOutputMute,
        cmd: &WeissAvcParamCmd,
    ) -> Result<(), Error> {
        assert_eq!(cmd.numeric_id, Self::PARAM_ID_SPDIF_COAXIAL_OUTPUT_MUTE);
        param.0 = cmd.value == 0;
        Ok(())
    }
}

/// Invert polarity of analog output.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct WeissAvcAnalogOutputPolarityInversion(pub bool);

impl WeissAvcParamConvert<WeissAvcAnalogOutputPolarityInversion> for WeissMan301Protocol {
    fn build_status_data(cmd: &mut WeissAvcParamCmd) -> Result<(), Error> {
        cmd.numeric_id = Self::PARAM_ID_ANALOG_OUTPUT_POLARITY_INVERSION;
        cmd.value = u32::MAX;
        Ok(())
    }

    fn build_control_data(
        param: &WeissAvcAnalogOutputPolarityInversion,
        cmd: &mut WeissAvcParamCmd,
    ) -> Result<(), Error> {
        cmd.numeric_id = Self::PARAM_ID_ANALOG_OUTPUT_POLARITY_INVERSION;
        cmd.value = if param.0 { 1 } else { 0 };
        Ok(())
    }

    fn parse_response_data(
        param: &mut WeissAvcAnalogOutputPolarityInversion,
        cmd: &WeissAvcParamCmd,
    ) -> Result<(), Error> {
        assert_eq!(
            cmd.numeric_id,
            Self::PARAM_ID_ANALOG_OUTPUT_POLARITY_INVERSION
        );
        param.0 = cmd.value > 0;
        Ok(())
    }
}

/// The type of oversampling filter in analog output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WeissAvcAnalogOutputFilterType {
    A,
    B,
}

impl Default for WeissAvcAnalogOutputFilterType {
    fn default() -> Self {
        Self::A
    }
}

impl WeissAvcParamConvert<WeissAvcAnalogOutputFilterType> for WeissMan301Protocol {
    fn build_status_data(cmd: &mut WeissAvcParamCmd) -> Result<(), Error> {
        cmd.numeric_id = Self::PARAM_ID_ANALOG_OUTPUT_FILTER_TYPE;
        cmd.value = u32::MAX;
        Ok(())
    }

    fn build_control_data(
        param: &WeissAvcAnalogOutputFilterType,
        cmd: &mut WeissAvcParamCmd,
    ) -> Result<(), Error> {
        cmd.numeric_id = Self::PARAM_ID_ANALOG_OUTPUT_FILTER_TYPE;
        cmd.value = if param.eq(&WeissAvcAnalogOutputFilterType::B) {
            1
        } else {
            0
        };
        Ok(())
    }

    fn parse_response_data(
        param: &mut WeissAvcAnalogOutputFilterType,
        cmd: &WeissAvcParamCmd,
    ) -> Result<(), Error> {
        assert_eq!(cmd.numeric_id, Self::PARAM_ID_ANALOG_OUTPUT_FILTER_TYPE);
        *param = if cmd.value > 0 {
            WeissAvcAnalogOutputFilterType::B
        } else {
            WeissAvcAnalogOutputFilterType::A
        };
        Ok(())
    }
}

/// Mute analog output.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct WeissAvcAnalogOutputMute(pub bool);

impl WeissAvcParamConvert<WeissAvcAnalogOutputMute> for WeissMan301Protocol {
    fn build_status_data(cmd: &mut WeissAvcParamCmd) -> Result<(), Error> {
        cmd.numeric_id = Self::PARAM_ID_ANALOG_OUTPUT_MUTE;
        cmd.value = u32::MAX;
        Ok(())
    }

    fn build_control_data(
        param: &WeissAvcAnalogOutputMute,
        cmd: &mut WeissAvcParamCmd,
    ) -> Result<(), Error> {
        cmd.numeric_id = Self::PARAM_ID_ANALOG_OUTPUT_MUTE;
        cmd.value = if param.0 { 0 } else { 1 };
        Ok(())
    }

    fn parse_response_data(
        param: &mut WeissAvcAnalogOutputMute,
        cmd: &WeissAvcParamCmd,
    ) -> Result<(), Error> {
        assert_eq!(cmd.numeric_id, Self::PARAM_ID_ANALOG_OUTPUT_MUTE);
        param.0 = cmd.value == 0;
        Ok(())
    }
}

/// The level of analog output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WeissAvcAnalogOutputLevel {
    /// 0 dB.
    Zero,
    /// -10 dB.
    NegativeTen,
    /// -20 dB.
    NegativeTwenty,
    /// -30 dB.
    NegativeThirty,
}

impl Default for WeissAvcAnalogOutputLevel {
    fn default() -> Self {
        Self::Zero
    }
}

impl WeissAvcParamConvert<WeissAvcAnalogOutputLevel> for WeissMan301Protocol {
    fn build_status_data(cmd: &mut WeissAvcParamCmd) -> Result<(), Error> {
        cmd.numeric_id = Self::PARAM_ID_ANALOG_OUTPUT_LEVEL;
        cmd.value = u32::MAX;
        Ok(())
    }

    fn build_control_data(
        param: &WeissAvcAnalogOutputLevel,
        cmd: &mut WeissAvcParamCmd,
    ) -> Result<(), Error> {
        cmd.numeric_id = Self::PARAM_ID_ANALOG_OUTPUT_LEVEL;
        cmd.value = match param {
            WeissAvcAnalogOutputLevel::NegativeThirty => 3,
            WeissAvcAnalogOutputLevel::NegativeTwenty => 2,
            WeissAvcAnalogOutputLevel::NegativeTen => 1,
            WeissAvcAnalogOutputLevel::Zero => 0,
        };
        Ok(())
    }

    fn parse_response_data(
        param: &mut WeissAvcAnalogOutputLevel,
        cmd: &WeissAvcParamCmd,
    ) -> Result<(), Error> {
        assert_eq!(cmd.numeric_id, Self::PARAM_ID_ANALOG_OUTPUT_LEVEL);
        *param = match cmd.value {
            3 => WeissAvcAnalogOutputLevel::NegativeThirty,
            2 => WeissAvcAnalogOutputLevel::NegativeTwenty,
            1 => WeissAvcAnalogOutputLevel::NegativeTen,
            _ => WeissAvcAnalogOutputLevel::Zero,
        };
        Ok(())
    }
}

/// The operation for parameters in Weiss AV/C protocol.
pub trait WeissAvcParamOperation<T>: WeissAvcParamConvert<T> {
    /// Cache current state of parameter.
    fn cache_param(fcp: &WeissAvc, param: &mut T, timeout_ms: u32) -> Result<(), Error>;
    /// Update the state of parameter.
    fn update_param(fcp: &WeissAvc, param: &mut T, timeout_ms: u32) -> Result<(), Error>;
}

impl<T> WeissAvcParamOperation<T> for WeissMan301Protocol
where
    WeissMan301Protocol: WeissAvcParamConvert<T>,
{
    fn cache_param(fcp: &WeissAvc, param: &mut T, timeout_ms: u32) -> Result<(), Error> {
        let mut cmd = WeissAvcParamCmd::default();
        Self::build_status_data(&mut cmd)?;
        fcp.status(&AvcAddr::Unit, &mut cmd, timeout_ms)?;
        Self::parse_response_data(param, &cmd)?;
        Ok(())
    }

    fn update_param(fcp: &WeissAvc, param: &mut T, timeout_ms: u32) -> Result<(), Error> {
        let mut cmd = WeissAvcParamCmd::default();
        Self::build_control_data(param, &mut cmd)?;
        fcp.control(&AvcAddr::Unit, &mut cmd, timeout_ms)?;
        Self::parse_response_data(param, &cmd)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn weiss_avc_param_command_operands() {
        let operands = [
            0x00, 0x1c, 0x6a, 0x01, 0x7f, 0x80, 0x02, 0x76, 0x54, 0x32, 0x10, 0xfe, 0xdc, 0xba,
            0x98, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff,
        ];
        let mut op = WeissAvcParamCmd::default();
        AvcStatus::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.op.class_id, 0x01);
        assert_eq!(op.op.sequence_id, 0x7f);
        assert_eq!(op.op.command_id, 0x8002);
        assert_eq!(op.op.arguments, &operands[7..]);
        assert_eq!(op.numeric_id, 0x76543210);
        assert_eq!(op.value, 0xfedcba98);

        let target = AvcStatus::build_operands(&mut op, &AvcAddr::Unit).unwrap();
        assert_eq!(&target[..4], &operands[..4]);
        // The value of sequence_id field is never matched.
        assert_eq!(&target[5..], &operands[5..]);

        let target = AvcControl::build_operands(&mut op, &AvcAddr::Unit).unwrap();
        assert_eq!(&target[..4], &operands[..4]);
        // The value of sequence_id field is never matched.
        assert_eq!(&target[5..], &operands[5..]);

        let mut op = WeissAvcParamCmd::default();
        AvcControl::parse_operands(&mut op, &AvcAddr::Unit, &operands).unwrap();
        assert_eq!(op.op.class_id, 0x01);
        assert_eq!(op.op.sequence_id, 0x7f);
        assert_eq!(op.op.command_id, 0x8002);
        assert_eq!(op.op.arguments, &operands[7..]);
        assert_eq!(op.numeric_id, 0x76543210);
        assert_eq!(op.value, 0xfedcba98);
    }
}
