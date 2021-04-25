//! ImGUI inspector

use imgui::{im_str, Ui};

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

// impl Inspect for i32 {
//     fn inspect(&mut self, ui: &mut Ui, label: &str) {
//         let _changed = ui.input_int(&im_str!("{}", label), self).build();
//     }
// }
