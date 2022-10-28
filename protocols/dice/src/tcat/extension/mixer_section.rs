// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Mixer section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for mixer section
//! in protocol extension defined by TCAT for ASICs of DICE.
use super::{caps_section::*, *};

/// Parameters of saturation in mixer section.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct MixerSaturationParams(pub Vec<bool>);

/// Parameters of coefficients in mixer section.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct MixerCoefficientParams(pub Vec<Vec<u16>>);

const SATURATION_OFFSET: usize = 0x00;
const COEFF_OFFSET: usize = 0x04;

const MAX_OUTPUT_COUNT: usize = 16;
const MAX_INPUT_COUNT: usize = 18;

fn calculate_mixer_coefficients_size() -> usize {
    4 * MAX_OUTPUT_COUNT * MAX_INPUT_COUNT
}

fn serialize_mixer_coefficients<T: AsRef<[u16]>>(
    coefs: &[T],
    caps: &MixerCaps,
    raw: &mut [u8],
) -> Result<(), String> {
    assert!(raw.len() >= calculate_mixer_coefficients_size());

    coefs
        .iter()
        .take(caps.output_count as usize)
        .enumerate()
        .for_each(|(i, entries)| {
            entries
                .as_ref()
                .iter()
                .take(caps.input_count as usize)
                .enumerate()
                .for_each(|(j, &coef)| {
                    let pos = 4 * (i * MAX_INPUT_COUNT + j);
                    serialize_u32(&(coef as u32), &mut raw[pos..(pos + 4)]);
                });
        });

    Ok(())
}

fn deserialize_mixer_coefficients<T: AsMut<[u16]>>(
    coefs: &mut [T],
    caps: &MixerCaps,
    raw: &[u8],
) -> Result<(), String> {
    assert!(raw.len() >= calculate_mixer_coefficients_size());

    coefs
        .iter_mut()
        .take(caps.output_count as usize)
        .enumerate()
        .for_each(|(i, entries)| {
            entries
                .as_mut()
                .iter_mut()
                .take(caps.input_count as usize)
                .enumerate()
                .for_each(|(j, coef)| {
                    let pos = 4 * (i * MAX_INPUT_COUNT + j);
                    let mut val = 0u32;
                    deserialize_u32(&mut val, &raw[pos..(pos + 4)]);
                    *coef = val as u16
                });
        });

    Ok(())
}

impl<O: TcatExtensionOperation> TcatExtensionSectionParamsOperation<MixerSaturationParams> for O {
    fn cache_extension_whole_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        params: &mut MixerSaturationParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if !caps.mixer.is_exposed {
            Err(Error::new(
                ProtocolExtensionError::Mixer,
                "Mixer is not available",
            ))?
        }

        let mut raw = [0; 4];
        Self::read_extension(
            req,
            node,
            &sections.mixer,
            SATURATION_OFFSET,
            &mut raw,
            timeout_ms,
        )?;

        let mut val = 0u32;
        deserialize_u32(&mut val, &raw);

        params
            .0
            .resize_with(caps.mixer.output_count as usize, Default::default);
        params
            .0
            .iter_mut()
            .enumerate()
            .for_each(|(i, saturation)| *saturation = val & (1 << i) > 0);

        Ok(())
    }
}

impl<O: TcatExtensionOperation> TcatExtensionSectionParamsOperation<MixerCoefficientParams> for O {
    fn cache_extension_whole_params(
        req: &FwReq,
        node: &FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        params: &mut MixerCoefficientParams,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if !caps.mixer.is_exposed {
            Err(Error::new(
                ProtocolExtensionError::Mixer,
                "Mixer is not available",
            ))?
        }

        let mut raw = vec![0u8; calculate_mixer_coefficients_size()];
        Self::read_extension(
            req,
            node,
            &sections.mixer,
            COEFF_OFFSET,
            &mut raw,
            timeout_ms,
        )?;

        params
            .0
            .resize_with(caps.mixer.output_count as usize, Default::default);
        params
            .0
            .iter_mut()
            .for_each(|coefs| coefs.resize_with(caps.mixer.input_count as usize, Default::default));
        deserialize_mixer_coefficients(&mut params.0, &caps.mixer, &raw)
            .map_err(|cause| Error::new(ProtocolExtensionError::Mixer, &cause))
    }
}

/// Protocol implementation of mixer section.
#[derive(Default)]
pub struct MixerSectionProtocol;

impl MixerSectionProtocol {
    const SATURATION_OFFSET: usize = 0x00;
    const COEFF_OFFSET: usize = 0x04;

    /// Cache state of hardware for mixer saturation.
    pub fn cache_mixer_whole_saturation(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        saturations: &mut [bool],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if !caps.mixer.is_exposed {
            Err(Error::new(
                ProtocolExtensionError::Mixer,
                "Mixer is not available",
            ))?
        }

        let mut raw = [0; 4];
        extension_read(
            req,
            node,
            sections.mixer.offset + Self::SATURATION_OFFSET,
            &mut raw,
            timeout_ms,
        )?;

        let mut val = 0u32;
        deserialize_u32(&mut val, &raw);

        saturations
            .iter_mut()
            .enumerate()
            .for_each(|(i, saturation)| *saturation = val & (1 << i) > 0);

        Ok(())
    }

    /// Cache state of hardware for mixer coefficients.
    pub fn cache_mixer_whole_coefficients<T: AsMut<[u16]>>(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        coefs: &mut [T],
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if !caps.mixer.is_exposed {
            Err(Error::new(
                ProtocolExtensionError::Mixer,
                "Mixer is not available",
            ))?
        }

        let mut raw = vec![0u8; calculate_mixer_coefficients_size()];
        extension_read(
            req,
            node,
            sections.mixer.offset + Self::COEFF_OFFSET,
            &mut raw,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::Mixer, &e.to_string()))?;

        deserialize_mixer_coefficients(coefs, &caps.mixer, &raw)
            .map_err(|cause| Error::new(ProtocolExtensionError::Mixer, &cause))
    }

    /// Update state of hardware for mixer coefficients.
    pub fn update_mixer_partial_coefficients<T, U>(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        coefs: &[T],
        prev: &mut [U],
        timeout_ms: u32,
    ) -> Result<(), Error>
    where
        T: AsRef<[u16]>,
        U: AsRef<[u16]> + AsMut<[u16]>,
    {
        let mut new = vec![0u8; calculate_mixer_coefficients_size()];
        serialize_mixer_coefficients(coefs, &caps.mixer, &mut new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Mixer, &cause))?;

        let mut old = vec![0u8; calculate_mixer_coefficients_size()];
        serialize_mixer_coefficients(prev, &caps.mixer, &mut old)
            .map_err(|cause| Error::new(ProtocolExtensionError::Mixer, &cause))?;

        (0..calculate_mixer_coefficients_size())
            .step_by(4)
            .try_for_each(|pos| {
                if new[pos..(pos + 4)] != old[pos..(pos + 4)] {
                    extension_write(
                        req,
                        node,
                        sections.mixer.offset + Self::COEFF_OFFSET + pos,
                        &mut new[pos..(pos + 4)],
                        timeout_ms,
                    )
                } else {
                    Ok(())
                }
            })?;

        deserialize_mixer_coefficients(prev, &caps.mixer, &new)
            .map_err(|cause| Error::new(ProtocolExtensionError::Mixer, &cause))
    }
}
