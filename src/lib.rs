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

use grue2d::{app::Platform, hot_crate, GrueRl};
use std::{env, path::PathBuf};

/// The game
pub struct SnowRl {
    pub grue: GrueRl,
    pub plugin: hot_crate::HotLibrary,
}

impl SnowRl {
    pub fn new(grue: GrueRl) -> Self {
        let plugin = {
            let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
            grue2d::hot_crate::HotLibrary::load(
                root.join("Cargo.toml"),
                root.join("crates/plugins/Cargo.toml"),
            )
            .unwrap()
        };

        Self { grue, plugin }
    }
}

#[cfg(feature = "sdl2")]
mod impl_ {
    use super::*;

    use rokol::gfx as rg;
    use sdl2::event::Event;
    use std::time::Duration;

    impl SnowRl {
        pub fn event(&mut self, ev: &Event, platform: &mut Platform) {
            self.grue.event(ev, platform);
        }

        pub fn update(&mut self, dt: Duration, platform: &mut Platform) {
            self.grue.update(dt, platform);
        }

        pub fn render(&mut self, dt: Duration, platform: &mut Platform) {
            self.grue.pre_render(dt, platform);
            self.grue.render_default();
            self.grue.post_render(dt, platform);
            self.grue.on_end_frame();

            rg::commit();
            platform.swap_window();
        }
    }
}
