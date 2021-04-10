/*!
Platform-dependent implementation
*/

#[cfg(feature = "sdl2")]
mod impl_ {
    use std::time::Duration;

    use grue2d::platform::{Lifecycle, PlatformLifetime};
    use rokol::gfx as rg;

    use crate::SnowRl;

    impl Lifecycle for SnowRl {
        type Event = sdl2::event::Event;

        fn event(&mut self, ev: Self::Event) {
            self.grue.event(&ev);
        }

        fn update(&mut self, dt: Duration, platform: &mut PlatformLifetime) {
            self.pre_update(dt, platform);
            self.grue.update(dt, platform);
        }

        fn render(&mut self, dt: Duration, platform: &mut PlatformLifetime) {
            self.grue.pre_render(dt, platform);
            self.render(dt, platform);
            self.grue.on_end_frame();
            rg::commit();
            platform.swap_window();
        }
    }
}

// /// Lifecycle forced by `rokol`
// impl RApp for SnowRl {
//     fn event(&mut self, ev: &Event) {
//         self.grue.event(ev);
//     }
//
//     /// Create our own lifecycle
//     fn frame(&mut self) {
//         self.pre_update();
//         self.grue.update();
//         self.render();
//         self.grue.on_end_frame();
//         rg::commit();
//     }
// }
