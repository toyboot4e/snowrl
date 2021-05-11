/*!
`paste::paste!` concats identifiers in declarative macro with `[< .. >]` syntax
*/

use std::{collections::VecDeque, marker::PhantomData, num::NonZeroU32};

use imgui::{im_str, Ui};

use crate::utils::inspect::{self, Inspect};

macro_rules! im_self {
    ($ty:ident, $method:ident) => {
        impl Inspect for $ty {
            fn inspect(&mut self, ui: &Ui, label: &str) {
                let _changed = ui.$method(&im_str!("{}", label), self);
            }
        }
    };
}

macro_rules! im_im {
    ($ty:ident, $method:ident) => {
        impl Inspect for $ty {
            fn inspect(&mut self, ui: &Ui, label: &str) {
                let _changed = ui.$method(&im_str!("{}", label), &im_str!("{}", self));
            }
        }
    };
}

im_self!(bool, checkbox);

im_im!(str, label_text);
im_im!(String, label_text);
// TODO: char?

macro_rules! im_input {
    ($ty:ident, $as:ty, $method:ident) => {
        impl Inspect for $ty {
            fn inspect(&mut self, ui: &Ui, label: &str) {
                let mut x = *self as $as;
                if ui.$method(&im_str!("{}", label), &mut x).build() {
                    *self = x as $ty;
                }
            }
        }
    };
}

macro_rules! im_input_array {
    ($ty:ty, $N:expr, $as:ty, $method:ident) => {
        impl Inspect for [$ty; $N] {
            #[allow(warnings)]
            fn inspect(&mut self, ui: &Ui, label: &str) {
                use arraytools::ArrayTools;
                let mut xs = self.map(|x| x as $as);
                let label = &im_str!("{}", label);
                if ui.$method(label, &mut xs).build() {
                    *self = xs.map(|x| x as $ty);
                }
            }
        }
    };
}

macro_rules! im_drag {
    ($ty:ty, $N:expr, $as:ty) => {
        impl Inspect for [$ty; $N] {
            #[allow(warnings)]
            fn inspect(&mut self, ui: &Ui, label: &str) {
                use arraytools::ArrayTools;
                let mut xs = self.map(|x| x as $as);
                let label = &im_str!("{}", label);
                if imgui::Drag::new(label).build_array(ui, &mut xs) {
                    *self = xs.map(|x| x as $ty);
                }
            }
        }
    };
}

impl<T> Inspect for [T; 0] {
    fn inspect(&mut self, _ui: &Ui, _label: &str) {}
}

macro_rules! impl_array {
    ($ty:ty, $as:ty, $method:ident) => {
        paste::paste! {
            im_input!($ty, $as, $method);
            im_input_array!($ty, 2, $as, [<$method 2>]);
            im_input_array!($ty, 3, $as, [<$method 3>]);
            im_input_array!($ty, 4, $as, [<$method 4>]);
        }
    };
}

impl_array!(f32, f32, input_float);
impl_array!(f64, f32, input_float);

impl_array!(i8, i32, input_int);
impl_array!(i32, i32, input_int);

impl_array!(u8, i32, input_int);
impl_array!(u32, i32, input_int);
impl_array!(u64, i32, input_int);

impl_array!(isize, i32, input_int);
impl_array!(usize, i32, input_int);

/// impl Inspect for `(T0, T1, ..)`
macro_rules! impl_tuple {
    ($($i:expr),*) => {
        paste::paste! {
            impl<$([<T $i>]),*> Inspect for ($([<T $i>]),*)
            where
                $([<T $i>]: Inspect,)*
            {
                fn inspect(&mut self, ui: &Ui, label: &str) {
                    inspect::nest(ui, label, || {
                        $(
                            &mut self.$i.inspect(ui, stringify!($i));
                        )*
                    });
                }
            }
        }
    };
}

impl_tuple!(0, 1);
impl_tuple!(0, 1, 2);
impl_tuple!(0, 1, 2, 3);

// non-zero types

impl Inspect for NonZeroU32 {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        self.clone().get().inspect(ui, label);
    }
}

// idiomatic types

macro_rules! impl_ignore {
    ($ty:ty) => {
        impl<T> Inspect for $ty {
            fn inspect(&mut self, _ui: &Ui, _label: &str) {}
        }
    };
}

impl_ignore!(PhantomData<T>);

impl<T: Inspect> Inspect for Option<T> {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        match self {
            Some(x) => x.inspect(ui, label),
            None => ui.label_text(&im_str!("{}", label), im_str!("None")),
        }
    }
}

// collections

/// Implements `Inspect` with iterator
macro_rules! impl_seq {
    ($ty:ident) => {
        impl<T: Inspect> Inspect for $ty<T> {
            fn inspect(&mut self, ui: &Ui, label: &str) {
                inspect::inspect_seq(self.iter_mut(), ui, label)
            }
        }
    };
}

impl_seq!(Vec);
impl_seq!(VecDeque);

// more std types

impl Inspect for std::time::Duration {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        let time = self.as_secs_f32();
        ui.label_text(&im_str!("{}", label), &im_str!("{}", time));
    }
}
