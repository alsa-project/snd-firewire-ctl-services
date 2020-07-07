// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use alsactl::*;
use glib::IsA;
use glib::{Error, FileError};

pub struct CardCntr {
    pub card: alsactl::Card,
    entries: Vec<alsactl::ElemValue>,
}

pub trait CtlModel<O: IsA<hinawa::SndUnit>> {
    fn load(&mut self, unit: &O, card_cntr: &mut CardCntr) -> Result<(), Error>;
    fn read(
        &mut self,
        unit: &O,
        elem_id: &alsactl::ElemId,
        elem_value: &mut alsactl::ElemValue,
    ) -> Result<bool, Error>;
    fn write(
        &mut self,
        unit: &O,
        elem_id: &alsactl::ElemId,
        old: &alsactl::ElemValue,
        new: &alsactl::ElemValue,
    ) -> Result<bool, Error>;
}

pub trait MonitorModel<O: IsA<hinawa::SndUnit>> {
    fn get_monitored_elems(&mut self, elem_id_list: &mut Vec<alsactl::ElemId>);
    fn monitor_unit(&mut self, unit: &O) -> Result<(), Error>;
    fn monitor_elems(
        &mut self,
        unit: &O,
        elem_id: &alsactl::ElemId,
        old: &alsactl::ElemValue,
        new: &mut alsactl::ElemValue,
    ) -> Result<bool, Error>;
}

impl Drop for CardCntr {
    fn drop(&mut self) {
        self.entries.iter().filter_map(|v| v.get_property_elem_id()).for_each(|elem_id| {
            let _ = self.card.remove_elems(&elem_id);
        });
    }
}

impl CardCntr {
    pub fn new() -> Self {
        CardCntr {
            card: alsactl::Card::new(),
            entries: Vec::new(),
        }
    }

    pub fn add_bool_elems(
        &mut self,
        elem_id: &alsactl::ElemId,
        elem_count: usize,
        value_count: usize,
        unlock: bool,
    ) -> Result<Vec<alsactl::ElemId>, Error> {
        let elem_info = alsactl::ElemInfo::new(ElemType::Boolean)?;
        elem_info.set_property_value_count(value_count as u32);

        let access = alsactl::ElemAccessFlag::READ
            | alsactl::ElemAccessFlag::WRITE
            | alsactl::ElemAccessFlag::VOLATILE;
        elem_info.set_property_access(access);

        self.register_elems(&elem_id, elem_count, &elem_info, None, unlock)
    }

    pub fn add_enum_elems(
        &mut self,
        elem_id: &alsactl::ElemId,
        elem_count: usize,
        value_count: usize,
        labels: &[&str],
        tlv: Option<&[i32]>,
        unlock: bool,
    ) -> Result<Vec<alsactl::ElemId>, Error> {
        let elem_info = alsactl::ElemInfo::new(ElemType::Enumerated)?;
        elem_info.set_property_value_count(value_count as u32);
        elem_info.set_enum_data(&labels)?;

        let access = alsactl::ElemAccessFlag::READ
            | alsactl::ElemAccessFlag::WRITE
            | alsactl::ElemAccessFlag::VOLATILE;
        elem_info.set_property_access(access);

        self.register_elems(&elem_id, elem_count, &elem_info, tlv, unlock)
    }

    pub fn add_bytes_elems(
        &mut self,
        elem_id: &alsactl::ElemId,
        elem_count: usize,
        value_count: usize,
        tlv: Option<&[i32]>,
        unlock: bool,
    ) -> Result<Vec<alsactl::ElemId>, Error> {
        let elem_info = alsactl::ElemInfo::new(ElemType::Bytes)?;
        elem_info.set_property_value_count(value_count as u32);

        let mut access = alsactl::ElemAccessFlag::READ
            | alsactl::ElemAccessFlag::WRITE
            | alsactl::ElemAccessFlag::VOLATILE;
        if tlv != None {
            access |= alsactl::ElemAccessFlag::TLV_READ | alsactl::ElemAccessFlag::TLV_WRITE;
        }
        elem_info.set_property_access(access);

        self.register_elems(&elem_id, elem_count, &elem_info, tlv, unlock)
    }

    pub fn add_int_elems(
        &mut self,
        elem_id: &alsactl::ElemId,
        elem_count: usize,
        min: i32,
        max: i32,
        step: i32,
        value_count: usize,
        tlv: Option<&[i32]>,
        unlock: bool,
    ) -> Result<Vec<ElemId>, Error> {
        let elem_info = alsactl::ElemInfo::new(ElemType::Integer)?;
        elem_info.set_property_value_count(value_count as u32);
        elem_info.set_int_data(&[min, max, step])?;

        let mut access = alsactl::ElemAccessFlag::READ
            | alsactl::ElemAccessFlag::WRITE
            | alsactl::ElemAccessFlag::VOLATILE;
        if tlv != None {
            access |= alsactl::ElemAccessFlag::TLV_READ | alsactl::ElemAccessFlag::TLV_WRITE;
        }
        elem_info.set_property_access(access);

        self.register_elems(&elem_id, elem_count, &elem_info, tlv, unlock)
    }

    fn register_elems<P>(
        &mut self,
        elem_id: &alsactl::ElemId,
        elem_count: usize,
        elem_info: &P,
        tlv: Option<&[i32]>,
        unlock: bool,
    ) -> Result<Vec<alsactl::ElemId>, Error>
    where
        P: IsA<alsactl::ElemInfo>,
    {
        // If already registered, reuse them if possible.
        let elem_id_list = self.card.get_elem_id_list()?;
        let elem_id_list = match elem_id_list.iter().position(|eid| eid.eq(elem_id)) {
            Some(_) => {
                let elem_id_list: Vec::<alsactl::ElemId> = elem_id_list.into_iter().filter(|eid| {
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

                    if einfo.get_property_access().contains(alsactl::ElemAccessFlag::OWNER) {
                        // Programming error.
                        let label = format!("{} is already added by runtime.", elem_id.get_name());
                        Err(Error::new(FileError::Inval, &label))
                    } else if einfo.get_property_access().contains(alsactl::ElemAccessFlag::LOCK) {
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
            None => self.card.add_elems(elem_id, elem_count as u32, elem_info)?
        };

        elem_id_list.iter().try_for_each(|elem_id| {
            match self.card.get_elem_info(&elem_id) {
                Ok(elem_info) => match elem_info.get_property_elem_id() {
                    Some(elem_id) => {
                        let mut v = alsactl::ElemValue::new();
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
            }
        })?;

        if let Some(cntr) = tlv {
            elem_id_list.iter().try_for_each(|elem_id| {
                self.card.write_elem_tlv(&elem_id, cntr)
            })?;
        }

        if unlock {
            elem_id_list.iter().for_each(|elem_id|{
                // Ignore any errors.
                let _ = self.card.lock_elem(&elem_id, false);
            });
        }

        Ok(elem_id_list)
    }

    pub fn monitor_elems<O, T>(
        &mut self,
        unit: &O,
        elem_id_list: &Vec<alsactl::ElemId>,
        ctl_model: &mut T,
    ) -> Result<(), Error>
    where
        O: IsA<hinawa::SndUnit>,
        T: CtlModel<O> + MonitorModel<O>,
    {
        elem_id_list.iter().for_each(|elem_id| {
            for v in &mut self.entries {
                let e = match v.get_property_elem_id() {
                    Some(e) => e,
                    None => continue,
                };

                if e != *elem_id {
                    continue;
                }

                let mut val = alsactl::ElemValue::new();

                if let Ok(res) = ctl_model.monitor_elems(unit, &e, v, &mut val) {
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
        });

        Ok(())
    }

    pub fn dispatch_elem_event<O, T>(
        &mut self,
        unit: &O,
        elem_id: &alsactl::ElemId,
        events: &alsactl::ElemEventMask,
        ctl_model: &mut T,
    ) -> Result<(), Error>
    where
        O: IsA<hinawa::SndUnit>,
        T: CtlModel<O>,
    {
        if events.contains(alsactl::ElemEventMask::REMOVE) {
            self.entries.retain(|v| match v.get_property_elem_id() {
                Some(e) => e != *elem_id,
                None => true,
            });
            return Ok(());
        }

        if events.contains(alsactl::ElemEventMask::ADD) {
            for v in &mut self.entries {
                let e = match v.get_property_elem_id() {
                    Some(e) => e,
                    None => continue,
                };

                if e != *elem_id {
                    continue;
                }

                let mut val = alsactl::ElemValue::new();

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

        if events.contains(alsactl::ElemEventMask::VALUE) {
            for v in &mut self.entries {
                let e = match v.get_property_elem_id() {
                    Some(e) => e,
                    None => continue,
                };

                if e != *elem_id {
                    continue;
                }

                let mut val = alsactl::ElemValue::new();
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
}
