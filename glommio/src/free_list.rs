// Unless explicitly stated otherwise all files in this repository are licensed under the
// MIT/Apache-2.0 License, at your convenience
//
// This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2020 Datadog, Inc.
//
use std::marker::PhantomData;
use std::mem;
use std::ops;
use std::fmt;

#[derive(Debug)]
pub(crate) struct Idx<T> {
    raw: usize,
    _ty: PhantomData<fn() -> T>,
}

impl<T: fmt::Debug> Copy for Idx<T> {}
impl<T: fmt::Debug> Clone for Idx<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: fmt::Debug> PartialEq for Idx<T> {
    fn eq(&self, other: &Idx<T>) -> bool {
        self.raw == other.raw
    }
}
impl<T: fmt::Debug> Eq for Idx<T> {}

impl<T: fmt::Debug> Idx<T> {
    pub(crate) fn from_raw(raw: usize) -> Idx<T> {
        Idx {
            raw,
            _ty: PhantomData,
        }
    }
    pub(crate) fn to_raw(self) -> usize {
        self.raw
    }
}

#[derive(Debug)]
pub(crate) struct FreeList<T: fmt::Debug> {
    first_free: Option<Idx<T>>,
    slots: Vec<Slot<T>>,
}

impl<T: fmt::Debug> Default for FreeList<T> {
    fn default() -> Self {
        FreeList {
            first_free: None,
            slots: Vec::new(),
        }
    }
}

impl<T: fmt::Debug> FreeList<T> {
    pub(crate) fn alloc(&mut self, item: T) -> Idx<T> {
        let slot = Slot::Full { item };
        match self.first_free {
            Some(idx) => {
                self.first_free = match mem::replace(&mut self.slots[idx.to_raw()], slot) {
                    Slot::Free { next_free } => next_free,
                    Slot::Full { .. } => {
                        panic!("id {:?} was full already. First free: {:?} Status: {:?}", idx,  self.first_free, self.slots);
                    }
                };
                idx
            }
            None => {
                let idx = Idx::from_raw(self.slots.len());
                self.slots.push(slot);
                idx
            }
        }
    }
    pub(crate) fn dealloc(&mut self, idx: Idx<T>) -> T {
        let slot = Slot::Free {
            next_free: mem::replace(&mut self.first_free, Some(idx)),
        };
        match mem::replace(&mut self.slots[idx.to_raw()], slot) {
            Slot::Full { item } => item,
            Slot::Free { .. } => {
                panic!("id {:?} was empty already, First free {:?}, Status: {:?}", idx,  self.first_free, self.slots);
            }
        }
    }
}

impl<T: fmt::Debug> ops::Index<Idx<T>> for FreeList<T> {
    type Output = T;

    fn index(&self, idx: Idx<T>) -> &T {
        match &self.slots[idx.to_raw()] {
            Slot::Free { .. } => unreachable!(),
            Slot::Full { item } => item,
        }
    }
}

impl<T: fmt::Debug> ops::IndexMut<Idx<T>> for FreeList<T> {
    fn index_mut(&mut self, idx: Idx<T>) -> &mut T {
        match &mut self.slots[idx.to_raw()] {
            Slot::Free { .. } => unreachable!(),
            Slot::Full { item } => item,
        }
    }
}

#[derive(Debug)]
enum Slot<T: fmt::Debug> {
    Free { next_free: Option<Idx<T>> },
    Full { item: T },
}

#[test]
fn free_list_smoke_test() {
    let mut free_list: FreeList<&str> = FreeList::default();

    let hello = free_list.alloc("hello");
    assert_eq!(hello, Idx::from_raw(0));

    let world = free_list.alloc("world");
    assert_eq!(world, Idx::from_raw(1));

    assert_eq!(free_list[hello], "hello");
    assert_eq!(free_list[world], "world");

    free_list.dealloc(hello);

    let goodbye = free_list.alloc("goodbye");
    assert_eq!(goodbye, Idx::from_raw(0));

    free_list.dealloc(goodbye);
    free_list.dealloc(world);

    let a = free_list.alloc("a");
    let b = free_list.alloc("b");
    let c = free_list.alloc("c");
    assert_eq!(a, Idx::from_raw(1));
    assert_eq!(b, Idx::from_raw(0));
    assert_eq!(c, Idx::from_raw(2));
}
