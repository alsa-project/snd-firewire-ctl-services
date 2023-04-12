// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2021 Takashi Sakamoto

use {super::*, protocols::loud::*};

#[derive(Default, Debug)]
pub struct LinkFwModel {
    avc: OxfwAvc,
    common_ctl: CommonCtl<OxfwAvc, LinkFwProtocol>,
    specific_ctl: SpecificCtl,
}

const FCP_TIMEOUT_MS: u32 = 100;

impl CtlModel<(SndUnit, FwNode)> for LinkFwModel {
    fn cache(&mut self, unit: &mut (SndUnit, FwNode)) -> Result<(), Error> {
        self.avc.bind(&unit.1)?;

        self.common_ctl.detect(&mut self.avc, FCP_TIMEOUT_MS)?;

        self.common_ctl.cache(&mut self.avc, FCP_TIMEOUT_MS)?;
        self.specific_ctl.cache(&mut self.avc, FCP_TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
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

impl NotifyModel<(SndUnit, FwNode), bool> for LinkFwModel {
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

#[derive(Default, Debug)]
struct SpecificCtl(LinkFwInputSource);

fn input_source_to_str(src: &LinkFwInputSource) -> &str {
    match src {
        LinkFwInputSource::Analog => "Analog-input",
        LinkFwInputSource::Digital => "S/PDIF-input",
    }
}

const CAPTURE_SOURCE_NAME: &str = "capture-source";

impl SpecificCtl {
    const SRCS: [LinkFwInputSource; 2] = [LinkFwInputSource::Analog, LinkFwInputSource::Digital];

    fn cache(&mut self, avc: &mut OxfwAvc, timeout_ms: u32) -> Result<(), Error> {
        let res = LinkFwProtocol::cache(avc, &mut self.0, timeout_ms);
        debug!(params = ?self.0, ?res);
        res
    }

    fn load(&self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        let labels: Vec<&str> = Self::SRCS.iter().map(|s| input_source_to_str(s)).collect();
        let elem_id = ElemId::new_by_name(ElemIfaceType::Mixer, 0, 0, CAPTURE_SOURCE_NAME, 0);
        card_cntr
            .add_enum_elems(&elem_id, 1, 1, &labels, None, true)
            .map(|_| ())
    }

    fn read(&self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CAPTURE_SOURCE_NAME => {
                let pos = Self::SRCS.iter().position(|s| self.0.eq(s)).unwrap();
                elem_value.set_enum(&[pos as u32]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        avc: &mut OxfwAvc,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CAPTURE_SOURCE_NAME => {
                let pos = elem_value.enumerated()[0] as usize;
                let src = Self::SRCS
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!("Invalid value for index of signal source: {}", pos);
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                let res = LinkFwProtocol::update(avc, &src, &mut self.0, timeout_ms);
                debug!(params = ?self.0, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }
}
