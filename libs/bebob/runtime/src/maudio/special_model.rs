// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use glib::Error;

use hinawa::{FwReq, FwFcpExt};
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemValue};

use core::card_cntr::*;

use bebob_protocols::{*, maudio::special::*};

use crate::common_ctls::*;
use super::special_ctls::{MeterCtl, StateCache, MixerCtl, InputCtl, OutputCtl, AuxCtl, HpCtl};

pub type Fw1814Model = SpecialModel<Fw1814ClkProtocol>;
pub type ProjectMixModel = SpecialModel<ProjectMixClkProtocol>;

pub struct SpecialModel<T: MediaClockFrequencyOperation + Default> {
    avc: BebobAvc,
    req: FwReq,
    clk_ctl: ClkCtl<T>,
    meter_ctl: MeterCtl,
    cache: StateCache,
}

const FCP_TIMEOUT_MS: u32 = 200;

#[derive(Default)]
struct ClkCtl<T: MediaClockFrequencyOperation + Default>(Vec<ElemId>, T);

impl<T: MediaClockFrequencyOperation + Default> MediaClkFreqCtlOperation<T> for ClkCtl<T> {}

impl<T: MediaClockFrequencyOperation + Default> Default for SpecialModel<T> {
    fn default() -> Self {
        Self {
            avc: Default::default(),
            req: Default::default(),
            clk_ctl: Default::default(),
            meter_ctl: MeterCtl::new(),
            cache: StateCache::new(),
        }
    }
}

impl<T: MediaClockFrequencyOperation + Default> CtlModel<SndUnit> for SpecialModel<T> {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.avc.as_ref().bind(&unit.get_node())?;

        self.clk_ctl.load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.meter_ctl.load(unit, &self.req, &self.avc, card_cntr)?;

        MixerCtl::load(&mut self.cache, card_cntr)?;
        InputCtl::load(&mut self.cache, card_cntr)?;
        OutputCtl::load(&mut self.cache, card_cntr)?;
        AuxCtl::load(&mut self.cache, card_cntr)?;
        HpCtl::load(&mut self.cache, card_cntr)?;

        self.cache.upload(unit, &self.req)?;

        Ok(())
    }

    fn read(&mut self, _: &mut SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if MixerCtl::read(&mut self.cache, elem_id, elem_value)? {
            Ok(true)
        } else if InputCtl::read(&mut self.cache, elem_id, elem_value)? {
            Ok(true)
        } else if OutputCtl::read(&mut self.cache, elem_id, elem_value)? {
            Ok(true)
        } else if AuxCtl::read(&mut self.cache, elem_id, elem_value)? {
            Ok(true)
        } else if HpCtl::read(&mut self.cache, elem_id, elem_value)? {
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
        } else if MixerCtl::write(&mut self.cache, unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if InputCtl::write(&mut self.cache, unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if OutputCtl::write(&mut self.cache, unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if AuxCtl::write(&mut self.cache, unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else if HpCtl::write(&mut self.cache, unit, &self.req, elem_id, old, new)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<T: MediaClockFrequencyOperation + Default> MeasureModel<SndUnit> for SpecialModel<T> {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctl.measure_elems);
    }

    fn measure_states(&mut self, unit: &mut SndUnit) -> Result<(), Error> {
        self.meter_ctl.measure_states(unit, &self.req, &self.avc)
    }

    fn measure_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.meter_ctl.measure_elem(elem_id, elem_value)
    }
}

impl<T: MediaClockFrequencyOperation + Default> NotifyModel<SndUnit, bool> for SpecialModel<T> {
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
        let mut ctl = ClkCtl::<Fw1814ClkProtocol>::default();

        let error = ctl.load_freq(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));

        let mut ctl = ClkCtl::<ProjectMixClkProtocol>::default();

        let error = ctl.load_freq(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
