/*!
Pool with reference-counted handles

# About

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
    marker::PhantomData,
    ops,
    sync::mpsc::{channel, Receiver, Sender},
};

type Gen = std::num::NonZeroU32;
type GenCounter = u32;
type Index = u32;
type RefCount = u16;

/// Message for reference counting (New | Drop)
#[derive(Debug, Clone, PartialEq, Eq)]
enum Message {
    New(Index),
    Drop(Index),
}

#[derive(Debug, Clone)]
struct PoolEntry<T> {
    item: T,
    /// None if this entry is invalid
    gen: Option<Gen>,
    ref_count: RefCount,
}

/// Dynamic array with reference-counted [`Handle`]s
#[derive(Debug)]
pub struct Pool<T> {
    /// NOTE: we never [`Vec::remove`] because it aligns other items changes corresponding indices
    /// for stored items
    entries: Vec<PoolEntry<T>>,
    /// Generation counter per [`Pool`] (another option is per slot)
    gen_count: GenCounter,
    /// Receives [`Message`]s from [`Handle`]s to handle reference countings
    receiver: Receiver<Message>,
    /// Just cloned and passed to [`Handle`]s
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
    pub fn items(&self) -> impl Iterator<Item = &T> {
        self.entries
            .iter()
            .filter(|e| e.gen.is_some())
            .map(|e| &e.item)
    }

    /// Mutable iterator of valid items in this pool
    pub fn items_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.entries
            .iter_mut()
            .filter(|e| e.gen.is_some())
            .map(|e| &mut e.item)
    }

    fn find_slot(&mut self) -> Option<usize> {
        for i in 0..self.entries.len() {
            if let Some(e) = self.entries.get(i) {
                if e.gen.is_none() {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Adds an item and returns a reference-counted [`Handle`] for it
    pub fn add(&mut self, item: T) -> Handle<T> {
        let entry = PoolEntry {
            item,
            gen: Gen::new(self.gen_count),
            ref_count: 1,
        };

        self.gen_count += 1;

        let index = match self.find_slot() {
            Some(i) => {
                self.entries[i] = entry;
                i
            }
            None => {
                if self.entries.len() < self.entries.capacity() {
                    let i = self.entries.len();
                    self.entries.push(entry);
                    i
                } else {
                    let i = self.entries.capacity();
                    self.entries.push(entry);
                    i
                }
            }
        };

        Handle {
            index: index as Index,
            sender: self.sender.clone(),
            _phantom: Default::default(),
        }
    }

    /// Update reference counting of internal items
    pub fn sync_refcount(&mut self) {
        while let Ok(mes) = self.receiver.try_recv() {
            match mes {
                Message::New(ix) => {
                    self.entries[ix as usize].ref_count += 1;
                }
                Message::Drop(ix) => {
                    self.entries[ix as usize].ref_count -= 1;
                    if self.entries[ix as usize].ref_count == 0 {
                        // invalidate the entry
                        self.entries[ix as usize].gen = None;
                    }
                }
            }
        }
    }

    /// Tries to get a reference from a [`WeakHandle`]
    ///
    /// Use indexer for strong [`Handle`]s
    pub fn get(&self, weak_handle: &WeakHandle<T>) -> Option<&T> {
        let entry = &self.entries[weak_handle.index as usize];
        if let Some(gen) = entry.gen {
            if gen == weak_handle.gen {
                return Some(&entry.item);
            }
        }
        None
    }

    /// Tries to get a mutable reference from a [`WeakHandle`]
    ///
    /// Use indexer for strong [`Handle`]s
    pub fn get_mut(&mut self, weak_handle: &WeakHandle<T>) -> Option<&mut T> {
        let entry = &mut self.entries[weak_handle.index as usize];
        if let Some(gen) = entry.gen {
            if gen == weak_handle.gen {
                return Some(&mut entry.item);
            }
        }
        None
    }
}

impl<T> ops::Index<&Handle<T>> for Pool<T> {
    type Output = T;
    fn index(&self, handle: &Handle<T>) -> &Self::Output {
        &self.entries[handle.index as usize].item
    }
}

impl<T> ops::IndexMut<&Handle<T>> for Pool<T> {
    fn index_mut(&mut self, handle: &Handle<T>) -> &mut Self::Output {
        &mut self.entries[handle.index as usize].item
    }
}

impl<T> ops::Index<&WeakHandle<T>> for Pool<T> {
    type Output = T;
    fn index(&self, handle: &WeakHandle<T>) -> &Self::Output {
        let entry = &self.entries[handle.index as usize];
        assert!(entry.gen.is_some() && entry.gen.unwrap() == handle.gen);
        &entry.item
    }
}

impl<T> ops::IndexMut<&WeakHandle<T>> for Pool<T> {
    fn index_mut(&mut self, handle: &WeakHandle<T>) -> &mut Self::Output {
        let entry = &mut self.entries[handle.index as usize];
        assert!(entry.gen.is_some() && entry.gen.unwrap() == handle.gen);
        &mut entry.item
    }
}

/// Owing index to an item in a [`Pool`]
///
/// The identity is NOT guaranteed if you have two `Pools` of the same type.
#[derive(Debug)]
pub struct Handle<T> {
    index: Index,
    sender: Sender<Message>,
    _phantom: PhantomData<T>,
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        // TODO: fix it because it dies if the pool is already dead?
        self.sender.send(Message::New(self.index)).unwrap();
        Self {
            index: self.index,
            sender: self.sender.clone(),
            _phantom: Default::default(),
        }
    }
}

impl<T> Drop for Handle<T> {
    fn drop(&mut self) {
        // TODO: fix it because it dies if the pool is already dead?
        self.sender.send(Message::Drop(self.index)).unwrap();
    }
}

/// Non-owing index to an item in a [`Pool`] with generational index that can the interested item
pub struct WeakHandle<T> {
    index: Index,
    gen: Gen,
    _phantom: PhantomData<T>,
}

#[cfg(test)]
mod test {

    //
}
