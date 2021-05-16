/*!
Integrate plugins
*/

pub extern crate grue2d;

use std::{env, path::PathBuf};

use anyhow::*;

use grue2d::{
    app::Platform,
    hot_crate::{HotCrate, Symbol},
    GrueRl, Plugin,
};

type Load<'a> = Symbol<'a, unsafe extern "C" fn() -> Box<dyn Plugin>>;

pub fn init() -> Result<(Platform, SnowRl)> {
    let plugin_crate = {
        let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

        grue2d::hot_crate::HotCrate::load(
            root.join("Cargo.toml"),
            root.join("crates/plugins/Cargo.toml"),
        )
        .unwrap()
    };

    let load: Load = unsafe { plugin_crate.get(b"load").unwrap() };
    let plugin = unsafe { load() };

    let (platform, (data, ctrl, fsm)) = plugins::PluginA {}.init_game()?;
    let grue = GrueRl::new(&platform, data, ctrl, fsm)?;

    Ok((
        platform,
        SnowRl {
            grue,
            plugin,
            plugin_crate,
        },
    ))
}

/// Run the game with plugins
pub struct SnowRl {
    pub grue: GrueRl,
    pub plugin: Box<dyn Plugin>,
    pub plugin_crate: HotCrate,
}

impl SnowRl {
    fn load_plugin(&mut self, platform: &mut Platform) {
        let load: Load = unsafe { self.plugin_crate.get(b"load") }.unwrap();
        let mut plugin = unsafe { load() };
        plugin.on_load(&mut self.grue, platform);

        self.plugin = plugin;
    }

    fn try_reload_plugin(&mut self, platform: &mut Platform) -> bool {
        let reload = self.plugin_crate.try_reload().unwrap();

        if reload {
            self.load_plugin(platform);
        }

        reload
    }
}

#[cfg(feature = "sdl2")]
mod impl_ {
    use super::*;

    use sdl2::event::Event;
    use std::time::Duration;

    impl SnowRl {
        pub fn event(&mut self, ev: &Event, platform: &mut Platform) {
            self.plugin.event(&mut self.grue, ev, platform);
        }

        pub fn update(&mut self, dt: Duration, platform: &mut Platform) {
            self.try_reload_plugin(platform);
            self.plugin.update(&mut self.grue, dt, platform);
        }

        pub fn render(&mut self, dt: Duration, platform: &mut Platform) {
            self.plugin.render(&mut self.grue, dt, platform);
        }
    }
}
