//! ImGUI inspector

mod crate_impls;
mod std_impls;

use imgui::Ui;

/// Derive ImGUI runtime inspector
pub trait Inspect {
    fn inspect(&mut self, ui: &Ui, label: &str);
}
