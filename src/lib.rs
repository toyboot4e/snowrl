/*!
Snow the roguelike game that extends [`grue2d`] framework
*/

pub extern crate grue2d;

pub mod init;
pub mod utils;

pub mod play;
pub mod prelude;
pub mod scenes;
pub mod states;

use {
    grue2d::{app::Platform, hot_crate, GrueRl},
    std::time::Duration,
};

/// The game
pub struct SnowRl {
    pub grue: GrueRl,
    pub plugin: hot_crate::HotLibrary,
}

impl SnowRl {
    pub fn new(grue: GrueRl) -> Self {
        let plugin = {
            let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
            grue2d::hot_crate::HotLibrary::load(
                root.join("Cargo.toml"),
                root.join("crates/plugins/Cargo.toml"),
            )
            .unwrap()
        };

        Self { grue, plugin }
    }
}

/// Our game lifecycle
impl SnowRl {
    #[inline]
    fn pre_update(&mut self, _dt: Duration, _platform: &mut Platform) {
        //
    }

    #[inline]
    fn render(&mut self, _dt: Duration, _platform: &mut Platform) {
        self.grue.render_default();
    }
}

#[cfg(feature = "sdl2")]
mod impl_ {
    use std::time::Duration;

    use grue2d::app::{Lifecycle, Platform};
    use rokol::gfx as rg;

    use crate::SnowRl;

    impl Lifecycle for SnowRl {
        type Event = sdl2::event::Event;

        fn event(&mut self, ev: Self::Event, platform: &mut Platform) {
            self.grue.event(&ev, platform);
        }

        fn update(&mut self, dt: Duration, platform: &mut Platform) {
            self.pre_update(dt, platform);
            self.grue.update(dt, platform);
        }

        fn render(&mut self, dt: Duration, platform: &mut Platform) {
            self.grue.pre_render(dt, platform);
            self.render(dt, platform);
            self.grue.post_render(dt, platform);
            self.grue.on_end_frame();
            rg::commit();
            platform.swap_window();
        }
    }
}
