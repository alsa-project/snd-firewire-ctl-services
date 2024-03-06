// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

#![doc = include_str!("../README.md")]

pub mod apogee;
pub mod griffin;
pub mod lacie;
pub mod loud;
pub mod oxford;
pub mod tascam;

use {
    glib::{prelude::IsA, Error, FileError},
    hinawa::{
        prelude::{FwFcpExt, FwFcpExtManual, FwReqExtManual},
        FwFcp, FwNode, FwReq, FwTcode,
    },
    oxford::*,
    ta1394_avc_audio::{amdtp::*, *},
    ta1394_avc_ccm::*,
    ta1394_avc_general::{general::*, *},
    ta1394_avc_stream_format::*,
};

/// The implementation of AV/C transaction.
#[derive(Default, Debug)]
pub struct OxfwAvc(FwFcp);

impl Ta1394Avc<Error> for OxfwAvc {
    fn transaction(&self, command_frame: &[u8], timeout_ms: u32) -> Result<Vec<u8>, Error> {
        let mut resp = vec![0; Self::FRAME_SIZE];
        self.0
            .avc_transaction(&command_frame, &mut resp, timeout_ms)
            .map(|_| resp)
    }
}

impl OxfwAvc {
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

fn from_avc_err(err: Ta1394AvcError<Error>) -> Error {
    match err {
        Ta1394AvcError::CmdBuild(cause) => Error::new(FileError::Inval, &cause.to_string()),
        Ta1394AvcError::CommunicationFailure(cause) => cause,
        Ta1394AvcError::RespParse(cause) => Error::new(FileError::Io, &cause.to_string()),
    }
}

/// Operation for read-only parameters by AV/C command in FCP.
pub trait OxfwFcpParamsOperation<P, T>
where
    P: Ta1394Avc<Error>,
{
    /// Cache state of hardware for the parameter.
    fn cache(avc: &mut P, params: &mut T, timeout_ms: u32) -> Result<(), Error>;
}

/// Operation for mutable parameters by AV/C command in FCP.
pub trait OxfwFcpMutableParamsOperation<P, T>
where
    P: Ta1394Avc<Error>,
{
    /// Update state of hardware when detecting any change between the given parameters.
    fn update(avc: &mut P, params: &T, prev: &mut T, timeout_ms: u32) -> Result<(), Error>;
}

/// Specification of audio function block.
pub trait OxfwAudioFbSpecification {
    /// The numeric identifier of audio function block for volume control.
    const VOLUME_FB_ID: u8;
    /// The numeric identifier of audio function block for mute control.
    const MUTE_FB_ID: u8;

    /// List to map raw data into preferred data order.
    const CHANNEL_MAP: &'static [usize];

    /// The minimum value of volume.
    const VOLUME_MIN: i16 = VolumeData::VALUE_NEG_INFINITY;
    /// The maximum value of volume.
    const VOLUME_MAX: i16 = VolumeData::VALUE_ZERO;

    /// Instantiate parameters for output volume.
    fn create_output_volume_params() -> OxfwOutputVolumeParams {
        OxfwOutputVolumeParams(vec![Default::default(); Self::CHANNEL_MAP.len()])
    }
}

/// Parameters of volume for output..
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxfwOutputVolumeParams(pub Vec<i16>);

impl<O, P> OxfwFcpParamsOperation<P, OxfwOutputVolumeParams> for O
where
    O: OxfwAudioFbSpecification,
    P: Ta1394Avc<Error>,
{
    fn cache(
        avc: &mut P,
        params: &mut OxfwOutputVolumeParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        assert!(params.0.len() >= Self::CHANNEL_MAP.len());
        let mut op = AudioFeature::new(
            Self::VOLUME_FB_ID,
            CtlAttr::Current,
            AudioCh::All,
            FeatureCtl::Volume(VolumeData::new(params.0.len())),
        );
        avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
            .map_err(|err| from_avc_err(err))?;
        if let FeatureCtl::Volume(data) = op.ctl {
            data.0
                .iter()
                .zip(Self::CHANNEL_MAP)
                .for_each(|(&vol, &pos)| {
                    params.0[pos] = vol;
                });
        }
        Ok(())
    }
}

impl<O, P> OxfwFcpMutableParamsOperation<P, OxfwOutputVolumeParams> for O
where
    O: OxfwAudioFbSpecification,
    P: Ta1394Avc<Error>,
{
    fn update(
        avc: &mut P,
        params: &OxfwOutputVolumeParams,
        prev: &mut OxfwOutputVolumeParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if params != prev {
            let vols: Vec<i16> = Self::CHANNEL_MAP.iter().map(|&pos| params.0[pos]).collect();
            let mut op = AudioFeature::new(
                Self::VOLUME_FB_ID,
                CtlAttr::Current,
                AudioCh::All,
                FeatureCtl::Volume(VolumeData(vols)),
            );
            avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
                .map_err(|err| from_avc_err(err))?;
        }
        prev.0.iter_mut().zip(&params.0).for_each(|(o, n)| *o = *n);
        Ok(())
    }
}

/// Parameters of mute for output.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct OxfwOutputMuteParams(pub bool);

impl<O, P> OxfwFcpParamsOperation<P, OxfwOutputMuteParams> for O
where
    O: OxfwAudioFbSpecification,
    P: Ta1394Avc<Error>,
{
    fn cache(avc: &mut P, params: &mut OxfwOutputMuteParams, timeout_ms: u32) -> Result<(), Error> {
        let mut op = AudioFeature::new(
            Self::MUTE_FB_ID,
            CtlAttr::Current,
            AudioCh::Master,
            FeatureCtl::Mute(vec![Default::default()]),
        );
        avc.status(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
            .map_err(|err| from_avc_err(err))?;
        if let FeatureCtl::Mute(data) = op.ctl {
            params.0 = data[0]
        }
        Ok(())
    }
}

impl<O, P> OxfwFcpMutableParamsOperation<P, OxfwOutputMuteParams> for O
where
    O: OxfwAudioFbSpecification,
    P: Ta1394Avc<Error>,
{
    fn update(
        avc: &mut P,
        params: &OxfwOutputMuteParams,
        prev: &mut OxfwOutputMuteParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if params != prev {
            let mut op = AudioFeature::new(
                Self::MUTE_FB_ID,
                CtlAttr::Current,
                AudioCh::Master,
                FeatureCtl::Mute(vec![params.0]),
            );
            avc.control(&AUDIO_SUBUNIT_0_ADDR, &mut op, timeout_ms)
                .map_err(|err| from_avc_err(err))?;
        }
        prev.0 = params.0;
        Ok(())
    }
}

/// Parameters for stream formats.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct OxfwStreamFormatState {
    /// Direction for packet stream.
    pub direction: PlugDirection,
    /// Available stream formats.
    pub format_entries: Vec<CompoundAm824Stream>,
    /// Whether to assumed or not.
    pub assumed: bool,
}

fn compound_am824_from_format(stream_format: &StreamFormat) -> Result<&CompoundAm824Stream, Error> {
    stream_format.as_compound_am824_stream().ok_or_else(|| {
        let msg = "Compound AM824 stream format is not available";
        Error::new(FileError::Nxio, &msg)
    })
}

const SUPPORTED_RATES: &[u32] = &[32000, 44100, 48000, 88200, 96000, 176400, 192000];

/// Operation for stream format.
pub trait OxfwStreamFormatOperation<T>
where
    T: Ta1394Avc<Error>,
{
    /// Detect available stream formats.
    fn detect_stream_formats(
        avc: &mut T,
        params: &mut OxfwStreamFormatState,
        is_output: bool,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let direction = if is_output {
            PlugDirection::Output
        } else {
            PlugDirection::Input
        };
        let plug_addr = PlugAddr {
            direction,
            mode: PlugAddrMode::Unit(UnitPlugData {
                unit_type: UnitPlugType::Pcr,
                plug_id: 0,
            }),
        };

        let mut op = ExtendedStreamFormatList::new(&plug_addr, 0);

        if avc.status(&AvcAddr::Unit, &mut op, timeout_ms).is_ok() {
            loop {
                compound_am824_from_format(&op.stream_format)
                    .map(|stream_format| params.format_entries.push(stream_format.clone()))?;

                op.index += 1;
                if let Err(err) = avc.status(&AvcAddr::Unit, &mut op, timeout_ms) {
                    if err == Ta1394AvcError::RespParse(AvcRespParseError::UnexpectedStatus) {
                        break;
                    } else {
                        Err(from_avc_err(err))?;
                    }
                }
            }

            params.assumed = false;
        } else {
            // Fallback. At first, retrieve current format information.
            let mut op = ExtendedStreamFormatSingle::new(&plug_addr);
            avc.status(&AvcAddr::Unit, &mut op, timeout_ms)
                .map_err(|err| from_avc_err(err))?;

            let stream_format = compound_am824_from_format(&op.stream_format)?;

            // Next, inquire supported sampling rates and make entries.
            SUPPORTED_RATES.iter().for_each(|&freq| {
                let fdf: [u8; 3] = AmdtpFdf::new(AmdtpEventType::Am824, false, freq).into();
                let fmt = PlugSignalFormat {
                    plug_id: 0,
                    fmt: FMT_IS_AMDTP,
                    fdf,
                };

                if direction == PlugDirection::Input {
                    let mut op = InputPlugSignalFormat(fmt);
                    if avc
                        .specific_inquiry(&AvcAddr::Unit, &mut op, timeout_ms)
                        .is_err()
                    {
                        return;
                    }
                } else {
                    let mut op = OutputPlugSignalFormat(fmt);
                    if avc
                        .specific_inquiry(&AvcAddr::Unit, &mut op, timeout_ms)
                        .is_err()
                    {
                        return;
                    }
                }

                let mut entry = stream_format.clone();
                entry.freq = freq;
                params.format_entries.push(entry);
            });

            params.assumed = true;
        }

        params.direction = direction;

        Ok(())
    }

    /// Read current stream format.
    fn read_stream_format(
        avc: &mut T,
        params: &OxfwStreamFormatState,
        timeout_ms: u32,
    ) -> Result<usize, Error> {
        let plug_addr = PlugAddr {
            direction: params.direction,
            mode: PlugAddrMode::Unit(UnitPlugData {
                unit_type: UnitPlugType::Pcr,
                plug_id: 0,
            }),
        };
        let mut op = ExtendedStreamFormatSingle::new(&plug_addr);
        avc.status(&AvcAddr::Unit, &mut op, timeout_ms)
            .map_err(|err| from_avc_err(err))?;
        let stream_format = compound_am824_from_format(&op.stream_format)?;
        params
            .format_entries
            .iter()
            .position(|fmt| stream_format.eq(fmt))
            .ok_or_else(|| {
                let msg = "Detected stream format is not matched";
                Error::new(FileError::Nxio, &msg)
            })
    }

    /// Write stream format to change parameters of isochronous packet streaming.
    fn write_stream_format(
        avc: &mut T,
        params: &OxfwStreamFormatState,
        format_index: usize,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let format = params
            .format_entries
            .iter()
            .nth(format_index)
            .cloned()
            .unwrap();

        if !params.assumed {
            let plug_addr = PlugAddr {
                direction: params.direction,
                mode: PlugAddrMode::Unit(UnitPlugData {
                    unit_type: UnitPlugType::Pcr,
                    plug_id: 0,
                }),
            };
            let mut op = ExtendedStreamFormatSingle::new(&plug_addr);
            op.stream_format = StreamFormat::Am(AmStream::CompoundAm824(format));
            avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
                .map_err(|err| from_avc_err(err))
        } else {
            let fdf: [u8; 3] = AmdtpFdf::new(AmdtpEventType::Am824, false, format.freq).into();
            let fmt = PlugSignalFormat {
                plug_id: 0,
                fmt: FMT_IS_AMDTP,
                fdf,
            };
            if params.direction == PlugDirection::Input {
                let mut op = InputPlugSignalFormat(fmt);
                avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
                    .map_err(|err| from_avc_err(err))
            } else {
                let mut op = OutputPlugSignalFormat(fmt);
                avc.control(&AvcAddr::Unit, &mut op, timeout_ms)
                    .map_err(|err| from_avc_err(err))
            }
        }
    }
}
