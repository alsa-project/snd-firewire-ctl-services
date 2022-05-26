// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::{Error, FileError};

use hinawa::FwFcpExt;
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemIfaceType, ElemValue, ElemValueExt, ElemValueExtManual};

use core::card_cntr::*;

use oxfw_protocols::tascam::*;

use super::common_ctl::CommonCtl;

#[derive(Default, Debug)]
pub struct TascamModel {
    avc: TascamAvc,
    common_ctl: CommonCtl,
}

fn display_mode_to_str(mode: &FireoneDisplayMode) -> &str {
    match mode {
        FireoneDisplayMode::Off => "always-off",
        FireoneDisplayMode::AlwaysOn => "always-on",
        FireoneDisplayMode::Breathe => "breathe",
        FireoneDisplayMode::Metronome => "metronome",
        FireoneDisplayMode::MidiClockRotate => "midi-clock-rotate",
        FireoneDisplayMode::MidiClockFlash => "midi-clock-flash",
        FireoneDisplayMode::JogSlowRotate => "jog-slow-rotate",
        FireoneDisplayMode::JogTrack => "jog-track",
    }
}

fn midi_message_mode_to_str(mode: &FireoneMidiMessageMode) -> &str {
    match mode {
        FireoneMidiMessageMode::Native => "native",
        FireoneMidiMessageMode::MackieHuiEmulation => "mackie-hui-emulation",
    }
}

fn input_mode_to_str(mode: &FireoneInputMode) -> &str {
    match mode {
        FireoneInputMode::Stereo => "stereo",
        FireoneInputMode::Monaural => "monaural",
    }
}

const FCP_TIMEOUT_MS: u32 = 100;

const DISPLAY_MODE_NAME: &str = "display-mode";
const MESSAGE_MODE_NAME: &str = "message-mode";
const INPUT_MODE_NAME: &str = "input-mode";
const FIRMWARE_VERSION_NAME: &str = "firmware-version";

impl TascamModel {
    const DISPLAY_MODES: [FireoneDisplayMode; 8] = [
        FireoneDisplayMode::Off,
        FireoneDisplayMode::AlwaysOn,
        FireoneDisplayMode::Breathe,
        FireoneDisplayMode::Metronome,
        FireoneDisplayMode::MidiClockRotate,
        FireoneDisplayMode::MidiClockFlash,
        FireoneDisplayMode::JogSlowRotate,
        FireoneDisplayMode::JogTrack,
    ];
    const MESSAGE_MODES: [FireoneMidiMessageMode; 2] = [
        FireoneMidiMessageMode::Native,
        FireoneMidiMessageMode::MackieHuiEmulation,
    ];
    const INPUT_MODES: [FireoneInputMode; 2] =
        [FireoneInputMode::Stereo, FireoneInputMode::Monaural];
}

impl CtlModel<SndUnit> for TascamModel {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.0.bind(&unit.get_node())?;

        self.common_ctl.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;

        let labels: Vec<&str> = Self::DISPLAY_MODES
            .iter()
            .map(|m| display_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, DISPLAY_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = Self::MESSAGE_MODES
            .iter()
            .map(|m| midi_message_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MESSAGE_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = Self::INPUT_MODES
            .iter()
            .map(|m| input_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, INPUT_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, FIRMWARE_VERSION_NAME, 0);
        let _ = card_cntr.add_bytes_elems(&elem_id, 1, 1, None, false)?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut SndUnit,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self
            .common_ctl
            .read(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)?
        {
            return Ok(true);
        } else {
            match elem_id.get_name().as_str() {
                DISPLAY_MODE_NAME => {
                    let mut mode = FireoneDisplayMode::default();
                    FireoneProtocol::read_display_mode(&mut self.avc, &mut mode, FCP_TIMEOUT_MS)?;
                    let pos = Self::DISPLAY_MODES
                        .iter()
                        .position(|m| m.eq(&mode))
                        .unwrap();
                    elem_value.set_enum(&[pos as u32]);
                    Ok(true)
                }
                MESSAGE_MODE_NAME => {
                    let mut mode = FireoneMidiMessageMode::default();
                    FireoneProtocol::read_midi_message_mode(
                        &mut self.avc,
                        &mut mode,
                        FCP_TIMEOUT_MS,
                    )?;
                    let pos = Self::MESSAGE_MODES
                        .iter()
                        .position(|m| m.eq(&mode))
                        .unwrap();
                    elem_value.set_enum(&[pos as u32]);
                    Ok(true)
                }
                INPUT_MODE_NAME => {
                    let mut mode = FireoneInputMode::default();
                    FireoneProtocol::read_input_mode(&mut self.avc, &mut mode, FCP_TIMEOUT_MS)?;
                    let pos = Self::INPUT_MODES.iter().position(|m| m.eq(&mode)).unwrap();
                    elem_value.set_enum(&[pos as u32]);
                    Ok(true)
                }
                FIRMWARE_VERSION_NAME => {
                    let mut version = 0;
                    FireoneProtocol::read_firmware_version(
                        &mut self.avc,
                        &mut version,
                        FCP_TIMEOUT_MS,
                    )?;
                    elem_value.set_bytes(&[version]);
                    Ok(true)
                }
                _ => Ok(false),
            }
        }
    }

    fn write(
        &mut self,
        unit: &mut SndUnit,
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .common_ctl
            .write(unit, &self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            return Ok(true);
        } else {
            match elem_id.get_name().as_str() {
                DISPLAY_MODE_NAME => {
                    let mut vals = [0];
                    new.get_enum(&mut vals);
                    let &mode = Self::DISPLAY_MODES
                        .iter()
                        .nth(vals[0] as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index for display modes: {}", vals[0]);
                            Error::new(FileError::Inval, &msg)
                        })?;
                    FireoneProtocol::write_display_mode(&mut self.avc, mode, FCP_TIMEOUT_MS)
                        .map(|_| true)
                }
                MESSAGE_MODE_NAME => {
                    let mut vals = [0];
                    new.get_enum(&mut vals);
                    let &mode = Self::MESSAGE_MODES
                        .iter()
                        .nth(vals[0] as usize)
                        .ok_or_else(|| {
                            let msg = format!("Invalid index for midi message modes: {}", vals[0]);
                            Error::new(FileError::Inval, &msg)
                        })?;
                    FireoneProtocol::write_midi_message_mode(&mut self.avc, mode, FCP_TIMEOUT_MS)
                        .map(|_| true)
                }
                INPUT_MODE_NAME => {
                    let mut vals = [0];
                    new.get_enum(&mut vals);
                    let &mode =
                        Self::INPUT_MODES
                            .iter()
                            .nth(vals[0] as usize)
                            .ok_or_else(|| {
                                let msg = format!("Invalid index for input modes: {}", vals[0]);
                                Error::new(FileError::Inval, &msg)
                            })?;
                    FireoneProtocol::write_input_mode(&mut self.avc, mode, FCP_TIMEOUT_MS)
                        .map(|_| true)
                }
                _ => Ok(false),
            }
        }
    }
}

impl NotifyModel<SndUnit, bool> for TascamModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_list);
    }

    fn parse_notification(&mut self, _: &mut SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndUnit,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.common_ctl
            .read(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)
    }
}
