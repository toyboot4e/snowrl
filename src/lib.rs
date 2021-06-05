/*!
Integrate plugins
*/

pub extern crate grue2d;
pub extern crate plugins;

use std::{env, path::PathBuf};

use anyhow::*;

use grue2d::{
    app::Platform,
    hot_crate::{HotCrate, Symbol},
    GrueRl, Plugin,
};

type Load<'a> = Symbol<'a, unsafe extern "C" fn() -> Box<dyn Plugin>>;

/// Create a window and initialize the game
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

    let (platform, (mut data, ctrl, fsm)) = plugins::PluginA {}.init_game()?;

    let mut node = snow2d::ui::Node::from(snow2d::ui::node::Surface::None);
    node.z_order = 1.0;
    node.layer = grue2d::game::data::res::UiLayer::Screen.to_layer();

    let cfg = markup::RenderConfig {
        font_family: data.ice.snow.fontbook.families[data.res.fonts.default].clone(),
        fontsize: 22.0,
        nl_space: 4.0,
    };

    let text = markup::Renderer {
        fb: &mut data.ice.snow.fontbook,
        kbd_icons: &mut data.res.kbd_icons,
        pool: &mut data.res.ui.nodes,
        default_node: &node,
    }
    .run(
        &cfg,
        r#"Markup with :b[bold] text.

Keyboard key :kbd[x]!

    Third line of text!"#,
    )
    .unwrap();

    // TODO: positioning
    // let node = &mut data.res.ui.nodes[&text.root];
    // node.params.pos = [100.0, 100.0].into();

    let grue = GrueRl::new(&platform, data, ctrl, fsm)?;

    Ok((
        platform,
        SnowRl {
            grue,
            plugin,
            plugin_crate,
            text,
        },
    ))
}

use grue2d::markup::{self, TextHandle};

/// Run the game with plugins
pub struct SnowRl {
    pub grue: GrueRl,
    pub plugin: Box<dyn Plugin>,
    pub plugin_crate: HotCrate,
    pub text: TextHandle,
}

impl SnowRl {
    fn load_plugin(&mut self, platform: &mut Platform) {
        let load: Load = unsafe { self.plugin_crate.get(b"load") }.unwrap();
        let mut plugin = unsafe { load() };
        plugin.on_load(&mut self.grue, platform);

        self.plugin = plugin;
    }

    fn try_reload_plugin(&mut self, platform: &mut Platform) -> bool {
        // it turned out `sokol` can't be reloaded
        false
    }
}

#[cfg(feature = "sdl2")]
mod impl_ {
    use super::*;

    use sdl2::event::Event;
    use std::time::Duration;

    impl SnowRl {
        #[inline(always)]
        pub fn event(&mut self, ev: &Event, platform: &mut Platform) {
            self.plugin.event(&mut self.grue, ev, platform);
        }

        #[inline(always)]
        pub fn update(&mut self, dt: Duration, platform: &mut Platform) {
            self.try_reload_plugin(platform);
            self.plugin.update(&mut self.grue, dt, platform);
        }

        #[inline(always)]
        pub fn render(&mut self, dt: Duration, platform: &mut Platform) {
            self.plugin.render(&mut self.grue, dt, platform);
        }
    }
}
