use std::{collections::HashMap, num::NonZeroU32};

use imgui::{im_str, Ui};

use super::Inspect;

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

// `paste::paste!` concats identifiers in declarative macro with `[< .. >]` syntax
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
impl_array!(i32, i32, input_int);
impl_array!(u32, i32, input_int);
impl_array!(usize, i32, input_int);

impl<T> Inspect for [T; 0] {
    fn inspect(&mut self, _ui: &Ui, _label: &str) {}
}

// non-zero types

impl Inspect for NonZeroU32 {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        self.clone().get().inspect(ui, label);
    }
}

// idiomatic types

impl<T> Inspect for std::marker::PhantomData<T> {
    fn inspect(&mut self, _ui: &Ui, _label: &str) {}
}

impl<T: Inspect> Inspect for Option<T> {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        if let Some(item) = self.as_mut() {
            item.inspect(ui, label);
        } else {
            ui.label_text(&im_str!("{}", label), im_str!("None"));
        }
    }
}

// collections

impl<K: Inspect, V: Inspect> Inspect for HashMap<K, V> {
    fn inspect(&mut self, ui: &Ui, _label: &str) {
        ui.text(&im_str!("TODO: HashMap"));
    }
}

// more std types
