// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use glib::Error;
use glib::IsA;

use alsactl::{ElemValueExt, ElemValueExtManual};

pub trait ElemValueAccessor<T>: IsA<alsactl::ElemValue>
where
    T: Copy + Clone + Default + Eq + PartialEq,
{
    fn set(&self, vals: &[T]);
    fn get(&self, vals: &mut [T]);

    fn set_vals<F>(&self, len: usize, mut cb: F) -> Result<(), Error>
    where
        F: FnMut(usize) -> Result<T, Error>,
    {
        let mut vals = vec![Default::default(); len];
        vals.iter_mut().enumerate().try_for_each(|(ch, v)| {
            *v = cb(ch)?;
            Ok(())
        })?;
        self.set(&vals);
        Ok(())
    }

    fn get_vals<F>(&self, old: &Self, len: usize, mut cb: F) -> Result<(), Error>
    where
        F: FnMut(usize, T) -> Result<(), Error>,
    {
        let mut vals = vec![Default::default(); len * 2];
        self.get(&mut vals[..len]);
        old.get(&mut vals[len..]);
        vals[..len]
            .iter()
            .zip(vals[len..].iter())
            .enumerate()
            .filter(|(_, (n, o))| *n != *o)
            .try_for_each(|(ch, (v, _))| {
                cb(ch, *v)?;
                Ok(())
            })?;
        Ok(())
    }

    fn set_val<F>(&self, mut cb: F) -> Result<(), Error>
    where
        F: FnMut() -> Result<T, Error>,
    {
        let mut vals = [Default::default()];
        vals[0] = cb()?;
        self.set(&vals);
        Ok(())
    }

    fn get_val<F>(&self, mut cb: F) -> Result<(), Error>
    where
        F: FnMut(T) -> Result<(), Error>,
    {
        let mut vals = [Default::default()];
        self.get(&mut vals);
        cb(vals[0])
    }
}

impl ElemValueAccessor<bool> for alsactl::ElemValue {
    fn set(&self, vals: &[bool]) {
        self.set_bool(vals);
    }

    fn get(&self, vals: &mut [bool]) {
        self.get_bool(vals);
    }
}

impl ElemValueAccessor<u8> for alsactl::ElemValue {
    fn set(&self, vals: &[u8]) {
        self.set_bytes(vals);
    }

    fn get(&self, vals: &mut [u8]) {
        self.get_bytes(vals);
    }
}

impl ElemValueAccessor<i32> for alsactl::ElemValue {
    fn set(&self, vals: &[i32]) {
        self.set_int(vals);
    }

    fn get(&self, vals: &mut [i32]) {
        self.get_int(vals);
    }
}

impl ElemValueAccessor<u32> for alsactl::ElemValue {
    fn set(&self, vals: &[u32]) {
        self.set_enum(vals);
    }

    fn get(&self, vals: &mut [u32]) {
        self.get_enum(vals);
    }
}

impl ElemValueAccessor<i64> for alsactl::ElemValue {
    fn set(&self, vals: &[i64]) {
        self.set_int64(vals);
    }

    fn get(&self, vals: &mut [i64]) {
        self.get_int64(vals);
    }
}
