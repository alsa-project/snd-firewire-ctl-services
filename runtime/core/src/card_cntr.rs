// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    alsactl::{prelude::*, *},
    glib::FileError,
    tracing::{debug, debug_span, enabled, Level},
};

#[derive(Default)]
pub struct CardCntr {
    pub card: Card,
    entries: Vec<(ElemInfo, ElemValue)>,
}

pub trait CtlModel<O: Sized> {
    fn cache(&mut self, _: &mut O) -> Result<(), Error>;
    fn load(&mut self, card_cntr: &mut CardCntr) -> Result<(), Error>;
    fn read(
        &mut self,
        unit: &mut O,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error>;
    fn write(
        &mut self,
        unit: &mut O,
        elem_id: &ElemId,
        old: &ElemValue,
        new: &ElemValue,
    ) -> Result<bool, Error>;
}

pub trait MeasureModel<O: Sized> {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>);
    fn measure_states(&mut self, unit: &mut O) -> Result<(), Error>;
    fn measure_elem(
        &mut self,
        unit: &O,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error>;
}

pub trait NotifyModel<O: Sized, N> {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>);
    fn parse_notification(&mut self, unit: &mut O, notice: &N) -> Result<(), Error>;
}

impl Drop for CardCntr {
    fn drop(&mut self) {
        self.entries
            .iter()
            .filter_map(|(elem_info, _)| elem_id_from_elem_info(elem_info))
            .for_each(|elem_id| {
                let _ = self.card.remove_elems(&elem_id);
            });
    }
}

fn elem_id_from_elem_info(elem_info: &ElemInfo) -> Option<ElemId> {
    match elem_info {
        ElemInfo::Iec60958(info) => info.elem_id(),
        ElemInfo::Boolean(info) => info.elem_id(),
        ElemInfo::Bytes(info) => info.elem_id(),
        ElemInfo::Integer(info) => info.elem_id(),
        ElemInfo::Integer64(info) => info.elem_id(),
        ElemInfo::Enumerated(info) => info.elem_id(),
    }
}

fn match_elem_id(elem_info: &ElemInfo, elem_id: &ElemId) -> bool {
    elem_id_from_elem_info(elem_info)
        .map(|e| e.eq(elem_id))
        .unwrap_or_default()
}

fn dump_elem_info(elem_info: &ElemInfo) {
    if let Some(elem_id) = elem_id_from_elem_info(elem_info) {
        match elem_info {
            ElemInfo::Iec60958(info) => {
                debug!(
                    numid=?elem_id.numid(),
                    access=?info.access(),
                    elem_type=?info.elem_type(),
                    owner=?info.owner()
                );
            }
            ElemInfo::Boolean(info) => {
                debug!(
                    numid=?elem_id.numid(),
                    access=?info.access(),
                    elem_type=?info.elem_type(),
                    owner=?info.owner(),
                    value_count=?info.value_count()
                );
            }
            ElemInfo::Bytes(info) => {
                debug!(
                    numid=?elem_id.numid(),
                    access=?info.access(),
                    elem_type=?info.elem_type(),
                    owner=?info.owner(),
                    value_count=?info.value_count()
                );
            }
            ElemInfo::Integer(info) => {
                debug!(
                    numid=?elem_id.numid(),
                    access=?info.access(),
                    elem_type=?info.elem_type(),
                    owner=?info.owner(),
                    value_count=?info.value_count(),
                    value_min=?info.value_min(),
                    value_max=?info.value_max(),
                    value_step=?info.value_step()
                );
            }
            ElemInfo::Integer64(info) => {
                debug!(
                    numid=?elem_id.numid(),
                    access=?info.access(),
                    elem_type=?info.elem_type(),
                    owner=?info.owner(),
                    value_count=?info.value_count(),
                    value_min=?info.value_min(),
                    value_max=?info.value_max(),
                    value_step=?info.value_step()
                );
            }
            ElemInfo::Enumerated(info) => {
                let labels: Vec<String> = info
                    .labels()
                    .iter()
                    .map(|label| label.to_string())
                    .collect();
                debug!(
                    numid=?elem_id.numid(),
                    access=?info.access(),
                    elem_type=?info.elem_type(),
                    owner=?info.owner(),
                    value_count=?info.value_count(),
                    ?labels
                );
            }
        }
    }
}

fn value_array_literal(elem_info: &ElemInfo, elem_value: &ElemValue) -> String {
    match elem_info {
        ElemInfo::Iec60958(_) => {
            format!(
                "{:?} {:?}",
                elem_value.iec60958_channel_status(),
                elem_value.iec60958_user_data()
            )
        }
        ElemInfo::Boolean(info) => {
            let count = info.value_count() as usize;
            format!("{:?}", &elem_value.boolean()[..count])
        }
        ElemInfo::Bytes(info) => {
            let count = info.value_count() as usize;
            format!("{:?}", &elem_value.bytes()[..count])
        }
        ElemInfo::Integer(info) => {
            let count = info.value_count() as usize;
            format!("{:?}", &elem_value.int()[..count])
        }
        ElemInfo::Integer64(info) => {
            let count = info.value_count() as usize;
            format!("{:?}", &elem_value.int64()[..count])
        }
        ElemInfo::Enumerated(info) => {
            let count = info.value_count() as usize;
            format!("{:?}", &elem_value.enumerated()[..count])
        }
    }
}

impl CardCntr {
    pub fn add_bool_elems(
        &mut self,
        elem_id: &ElemId,
        elem_count: usize,
        value_count: usize,
        unlock: bool,
    ) -> Result<Vec<ElemId>, Error> {
        let _entry = debug_span!("boolean").entered();

        let elem_info = ElemInfoBoolean::new();
        elem_info.set_value_count(value_count as u32);

        let access = ElemAccessFlag::READ | ElemAccessFlag::WRITE | ElemAccessFlag::VOLATILE;
        elem_info.set_access(access);

        let res = self.register_elems(&elem_id, elem_count, &elem_info, None, unlock);
        debug!(
            name = ?elem_id.name().as_str(),
            iface = ?elem_id.iface(),
            device_id = ?elem_id.device_id(),
            subdevice_id = ?elem_id.subdevice_id(),
            index = ?elem_id.index(),
            ?elem_count,
            ?unlock,
            ?res,
        );
        res
    }

    pub fn add_enum_elems<O>(
        &mut self,
        elem_id: &ElemId,
        elem_count: usize,
        value_count: usize,
        labels: &[O],
        tlv: Option<&[u32]>,
        unlock: bool,
    ) -> Result<Vec<ElemId>, Error>
    where
        O: AsRef<str>,
    {
        let _entry = debug_span!("enumerated").entered();

        let entries = labels
            .iter()
            .map(|entry| entry.as_ref())
            .collect::<Vec<&str>>();

        let elem_info = ElemInfoEnumerated::new();
        elem_info.set_value_count(value_count as u32);
        elem_info.set_labels(&entries);

        let access = ElemAccessFlag::READ | ElemAccessFlag::WRITE | ElemAccessFlag::VOLATILE;
        elem_info.set_access(access);

        let res = self.register_elems(&elem_id, elem_count, &elem_info, tlv, unlock);
        debug!(
            name = ?elem_id.name().as_str(),
            iface = ?elem_id.iface(),
            device_id = ?elem_id.device_id(),
            subdevice_id = ?elem_id.subdevice_id(),
            index = ?elem_id.index(),
            ?elem_count,
            ?value_count,
            ?entries,
            ?tlv,
            ?unlock,
            ?res,
        );
        res
    }

    pub fn add_bytes_elems(
        &mut self,
        elem_id: &ElemId,
        elem_count: usize,
        value_count: usize,
        tlv: Option<&[u32]>,
        unlock: bool,
    ) -> Result<Vec<ElemId>, Error> {
        let _entry = debug_span!("bytes").entered();

        let elem_info = ElemInfoBytes::new();
        elem_info.set_value_count(value_count as u32);

        let mut access = ElemAccessFlag::READ | ElemAccessFlag::WRITE | ElemAccessFlag::VOLATILE;
        if tlv != None {
            access |= ElemAccessFlag::TLV_READ | ElemAccessFlag::TLV_WRITE;
        }
        elem_info.set_access(access);

        let res = self.register_elems(&elem_id, elem_count, &elem_info, tlv, unlock);
        debug!(
            name = ?elem_id.name().as_str(),
            iface = ?elem_id.iface(),
            device_id = ?elem_id.device_id(),
            subdevice_id = ?elem_id.subdevice_id(),
            index = ?elem_id.index(),
            ?elem_count,
            ?value_count,
            ?tlv,
            ?unlock,
            ?res,
        );
        res
    }

    pub fn add_int_elems(
        &mut self,
        elem_id: &ElemId,
        elem_count: usize,
        min: i32,
        max: i32,
        step: i32,
        value_count: usize,
        tlv: Option<&[u32]>,
        unlock: bool,
    ) -> Result<Vec<ElemId>, Error> {
        let _entry = debug_span!("integer").entered();

        let elem_info = ElemInfoInteger::new();
        elem_info.set_value_count(value_count as u32);
        elem_info.set_value_min(min);
        elem_info.set_value_max(max);
        elem_info.set_value_step(step);

        let mut access = ElemAccessFlag::READ | ElemAccessFlag::WRITE | ElemAccessFlag::VOLATILE;
        if tlv != None {
            access |= ElemAccessFlag::TLV_READ | ElemAccessFlag::TLV_WRITE;
        }
        elem_info.set_access(access);

        let res = self.register_elems(&elem_id, elem_count, &elem_info, tlv, unlock);
        debug!(
            name = ?elem_id.name().as_str(),
            iface = ?elem_id.iface(),
            device_id = ?elem_id.device_id(),
            subdevice_id = ?elem_id.subdevice_id(),
            index = ?elem_id.index(),
            ?elem_count,
            ?min,
            ?max,
            ?step,
            ?value_count,
            ?tlv,
            ?unlock,
            ?res,
        );
        res
    }

    pub fn add_iec60958_elem(
        &mut self,
        elem_id: &ElemId,
        elem_count: usize,
        unlock: bool,
    ) -> Result<ElemId, Error> {
        let _entry = debug_span!("iec60958").entered();

        let elem_info = ElemInfoIec60958::new();

        let access = ElemAccessFlag::READ | ElemAccessFlag::WRITE | ElemAccessFlag::VOLATILE;
        elem_info.set_access(access);

        let res = self.register_elems(&elem_id, elem_count, &elem_info, None, unlock);

        debug!(
            name = ?elem_id.name().as_str(),
            iface = ?elem_id.iface(),
            device_id = ?elem_id.device_id(),
            subdevice_id = ?elem_id.subdevice_id(),
            index = ?elem_id.index(),
            ?elem_count,
            ?unlock,
            ?res,
        );

        let mut elem_id_list = res?;
        Ok(elem_id_list.remove(0))
    }

    fn register_elems<O: AsRef<ElemInfoCommon>>(
        &mut self,
        elem_id: &ElemId,
        elem_count: usize,
        elem_info: &O,
        tlv: Option<&[u32]>,
        unlock: bool,
    ) -> Result<Vec<ElemId>, Error> {
        let _enter = debug_span!("register").entered();

        // If already registered, reuse them if possible.
        let elem_id_list = self.card.elem_id_list()?;
        let elem_id_list = match elem_id_list.iter().position(|eid| eid.eq(elem_id)) {
            Some(_) => {
                let elem_id_list: Vec::<ElemId> = elem_id_list.into_iter().filter(|eid| {
                    eid.name() == elem_id.name() &&
                    eid.device_id() == elem_id.device_id() &&
                    eid.subdevice_id() == elem_id.subdevice_id() &&
                    eid.iface() == elem_id.iface()
                }).collect();

                if elem_id_list.len() != elem_count {
                    // The count of elements is unexpected.
                    let label = format!("{} is already added however the count is unexpected.", elem_id.name());
                    return Err(Error::new(FileError::Inval, &label));
                }

                elem_id_list.iter().try_for_each(|elem_id| {
                    let info = self.card.elem_info(elem_id)?;

                    if info.as_ref().access().contains(ElemAccessFlag::OWNER) {
                        // Programming error.
                        let label = format!("{} is already added by runtime.", elem_id.name());
                        Err(Error::new(FileError::Inval, &label))
                    } else if info.as_ref().access().contains(ElemAccessFlag::LOCK) {
                        // The other process locks the element.
                        let label = format!("{} is locked by the other process.", elem_id.name());
                        Err(Error::new(FileError::Inval, &label))
                    } else if info.as_ref().elem_type() != elem_info.as_ref().elem_type() {
                        // The existent element has unexpected type.
                        let label = format!("{} is already added but has unexpected type.", elem_id.name());
                        Err(Error::new(FileError::Inval, &label))
                    } else {
                        Ok(())
                    }
                })?;

                // Reuse the list of element identifiers.
                elem_id_list
            }
            None => {
                self.card.add_elems(elem_id, elem_count as u32, elem_info)
                    .map_err(|e| {
                        if let Some(CardError::Failed) = e.kind::<CardError>() {
                            if e.to_string() == "ioctl(ELEM_ADD) 12(Cannot allocate memory)" {
                                let mut msg = String::new();
                                msg.push_str("Allocation of user-defined element set reached capacity of snd.ko\n");
                                msg.push_str("This can be fixed by using Linux kernel v5.13 or later,\n");
                                msg.push_str("or by using snd.ko pached to extend the capacity.\n");
                                msg.push_str("The capacity is defined as 'MAX_USER_CONTROLS'");
                                msg.push_str("located in 'sound/core/control.c'.");
                                eprintln!("{}", msg);
                            }
                        }
                        e
                    })?
            }
        };

        elem_id_list
            .iter()
            .try_for_each(|elem_id| match self.card.elem_info(&elem_id) {
                Ok(elem_info) => match elem_info.as_ref().elem_id() {
                    Some(elem_id) => {
                        let mut v = ElemValue::new();
                        self.card.read_elem_value(&elem_id, &mut v)?;

                        debug!(
                            numid = ?elem_id.numid(),
                            name = ?elem_id.name().as_str(),
                            iface = ?elem_id.iface(),
                            device_id = ?elem_id.device_id(),
                            subdevice_id = ?elem_id.subdevice_id(),
                            index = ?elem_id.index(),
                        );

                        if enabled!(Level::DEBUG) {
                            dump_elem_info(&elem_info);
                        }

                        self.entries.push((elem_info, v));
                        Ok(())
                    }
                    None => {
                        let _ = self.card.remove_elems(&elem_id_list[0]);
                        let label = "Unexpected result to detect element id";
                        Err(Error::new(FileError::Io, label))
                    }
                },
                Err(err) => {
                    let _ = self.card.remove_elems(&elem_id_list[0]);
                    Err(err)
                }
            })?;

        if let Some(cntr) = tlv {
            elem_id_list.iter().try_for_each(|elem_id| {
                let res = self.card.write_elem_tlv(&elem_id, &cntr);
                debug!(numid=?elem_id.numid(), ?tlv, ?res);
                res
            })?;
        }

        if unlock {
            elem_id_list.iter().for_each(|elem_id| {
                // Ignore any errors.
                let res = self.card.lock_elem(&elem_id, false);
                debug!(numid=?elem_id.numid(), ?unlock, ?res);
            });
        }

        Ok(elem_id_list)
    }

    pub fn dispatch_elem_event<O, T>(
        &mut self,
        unit: &mut O,
        elem_id: &ElemId,
        events: &ElemEventMask,
        ctl_model: &mut T,
    ) -> Result<(), Error>
    where
        O: Sized,
        T: CtlModel<O>,
    {
        if events.contains(ElemEventMask::REMOVE) {
            let _enter = debug_span!("remove").entered();

            debug!(numid = ?elem_id.numid());

            self.entries
                .retain(|(elem_info, _)| match_elem_id(elem_info, elem_id));
            return Ok(());
        }

        if events.contains(ElemEventMask::ADD) {
            let _enter = debug_span!("add").entered();

            for (elem_info, v) in &mut self.entries {
                if !match_elem_id(elem_info, elem_id) {
                    continue;
                }

                let mut val = ElemValue::new();

                let _enter = debug_span!("hardware").entered();

                let res = ctl_model.read(unit, &elem_id, &mut val);
                debug!(
                    numid = elem_id.numid(),
                    values = value_array_literal(elem_info, &val),
                    ?res,
                );

                _enter.exit();

                if let Ok(res) = res {
                    if !res || v.equal(&val) {
                        continue;
                    }

                    let _enter = debug_span!("kernel").entered();

                    let res = self.card.write_elem_value(&elem_id, &val);
                    debug!(
                        numid = elem_id.numid(),
                        values = value_array_literal(elem_info, &val),
                        ?res,
                    );

                    _enter.exit();

                    if res.is_err() {
                        continue;
                    }

                    *v = val;
                }
            }
        }

        if events.contains(ElemEventMask::VALUE) {
            let _enter = debug_span!("value").entered();

            for (elem_info, v) in &mut self.entries {
                if !match_elem_id(elem_info, elem_id) {
                    continue;
                }

                let _enter = debug_span!("local").entered();

                debug!(
                    numid = elem_id.numid(),
                    values = value_array_literal(elem_info, &v),
                );

                _enter.exit();

                let _enter = debug_span!("kernel").entered();

                let mut val = ElemValue::new();
                let res = self.card.read_elem_value(&elem_id, &mut val);
                debug!(
                    numid = elem_id.numid(),
                    values = value_array_literal(elem_info, &val),
                    ?res,
                );

                _enter.exit();

                // No need to update the hardware.
                if res.is_err() || v.equal(&val) {
                    continue;
                }

                let _enter = debug_span!("hardware").entered();

                let res = ctl_model.write(unit, &elem_id, v, &val);
                debug!(
                    numid = elem_id.numid(),
                    old_values = value_array_literal(elem_info, &v),
                    new_values = value_array_literal(elem_info, &val),
                    ?res,
                );

                _enter.exit();

                if let Ok(res) = res {
                    if res {
                        *v = val;
                        return Ok(());
                    }
                } else {
                    // Back to old values.
                    self.card.write_elem_value(&elem_id, v)?;
                    res?;
                }
            }
        }

        Ok(())
    }

    pub fn measure_elems<O, T>(
        &mut self,
        unit: &mut O,
        elem_id_list: &Vec<ElemId>,
        ctl_model: &mut T,
    ) -> Result<(), Error>
    where
        O: Sized,
        T: CtlModel<O> + MeasureModel<O>,
    {
        let card = &self.card;
        let entries = &mut self.entries;

        let enter = debug_span!("cache").entered();
        ctl_model.measure_states(unit)?;
        enter.exit();

        elem_id_list.iter().try_for_each(|elem_id| {
            entries
                .iter_mut()
                .filter(|(elem_info, _)| match_elem_id(elem_info, elem_id))
                .try_for_each(|(elem_info, elem_value)| {
                    let _enter = debug_span!("hardware").entered();

                    let res = ctl_model.measure_elem(unit, elem_id, elem_value);
                    debug!(
                        numid = elem_id.numid(),
                        values = value_array_literal(elem_info, &elem_value),
                        ?res,
                    );
                    res?;

                    _enter.exit();

                    let _enter = debug_span!("kernel").entered();
                    let res = card.write_elem_value(elem_id, elem_value);
                    debug!(
                        numid = elem_id.numid(),
                        values = value_array_literal(elem_info, &elem_value),
                        ?res,
                    );
                    _enter.exit();

                    res
                })
        })
    }

    pub fn dispatch_notification<O, N, T>(
        &mut self,
        unit: &mut O,
        notification: &N,
        elem_id_list: &Vec<ElemId>,
        ctl_model: &mut T,
    ) -> Result<(), Error>
    where
        O: Sized,
        T: CtlModel<O> + NotifyModel<O, N>,
    {
        let card = &self.card;
        let entries = &mut self.entries;

        let _enter = debug_span!("cache").entered();
        ctl_model.parse_notification(unit, notification)?;
        _enter.exit();

        elem_id_list.iter().try_for_each(|elem_id| {
            entries
                .iter_mut()
                .filter(|(elem_info, _)| match_elem_id(elem_info, elem_id))
                .try_for_each(|(elem_info, elem_value)| {
                    let _enter = debug_span!("hardware").entered();
                    let res = ctl_model.read(unit, elem_id, elem_value);
                    debug!(
                        numid = elem_id.numid(),
                        values = value_array_literal(elem_info, &elem_value),
                        ?res,
                    );
                    res?;

                    _enter.exit();

                    let _enter = debug_span!("kernel").entered();
                    let res = card.write_elem_value(elem_id, elem_value);
                    debug!(
                        numid = elem_id.numid(),
                        values = value_array_literal(elem_info, &elem_value),
                        ?res,
                    );

                    _enter.exit();

                    res
                })
        })
    }
}
