/*!
SnowRL the roguelike game
*/

#![feature(const_raw_ptr_deref)]

pub extern crate gui;
pub extern crate rokol;
pub extern crate snow2d;

pub mod consts;
pub mod paths;
pub mod states;

mod init;
pub use init::init;

use snow2d::utils::arena::Slot;

use gui::prelude::*;

/// SnowRL the game
#[derive(Debug)]
pub struct SnowRl {
    pub god: God,
    pub fsm: Fsm<God>,
    pub w_rdr: WorldRenderer,
}

#[cfg(feature = "sdl2")]
mod impl_ {
    use super::*;

    use sdl2::event::Event;
    use std::time::Duration;

    impl SnowRl {
        #[inline(always)]
        pub fn event(&mut self, ev: &Event, _platform: &mut Platform) {
            self.god.ice.event(ev);
        }

        #[inline(always)]
        pub fn update(&mut self, dt: Duration, platform: &mut Platform) {
            self.pre_update(dt, platform);
            self.fsm.update(&mut self.god);
            self.post_update(dt, platform);
        }

        pub const DEFAULT_RENDER_SCHEDULE: &'static [DrawStage] = &[
            DrawStage::MapDown,
            DrawStage::UiLayer(UiLayer::Actors),
            DrawStage::UiLayer(UiLayer::OnActors),
            DrawStage::MapUp,
            DrawStage::Shadow,
            DrawStage::UiLayer(UiLayer::OnShadow),
            DrawStage::Snow,
            DrawStage::UiLayer(UiLayer::Screen),
        ];

        #[inline(always)]
        pub fn render(&mut self, dt: Duration, platform: &mut Platform) {
            // FIXME:
            let window = self.god.ice.gfx.window.clone();
            self.god.ice.pre_render(window);
            DrawStage::draw_schedule(Self::DEFAULT_RENDER_SCHEDULE, self);
            self.god.ice.post_render(dt);

            #[cfg(debug_assertions)]
            self.god.debug_render(&mut platform.win);

            self.god.ice.on_end_frame();
            rg::commit();
            platform.swap_window();
        }

        #[inline(always)]
        fn pre_update(&mut self, dt: Duration, _platform: &mut Platform) {
            self.god.ice.pre_update(dt);
            self.god.gui.update(&mut self.god.ice);
            self.god.res.vi.update(&self.god.ice.input, dt);
        }

        #[inline(always)]
        fn post_update(&mut self, dt: Duration, _platform: &mut Platform) {
            // shadow
            // FIXME: don't hard code player detection
            let player_view_index = self.god.gui.actors.upgrade(Slot::from_raw(0)).unwrap();
            let player_view = &self.god.gui.actors.get(player_view_index).unwrap();
            let player_mdl = &self.god.gui.vm.ents[player_view.mdl];

            self.god
                .gui
                .shadow
                .post_update(dt, &self.god.gui.vm.map, player_mdl.pos);

            // camera
            let player_pos = player_view.img.pos_world_centered(&self.god.gui.map.tiled);
            self.god.gui.cam_follow.update_follow(
                &mut self.god.gui.cam,
                player_pos,
                Vec2f::from(self.god.ice.gfx.window.size_f32()),
            );

            // sprites
            self.god.res.ui.update(dt);
            self.god.res.dir_anims.update(dt, &mut self.god.res.ui);

            // renderer
            self.w_rdr.post_update(&self.god.gui.vm, dt);
        }

        // #[inline(always)]
        // fn debug_render(&mut self, dt: Duration, platform: &mut Platform) {
        //     //
        // }
    }
}

/// Drawing stage for rendering schedule
#[derive(Debug, Clone, Copy)]
pub enum DrawStage {
    /// Specific UI (sprite) layer
    UiLayer(UiLayer),
    /// Down parts of the map
    MapDown,
    /// Up parts of the map
    MapUp,
    /// FoV/FoW
    Shadow,
    /// FoV/The snow shadery
    Snow,
    /// Clear screen with cornflower blue
    ClearScreen,
}

impl DrawStage {
    pub fn draw_schedule(schedule: &[Self], rl: &mut SnowRl) {
        for stage in schedule {
            stage.draw(rl);
        }
    }

    const PA_BLUE: rg::PassAction =
        rg::PassAction::clear_const([100.0 / 255.0, 149.0 / 255.0, 237.0 / 255.0, 250.0 / 255.0]);

    pub fn draw(self, app: &mut SnowRl) {
        let (data, world_render) = (&mut app.god, &mut app.w_rdr);
        let cam_mat = data.gui.cam.to_mat4();

        let (ice, res, world, cfg) = (&mut data.ice, &mut data.res, &mut data.gui, &data.cfg);
        let dt = ice.dt();

        match self {
            DrawStage::UiLayer(ui_layer) => {
                if ui_layer == UiLayer::Actors {
                    // FIXME: we're assuming `OnActors` is drawn actor `Actors`
                    world_render.setup_actor_nodes(world, &mut res.ui, dt);
                }

                let mut screen = ice
                    .gfx
                    .screen()
                    .transform(match ui_layer.to_layer().coord {
                        CoordSystem::Screen => None,
                        CoordSystem::World => Some(cam_mat),
                    })
                    .build();

                res.ui.render_range(ui_layer.to_draw_range(), &mut screen);
            }
            DrawStage::MapDown => {
                let mut screen = ice
                    .gfx
                    .screen()
                    .pa(Some(&rg::PassAction::LOAD))
                    .transform(Some(cam_mat))
                    .build();
                WorldRenderer::render_map(&mut screen, world, 0..100);
            }
            DrawStage::MapUp => {
                let mut screen = ice
                    .gfx
                    .screen()
                    .pa(Some(&Self::PA_BLUE))
                    .transform(Some(cam_mat))
                    .build();
                WorldRenderer::render_map(&mut screen, world, 100..);
            }
            DrawStage::Shadow => {
                world_render.render_shadow(&mut ice.gfx, world, &cfg.shadow_cfg);
            }
            DrawStage::Snow => {
                world_render.render_snow(&ice.gfx.window, &ice.gfx.clock, &cfg.snow_cfg);
            }
            DrawStage::ClearScreen => {
                // TODO: is this inefficient
                let _screen = ice.gfx.screen().pa(Some(&Self::PA_BLUE)).build();
            }
        }
    }
}
