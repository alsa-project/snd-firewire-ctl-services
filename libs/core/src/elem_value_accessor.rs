// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto

use {
    super::*,
    alsactl::{prelude::*, ElemValue},
    glib::IsA,
};

pub trait ElemValueAccessor<T>: IsA<ElemValue>
where
    T: Copy + Clone + Default + Eq + PartialEq,
{
    fn set(&self, vals: &[T]);
    fn get<F>(&self, len: usize, cb: F) -> Result<(), Error>
    where
        F: FnMut(&[T]) -> Result<(), Error>;

    fn set_vals<F>(&self, len: usize, mut cb: F) -> Result<(), Error>
    where
        F: FnMut(usize) -> Result<T, Error>,
    {
        self.get(len, |prev| {
            let mut vals = prev.to_owned();
            vals.iter_mut()
                .enumerate()
                .try_for_each(|(ch, v)| cb(ch).map(|val| *v = val))
                .map(|_| self.set(&vals))
        })
    }

    fn get_vals<F>(&self, old: &Self, len: usize, mut cb: F) -> Result<(), Error>
    where
        F: FnMut(usize, T) -> Result<(), Error>,
    {
        self.get(len, |curr| {
            old.get(len, |prev| {
                curr.iter()
                    .zip(prev)
                    .enumerate()
                    .filter(|(_, (n, o))| !n.eq(o))
                    .try_for_each(|(ch, (v, _))| cb(ch, *v))
            })
        })
    }

    fn set_val<F>(&self, mut cb: F) -> Result<(), Error>
    where
        F: FnMut() -> Result<T, Error>,
    {
        cb().map(|val| self.set(&[val]))
    }

    fn get_val<F>(&self, mut cb: F) -> Result<(), Error>
    where
        F: FnMut(T) -> Result<(), Error>,
    {
        self.get(1, |vals| cb(vals[0]))
    }
}

impl ElemValueAccessor<bool> for ElemValue {
    fn set(&self, vals: &[bool]) {
        self.set_bool(vals)
    }

    fn get<F>(&self, len: usize, mut cb: F) -> Result<(), Error>
    where
        F: FnMut(&[bool]) -> Result<(), Error>,
    {
        cb(&self.boolean()[..len])
    }
}

impl ElemValueAccessor<u8> for ElemValue {
    fn set(&self, vals: &[u8]) {
        self.set_bytes(vals)
    }

    fn get<F>(&self, len: usize, mut cb: F) -> Result<(), Error>
    where
        F: FnMut(&[u8]) -> Result<(), Error>,
    {
        cb(&self.bytes()[..len])
    }
}

impl ElemValueAccessor<i32> for ElemValue {
    fn set(&self, vals: &[i32]) {
        self.set_int(vals)
    }

    fn get<F>(&self, len: usize, mut cb: F) -> Result<(), Error>
    where
        F: FnMut(&[i32]) -> Result<(), Error>,
    {
        cb(&self.int()[..len])
    }
}

impl ElemValueAccessor<u32> for ElemValue {
    fn set(&self, vals: &[u32]) {
        self.set_enum(vals)
    }

    fn get<F>(&self, len: usize, mut cb: F) -> Result<(), Error>
    where
        F: FnMut(&[u32]) -> Result<(), Error>,
    {
        cb(&self.enumerated()[..len])
    }
}

impl ElemValueAccessor<i64> for ElemValue {
    fn set(&self, vals: &[i64]) {
        self.set_int64(vals)
    }

    fn get<F>(&self, len: usize, mut cb: F) -> Result<(), Error>
    where
        F: FnMut(&[i64]) -> Result<(), Error>,
    {
        cb(&self.int64()[..len])
    }
}
