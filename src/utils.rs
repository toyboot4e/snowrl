/// Lifetime-free pointer to type `T`
///
/// This is dangerous but works on certain senarios.
#[derive(Debug)]
pub struct Cheat<T> {
    ptr: *mut T,
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
