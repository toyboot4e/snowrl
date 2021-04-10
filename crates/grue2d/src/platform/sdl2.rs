//! Rust-SDL2 support

use sdl2::event::Event;

use super::PlatformLifetime;
use crate::GrueRl;

/// Lifecycle methods
impl GrueRl {
    pub fn event(&mut self, ev: &Event) {
        self.data.ice.event(ev);
    }

    pub fn update(&mut self, dt: std::time::Duration, _platform: &mut PlatformLifetime) {
        // TODO: set dt
        self.pre_update(dt);
        self.fsm.update(&mut self.data, &mut self.ctrl);
        self.post_update();
    }

    pub fn pre_render(&mut self, _dt: std::time::Duration, platform: &mut PlatformLifetime) {
        let size = platform.win.size();

        self.data.ice.pre_render(snow2d::gfx::WindowState {
            w: size.0,
            h: size.1,
            // FIXME: never hard code this value
            // dpi_scale: [2.0, 2.0],
            dpi_scale: [1.0, 1.0],
        });
    }

    pub fn on_end_frame(&mut self) {
        self.data.ice.on_end_frame();
    }
}
