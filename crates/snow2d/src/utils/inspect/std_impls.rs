use std::{collections::HashMap, num::NonZeroU32};

use arraytools::ArrayTools;
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

macro_rules! impl_array_as {
    ($ty:ty, $as:ty, $method:ident) => {
        #[allow(warnings)]
        impl Inspect for $ty {
            fn inspect(&mut self, ui: &Ui, label: &str) {
                let mut xs = self.map(|x| x as $as);
                let _changed = ui.$method(&im_str!("{}", label), &mut xs).build();
                *self = xs;
            }
        }
    };
}

im_self!(bool, checkbox);

im_im!(str, label_text);
im_im!(String, label_text);

im_self!(f32, input_float);
impl_array_as!([f32; 2], f32, input_float2);
impl_array_as!([f32; 3], f32, input_float3);
impl_array_as!([f32; 4], f32, input_float4);

// TODO: support [f32; 32]

// im_self!(f64, input_float);

im_self!(i32, input_int);
impl_array_as!([i32; 2], i32, input_int2);
impl_array_as!([i32; 3], i32, input_int3);
impl_array_as!([i32; 4], i32, input_int4);

// im_self!(i64, label_int);

impl<T> Inspect for [T; 0] {
    fn inspect(&mut self, _ui: &Ui, _label: &str) {}
}

// TODO:
impl Inspect for usize {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        let mut i = *self as i32;
        if ui.input_int(&im_str!("{}", label), &mut i).build() {
            *self = i as usize;
        }
    }
}

impl Inspect for u32 {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        let mut i = *self as i32;
        if ui.input_int(&im_str!("{}", label), &mut i).build() {
            *self = i as u32;
        }
    }
}

impl Inspect for [u32; 2] {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        let mut i = [self[0] as i32, self[1] as i32];
        if ui.input_int2(&im_str!("{}", label), &mut i).build() {
            self[0] = i[0] as u32;
            self[1] = i[1] as u32;
        }
    }
}

impl Inspect for [u32; 3] {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        let mut i = [self[0] as i32, self[1] as i32, self[2] as i32];
        if ui.input_int3(&im_str!("{}", label), &mut i).build() {
            self[0] = i[0] as u32;
            self[1] = i[1] as u32;
            self[2] = i[2] as u32;
        }
    }
}

impl Inspect for [u32; 4] {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        let mut i = [
            self[0] as i32,
            self[1] as i32,
            self[2] as i32,
            self[3] as i32,
        ];
        if ui.input_int4(&im_str!("{}", label), &mut i).build() {
            self[0] = i[0] as u32;
            self[1] = i[1] as u32;
            self[2] = i[2] as u32;
            self[3] = i[3] as u32;
        }
    }
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
