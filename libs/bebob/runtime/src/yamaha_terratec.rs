// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use glib::Error;

use hinawa::{SndUnit, SndUnitExt, FwFcpExt};
use alsactl::{ElemId, ElemValue};

use core::card_cntr::*;

use bebob_protocols::{*, yamaha_terratec::*};

use super::common_ctls::*;

#[derive(Default)]
pub struct GoPhase24CoaxModel {
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    mixer_src_ctl: MixerSourceCtl,
    mixer_out_ctl: CoaxMixerOutputCtl,
}

#[derive(Default)]
pub struct GoPhase24OptModel {
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    phys_out_ctl: OptPhysOutputCtl,
    mixer_src_ctl: MixerSourceCtl,
    mixer_out_ctl: OptMixerOutputCtl,
}

const FCP_TIMEOUT_MS: u32 = 100;

#[derive(Default)]
struct ClkCtl(Vec<ElemId>);

impl MediaClkFreqCtlOperation<GoPhase24ClkProtocol> for ClkCtl {}

impl SamplingClkSrcCtlOperation<GoPhase24ClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &["Internal", "S/PDIF"];
}

#[derive(Default)]
struct MixerSourceCtl;

impl AvcLevelCtlOperation<GoPhase24MixerSourceProtocol> for MixerSourceCtl {
    const LEVEL_NAME: &'static str = "mixer-source-gain";
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-input-1", "analog-input-2",
        "digital-input-1", "digital-input-2",
        "stream-input-1", "stream-input-2",
        "stream-input-3", "stream-input-4",
        "stream-input-5", "stream-input-6",
    ];
}

impl AvcMuteCtlOperation<GoPhase24MixerSourceProtocol> for MixerSourceCtl {
    const MUTE_NAME: &'static str = "mixer-source-mute";
}

#[derive(Default)]
struct OptPhysOutputCtl;

impl AvcLevelCtlOperation<GoPhase24OptPhysOutputProtocol> for OptPhysOutputCtl {
    const LEVEL_NAME: &'static str = "phy-output-volume";
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-output-1", "analog-output-2", "analog-output-3", "analog-output-4",
    ];
}

impl AvcMuteCtlOperation<GoPhase24OptPhysOutputProtocol> for OptPhysOutputCtl {
    const MUTE_NAME: &'static str = "phys-output-mute";
}

#[derive(Default)]
struct CoaxMixerOutputCtl;

impl AvcLevelCtlOperation<GoPhase24CoaxMixerOutputProtocol> for CoaxMixerOutputCtl {
    const LEVEL_NAME: &'static str = "mixer-output-volume";
    const PORT_LABELS: &'static [&'static str] = &["mixer-output-1", "mixer-output-2"];
}

impl AvcMuteCtlOperation<GoPhase24CoaxMixerOutputProtocol> for CoaxMixerOutputCtl {
    const MUTE_NAME: &'static str = "mixer-output-mute";
}

#[derive(Default)]
struct OptMixerOutputCtl;

impl AvcLevelCtlOperation<GoPhase24OptMixerOutputProtocol> for OptMixerOutputCtl {
    const LEVEL_NAME: &'static str = "mixer-output-volume";
    const PORT_LABELS: &'static [&'static str] = &["mixer-output-1", "mixer-output-2"];
}

impl AvcMuteCtlOperation<GoPhase24OptMixerOutputProtocol> for OptMixerOutputCtl {
    const MUTE_NAME: &'static str = "mixer-output-mute";
}

impl CtlModel<SndUnit> for GoPhase24CoaxModel {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.get_node())?;

        self.clk_ctl.load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl.load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.mixer_src_ctl.load_level(card_cntr)?;
        self.mixer_src_ctl.load_mute(card_cntr)?;
        self.mixer_out_ctl.load_level(card_cntr)?;
        self.mixer_out_ctl.load_mute(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, _: &mut SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.clk_ctl.read_src(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_src_ctl.read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_src_ctl.read_mute(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_out_ctl.read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_out_ctl.read_mute(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndUnit, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.write_freq(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS * 3)? {
            Ok(true)
        } else if self.clk_ctl.write_src(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_src_ctl.write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
             Ok(true)
        } else if self.mixer_src_ctl.write_mute(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
             Ok(true)
        } else if self.mixer_out_ctl.write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
             Ok(true)
        } else if self.mixer_out_ctl.write_mute(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
             Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndUnit, bool> for GoPhase24CoaxModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.0);
    }

    fn parse_notification(&mut self, _: &mut SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)
    }
}

impl CtlModel<SndUnit> for GoPhase24OptModel {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.get_node())?;

        self.clk_ctl.load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl.load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.phys_out_ctl.load_level(card_cntr)?;
        self.phys_out_ctl.load_mute(card_cntr)?;
        self.mixer_src_ctl.load_level(card_cntr)?;
        self.mixer_src_ctl.load_mute(card_cntr)?;
        self.mixer_out_ctl.load_level(card_cntr)?;
        self.mixer_out_ctl.load_mute(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, _: &mut SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.clk_ctl.read_src(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.phys_out_ctl.read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.phys_out_ctl.read_mute(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_src_ctl.read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_src_ctl.read_mute(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_out_ctl.read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_out_ctl.read_mute(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &mut SndUnit, elem_id: &ElemId, old: &ElemValue, new: &ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.write_freq(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS * 3)? {
            Ok(true)
        } else if self.clk_ctl.write_src(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.phys_out_ctl.write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
             Ok(true)
        } else if self.phys_out_ctl.write_mute(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
             Ok(true)
        } else if self.mixer_src_ctl.write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
             Ok(true)
        } else if self.mixer_src_ctl.write_mute(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
             Ok(true)
        } else if self.mixer_out_ctl.write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
             Ok(true)
        } else if self.mixer_out_ctl.write_mute(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
             Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndUnit, bool> for GoPhase24OptModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctl.0);
    }

    fn parse_notification(&mut self, _: &mut SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alsactl::CardError;

    #[test]
    fn test_clk_ctl_definition() {
        let mut card_cntr = CardCntr::new();
        let mut ctl = ClkCtl::default();

        let error = ctl.load_freq(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }

    #[test]
    fn test_level_ctl_definition() {
        let mut card_cntr = CardCntr::new();

        let ctl = OptPhysOutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let ctl = CoaxMixerOutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let ctl = MixerSourceCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let ctl = OptMixerOutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
