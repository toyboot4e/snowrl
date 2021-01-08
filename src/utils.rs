//! Utilities

use std::time::Duration;

/// Double buffer
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

/// Double buffer and interpolation value
#[derive(Debug, Clone)]
pub struct DoubleTrack<T> {
    /// Front
    pub a: T,
    /// Back
    pub b: T,
    /// Interpolation value
    pub dt: Duration,
}

impl<T: Default> Default for DoubleTrack<T> {
    fn default() -> Self {
        Self {
            a: Default::default(),
            b: Default::default(),
            dt: Default::default(),
        }
    }
}

impl<T> DoubleTrack<T> {
    /// TODO: maybe improve efficiency
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.a, &mut self.b);
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
    pub fn new(reference: &T) -> Self {
        Self {
            ptr: reference as *const _ as *mut _,
        }
    }

    pub fn empty() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
        }
    }

    /// Explicit cast to `T`
    ///
    /// `deref_mut` without importing `DerefMut`.
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

/// Shorthand for `Cheat::new`
pub fn cheat<T>(reference: &T) -> Cheat<T> {
    Cheat::new(reference)
}
