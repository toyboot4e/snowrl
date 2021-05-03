//! ImGUI inspector

mod crate_impls;
mod std_impls;

use imgui::{im_str, Ui};

/// Derive ImGUI runtime inspector
pub trait Inspect {
    fn inspect(&mut self, ui: &Ui, label: &str);
}

pub fn nest(ui: &Ui, label: &str, closure: impl FnOnce()) {
    imgui::TreeNode::new(&imgui::im_str!("{}", label))
        .flags(imgui::TreeNodeFlags::OPEN_ON_ARROW | imgui::TreeNodeFlags::OPEN_ON_DOUBLE_CLICK)
        .build(ui, closure)
}

pub fn inspect_seq<'a, T: Inspect + 'static>(
    xs: impl Iterator<Item = &'a mut T>,
    ui: &Ui,
    label: &str,
) {
    self::nest(ui, label, || {
        for (i, x) in xs.enumerate() {
            x.inspect(ui, im_str!("{}", i).to_str());
        }
    })
}
