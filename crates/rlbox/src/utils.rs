//! Utilities

/// Double buffer
#[derive(Debug, Clone)]
pub struct Double<T> {
    /// Front buffer at initial state
    pub a: T,
    /// Back buffer at initial state
    pub b: T,
    /// True then `a` is front
    counter: bool,
}

impl<T: Default> Default for Double<T> {
    fn default() -> Self {
        Self {
            a: T::default(),
            b: T::default(),
            counter: true,
        }
    }
}

impl<T> Double<T> {
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

    pub fn into_front(self) -> T {
        if self.counter {
            self.a
        } else {
            self.b
        }
    }

    pub fn into_back(self) -> T {
        if self.counter {
            self.b
        } else {
            self.a
        }
    }

    pub fn front(&self) -> &T {
        if self.counter {
            &self.a
        } else {
            &self.b
        }
    }

    pub fn front_mut(&mut self) -> &mut T {
        if self.counter {
            &mut self.a
        } else {
            &mut self.b
        }
    }

    pub fn back(&self) -> &T {
        if self.counter {
            &self.b
        } else {
            &self.a
        }
    }

    pub fn back_mut(&mut self) -> &mut T {
        if self.counter {
            &mut self.b
        } else {
            &mut self.a
        }
    }
}
