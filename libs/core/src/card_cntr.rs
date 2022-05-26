// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    alsactl::*,
    glib::{FileError, IsA},
    hinawa::SndUnit,
};

pub struct CardCntr {
    pub card: Card,
    entries: Vec<ElemValue>,
}

pub trait CtlModel<O: IsA<SndUnit>> {
    fn load(&mut self, unit: &mut O, card_cntr: &mut CardCntr) -> Result<(), Error>;
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

pub trait MeasureModel<O: IsA<SndUnit>> {
    fn get_measure_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>);
    fn measure_states(&mut self, unit: &mut O) -> Result<(), Error>;
    fn measure_elem(
        &mut self,
        unit: &O,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error>;
}

pub trait NotifyModel<O: IsA<SndUnit>, N> {
    fn get_notified_elem_list(&mut self, elem_id_list: &mut Vec<ElemId>);
    fn parse_notification(&mut self, unit: &mut O, notice: &N) -> Result<(), Error>;
    fn read_notified_elem(
        &mut self,
        unit: &O,
        elem_id: &ElemId,
        elem_value: &mut ElemValue,
    ) -> Result<bool, Error>;
}

impl Drop for CardCntr {
    fn drop(&mut self) {
        self.entries
            .iter()
            .filter_map(|v| v.get_property_elem_id())
            .for_each(|elem_id| {
                let _ = self.card.remove_elems(&elem_id);
            });
    }
}

impl CardCntr {
    pub fn new() -> Self {
        CardCntr {
            card: Card::new(),
            entries: Vec::new(),
        }
    }

    pub fn add_bool_elems(
        &mut self,
        elem_id: &ElemId,
        elem_count: usize,
        value_count: usize,
        unlock: bool,
    ) -> Result<Vec<ElemId>, Error> {
        let elem_info = ElemInfo::new(ElemType::Boolean)?;
        elem_info.set_property_value_count(value_count as u32);

        let access = ElemAccessFlag::READ
            | ElemAccessFlag::WRITE
            | ElemAccessFlag::VOLATILE;
        elem_info.set_property_access(access);

        self.register_elems(&elem_id, elem_count, &elem_info, None, unlock)
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
        let entries = labels
            .iter()
            .map(|entry| entry.as_ref())
            .collect::<Vec<&str>>();

        let elem_info = ElemInfo::new(ElemType::Enumerated)?;
        elem_info.set_property_value_count(value_count as u32);
        elem_info.set_enum_data(&entries)?;

        let access = ElemAccessFlag::READ
            | ElemAccessFlag::WRITE
            | ElemAccessFlag::VOLATILE;
        elem_info.set_property_access(access);

        self.register_elems(&elem_id, elem_count, &elem_info, tlv, unlock)
    }

    pub fn add_bytes_elems(
        &mut self,
        elem_id: &ElemId,
        elem_count: usize,
        value_count: usize,
        tlv: Option<&[u32]>,
        unlock: bool,
    ) -> Result<Vec<ElemId>, Error> {
        let elem_info = ElemInfo::new(ElemType::Bytes)?;
        elem_info.set_property_value_count(value_count as u32);

        let mut access = ElemAccessFlag::READ
            | ElemAccessFlag::WRITE
            | ElemAccessFlag::VOLATILE;
        if tlv != None {
            access |= ElemAccessFlag::TLV_READ | ElemAccessFlag::TLV_WRITE;
        }
        elem_info.set_property_access(access);

        self.register_elems(&elem_id, elem_count, &elem_info, tlv, unlock)
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
        let elem_info = ElemInfo::new(ElemType::Integer)?;
        elem_info.set_property_value_count(value_count as u32);
        elem_info.set_int_data(&[min, max, step])?;

        let mut access = ElemAccessFlag::READ
            | ElemAccessFlag::WRITE
            | ElemAccessFlag::VOLATILE;
        if tlv != None {
            access |= ElemAccessFlag::TLV_READ | ElemAccessFlag::TLV_WRITE;
        }
        elem_info.set_property_access(access);

        self.register_elems(&elem_id, elem_count, &elem_info, tlv, unlock)
    }

    pub fn add_iec60958_elem(
        &mut self,
        elem_id: &ElemId,
        elem_count: usize,
        unlock: bool,
    ) -> Result<ElemId, Error> {
        let elem_info = ElemInfo::new(ElemType::Iec60958)?;
        elem_info.set_property_value_count(1);

        let access = ElemAccessFlag::READ
            | ElemAccessFlag::WRITE
            | ElemAccessFlag::VOLATILE;
        elem_info.set_property_access(access);

        let mut elem_id_list =
            self.register_elems(&elem_id, elem_count, &elem_info, None, unlock)?;

        Ok(elem_id_list.remove(0))
    }

    fn register_elems<P>(
        &mut self,
        elem_id: &ElemId,
        elem_count: usize,
        elem_info: &P,
        tlv: Option<&[u32]>,
        unlock: bool,
    ) -> Result<Vec<ElemId>, Error>
    where
        P: IsA<ElemInfo>,
    {
        // If already registered, reuse them if possible.
        let elem_id_list = self.card.get_elem_id_list()?;
        let elem_id_list = match elem_id_list.iter().position(|eid| eid.eq(elem_id)) {
            Some(_) => {
                let elem_id_list: Vec::<ElemId> = elem_id_list.into_iter().filter(|eid| {
                    eid.get_name() == elem_id.get_name() &&
                    eid.get_device_id() == elem_id.get_device_id() &&
                    eid.get_subdevice_id() == elem_id.get_subdevice_id() &&
                    eid.get_iface() == elem_id.get_iface()
                }).collect();

                if elem_id_list.len() != elem_count {
                    // The count of elements is unexpected.
                    let label = format!("{} is already added however the count is unexpected.", elem_id.get_name());
                    return Err(Error::new(FileError::Inval, &label));
                }

                elem_id_list.iter().try_for_each(|elem_id| {
                    let einfo = self.card.get_elem_info(elem_id)?;

                    if einfo.get_property_access().contains(ElemAccessFlag::OWNER) {
                        // Programming error.
                        let label = format!("{} is already added by runtime.", elem_id.get_name());
                        Err(Error::new(FileError::Inval, &label))
                    } else if einfo.get_property_access().contains(ElemAccessFlag::LOCK) {
                        // The other process locks the element.
                        let label = format!("{} is locked by the other process.", elem_id.get_name());
                        Err(Error::new(FileError::Inval, &label))
                    } else if einfo.get_property_type() != elem_info.get_property_type() {
                        // The existent element has unexpected type.
                        let label = format!("{} is already added but has unexpected type.", elem_id.get_name());
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
            .try_for_each(|elem_id| match self.card.get_elem_info(&elem_id) {
                Ok(elem_info) => match elem_info.get_property_elem_id() {
                    Some(elem_id) => {
                        let mut v = ElemValue::new();
                        self.card.read_elem_value(&elem_id, &mut v)?;
                        self.entries.push(v);
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
            elem_id_list
                .iter()
                .try_for_each(|elem_id| self.card.write_elem_tlv(&elem_id, &cntr))?;
        }

        if unlock {
            elem_id_list.iter().for_each(|elem_id| {
                // Ignore any errors.
                let _ = self.card.lock_elem(&elem_id, false);
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
        O: IsA<SndUnit>,
        T: CtlModel<O>,
    {
        if events.contains(ElemEventMask::REMOVE) {
            self.entries.retain(|v| match v.get_property_elem_id() {
                Some(e) => e != *elem_id,
                None => true,
            });
            return Ok(());
        }

        if events.contains(ElemEventMask::ADD) {
            for v in &mut self.entries {
                let e = match v.get_property_elem_id() {
                    Some(e) => e,
                    None => continue,
                };

                if e != *elem_id {
                    continue;
                }

                let mut val = ElemValue::new();

                if let Ok(res) = ctl_model.read(unit, &e, &mut val) {
                    if !res {
                        continue;
                    }

                    if v.equal(&val) {
                        continue;
                    }

                    if let Err(_) = self.card.write_elem_value(&e, &val) {
                        continue;
                    }

                    *v = val;
                }
            }
        }

        if events.contains(ElemEventMask::VALUE) {
            for v in &mut self.entries {
                let e = match v.get_property_elem_id() {
                    Some(e) => e,
                    None => continue,
                };

                if e != *elem_id {
                    continue;
                }

                let mut val = ElemValue::new();
                if self.card.read_elem_value(&e, &mut val).is_err() {
                    continue;
                }

                // No need to update the hardware.
                if v.equal(&val) {
                    continue;
                }

                match ctl_model.write(unit, &e, v, &val) {
                    Ok(res) => {
                        if res {
                            *v = val;
                            return Ok(());
                        }
                    }
                    Err(err) => {
                        // Back to old values.
                        self.card.write_elem_value(&e, v)?;
                        return Err(err);
                    }
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
        O: IsA<SndUnit>,
        T: CtlModel<O> + MeasureModel<O>,
    {
        let card = &self.card;
        let entries = &mut self.entries;

        ctl_model.measure_states(unit)?;

        elem_id_list.iter().try_for_each(|elem_id| {
            entries
                .iter_mut()
                .filter(|elem_value| match elem_value.get_property_elem_id() {
                    Some(eid) => eid == *elem_id,
                    None => false,
                })
                .try_for_each(|elem_value| {
                    if ctl_model.measure_elem(unit, elem_id, elem_value)? {
                        card.write_elem_value(elem_id, elem_value)?;
                    }

                    Ok(())
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
        O: IsA<SndUnit>,
        T: CtlModel<O> + NotifyModel<O, N>,
    {
        let card = &self.card;
        let entries = &mut self.entries;

        ctl_model.parse_notification(unit, notification)?;

        elem_id_list.iter().try_for_each(|elem_id| {
            entries
                .iter_mut()
                .filter(|elem_value| match elem_value.get_property_elem_id() {
                    Some(eid) => eid == *elem_id,
                    None => false,
                })
                .try_for_each(|elem_value| {
                    if ctl_model.read_notified_elem(unit, elem_id, elem_value)? {
                        card.write_elem_value(elem_id, elem_value)?;
                    }

                    Ok(())
                })
        })
    }
}
