/*!
Trivial utilities
 */

pub mod consts {
    //! TODO: remove

    /// Duration in seconds to change direction in 45 degrees
    pub const CHANGE_DIR_TIME: f32 = 1.0 / 60.0;
}

/// Raw double buffer with interpolation value
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
