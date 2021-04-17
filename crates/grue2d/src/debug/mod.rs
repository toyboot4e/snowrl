/*!
Debug-only module
*/

use anyhow::*;
use imgui_backends::{helper::QuickStart, platform::ImGuiSdl2, renderer::ImGuiRokolGfx};

/// ImGUI backend
pub type Backend = imgui_backends::Backend<ImGuiSdl2, ImGuiRokolGfx>;

use crate::Platform;

// FIXME: hard-coded value
const W: usize = 1280;
const H: usize = 720;

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
