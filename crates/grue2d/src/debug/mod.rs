/*!
Debug-only module
*/

// TODO: rokol ImGUI renderer

// TODO: show game in ImGUI window
// use a stack of `RenderTexture`s which can override backbuffer size.

/// ImGUI backend
pub type Backend = imgui_backends::Backend<ImGuiSdl2, ImGuiRokolGfx>;
pub type BackendUi<'a> = imgui_backends::BackendUi<'a, ImGuiSdl2, ImGuiRokolGfx>;

use anyhow::*;
use imgui::{self as ig, im_str};
use imgui_backends::{helper::QuickStart, platform::ImGuiSdl2, renderer::ImGuiRokolGfx};

use snow2d::utils::Inspect;

use crate::{
    app::Platform,
    game::{Control, Data},
};

// FIXME: hard-coded value
const W: usize = 1280;
const H: usize = 720;

#[derive(Debug, Clone)]
pub struct DebugState {
    //
}

impl Default for DebugState {
    fn default() -> Self {
        Self {}
    }
}

pub fn create_backend(platform: &Platform) -> Result<Backend> {
    let mut imgui = QuickStart {
        display_size: [W as f32, H as f32],
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
    pub fn debug_render(&mut self, data: &mut Data, _ctrl: &mut Control, ui: &mut BackendUi) {
        // ui.show_demo_window(&mut true);

        ig::Window::new(im_str!("Test"))
            .size([400.0, 100.], ig::Condition::FirstUseEver)
            // semi-transparent
            .bg_alpha(0.5)
            .build(ui, || {
                let cfg = data.cfg.clone();
                data.cfg.inspect(ui, "configuration");
                if data.cfg.vol != cfg.vol {
                    data.ice.audio.set_global_volume(data.cfg.vol);
                }

                data.world.inspect(ui, "world");
            });

        // self::show_anim_queue(data, ctrl, ui);
    }

    // fn show_anim_queue(data: &mut Data, ctrl: &mut Control, ui: &mut BackendUi) {
    //     let anims = ctrl.rogue.anims.anims();
    // }
}
