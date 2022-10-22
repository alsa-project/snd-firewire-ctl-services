// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Application section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for application
//! section in protocol extension defined by TCAT for ASICs of DICE.

use super::*;

/// Serialize and deserialize parameters in application section.
pub trait ApplSectionParamsSerdes<T>
where
    T: Default + std::fmt::Debug,
{
    /// Offset in application section.
    const APPL_PARAMS_OFFSET: usize;

    /// Size of raw data.
    const APPL_PARAMS_SIZE: usize;

    /// Serialize parameters.
    fn serialize_appl_params(params: &T, raw: &mut [u8]) -> Result<(), String>;

    /// Deserialize parameters.
    fn deserialize_appl_params(params: &mut T, raw: &[u8]) -> Result<(), String>;
}

/// Operation for parameters in application section of TCAT protocol extension.
pub trait TcatApplSectionParamsOperation<T>:
    TcatExtensionOperation + ApplSectionParamsSerdes<T>
where
    T: Default + std::fmt::Debug,
{
    /// Cache state of hardware for whole parameters.
    fn cache_appl_whole_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        params: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut raw = vec![0u8; Self::APPL_PARAMS_SIZE];
        Self::read_extension(
            req,
            node,
            &sections.application,
            Self::APPL_PARAMS_OFFSET,
            &mut raw,
            timeout_ms,
        )?;
        Self::deserialize_appl_params(params, &raw)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }
}

/// Operation for mutable parameters in application section of TCAT protocol extension.
pub trait TcatApplSectionMutableParamsOperation<T>:
    TcatExtensionOperation + ApplSectionParamsSerdes<T>
where
    T: Default + std::fmt::Debug,
{
    /// Update state of hardware for partial parameters.
    fn update_appl_partial_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        params: &T,
        prev: &mut T,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let mut new = vec![0u8; Self::APPL_PARAMS_SIZE];
        Self::serialize_appl_params(params, &mut new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        let mut old = vec![0u8; Self::APPL_PARAMS_SIZE];
        Self::serialize_appl_params(prev, &mut old)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))?;

        (0..Self::APPL_PARAMS_SIZE).step_by(4).try_for_each(|pos| {
            if new[pos..(pos + 4)] != old[pos..(pos + 4)] {
                Self::write_extension(
                    req,
                    node,
                    &sections.application,
                    Self::APPL_PARAMS_OFFSET,
                    &mut new[pos..(pos + 4)],
                    timeout_ms,
                )
            } else {
                Ok(())
            }
        })?;

        Self::deserialize_appl_params(prev, &new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Appl, &cause))
    }
}

/// Operation for notified parameters in application section of TCAT protocol extension.
pub trait TcatApplSectionNotifiedParamsOperation<T>: TcatApplSectionParamsOperation<T>
where
    T: Default + std::fmt::Debug,
{
    /// Update state of hardware for partial parameters.
    fn cache_appl_notified_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        params: &mut T,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error>;
}
