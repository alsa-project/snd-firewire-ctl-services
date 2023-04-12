// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {super::*, protocols::tascam::*};

#[derive(Default, Debug)]
pub struct TascamModel {
    avc: TascamAvc,
    common_ctl: CommonCtl<TascamAvc, FireoneProtocol>,
    specific_ctl: SpecificCtl,
}

const FCP_TIMEOUT_MS: u32 = 100;

impl CtlModel<(SndUnit, FwNode)> for TascamModel {
    fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.common_ctl.detect(&mut self.avc, FCP_TIMEOUT_MS)?;

        self.common_ctl.cache(&mut self.avc, FCP_TIMEOUT_MS)?;
        self.specific_ctl.cache(&mut self.avc, FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, _: &mut (SndUnit, FwNode), card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr)?;
        self.specific_ctl.load(card_cntr)?;

        Ok(())
    }

    fn read(
        &mut self,
        _: &mut (SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.common_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.specific_ctl.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndUnit, FwNode),
        elem_id: &ElemId,
        _: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self
            .common_ctl
            .write(&unit.0, &mut self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else if self
            .specific_ctl
            .write(&mut self.avc, elem_id, new, FCP_TIMEOUT_MS)?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndUnit, FwNode), bool> for TascamModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_id_list);
    }

    fn parse_notification(
        &mut self,
        _: &mut (SndUnit, FwNode),
        &locked: &bool,
    ) -> Result<(), Error> {
        if locked {
            self.common_ctl.cache(&mut self.avc, FCP_TIMEOUT_MS)?;
        }
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndUnit, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        self.common_ctl.read(elem_id, elem_value)
    }
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

const DISPLAY_MODE_NAME: &str = "display-mode";
const MIDI_MESSAGE_MODE_NAME: &str = "message-mode";
const INPUT_MODE_NAME: &str = "input-mode";
const FIRMWARE_VERSION_NAME: &str = "firmware-version";

#[derive(Default, Debug)]
struct SpecificCtl(SpecificParams);

impl SpecificCtl {
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

    const MIDI_MESSAGE_MODES: [FireoneMidiMessageMode; 2] = [
        FireoneMidiMessageMode::Native,
        FireoneMidiMessageMode::MackieHuiEmulation,
    ];

    const INPUT_MODES: [FireoneInputMode; 2] =
        [FireoneInputMode::Stereo, FireoneInputMode::Monaural];

    fn cache(&mut self, avc: &mut TascamAvc, timeout_ms: u32) -> Result<(), Error> {
        let res = FireoneProtocol::cache(avc, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::DISPLAY_MODES
            .iter()
            .map(|m| display_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, DISPLAY_MODE_NAME, 0);
        let _ = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;

        let labels: Vec<&str> = Self::MIDI_MESSAGE_MODES
            .iter()
            .map(|m| midi_message_mode_to_str(m))
            .collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, MIDI_MESSAGE_MODE_NAME, 0);
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

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            DISPLAY_MODE_NAME => {
                let params = &self.0;
                let pos = Self::DISPLAY_MODES
                    .iter()
                    .position(|m| params.display_mode.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            MIDI_MESSAGE_MODE_NAME => {
                let params = &self.0;
                let pos = Self::MIDI_MESSAGE_MODES
                    .iter()
                    .position(|m| params.midi_message_mode.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            INPUT_MODE_NAME => {
                let params = &self.0;
                let pos = Self::INPUT_MODES
                    .iter()
                    .position(|m| params.input_mode.eq(m))
                    .unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            FIRMWARE_VERSION_NAME => {
                let params = &self.0;
                elem_value.set_bytes(&[params.firmware_version]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        avc: &mut TascamAvc,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            DISPLAY_MODE_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.display_mode = Self::DISPLAY_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Display mode not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = FireoneProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            MIDI_MESSAGE_MODE_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.midi_message_mode = Self::MIDI_MESSAGE_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("MIDI message mode not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = FireoneProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            INPUT_MODE_NAME => {
                let mut params = self.0.clone();
                let pos = elem_value.enumerated()[0] as usize;
                params.input_mode = Self::INPUT_MODES
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Input mode not found for position {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = FireoneProtocol::update(avc, &params, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
