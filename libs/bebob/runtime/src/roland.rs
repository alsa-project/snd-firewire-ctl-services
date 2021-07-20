// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use glib::{Error, FileError};

use hinawa::FwFcpExt;
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemValue};

use core::card_cntr::*;

use bebob_protocols::{*, roland::*};

use crate::common_ctls::*;
use super::model::CLK_RATE_NAME;

#[derive(Default)]
pub struct FaModel {
    avc: BebobAvc,
    clk_ctl: ClkCtl,
}

const FCP_TIMEOUT_MS: u32 = 100;

// Read only, configured by hardware only.
#[derive(Default)]
struct ClkCtl(Vec<ElemId>);

impl MediaClkFreqCtlOperation<FaClkProtocol> for ClkCtl {
    fn write_freq(
        &self,
        _: &mut SndUnit,
        _: &BebobAvc,
        elem_id: &ElemId,
        _: &ElemValue,
        _: &ElemValue,
        _: u32,
    ) -> Result<bool, Error> {
        if elem_id.get_name().as_str() == CLK_RATE_NAME {
            Err(Error::new(FileError::Nxio, "Sampling rate is immutable from software"))
        } else {
            Ok(false)
        }
    }
}

impl CtlModel<SndUnit> for FaModel {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.get_node())?;

        self.clk_ctl.load_freq(card_cntr)?;

        Ok(())
    }

    fn read(&mut self, _: &mut SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(true)
        }
    }

    fn write(
        &mut self,
        unit: &mut SndUnit,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.clk_ctl.write_freq(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS * 3)? {
            Ok(true)
        } else {
            Ok(false)
        }
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
}
