//! ImGUI inspector

mod std_impls;
mod crate_impls;

use imgui::{im_str, Ui};

/// Derive ImGUI runtime inspector
pub trait Inspect {
    fn inspect(&mut self, ui: &Ui, label: &str);
}

