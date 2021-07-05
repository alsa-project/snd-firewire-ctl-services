// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{FwFcpExt, SndUnitExt};

use core::card_cntr;

use ta1394::{Ta1394Avc, AvcAddr, MUSIC_SUBUNIT_0};
use ta1394::general::UnitInfo;
use ta1394::ccm::{SignalAddr, SignalUnitAddr, SignalSubunitAddr};

use super::super::BebobAvc;
use super::super::common_ctls::ClkCtl;
use super::apogee_ctls::{HwCtl, DisplayCtl, OpticalCtl, InputCtl, OutputCtl, MixerCtl, RouteCtl, ResamplerCtl, MeterCtl};

pub struct EnsembleModel<'a>{
    avc: BebobAvc,
    clk_ctls: ClkCtl<'a>,
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

impl<'a> EnsembleModel<'a> {
    const FCP_TIMEOUT_MS: u32 = 100;

    const CLK_DST: SignalAddr = SignalAddr::Subunit(SignalSubunitAddr{
        subunit: MUSIC_SUBUNIT_0,
        plug_id: 7,
    });
    const CLK_SRCS: &'a [SignalAddr] = &[
        SignalAddr::Subunit(SignalSubunitAddr{
            subunit: MUSIC_SUBUNIT_0,
            plug_id: 7,
        }),
        SignalAddr::Unit(SignalUnitAddr::Ext(4)),
        SignalAddr::Unit(SignalUnitAddr::Ext(5)),
        SignalAddr::Unit(SignalUnitAddr::Ext(6)),
    ];

    const CLK_SRC_LABELS: &'a [&'a str] = &[
        "Internal",
        "S/PDIF coax",
        "Optical",
        "Word Clock",
    ];
}

impl<'a> Default for EnsembleModel<'a> {
    fn default() -> Self {
        Self{
            avc: Default::default(),
            clk_ctls: ClkCtl::new(&Self::CLK_DST, Self::CLK_SRCS, Self::CLK_SRC_LABELS),
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

impl<'a> card_cntr::CtlModel<hinawa::SndUnit> for EnsembleModel<'a> {
    fn load(&mut self, unit: &mut hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        self.avc.fcp.bind(&unit.get_node())?;

        let mut op = UnitInfo::new();
        self.avc.status(&AvcAddr::Unit, &mut op, Self::FCP_TIMEOUT_MS)?;
        self.avc.company_id = op.company_id;

        self.clk_ctls.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;
        self.hw_ctls.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;
        self.display_ctls.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;
        self.opt_iface_ctls.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;
        self.input_ctls.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;
        self.out_ctls.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;
        self.mixer_ctls.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;
        self.route_ctls.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;
        self.resampler_ctls.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;
        self.meter_ctls.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn read(&mut self, _: &mut hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)? {
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

    fn write(&mut self, unit: &mut hinawa::SndUnit, elem_id: &alsactl::ElemId,
             old: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.write(unit, &self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.hw_ctls.write(unit, &self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.display_ctls.write(&self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.opt_iface_ctls.write(&self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.input_ctls.write(&self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.out_ctls.write(&self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.mixer_ctls.write(&self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.route_ctls.write(&self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.resampler_ctls.write(&self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.meter_ctls.write(&self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(true)
        }
    }
}

impl<'a> card_cntr::MeasureModel<hinawa::SndUnit> for EnsembleModel<'a> {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.meter_ctls.measure_elem_list);
    }

    fn measure_states(&mut self, _: &mut hinawa::SndUnit) -> Result<(), Error> {
        self.meter_ctls.measure_states(&self.avc, Self::FCP_TIMEOUT_MS)
    }

    fn measure_elem(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        self.meter_ctls.measure_elem(elem_id, elem_value)
    }
}

impl<'a> card_cntr::NotifyModel<hinawa::SndUnit, bool> for EnsembleModel<'a> {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>) {
        elem_id_list.extend_from_slice(&self.clk_ctls.notified_elem_list);
    }

    fn parse_notification(&mut self, _: &mut hinawa::SndUnit, _: &bool) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId,
                          elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        self.clk_ctls.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)
    }
}
