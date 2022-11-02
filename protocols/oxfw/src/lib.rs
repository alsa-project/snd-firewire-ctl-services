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
    glib::{Error, FileError, IsA},
    hinawa::{
        prelude::{FwFcpExt, FwFcpExtManual, FwReqExtManual},
        FwFcp, FwNode, FwReq, FwTcode,
    },
    ta1394_avc_audio::*,
    ta1394_avc_ccm::*,
    ta1394_avc_general::{general::*, *},
};

/// The implementation of AV/C transaction.
#[derive(Default, Debug)]
pub struct OxfwAvc(FwFcp);

impl Ta1394Avc<Error> for OxfwAvc {
    fn transaction(&self, command_frame: &[u8], timeout_ms: u32) -> Result<Vec<u8>, Error> {
        let mut resp = vec![0; Self::FRAME_SIZE];
        self.0
            .avc_transaction(&command_frame, &mut resp, timeout_ms)
            .map(|len| {
                resp.truncate(len);
                resp
            })
    }
}

impl OxfwAvc {
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
