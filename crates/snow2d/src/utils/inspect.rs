//! ImGUI inspector

use std::{collections::HashMap, num::NonZeroU32};

use imgui::{im_str, Ui};

use crate::{
    input::Dir8,
    utils::arena::{Arena, Index},
};

/// Derive ImGUI runtime inspector
pub trait Inspect {
    fn inspect(&mut self, ui: &Ui, label: &str);
}

impl Inspect for bool {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        // let color = [1.0, 1.0, 0.0, 1.0];
        // let style = ui.push_style_color(imgui::StyleColor::Text, color);

        let _changed = ui.checkbox(&im_str!("{}", label), self);

        // style.pop(ui);
    }
}

impl Inspect for String {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        ui.label_text(&im_str!("{}", label), &im_str!("{}", self));
    }
}

impl Inspect for f32 {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        let _changed = ui.input_float(&im_str!("{}", label), self).build();
    }
}

impl Inspect for [f32; 2] {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        let _changed = ui.input_float2(&im_str!("{}", label), self).build();
    }
}

impl Inspect for [f32; 3] {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        let _changed = ui.input_float3(&im_str!("{}", label), self).build();
    }
}

impl Inspect for [f32; 4] {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        let _changed = ui.input_float4(&im_str!("{}", label), self).build();
    }
}

impl Inspect for i32 {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        let _changed = ui.input_int(&im_str!("{}", label), self).build();
    }
}

impl Inspect for [i32; 2] {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        let _changed = ui.input_int2(&im_str!("{}", label), self).build();
    }
}

impl Inspect for [i32; 3] {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        let _changed = ui.input_int3(&im_str!("{}", label), self).build();
    }
}

impl Inspect for [i32; 4] {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        let _changed = ui.input_int4(&im_str!("{}", label), self).build();
    }
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

// generics

impl<T> Inspect for std::sync::mpsc::Sender<T> {
    fn inspect(&mut self, _ui: &Ui, _label: &str) {}
}

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

impl<K: Inspect, V: Inspect> Inspect for HashMap<K, V> {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        ui.text(&im_str!("TODO: HashMap"));
    }
}

// external crate items

impl Inspect for Dir8 {
    fn inspect(&mut self, ui: &Ui, _label: &str) {
        ui.text("TODO: Dir8");
    }
}

impl<T: Inspect> Inspect for Arena<T> {
    fn inspect(&mut self, ui: &Ui, label: &str) {
        imgui::TreeNode::new(&imgui::im_str!("{}", label))
            .flags(imgui::TreeNodeFlags::OPEN_ON_ARROW | imgui::TreeNodeFlags::OPEN_ON_DOUBLE_CLICK)
            .build(ui, || {
                for i in 0..self.capacity() {
                    if let Some((_index, item)) = self.get_by_slot_mut(i as u32) {
                        item.inspect(ui, im_str!("{}", i).to_str());
                    }
                }
            });
    }
}

impl<T> Inspect for Index<T> {
    fn inspect(&mut self, ui: &Ui, _label: &str) {
        ui.text("TODO: Index<T>");
    }
}

use crate::{asset::Asset, gfx::tex::*};

impl Inspect for Asset<Texture2dDrop> {
    fn inspect(&mut self, ui: &Ui, _label: &str) {
        ui.text("TODO: Asset<T>");
    }
}
