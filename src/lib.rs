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
    grue2d::{
        agents::WorldRenderer,
        data::{res::UiLayer, Data},
        hot_crate,
        PlatformLifetime,
        GrueRl,
    },
    rokol::gfx as rg,
    snow2d::gfx::Color,
    std::time::Duration,
};

/// The game
///
/// See `platform_impl.rs` for the internal game loop.
pub struct SnowRl {
    pub grue: GrueRl,
    pub plugin: hot_crate::HotLibrary,
    pa_blue: rg::PassAction,
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

        Self {
            grue,
            plugin,
            pa_blue: rg::PassAction::clear(Color::CORNFLOWER_BLUE.to_normalized_array()),
        }
    }
}

/// Our game lifecycle
impl SnowRl {
    #[inline]
    fn pre_update(&mut self, _dt: Duration, _platform: &mut PlatformLifetime) {
        //
    }

    #[inline]
    fn render(&mut self, _dt: Duration, _platform: &mut PlatformLifetime) {
        let (data, agents) = (&mut self.grue.data, &mut self.grue.agents);
        let cam_mat = data.world.cam.to_mat4();

        {
            let (ice, res, world) = (&mut data.ice, &mut data.res, &mut data.world);
            let dt = ice.dt();

            {
                let mut screen = ice
                    .snow
                    .screen()
                    .pa(Some(&self.pa_blue))
                    .transform(Some(world.cam.to_mat4()))
                    .build();
                WorldRenderer::render_map(&mut screen, world, 0..100);
            }

            agents
                .world_render
                .setup_actor_nodes(world, &mut res.ui, dt);
            res.ui.layer_mut(UiLayer::Actors).render(ice, cam_mat);

            {
                let mut screen = ice
                    .snow
                    .screen()
                    .pa(None)
                    .transform(Some(world.cam.to_mat4()))
                    .build();
                WorldRenderer::render_map(&mut screen, world, 100..);
            }

            agents.world_render.render_shadow(&mut ice.snow, world);
            res.ui.layer_mut(UiLayer::OnShadow).render(ice, cam_mat);
            agents.world_render.render_snow(&ice.snow.window);
        }

        data.res
            .ui
            .layer_mut(UiLayer::Screen)
            .render(&mut data.ice, cam_mat);
    }
}

#[cfg(feature = "sdl2")]
mod impl_ {
    use std::time::Duration;

    use grue2d::{Lifecycle, PlatformLifetime};
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
