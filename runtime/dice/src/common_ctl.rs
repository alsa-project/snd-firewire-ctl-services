// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2022 Takashi Sakamoto

use super::*;

const CLK_RATE_NAME: &str = "clock-rate";
const CLK_SRC_NAME: &str = "clock-source";
const NICKNAME: &str = "nickname";
const LOCKED_CLK_SRC_NAME: &str = "locked-clock-source";
const SLIPPED_CLK_SRC_NAME: &str = "slipped-clock-source";

pub trait CommonCtlOperation<T>
where
    T: TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>,
{
    const CLK_CAPS_FIXUP: Option<(&'static [ClockRate], &'static [ClockSource])> = None;
    const CLK_SRC_LABELS_FIXUP: Option<&'static [&'static str]> = None;

    fn whole_cache(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        sections: &mut GeneralSections,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::whole_cache(req, node, &mut sections.global, timeout_ms);
        debug!(params = ?sections.global.params, ?res);
        res?;

        let res = T::whole_cache(req, node, &mut sections.tx_stream_format, timeout_ms);
        debug!(params = ?sections.tx_stream_format.params, ?res);
        res?;

        let res = T::whole_cache(req, node, &mut sections.rx_stream_format, timeout_ms);
        debug!(params = ?sections.rx_stream_format.params, ?res);
        res?;

        // Old firmware doesn't support it.
        if sections.ext_sync.size > 0 {
            let res = T::whole_cache(req, node, &mut sections.ext_sync, timeout_ms);
            debug!(params = ?sections.ext_sync.params, ?res);
            res?;
        }

        Ok(())
    }

    fn load(
        &mut self,
        card_cntr: &mut CardCntr,
        sections: &GeneralSections,
    ) -> Result<(Vec<ElemId>, Vec<ElemId>), Error> {
        let mut notified_elem_list = Vec::new();
        let mut measured_elem_list = Vec::new();

        let params = &sections.global.params;

        let labels: Vec<String> = params
            .avail_rates
            .iter()
            .map(|r| clock_rate_to_string(r))
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_RATE_NAME, 0);
        let mut elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        notified_elem_list.append(&mut elem_id_list);

        let labels: Vec<&str> = params
            .avail_sources
            .iter()
            .filter_map(|src| {
                params
                    .clock_source_labels
                    .iter()
                    .find(|(s, _)| src.eq(s))
                    .map(|(_, l)| l.as_str())
            })
            .collect();

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, CLK_SRC_NAME, 0);
        let mut elem_id_list = card_cntr.add_enum_elems(&elem_id, 1, 1, &labels, None, true)?;
        notified_elem_list.append(&mut elem_id_list);

        let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, NICKNAME, 0);
        card_cntr
            .add_bytes_elems(&elem_id, 1, NICKNAME_MAX_SIZE, None, true)
            .map(|mut elem_id_list| notified_elem_list.append(&mut elem_id_list))?;

        let labels: Vec<&str> = params
            .external_source_states
            .sources
            .iter()
            .filter_map(|src| {
                params
                    .clock_source_labels
                    .iter()
                    .find(|(s, _)| src.eq(s))
                    .map(|(_, l)| l.as_str())
            })
            .collect();

        if labels.len() > 0 {
            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, LOCKED_CLK_SRC_NAME, 0);
            let mut elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, labels.len(), false)?;
            notified_elem_list.append(&mut elem_id_list);

            let elem_id = ElemId::new_by_name(ElemIfaceType::Card, 0, 0, SLIPPED_CLK_SRC_NAME, 0);
            let mut elem_id_list = card_cntr.add_bool_elems(&elem_id, 1, labels.len(), false)?;
            measured_elem_list.append(&mut elem_id_list);
        }

        Ok((measured_elem_list, notified_elem_list))
    }

    fn read(
        &mut self,
        sections: &GeneralSections,
        elem_id: &ElemId,
        elem_value: &ElemValue,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_RATE_NAME => {
                let params = &sections.global.params;
                let pos = params
                    .avail_rates
                    .iter()
                    .position(|rate| rate.eq(&params.clock_config.rate))
                    .ok_or_else(|| {
                        let msg = format!(
                            "Unexpected value read for clock rate: {}",
                            clock_rate_to_string(&params.clock_config.rate),
                        );
                        Error::new(FileError::Io, &msg)
                    })
                    .map(|pos| pos as u32)?;
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            CLK_SRC_NAME => {
                let params = &sections.global.params;
                let pos = params
                    .avail_sources
                    .iter()
                    .position(|src| src.eq(&params.clock_config.src))
                    .ok_or_else(|| {
                        let msg = format!(
                            "Unexpected value read for clock source: {}",
                            clock_source_to_string(&params.clock_config.src)
                        );
                        Error::new(FileError::Io, &msg)
                    })
                    .map(|pos| pos as u32)?;
                elem_value.set_enum(&[pos]);
                Ok(true)
            }
            NICKNAME => {
                // NOTE: the maximum size of nickname bytes (=64) fits within the size of byte
                // value container (=512).
                let mut vals = [0u8; 512];
                let raw = sections.global.params.nickname.as_bytes();
                vals[..raw.len()].copy_from_slice(&raw);
                elem_value.set_bytes(&vals);
                Ok(true)
            }
            LOCKED_CLK_SRC_NAME => {
                let params = &sections.global.params;
                elem_value.set_bool(&params.external_source_states.locked);
                Ok(true)
            }
            SLIPPED_CLK_SRC_NAME => {
                let params = &sections.global.params;
                elem_value.set_bool(&params.external_source_states.slipped);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn write(
        &mut self,
        unit: &SndDice,
        req: &FwReq,
        node: &FwNode,
        sections: &mut GeneralSections,
        elem_id: &ElemId,
        elem_value: &ElemValue,
        timeout_ms: u32,
    ) -> Result<bool, Error> {
        match elem_id.name().as_str() {
            CLK_RATE_NAME => {
                let pos = elem_value.enumerated()[0] as usize;
                let mut params = sections.global.params.clone();
                let rate = params
                    .avail_rates
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!(
                            "Invalid value for index of rate: {} greater than {}",
                            pos,
                            params.avail_rates.len(),
                        );
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                params.clock_config.rate = rate;
                unit.lock()?;
                let res = T::partial_update(req, node, &params, &mut sections.global, timeout_ms);
                let _ = unit.unlock();
                debug!(?params, ?res);
                res.map(|_| true)
            }
            CLK_SRC_NAME => {
                let pos = elem_value.enumerated()[0] as usize;
                let mut params = sections.global.params.clone();
                let src = sections
                    .global
                    .params
                    .avail_sources
                    .iter()
                    .nth(pos)
                    .ok_or_else(|| {
                        let msg = format!(
                            "Invalid value for index of source: {} greater than {}",
                            pos,
                            params.avail_sources.len(),
                        );
                        Error::new(FileError::Inval, &msg)
                    })
                    .copied()?;
                params.clock_config.src = src;
                unit.lock()?;
                let res = T::partial_update(req, node, &params, &mut sections.global, timeout_ms);
                let _ = unit.unlock();
                debug!(?params, ?res);
                res.map(|_| true)
            }
            NICKNAME => {
                let vals = elem_value.bytes().to_vec();
                let mut params = sections.global.params.clone();
                params.nickname = String::from_utf8(vals).map_err(|e| {
                    let msg = format!("Invalid bytes for string: {}", e);
                    Error::new(FileError::Inval, &msg)
                })?;
                let res = T::partial_update(req, node, &params, &mut sections.global, timeout_ms);
                debug!(?params, ?res);
                res.map(|_| true)
            }
            _ => Ok(false),
        }
    }

    fn measure(
        &mut self,
        req: &FwReq,
        node: &FwNode,
        sections: &mut GeneralSections,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        let res = T::partial_cache(req, node, &mut sections.global, timeout_ms);
        debug!(params = ?sections.global.params, ?res);
        res?;

        // Old firmware doesn't support it.
        if sections.ext_sync.size > 0 {
            let res = T::whole_cache(req, node, &mut sections.ext_sync, timeout_ms);
            debug!(params = ?sections.ext_sync.params, ?res);
            res?;
        }

        Ok(())
    }

    fn parse_notification(
        &self,
        req: &FwReq,
        node: &FwNode,
        sections: &mut GeneralSections,
        msg: u32,
        timeout_ms: u32,
    ) -> Result<(), Error> {
        if T::notified(&sections.global, msg) {
            let res = T::whole_cache(req, node, &mut sections.global, timeout_ms);
            debug!(params = ?sections.global.params, ?res);
            res?;
        }
        if T::notified(&sections.tx_stream_format, msg) {
            let res = T::whole_cache(req, node, &mut sections.tx_stream_format, timeout_ms);
            debug!(params = ?sections.global.params, ?res);
            res?;
        }
        if T::notified(&sections.rx_stream_format, msg) {
            let res = T::whole_cache(req, node, &mut sections.rx_stream_format, timeout_ms);
            debug!(params = ?sections.global.params, ?res);
            res?;
        }

        Ok(())
    }
}
