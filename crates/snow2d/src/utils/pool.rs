/*!
Reference-counted pool

`pool` has to be used with care: [`Handle`] is a strong reference to an item. Unreference items will
be deleted in next sync call.

# Why not `Vec<T>`?

`usize` index for `Vec<T>` has some limitations:

1. The position of the item at the index can change on removing other items
2. The item at the index might be already deleted and another new item can be there (ABA problem)
3. The `usize` index can be used for unintentional `Vec<T>`

[`Pool`] handles these problems:

1. Invalidating an item don't change other items' positions
2. [`Handle`] is always valid because items have reference-counted lifetimes with them.
[`WeakHandle`] identifies their interested item with generational indices.
3. Handles have type parameters so the chances to use incorrect [`Pool`] reduce. (This is not
perfect as handles can be used for non-original pools).

Another approach would be using non-reference-counted [`Index`] in [`arena`].

[`arena`]: crate::utils::arena
[`Index`]: crate::utils::arena::Index
*/

use std::{
    cmp,
    marker::PhantomData,
    ops, slice,
    sync::mpsc::{channel, Receiver, Sender},
};

use derivative::Derivative;

type Gen = std::num::NonZeroU32;
type GenCounter = u32;
type RefCount = u16;

/// Newtype of `u32`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Slot(u32);

impl Slot {
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

/// Reference counting message (New | Drop)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Message {
    New(Slot),
    Drop(Slot),
}

/// Owing index to an item in a [`Pool`]
///
/// It can't identify the belonging [`Pool`].
#[derive(Debug)]
pub struct Handle<T> {
    slot: Slot,
    gen: Gen,
    sender: Sender<Message>,
    _phantom: PhantomData<fn() -> T>,
}

impl<T> cmp::PartialEq for Handle<T> {
    fn eq(&self, other: &Handle<T>) -> bool {
        // WARNING: it doesn't consider belonging pool
        self.gen == other.gen
    }
}

impl<T> Handle<T> {
    /// Index that corrresponds to memory location
    pub fn slot(&self) -> Slot {
        self.slot
    }

    pub fn downgrade(self) -> WeakHandle<T> {
        WeakHandle {
            slot: self.slot,
            gen: self.gen,
            _phantom: PhantomData,
        }
    }

    pub fn to_downgraded(&self) -> WeakHandle<T> {
        self.clone().downgrade()
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        // TODO: it dies if the pool is already dead?
        self.sender.send(Message::New(self.slot)).unwrap();
        Self {
            slot: self.slot,
            gen: self.gen,
            sender: self.sender.clone(),
            _phantom: Default::default(),
        }
    }
}

impl<T> Drop for Handle<T> {
    fn drop(&mut self) {
        // fails if the pool is already dead
        self.sender.send(Message::Drop(self.slot)).ok();
    }
}

/// Non-owing index to an item in a [`Pool`]
///
/// The item is identified with generational index.
#[derive(Derivative)]
#[derivative(Debug, PartialEq, Clone, Copy)]
pub struct WeakHandle<T> {
    slot: Slot,
    gen: Gen,
    _phantom: PhantomData<fn() -> T>,
}

impl<T> WeakHandle<T> {
    /// Index that corrresponds to memory location
    pub fn slot(&self) -> Slot {
        self.slot
    }
}

impl<T> From<Handle<T>> for WeakHandle<T> {
    fn from(h: Handle<T>) -> Self {
        Self {
            slot: h.slot,
            gen: h.gen,
            _phantom: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PoolEntry<T> {
    // TODO: refactor using an enum
    item: T,
    /// None if this entry is invalid
    gen: Option<Gen>,
    ref_count: RefCount,
}

/// Dynamic array with reference-counted [`Handle`]s
#[derive(Debug)]
pub struct Pool<T> {
    /// NOTE: we never call [`Vec::remove`]; it aligns (change positions of) other items.
    entries: Vec<PoolEntry<T>>,
    /// Generation counter per [`Pool`]. Another option is per slot.
    gen_count: GenCounter,
    /// Receives [`Message`]s from [`Handle`]s for handling reference countings
    receiver: Receiver<Message>,
    /// Cloned and passed to [`Handle`]s
    sender: Sender<Message>,
}

impl<T> Pool<T> {
    pub fn with_capacity(cap: usize) -> Self {
        let (sender, receiver) = channel::<Message>();

        Self {
            entries: Vec::with_capacity(cap),
            gen_count: 1,
            sender,
            receiver,
        }
    }

    /// Iterator of valid items in this pool
    pub fn iter(&self) -> impl Iterator<Item = &T>
    where
        T: 'static,
    {
        iters::Iter {
            entries: self.entries.iter(),
        }
    }

    /// Mutable iterator of valid items in this pool
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T>
    where
        T: 'static,
    {
        iters::IterMut {
            entries: self.entries.iter_mut(),
        }
    }

    /// Update reference counting of internal items and remove unreferenced nodes
    pub fn sync_refcounts(&mut self) {
        // TODO: can we assume messages are correctly ordered?
        while let Ok(mes) = self.receiver.try_recv() {
            match mes {
                Message::New(slot) => {
                    let e = &mut self.entries[slot.as_usize()];
                    e.ref_count += 1;
                }
                Message::Drop(slot) => {
                    let entry = &mut self.entries[slot.as_usize()];
                    entry.ref_count -= 1;
                    if entry.ref_count == 0 {
                        entry.gen = None;
                    }
                }
            }
        }
    }

    // /// Force removing a reference-counted node
    // pub unsafe fn remove_node(&mut self, slot: Slot) {
    //     let entry = &mut self.entries[slot.as_usize()];
    //     entry.gen = None;
    // }
}

/// Handle-based accessors
impl<T> Pool<T> {
    fn find_empty_slot(&mut self) -> Option<usize> {
        for i in 0..self.entries.len() {
            if let Some(e) = self.entries.get(i) {
                if e.gen.is_none() {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Returns a reference-counted [`Handle`] for the given item
    pub fn add(&mut self, item: impl Into<T>) -> Handle<T> {
        let item = item.into();

        let gen = Gen::new(self.gen_count);

        let entry = PoolEntry {
            item,
            gen: gen.clone(),
            ref_count: 1, // !
        };

        self.gen_count += 1;

        let slot = match self.find_empty_slot() {
            Some(i) => {
                self.entries[i] = entry;
                i
            }
            None => {
                let i = self.entries.len();
                self.entries.push(entry);
                i
            }
        };

        Handle {
            slot: Slot(slot as u32),
            gen: gen.unwrap(),
            sender: self.sender.clone(),
            _phantom: Default::default(),
        }
    }

    /// Tries to get a reference from a [`WeakHandle`]
    ///
    /// For strong [`Handle`]s, use index (`pool[handle]`).
    pub fn get(&self, weak_handle: &WeakHandle<T>) -> Option<&T> {
        let entry = &self.entries[weak_handle.slot.as_usize()];
        if entry.gen == Some(weak_handle.gen) {
            Some(&entry.item)
        } else {
            None
        }
    }

    /// Tries to get a mutable reference from a [`WeakHandle`]
    ///
    /// For strong [`Handle`]s, use index (`pool[handle]`).
    pub fn get_mut(&mut self, weak_handle: &WeakHandle<T>) -> Option<&mut T> {
        let entry = &mut self.entries[weak_handle.slot.as_usize()];
        if entry.gen == Some(weak_handle.gen) {
            Some(&mut entry.item)
        } else {
            None
        }
    }
}

impl<T> ops::Index<&Handle<T>> for Pool<T> {
    type Output = T;
    fn index(&self, handle: &Handle<T>) -> &Self::Output {
        &self.entries[handle.slot.as_usize()].item
    }
}

impl<T> ops::IndexMut<&Handle<T>> for Pool<T> {
    fn index_mut(&mut self, handle: &Handle<T>) -> &mut Self::Output {
        &mut self.entries[handle.slot.as_usize()].item
    }
}

impl<T> ops::Index<&WeakHandle<T>> for Pool<T> {
    type Output = T;
    fn index(&self, handle: &WeakHandle<T>) -> &Self::Output {
        let entry = &self.entries[handle.slot.as_usize()];
        assert!(entry.gen == Some(handle.gen));
        &entry.item
    }
}

impl<T> ops::IndexMut<&WeakHandle<T>> for Pool<T> {
    fn index_mut(&mut self, handle: &WeakHandle<T>) -> &mut Self::Output {
        let entry = &mut self.entries[handle.slot.as_usize()];
        assert!(entry.gen == Some(handle.gen));
        &mut entry.item
    }
}

/// Slot-based accessors
impl<T> Pool<T> {
    /// Retruns the item if it's valid
    pub fn get_by_slot(&self, slot: Slot) -> Option<&T> {
        let entry = self.entries.get(slot.as_usize())?;
        entry.gen.and(Some(&entry.item))
    }

    /// Retruns the item if it's valid
    pub fn get_mut_by_slot(&mut self, slot: Slot) -> Option<&mut T> {
        let entry = self.entries.get_mut(slot.as_usize())?;
        entry.gen.and(Some(&mut entry.item))
    }

    // TODO: use specific iterator?
    pub fn slots(&self) -> impl Iterator<Item = Slot> + '_ {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(i, entry)| entry.gen.and(Some(Slot(i as u32))))
    }

    /// Iterator of `(Slot, &T)`
    pub fn enumerate_items(&self) -> impl Iterator<Item = (Slot, &T)> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(i, entry)| entry.gen.and(Some((Slot(i as u32), &entry.item))))
    }

    /// Iterator of `(Slot, &mut T)`
    pub fn enumerate_items_mut(&mut self) -> impl Iterator<Item = (Slot, &mut T)> {
        self.entries
            .iter_mut()
            .enumerate()
            .filter_map(|(i, entry)| entry.gen.and(Some((Slot(i as u32), &mut entry.item))))
    }
}

pub mod iters {
    //! Iterator types of the `pool` module

    use super::*;

    pub struct Iter<'a, T: 'static> {
        // TODO: len: u32,
        pub(crate) entries: slice::Iter<'a, PoolEntry<T>>,
    }

    impl<'a, T: 'static> Iterator for Iter<'a, T> {
        type Item = &'a T;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                let e = self.entries.next()?;
                if e.gen.is_none() {
                    continue;
                }
                return Some(&e.item);
            }
        }
    }

    pub struct IterMut<'a, T: 'static> {
        // TODO: len: u32,
        pub(crate) entries: slice::IterMut<'a, PoolEntry<T>>,
    }

    impl<'a, T: 'static> Iterator for IterMut<'a, T> {
        type Item = &'a mut T;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                let e = self.entries.next()?;
                if e.gen.is_none() {
                    continue;
                }
                return Some(&mut e.item);
            }
        }
    }

    impl<'a, T: 'static> IntoIterator for &'a Pool<T> {
        type Item = &'a T;
        type IntoIter = Iter<'a, T>;
        fn into_iter(self) -> Self::IntoIter {
            Iter {
                entries: self.entries.iter(),
            }
        }
    }

    impl<'a, T: 'static> IntoIterator for &'a mut Pool<T> {
        type Item = &'a mut T;
        type IntoIter = IterMut<'a, T>;
        fn into_iter(self) -> Self::IntoIter {
            IterMut {
                entries: self.entries.iter_mut(),
            }
        }
    }
}
