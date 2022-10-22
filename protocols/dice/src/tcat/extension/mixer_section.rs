// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

//! Mixer section in protocol extension defined by TCAT for ASICs of DICE.
//!
//! The module includes structure, enumeration, and trait and its implementation for mixer section
//! in protocol extension defined by TCAT for ASICs of DICE.
use super::{caps_section::*, *};

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
                    (coef as u32).build_quadlet(&mut raw[pos..(pos + 4)]);
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
                    val.parse_quadlet(&raw[pos..(pos + 4)]);
                    *coef = val as u16
                });
        });

    Ok(())
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
        val.parse_quadlet(&raw);

        saturations
            .iter_mut()
            .enumerate()
            .for_each(|(i, saturation)| *saturation = val & (1 << i) > 0);

        Ok(())
    }

    pub fn read_saturation(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        timeout_ms: u32,
    ) -> Result<Vec<bool>, Error> {
        if !caps.mixer.is_exposed {
            Err(Error::new(
                ProtocolExtensionError::Mixer,
                "Mixer is not available",
            ))?
        }

        let mut data = [0; 4];
        extension_read(
            req,
            node,
            sections.mixer.offset + Self::SATURATION_OFFSET,
            &mut data,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::Mixer, &e.to_string()))
        .map(|_| {
            let val = u32::from_be_bytes(data);
            (0..caps.mixer.output_count)
                .map(|i| val & (1 << i) > 0)
                .collect::<Vec<_>>()
        })
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

    pub fn read_coef(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        dst: usize,
        src: usize,
        timeout_ms: u32,
    ) -> Result<u32, Error> {
        if !caps.mixer.is_exposed {
            Err(Error::new(
                ProtocolExtensionError::Mixer,
                "Mixer is not available",
            ))?
        }

        let offset = 4 * (src + dst * caps.mixer.input_count as usize);
        let mut data = [0; 4];
        extension_read(
            req,
            node,
            sections.mixer.offset + Self::COEFF_OFFSET + offset,
            &mut data,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::Mixer, &e.to_string()))
        .map(|_| u32::from_be_bytes(data))
    }

    pub fn write_coef(
        req: &mut FwReq,
        node: &mut FwNode,
        sections: &ExtensionSections,
        caps: &ExtensionCaps,
        dst: usize,
        src: usize,
        val: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if caps.mixer.is_readonly {
            Err(Error::new(
                ProtocolExtensionError::Mixer,
                "Mixer is immutable",
            ))?
        }

        let offset = 4 * (src + dst * caps.mixer.input_count as usize);
        let mut data = [0; 4];
        data.copy_from_slice(&val.to_be_bytes());
        extension_write(
            req,
            node,
            sections.mixer.offset + Self::COEFF_OFFSET + offset,
            &mut data,
            timeout_ms,
        )
        .map_err(|e| Error::new(ProtocolExtensionError::Mixer, &e.to_string()))
    }
}
