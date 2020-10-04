// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;

use hinawa::{FwFcpExt, SndUnitExt};

use crate::card_cntr;

use crate::ta1394::{Ta1394Avc, AvcAddr, MUSIC_SUBUNIT_0};
use crate::ta1394::general::UnitInfo;
use crate::ta1394::ccm::{SignalAddr, SignalUnitAddr, SignalSubunitAddr};

use crate::bebob::BebobAvc;
use crate::bebob::common_ctls::ClkCtl;
use super::apogee_ctls::HwCtl;

pub struct EnsembleModel<'a>{
    avc: BebobAvc,
    clk_ctls: ClkCtl<'a>,
    hw_ctls: HwCtl,
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

    pub fn new() -> Self {
        EnsembleModel{
            avc: BebobAvc::new(),
            clk_ctls: ClkCtl::new(&Self::CLK_DST, Self::CLK_SRCS, Self::CLK_SRC_LABELS),
            hw_ctls: HwCtl::new(),
        }
    }
}

impl<'a> card_cntr::CtlModel<hinawa::SndUnit> for EnsembleModel<'a> {
    fn load(&mut self, unit: &hinawa::SndUnit, card_cntr: &mut card_cntr::CardCntr)
        -> Result<(), Error>
    {
        self.avc.fcp.bind(&unit.get_node())?;

        let mut op = UnitInfo::new();
        self.avc.status(&AvcAddr::Unit, &mut op, Self::FCP_TIMEOUT_MS)?;
        self.avc.company_id = op.company_id;

        self.clk_ctls.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;
        self.hw_ctls.load(&self.avc, card_cntr, Self::FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn read(&mut self, _: &hinawa::SndUnit, elem_id: &alsactl::ElemId, elem_value: &mut alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.read(&self.avc, elem_id, elem_value, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.hw_ctls.read(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(&mut self, unit: &hinawa::SndUnit, elem_id: &alsactl::ElemId,
             old: &alsactl::ElemValue, new: &alsactl::ElemValue)
        -> Result<bool, Error>
    {
        if self.clk_ctls.write(unit, &self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else if self.hw_ctls.write(unit, &self.avc, elem_id, old, new, Self::FCP_TIMEOUT_MS)? {
            Ok(true)
        } else {
            Ok(true)
        }
    }
}
