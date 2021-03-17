/*!
Cheat the borrow checker using raw pointer
*/

/// Lifetime-free mutable reference to type `T`
///
/// # Safety
///
/// Make sure the pointer lives as long as required.
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
    #[inline]
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

    #[inline]
    pub unsafe fn as_mut(&self) -> &mut T {
        &mut *self.ptr
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

impl<T> AsRef<T> for Cheat<T> {
    fn as_ref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> AsMut<T> for Cheat<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}
