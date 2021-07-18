// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use glib::Error;

use hinawa::FwFcpExt;
use hinawa::{SndUnit, SndUnitExt};

use alsactl::{ElemId, ElemValue};

use core::card_cntr::*;

use bebob_protocols::{*, apogee::ensemble::*};

use crate::common_ctls::*;
use super::apogee_ctls::{HwCtl, DisplayCtl, OpticalCtl, InputCtl, OutputCtl, MixerCtl, RouteCtl, ResamplerCtl, MeterCtl};

const FCP_TIMEOUT_MS: u32 = 100;

pub struct EnsembleModel {
    avc: BebobAvc,
    clk_ctl: ClkCtl,
    hw_ctls: HwCtl,
    display_ctls: DisplayCtl,
    opt_iface_ctls: OpticalCtl,
    input_ctls: InputCtl,
    out_ctls: OutputCtl,
    mixer_ctls: MixerCtl,
    route_ctls: RouteCtl,
    resampler_ctls: ResamplerCtl,
    meter_ctls: MeterCtl,
}

#[derive(Default)]
struct ClkCtl(Vec<ElemId>);

impl MediaClkFreqCtlOperation<EnsembleClkProtocol> for ClkCtl {}

impl SamplingClkSrcCtlOperation<EnsembleClkProtocol> for ClkCtl {
    const SRC_LABELS: &'static [&'static str] = &[
        "Internal",
        "S/PDIF-coax",
        "Optical",
        "Word Clock",
    ];
}

impl Default for EnsembleModel {
    fn default() -> Self {
        Self {
            avc: Default::default(),
            clk_ctl: Default::default(),
            hw_ctls: HwCtl::new(),
            display_ctls: DisplayCtl::new(),
            opt_iface_ctls: OpticalCtl::new(),
            input_ctls: InputCtl::new(),
            out_ctls: OutputCtl::new(),
            mixer_ctls: MixerCtl::new(),
            route_ctls: RouteCtl::new(),
            resampler_ctls: ResamplerCtl::new(),
            meter_ctls: MeterCtl::new(),
        }
    }
}

impl CtlModel<SndUnit> for EnsembleModel {
    fn load(&mut self, unit: &mut SndUnit, card_cntr: &mut CardCntr)
        -> Result<(), Error>
    {
        self.avc.as_ref().bind(&unit.get_node())?;

        self.clk_ctl.load_freq(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.clk_ctl.load_src(card_cntr)
            .map(|mut elem_id_list| self.clk_ctl.0.append(&mut elem_id_list))?;

        self.hw_ctls.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;
        self.display_ctls.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;
        self.opt_iface_ctls.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;
        self.input_ctls.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;
        self.out_ctls.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;
        self.mixer_ctls.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;
        self.route_ctls.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;
        self.resampler_ctls.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;
        self.meter_ctls.load(&self.avc, card_cntr, FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn read(&mut self, _: &mut SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctl.read_freq(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.clk_ctl.read_src(&self.avc, elem_id, elem_value, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.hw_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.display_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.opt_iface_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.input_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.out_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.mixer_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.route_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.resampler_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else if self.meter_ctls.read(elem_id, elem_value)? {
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
        } else if self.clk_ctl.write_src(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS * 3)? {
            Ok(true)
        } else if self.hw_ctls.write(unit, &self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.display_ctls.write(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.opt_iface_ctls.write(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.input_ctls.write(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.out_ctls.write(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctls.write(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.route_ctls.write(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.resampler_ctls.write(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.meter_ctls.write(&self.avc, elem_id, old, new, FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(true)
        }
    }
}

impl MeasureModel<SndUnit> for EnsembleModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctls.measure_elem_list);
    }

    fn measure_states(&mut self, _: &mut SndUnit) -> Result<(), Error> {
        self.meter_ctls.measure_states(&self.avc, FCP_TIMEOUT_MS)
    }

    fn measure_elem(&mut self, _: &SndUnit, elem_id: &ElemId, elem_value: &mut ElemValue)
        -> Result<bool, Error>
    {
        self.meter_ctls.measure_elem(elem_id, elem_value)
    }
}

impl NotifyModel<SndUnit, bool> for EnsembleModel {
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

        let error = ctl.load_src(&mut card_cntr).unwrap_err();
        assert_eq!(error.kind::<CardError>(), Some(CardError::Failed));
    }
}
