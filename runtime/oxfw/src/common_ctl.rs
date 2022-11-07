// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, std::marker::PhantomData, ta1394_avc_general::*, ta1394_avc_stream_format::*};

#[derive(Default, Debug)]
pub struct CommonCtl<P, T>
where
    P: Debug + Ta1394Avc<Error>,
    T: Debug + OxfwStreamFormatOperation<P>,
{
    pub notified_elem_id_list: Vec<ElemId>,

    avail_freqs: Vec<u32>,

    curr_output_fmt: usize,
    curr_input_fmt: usize,

    output_fmts: OxfwStreamFormatState,
    input_fmts: OxfwStreamFormatState,

    _phantom0: PhantomData<T>,
    _phantom1: PhantomData<P>,
}

const CLK_RATE_NAME: &'static str = "sampling-rate";

impl<P, T> CommonCtl<P, T>
where
    P: Debug + Ta1394Avc<Error>,
    T: Debug + OxfwStreamFormatOperation<P>,
{
    pub fn detect(&mut self, avc: &mut P, timeout_ms: u32) -> Result<(), Error> {
        T::detect_stream_formats(avc, &mut self.output_fmts, true, timeout_ms)?;
        T::detect_stream_formats(avc, &mut self.input_fmts, false, timeout_ms)?;

        let mut avail_freqs = Vec::new();
        self.output_fmts
            .format_entries
            .iter()
            .chain(&self.input_fmts.format_entries)
            .for_each(|fmt| {
                if avail_freqs.iter().find(|freq| fmt.freq.eq(freq)).is_none() {
                    avail_freqs.push(fmt.freq);
                }
            });
        avail_freqs.sort();
        self.avail_freqs = avail_freqs;

        Ok(())
    }

    pub fn cache(&mut self, avc: &mut P, timeout_ms: u32) -> Result<(), Error> {
        if self.output_fmts.format_entries.len() > 0 {
            self.curr_output_fmt = T::read_stream_format(avc, &self.output_fmts, timeout_ms)?;
        }

        if self.input_fmts.format_entries.len() > 0 {
            self.curr_input_fmt = T::read_stream_format(avc, &self.input_fmts, timeout_ms)?;
        }

        Ok(())
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<String> = self
            .avail_freqs
            .iter()
            .map(|rate| rate.to_string())
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_RATE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|mut elem_id_list| {
                self.notified_elem_id_list.append(&mut elem_id_list);
            })
    }

    pub fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_RATE_NAME => {
                // NOTE: use current format of input as representative.
                let fmt = &self.input_fmts.format_entries[self.curr_input_fmt];
                let pos = self
                    .avail_freqs
                    .iter()
                    .position(|freq| fmt.freq.eq(freq))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn write(
        &mut self,
        unit: &SndUnit,
        avc: &mut P,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_RATE_NAME => {
                let pos = elem_value.enumerated()[0] as usize;
                let freq = self
                    .avail_freqs
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg =
                            format!("Sampling transfer frequency not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                unit.lock()?;
                let res = {
                    if self.output_fmts.format_entries.len() > 0 {
                        let idx = detect_index(
                            &self.output_fmts.format_entries,
                            freq,
                            self.curr_output_fmt,
                        )?;
                        T::write_stream_format(avc, &self.output_fmts, idx, timeout_ms)
                            .map(|_| self.curr_output_fmt = idx)?;
                    }
                    if self.input_fmts.format_entries.len() > 0 {
                        let idx = detect_index(
                            &self.input_fmts.format_entries,
                            freq,
                            self.curr_input_fmt,
                        )?;
                        T::write_stream_format(avc, &self.input_fmts, idx, timeout_ms)
                            .map(|_| self.curr_input_fmt = idx)?;
                    }
                    Ok(())
                };
                unit.unlock()?;
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}

fn detect_index(
    entries: &[CompoundAm824Stream],
    freq: u32,
    curr_idx: usize,
) -> Result<usize, Error> {
    match entries.iter().filter(|e| freq.eq(&e.freq)).count() {
        0 => {
            let msg = format!(
                "Stream format entry not found for sampling transfer frequenty {}",
                freq
            );
            Err(Error::new(FileError::Inval, &msg))
        }
        1 => {
            let pos = entries.iter().position(|e| freq.eq(&e.freq)).unwrap();
            Ok(pos)
        }
        _ => {
            // NOTE: some devices have multiple entries for the same sampling transfer frequency.
            // Let's use the entry which has the same channels as the current.
            let mut entry = entries[curr_idx].clone();
            entry.freq = freq;
            entries.iter().position(|e| entry.eq(e)).ok_or_else(|| {
                let msg = format!(
                    "Stream format entry not found for sampling transfer frequenty {}",
                    freq
                );
                Error::new(FileError::Inval, &msg)
            })
        }
    }
}

/// Control for output parameters.
#[derive(Debug)]
pub struct OutputCtl<P, T>
where
    P: Debug + Ta1394Avc<Error>,
    T: Debug
        + OxfwAudioFbSpecification
        + OxfwFcpParamsOperation<P, OxfwOutputMuteParams>
        + OxfwFcpMutableParamsOperation<P, OxfwOutputMuteParams>
        + OxfwFcpParamsOperation<P, OxfwOutputVolumeParams>
        + OxfwFcpMutableParamsOperation<P, OxfwOutputVolumeParams>,
{
    pub volume: OxfwOutputVolumeParams,
    pub mute: OxfwOutputMuteParams,
    voluntary: bool,
    _phantom0: PhantomData<P>,
    _phantom1: PhantomData<T>,
}

impl<T, P> Default for OutputCtl<P, T>
where
    P: Debug + Ta1394Avc<Error>,
    T: Debug
        + OxfwAudioFbSpecification
        + OxfwFcpParamsOperation<P, OxfwOutputMuteParams>
        + OxfwFcpMutableParamsOperation<P, OxfwOutputMuteParams>
        + OxfwFcpParamsOperation<P, OxfwOutputVolumeParams>
        + OxfwFcpMutableParamsOperation<P, OxfwOutputVolumeParams>,
{
    fn default() -> Self {
        Self {
            volume: T::create_output_volume_params(),
            mute: Default::default(),
            voluntary: Default::default(),
            _phantom0: Default::default(),
            _phantom1: Default::default(),
        }
    }
}

const VOL_NAME: &str = "PCM Playback Volume";
const MUTE_NAME: &str = "PCM Playback Switch";

impl<P, T> OutputCtl<P, T>
where
    P: Debug + Ta1394Avc<Error>,
    T: Debug
        + OxfwAudioFbSpecification
        + OxfwFcpParamsOperation<P, OxfwOutputMuteParams>
        + OxfwFcpMutableParamsOperation<P, OxfwOutputMuteParams>
        + OxfwFcpParamsOperation<P, OxfwOutputVolumeParams>
        + OxfwFcpMutableParamsOperation<P, OxfwOutputVolumeParams>,
{
    const PLAYBACK_COUNT: usize = T::CHANNEL_MAP.len();

    const VOLUME_MIN: i32 = T::VOLUME_MIN as i32;
    const VOLUME_MAX: i32 = T::VOLUME_MAX as i32;
    const VOLUME_STEP: i32 = 1;

    pub fn cache(&mut self, avc: &mut P, timeout_ms: u32) -> Result<(), Error> {
        if self.voluntary {
            T::cache(avc, &mut self.volume, timeout_ms)?;
            T::cache(avc, &mut self.mute, timeout_ms)?;
        }
        Ok(())
    }

    pub fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        // NOTE: I have a plan to remove control functionality from ALSA oxfw driver for future.
        let elem_id_list = card_cntr.card.elem_id_list()?;
        self.voluntary = elem_id_list
            .iter()
            .find(|elem_id| elem_id.name().as_str() == VOL_NAME)
            .is_none();
        if self.voluntary {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, VOL_NAME, 0);
            let _ = card_cntr.add_int_elems(
                &elem_id,
                1,
                Self::VOLUME_MIN as i32,
                Self::VOLUME_MAX as i32,
                Self::VOLUME_STEP as i32,
                Self::PLAYBACK_COUNT,
                None,
                true,
            )?;

            let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, MUTE_NAME, 0);
            let _ = card_cntr.add_bool_elems(&elem_id, 1, 1, true)?;
        }

        Ok(())
    }

    pub fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        if self.voluntary {
            match elem_id.name().as_str() {
                VOL_NAME => {
                    let vals: Vec<i32> = self.volume.0.iter().map(|&vol| vol as i32).collect();
                    elem_value.set_int(&vals);
                    Ok(true)
                }
                MUTE_NAME => {
                    elem_value.set_bool(&[self.mute.0]);
                    Ok(true)
                }
                _ => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    pub fn write(
        &mut self,
        avc: &mut P,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        if self.voluntary {
            match elem_id.name().as_str() {
                VOL_NAME => {
                    let mut params = self.volume.clone();
                    params
                        .0
                        .iter_mut()
                        .zip(elem_value.int())
                        .for_each(|(vol, &val)| *vol = val as i16);
                    T::update(avc, &params, &mut self.volume, timeout_ms).map(|_| true)
                }
                MUTE_NAME => {
                    let mut params = self.mute.clone();
                    params.0 = elem_value.boolean()[0];
                    T::update(avc, &params, &mut self.mute, timeout_ms).map(|_| true)
                }
                _ => Ok(false),
            }
        } else {
            Ok(false)
        }
    }
}
