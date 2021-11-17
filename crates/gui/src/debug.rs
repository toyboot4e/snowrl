use anyhow::*;

use imgui_backends::{imgui, helper::QuickStart, platform::ImGuiSdl2, renderer::ImGuiRokolGfx};

use snow2d::utils::Inspect;

use crate::{Gui, window::Platform};

pub type Backend = imgui_backends::Backend<ImGuiSdl2, ImGuiRokolGfx>;
pub type BackendUi<'a> = imgui_backends::BackendUi<'a, ImGuiSdl2, ImGuiRokolGfx>;

#[derive(Debug, Clone, Default)]
pub struct DebugState {
    //
}

pub fn create_backend(display_size: [f32; 2], platform: &Platform) -> Result<Backend> {
    let mut imgui = QuickStart {
        display_size,
        fontsize: 13.0,
        hidpi_factor: 1.0,
    }
    .create_context();

    let platform = ImGuiSdl2::new(&mut imgui, &platform.win);
    let renderer = ImGuiRokolGfx::new(&mut imgui)?;

    Ok(Backend {
        imgui,
        platform,
        renderer,
    })
}

impl DebugState {
    pub fn render(&mut self, ui: &mut BackendUi, gui: &mut Gui) {
        ui.show_demo_window(&mut true);

        imgui::Window::new("Runtime inspector")
            .size([200.0, 400.0], imgui::Condition::FirstUseEver)
            // semi-transparent
            .bg_alpha(0.5)
            .build(ui, || {
                ui.label_text("XXX", "YYY");
                gui.vm.entities.inspect(ui, "view model");
            });
    }
}
