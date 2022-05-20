// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use super::command_dsp_runtime::*;

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct Track16 {
    req: FwReq,
    resp: FwResp,
    sequence_number: u8,
}

impl CtlModel<SndMotu> for Track16 {
    fn load(&mut self, _: &mut SndMotu, _: &mut CardCntr) -> Result<(), Error> {
        Ok(())
    }

    fn read(&mut self, _: &mut SndMotu, _: &ElemId, _: &mut ElemValue) -> Result<bool, Error> {
        Ok(false)
    }

    fn write(
        &mut self,
        _: &mut SndMotu,
        _: &ElemId,
        _: &ElemValue,
        _: &ElemValue,
    ) -> Result<bool, Error> {
        Ok(false)
    }
}

impl NotifyModel<SndMotu, u32> for Track16 {
    fn get_notified_elem_list(&mut self, _: &mut Vec<ElemId>) {}

    fn parse_notification(&mut self, _: &mut SndMotu, _: &u32) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndMotu,
        _: &ElemId,
        _: &mut ElemValue,
    ) -> Result<bool, Error> {
        Ok(false)
    }
}

impl NotifyModel<SndMotu, Vec<DspCmd>> for Track16 {
    fn get_notified_elem_list(&mut self, _: &mut Vec<ElemId>) {}

    fn parse_notification(&mut self, _: &mut SndMotu, _: &Vec<DspCmd>) -> Result<(), Error> {
        Ok(())
    }

    fn read_notified_elem(
        &mut self,
        _: &SndMotu,
        _: &ElemId,
        _: &mut ElemValue,
    ) -> Result<bool, Error> {
        Ok(false)
    }
}

impl MeasureModel<SndMotu> for Track16 {
    fn get_measure_elem_list(&mut self, _: &mut Vec<ElemId>) {}

    fn measure_states(&mut self, _: &mut SndMotu) -> Result<(), Error> {
        Ok(())
    }

    fn measure_elem(&mut self, _: &SndMotu, _: &ElemId, _: &mut ElemValue) -> Result<bool, Error> {
        Ok(false)
    }
}

impl CommandDspModel for Track16 {
    fn prepare_message_handler<F>(&mut self, unit: &mut SndMotu, handler: F) -> Result<(), Error>
    where
        F: Fn(&FwResp, FwTcode, u64, u32, u32, u32, u32, &[u8]) -> FwRcode + 'static,
    {
        Track16Protocol::register_message_destination_address(
            &mut self.resp,
            &mut self.req,
            &mut unit.get_node(),
            TIMEOUT_MS,
        )?;
        self.resp.connect_requested2(handler);
        Ok(())
    }

    fn begin_messaging(&mut self, unit: &mut SndMotu) -> Result<(), Error> {
        Track16Protocol::begin_messaging(
            &mut self.req,
            &mut unit.get_node(),
            &mut self.sequence_number,
            TIMEOUT_MS,
        )
    }

    fn release_message_handler(&mut self, unit: &mut SndMotu) -> Result<(), Error> {
        Track16Protocol::cancel_messaging(
            &mut self.req,
            &mut unit.get_node(),
            &mut self.sequence_number,
            TIMEOUT_MS,
        )?;
        Track16Protocol::release_message_destination_address(
            &mut self.resp,
            &mut self.req,
            &mut unit.get_node(),
            TIMEOUT_MS,
        )?;
        Ok(())
    }
}
