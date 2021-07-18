// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::Error;

use hinawa::{SndUnit, SndUnitExt, FwFcpExt};
use alsactl::{ElemId, ElemValue};

use core::card_cntr::*;

use bebob_protocols::{*, esi::*};

use super::{common_ctls::*, model::OUT_VOL_NAME};

const FCP_TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct Quatafire610Model {
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    input_ctl: Quatafire610InputCtl,
    output_ctl: Quatafire610OutputCtl,
}

#[derive(Default)]
struct ClkCtl(Vec<ElemId>);

impl MediaClkFreqCtlOperation<Quatafire610ClkProtocol> for ClkCtl {}

impl SamplingClkSrcCtlOperation<Quatafire610ClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &[
        "Internal",
    ];
}

#[derive(Default)]
struct Quatafire610InputCtl;

impl AvcLevelCtlOperation<Quatafire610PhysInputProtocol> for Quatafire610InputCtl {
    const LEVEL_NAME: &'static str = "phys-input-gain";
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-input-1", "analog-input-2",
        "analog-input-1", "analog-input-2",
        "digital-input-1", "digital-input-2",
    ];
}

impl AvcLrBalanceCtlOperation<Quatafire610PhysInputProtocol> for Quatafire610InputCtl {
    const BALANCE_NAME: &'static str = "phys-input-balance";
}

#[derive(Default)]
struct Quatafire610OutputCtl;

impl AvcLevelCtlOperation<Quatafire610PhysOutputProtocol> for Quatafire610OutputCtl {
    const LEVEL_NAME: &'static str = OUT_VOL_NAME;
    const PORT_LABELS: &'static [&'static str] = &[
        "analog-output-1", "analog-output-2", "analog-output-3", "analog-output-4",
        "analog-output-5", "analog-output-6", "analog-output-7", "analog-output-8",
    ];
}

impl CtlModel<SndUnit> for Quatafire610Model {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.get_node())?;

        self.clk_ctl.load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl.load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.input_ctl.load_level(card_cntr)?;
        self.input_ctl.load_balance(card_cntr)?;
        self.output_ctl.load_level(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, _: &mut SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.clk_ctl.read_src(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.input_ctl.read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.input_ctl.read_balance(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
             Ok(true)
        } else if self.output_ctl.read_level(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
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
        } else if self.input_ctl.write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.input_ctl.write_balance(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.output_ctl.write_level(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
             Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<SndUnit, bool> for Quatafire610Model {
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

        let ctl = Quatafire610InputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let ctl = Quatafire610OutputCtl::default();
        let error = ctl.load_level(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
