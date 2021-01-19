//! Utilities

pub mod ez;

pub mod tweak {
    //! See [inline_tweak](https://docs.rs/inline_tweak/latest/inline_tweak/)

    pub use inline_tweak::{self, tweak, watch, Tweakable};
}

/// Raw double buffer
#[derive(Debug, Clone)]
pub struct Double<T> {
    /// Front
    pub a: T,
    /// Back
    pub b: T,
}

impl<T: Default> Default for Double<T> {
    fn default() -> Self {
        Self {
            a: Default::default(),
            b: Default::default(),
        }
    }
}

impl<T> Double<T> {
    /// TODO: maybe improve efficiency
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.a, &mut self.b);
    }
}

/// Raw double buffer and interpolation value
#[derive(Debug, Clone)]
pub struct DoubleTrack<T> {
    /// Front
    pub a: T,
    /// Back
    pub b: T,
    /// Interpolation value
    pub t: f32,
}

impl<T: Default> Default for DoubleTrack<T> {
    fn default() -> Self {
        Self {
            a: Default::default(),
            b: Default::default(),
            t: Default::default(),
        }
    }
}

impl<T> DoubleTrack<T> {
    /// TODO: maybe improve efficiency
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.a, &mut self.b);
    }
}

/// Double buffer that can internally swap buffers without copy
#[derive(Debug, Clone)]
pub struct DoubleSwap<T> {
    /// Front buffer at initial state
    a: T,
    /// Back buffer at initial state
    b: T,
    /// True then `a` is front
    counter: bool,
}

impl<T: Default> Default for DoubleSwap<T> {
    fn default() -> Self {
        Self {
            a: T::default(),
            b: T::default(),
            counter: true,
        }
    }
}

impl<T> DoubleSwap<T> {
    pub fn new(a: T, b: T) -> Self {
        Self {
            a,
            b,
            counter: true,
        }
    }

    /// Swaps front/back buffers
    pub fn swap(&mut self) {
        self.counter = !self.counter;
    }

    pub fn unwrap(self) -> [T; 2] {
        if self.counter {
            [self.a, self.b]
        } else {
            [self.b, self.a]
        }
    }

    pub fn into_a(self) -> T {
        if self.counter {
            self.a
        } else {
            self.b
        }
    }

    pub fn into_b(self) -> T {
        if self.counter {
            self.b
        } else {
            self.a
        }
    }

    /// Front
    pub fn a(&self) -> &T {
        if self.counter {
            &self.a
        } else {
            &self.b
        }
    }

    pub fn a_mut(&mut self) -> &mut T {
        if self.counter {
            &mut self.a
        } else {
            &mut self.b
        }
    }

    pub fn set_a(&mut self, x: T) {
        if self.counter {
            self.a = x;
        } else {
            self.b = x;
        }
    }

    /// Back
    pub fn b(&self) -> &T {
        if self.counter {
            &self.b
        } else {
            &self.a
        }
    }

    pub fn b_mut(&mut self) -> &mut T {
        if self.counter {
            &mut self.b
        } else {
            &mut self.a
        }
    }

    pub fn set_b(&mut self, x: T) {
        if self.counter {
            self.b = x;
        } else {
            self.a = x;
        }
    }
}

/// Lifetime-free mutable reference to type `T`
///
/// Be sure that the pointer lives as long as required.
///
/// I basicaly prefer `Cheat<T>` to `Rc<RefCell<T>>`.
#[derive(Debug)]
pub struct Cheat<T> {
    ptr: *mut T,
}

impl<T> Clone for Cheat<T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<T> Cheat<T> {
    pub unsafe fn new(reference: &T) -> Self {
        Self {
            ptr: reference as *const _ as *mut _,
        }
    }

    pub unsafe fn empty() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
        }
    }

    /// Explicit cast to `T`
    pub fn cheat(&mut self) -> &mut T {
        use std::ops::DerefMut;
        self.deref_mut()
    }
}

impl<T> std::ops::Deref for Cheat<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T> std::ops::DerefMut for Cheat<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}
