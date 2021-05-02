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
                if ui.$method(&im_str!("{}", label), &mut xs).build() {
                    *self = xs.map(|x| x as $ty);
                }
            }
        }
    };
}

im_input!(f32, f32, input_float);
im_input_array!(f32, 2, f32, input_float2);
im_input_array!(f32, 3, f32, input_float3);
im_input_array!(f32, 4, f32, input_float4);

im_input!(i32, i32, input_int);
im_input_array!(i32, 2, i32, input_int2);
im_input_array!(i32, 3, i32, input_int3);
im_input_array!(i32, 4, i32, input_int4);

im_input!(isize, i32, input_int);
im_input_array!(isize, 2, i32, input_int2);
im_input_array!(isize, 3, i32, input_int3);
im_input_array!(isize, 4, i32, input_int4);

im_input!(u32, i32, input_int);
im_input_array!(u32, 2, i32, input_int2);
im_input_array!(u32, 3, i32, input_int3);
im_input_array!(u32, 4, i32, input_int4);

im_input!(usize, i32, input_int);
im_input_array!(usize, 2, i32, input_int2);
im_input_array!(usize, 3, i32, input_int3);
im_input_array!(usize, 4, i32, input_int4);

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
