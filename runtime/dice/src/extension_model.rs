// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::{tcd22xx_ctl::*, *},
    protocols::tcat::{extension::*, tcd22xx_spec::*},
};

#[derive(Default)]
pub struct ExtensionModel {
    req: FwReq,
    sections: GeneralSections,
    extension_sections: ExtensionSections,
    ctl: CommonCtl,
    tcd22xx_ctl: ExtensionTcd22xxCtl,
}

const TIMEOUT_MS: u32 = 20;

impl ExtensionModel {
    pub fn cache(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.sections =
            GeneralProtocol::read_general_sections(&mut self.req, &mut unit.1, TIMEOUT_MS)?;

        Ok(())
    }
}

impl CtlModel<(SndDice, FwNode)> for ExtensionModel {
    fn load(
        &mut self,
        unit: &mut (SndDice, FwNode),
        card_cntr: &mut CardCntr,
    ) -> Result<(), Error> {
        let caps = GlobalSectionProtocol::read_clock_caps(
            &mut self.req,
            &mut unit.1,
            &self.sections,
            TIMEOUT_MS,
        )?;
        let src_labels = GlobalSectionProtocol::read_clock_source_labels(
            &mut self.req,
            &mut unit.1,
            &self.sections,
            TIMEOUT_MS,
        )?;
        self.ctl.load(card_cntr, &caps, &src_labels)?;

        let avail_rates = caps.get_rate_entries();
        let avail_sources = caps.get_src_entries(&src_labels);
        let clock_source_labels: Vec<(ClockSource, String)> = avail_sources
            .iter()
            .filter_map(|&src| {
                src.get_label(&src_labels, false)
                    .map(|l| (src, l.to_string()))
            })
            .collect();

        self.extension_sections =
            ProtocolExtension::read_extension_sections(&mut self.req, &mut unit.1, TIMEOUT_MS)?;
        self.tcd22xx_ctl.load(
            unit,
            &mut self.req,
            &self.extension_sections,
            &avail_rates,
            &avail_sources,
            &clock_source_labels,
            TIMEOUT_MS,
            card_cntr,
        )?;
        self.tcd22xx_ctl.cache(
            unit,
            &mut self.req,
            &self.sections,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;

        Ok(())
    }

    fn read(
        &mut self,
        unit: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.ctl.read(
            unit,
            &mut self.req,
            &self.sections,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.tcd22xx_ctl.read(
            unit,
            &mut self.req,
            &self.extension_sections,
            elem_id,
            elem_value,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn write(
        &mut self,
        unit: &mut (SndDice, FwNode),
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        if self.ctl.write(
            unit,
            &mut self.req,
            &self.sections,
            elem_id,
            old,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else if self.tcd22xx_ctl.write(
            unit,
            &mut self.req,
            &self.extension_sections,
            elem_id,
            old,
            new,
            TIMEOUT_MS,
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl NotifyModel<(SndDice, FwNode), u32> for ExtensionModel {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.notified_elem_list);
        self.tcd22xx_ctl.get_notified_elem_list(elem_id_list);
    }

    fn parse_notification(&mut self, unit: &mut (SndDice, FwNode), msg: &u32) -> Result<(), Error> {
        self.ctl
            .parse_notification(unit, &mut self.req, &self.sections, *msg, TIMEOUT_MS)?;
        self.tcd22xx_ctl.parse_notification(
            unit,
            &mut self.req,
            &self.sections,
            &self.extension_sections,
            TIMEOUT_MS,
            *msg,
        )?;
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.read_notified_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl MeasureModel<(SndDice, FwNode)> for ExtensionModel {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.ctl.measured_elem_list);
        self.tcd22xx_ctl.get_measured_elem_list(elem_id_list);
    }

    fn measure_states(&mut self, unit: &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.ctl
            .measure_states(unit, &mut self.req, &self.sections, TIMEOUT_MS)?;
        self.tcd22xx_ctl.measure_states(
            unit,
            &mut self.req,
            &self.extension_sections,
            TIMEOUT_MS,
        )?;
        Ok(())
    }

    fn measure_elem(
        &mut self,
        _: &(SndDice, FwNode),
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error> {
        if self.ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else if self.tcd22xx_ctl.measure_elem(elem_id, elem_value)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default)]
struct ExtensionProtocol;

impl Tcd22xxSpecOperation for ExtensionProtocol {
    const INPUTS: &'static [Input] = &[
        Input {
            id: SrcBlkId::Ins0,
            offset: 0,
            count: 16,
            label: None,
        },
        Input {
            id: SrcBlkId::Ins1,
            offset: 0,
            count: 16,
            label: None,
        },
        Input {
            id: SrcBlkId::Aes,
            offset: 0,
            count: 2,
            label: None,
        },
        Input {
            id: SrcBlkId::Aes,
            offset: 2,
            count: 2,
            label: None,
        },
        Input {
            id: SrcBlkId::Aes,
            offset: 4,
            count: 2,
            label: None,
        },
        Input {
            id: SrcBlkId::Aes,
            offset: 6,
            count: 2,
            label: None,
        },
        Input {
            id: SrcBlkId::Adat,
            offset: 0,
            count: 8,
            label: None,
        },
        Input {
            id: SrcBlkId::Adat,
            offset: 8,
            count: 8,
            label: None,
        },
    ];
    const OUTPUTS: &'static [Output] = &[
        Output {
            id: DstBlkId::Ins0,
            offset: 0,
            count: 16,
            label: None,
        },
        Output {
            id: DstBlkId::Ins1,
            offset: 0,
            count: 16,
            label: None,
        },
        Output {
            id: DstBlkId::Aes,
            offset: 0,
            count: 2,
            label: None,
        },
        Output {
            id: DstBlkId::Aes,
            offset: 2,
            count: 2,
            label: None,
        },
        Output {
            id: DstBlkId::Aes,
            offset: 4,
            count: 2,
            label: None,
        },
        Output {
            id: DstBlkId::Aes,
            offset: 6,
            count: 2,
            label: None,
        },
        Output {
            id: DstBlkId::Adat,
            offset: 0,
            count: 8,
            label: None,
        },
        Output {
            id: DstBlkId::Adat,
            offset: 8,
            count: 8,
            label: None,
        },
    ];
    const FIXED: &'static [SrcBlk] = &[
        SrcBlk {
            id: SrcBlkId::Ins0,
            ch: 0,
        },
        SrcBlk {
            id: SrcBlkId::Ins0,
            ch: 1,
        },
        SrcBlk {
            id: SrcBlkId::Ins0,
            ch: 2,
        },
        SrcBlk {
            id: SrcBlkId::Ins0,
            ch: 3,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 0,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 1,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 2,
        },
        SrcBlk {
            id: SrcBlkId::Ins1,
            ch: 3,
        },
    ];
}

#[derive(Default)]
struct ExtensionTcd22xxCtl(Tcd22xxCtl);

impl Tcd22xxCtlOperation<ExtensionProtocol> for ExtensionTcd22xxCtl {
    fn tcd22xx_ctl(&self) -> &Tcd22xxCtl {
        &self.0
    }

    fn tcd22xx_ctl_mut(&mut self) -> &mut Tcd22xxCtl {
        &mut self.0
    }
}

pub fn detect_extended_model(node: &mut FwNode) -> bool {
    let mut req = FwReq::default();
    let res = ProtocolExtension::read_extension_sections(&mut req, node, 100);

    // MEMO: workaround for old firmware. Invalidate the negative effect by failure of
    // previous transaction.
    if res.is_err() {
        let _ = GeneralProtocol::read_general_sections(&mut req, node, 100);
    }

    res.is_ok()
}
