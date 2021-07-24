/*!
Integrate plugins
*/

pub extern crate grue2d;
pub extern crate plugins;
pub extern crate rokol;
pub extern crate snow2d;

use anyhow::*;

use grue2d::{app::Platform, GrueRl, Plugin};

/// Create a window and initialize the game
pub fn init() -> Result<(Platform, SnowRl)> {
    let plugin = plugins::load();
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
        [100.0, 100.0],
        r#"Markup with :b[bold] text.
Keyboard key :kbd[x]!

    Third line of text!"#,
    )
    .unwrap();

    // TODO: scaled layout + scaled text
    // data.res.ui.nodes[&text.root].params.scale = [2.0, 2.0].into();

    let grue = GrueRl::new(&platform, data, ctrl, fsm)?;

    Ok((platform, SnowRl { grue, plugin, text }))
}

use grue2d::markup::{self, TextHandle};

/// Run the game with plugins
pub struct SnowRl {
    pub grue: GrueRl,
    pub plugin: plugins::PluginA,
    pub text: TextHandle,
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
            self.plugin.update(&mut self.grue, dt, platform);
        }

        #[inline(always)]
        pub fn render(&mut self, dt: Duration, platform: &mut Platform) {
            self.plugin.render(&mut self.grue, dt, platform);
        }
    }
}
