// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2024 Takashi Sakamoto

use {super::*, protocols::weiss::normal::*};

pub type Adc2Model = WeissNormalModel<WeissAdc2Protocol>;
pub type VestaModel = WeissNormalModel<WeissVestaProtocol>;
pub type Dac2Model = WeissNormalModel<WeissDac2Protocol>;
pub type Afi1Model = WeissNormalModel<WeissAfi1Protocol>;
pub type Dac202Model = WeissNormalModel<WeissDac202Protocol>;
pub type Int203Model = WeissNormalModel<WeissInt203Protocol>;

const TIMEOUT_MS: u32 = 100;

#[derive(Default)]
pub struct WeissNormalModel<T>
where
    T: TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>,
{
    req: FwReq,
    sections: GeneralSections,
    common_ctl: CommonCtl<T>,
}

impl<T> CtlModel<(SndDice, FwNode)> for WeissNormalModel<T>
where
    T: TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>,
{
    fn cache(&mut self, (_, node): &mut (SndDice, FwNode)) -> Result<(), Error> {
        T::read_general_sections(&self.req, &node, &mut self.sections, TIMEOUT_MS)?;

        self.common_ctl
            .cache_whole_params(&self.req, &node, &mut self.sections, TIMEOUT_MS)?;

        Ok(())
    }

    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error> {
        self.common_ctl.load(card_cntr)
    }

    fn read(&mut self, elem_id: &ElemId, elem_value: &mut ElemValue) -> Result<bool, Error> {
        self.common_ctl.read(elem_id, elem_value)
    }

    fn write(
        &mut self,
        (unit, node): &mut (SndDice, FwNode),
        elem_id: &ElemId,
        new: &ElemValue,
    ) -> Result<bool, Error> {
        self.common_ctl.write(
            &unit,
            &self.req,
            &node,
            &mut self.sections,
            elem_id,
            new,
            TIMEOUT_MS,
        )
    }
}

impl<T> NotifyModel<(SndDice, FwNode), u32> for WeissNormalModel<T>
where
    T: TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>,
{
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.notified_elem_id_list);
    }

    fn parse_notification(
        &mut self,
        (_, node): &mut (SndDice, FwNode),
        msg: &u32,
    ) -> Result<(), Error> {
        self.common_ctl
            .parse_notification(&self.req, &node, &mut self.sections, *msg, TIMEOUT_MS)
    }
}

impl<T> MeasureModel<(SndDice, FwNode)> for WeissNormalModel<T>
where
    T: TcatNotifiedSectionOperation<GlobalParameters>
        + TcatFluctuatedSectionOperation<GlobalParameters>
        + TcatMutableSectionOperation<GlobalParameters>
        + TcatNotifiedSectionOperation<TxStreamFormatParameters>
        + TcatNotifiedSectionOperation<RxStreamFormatParameters>
        + TcatSectionOperation<ExtendedSyncParameters>,
{
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>) {
        elem_id_list.extend_from_slice(&self.common_ctl.measured_elem_id_list);
    }

    fn measure_states(&mut self, (_, node): &mut (SndDice, FwNode)) -> Result<(), Error> {
        self.common_ctl
            .cache_partial_params(&self.req, &node, &mut self.sections, TIMEOUT_MS)
    }
}
