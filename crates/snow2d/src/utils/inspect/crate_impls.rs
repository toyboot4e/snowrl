use imgui::{im_str, Ui};

use crate::{
    asset::Asset,
    gfx::tex::*,
    input::Dir8,
    utils::arena::{Arena, Index},
};

use super::Inspect;

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

impl Inspect for Asset<Texture2dDrop> {
    fn inspect(&mut self, ui: &Ui, _label: &str) {
        ui.text("TODO: Asset<T>");
    }
}
