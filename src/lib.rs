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

mod platform_impl;

use {
    grue2d::{
        agents::WorldRenderer,
        data::{resources::UiLayer, Data},
        hot_crate,
        platform::PlatformLifetime,
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
    tmp: Vec<snow2d::utils::pool::Handle<snow2d::ui::node::Node>>,
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
            tmp: vec![],
        }
    }
}

/// Our game lifecycle
impl SnowRl {
    #[inline]
    fn pre_update(&mut self, _dt: Duration, _platform: &mut PlatformLifetime) {
        // self.test_transform();

        // // TODO: handle plugins properly
        // if self.grue.gl.ice.frame_count % 120 == 0 {
        //     use {grue2d::Plugin, hot_crate::libloading::Symbol};

        //     self.plugin.reload().unwrap();

        //     let load: Symbol<extern "C" fn() -> Box<dyn Plugin>> =
        //         unsafe { self.plugin.get(b"load") }.unwrap();
        //     println!("current plugin: {:?}", load());
        //     // plugin.close().unwrap();
        // }
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

        // Self::test_text_style(data);

        data.res
            .ui
            .layer_mut(UiLayer::Screen)
            .render(&mut data.ice, cam_mat);
    }

    fn test_text_style(data: &mut Data) {
        use snow2d::gfx::{geom2d::*, text::prelude::*};
        let text = "snow2d graphics!";

        let layout = TextView {
            text: &text,
            lines: vec![LineView {
                line_spans: vec![
                    LineSpanView {
                        text_slice: &text[0..6],
                        first_quad_ix: 0,
                        quad_span: QuadSpan { from: 0, to: 6 },
                        style: TextStyle {
                            color: [128, 128, 128, 255],
                            is_bold: false,
                        },
                    },
                    LineSpanView {
                        text_slice: &text[7..16],
                        first_quad_ix: 0,
                        quad_span: QuadSpan { from: 7, to: 16 },
                        style: TextStyle {
                            color: [255, 0, 0, 255],
                            is_bold: true,
                        },
                    },
                ],
            }],
        };

        let (font_set_handle, _) = data.ice.snow.fontbook.storage.get_by_slot(0).unwrap();
        snow2d::gfx::text::render_line(
            &layout.lines[0],
            text,
            Vec2f::new(100.0, 100.0),
            &mut data.ice.snow,
            font_set_handle,
        );
    }

    fn test_transform(&mut self) {
        if !self.tmp.is_empty() {
            return;
        }
        use crate::prelude::*;

        let data = &mut self.grue.data;
        let res = &mut data.res;
        let ice = &mut data.ice;

        let layer = res.ui.layer_mut(UiLayer::Screen);

        let tex: Asset<Texture2dDrop> = ice.assets.load_sync(paths::img::pochi::WHAT).unwrap();
        let sprite = SpriteData::builder(tex)
            .uv_rect([0.0, 0.0, 1.0 / 6.0, 1.0 / 4.0])
            .origin([0.5, 0.5])
            .build();

        let mut center: Node = sprite.clone().into();
        center.params.pos = [640.0, 320.0].into();
        let parent = layer.nodes.add(center);

        let mut b = layer.anims.builder();

        // TODO: repeat
        b.dt(ez::EasedDt::new(1.0, ez::Ease::SinIn));
        let child1: Node = sprite.clone().into();
        let child1 = layer.nodes.attach_child(&parent, child1);
        b.node(&child1);
        b.ease(ez::Ease::Linear).rot([0.0, 6.28]);
        b.ease(ez::Ease::SinIn).x([-200.0, 0.0]);
        b.ease(ez::Ease::SinOut).y([0.0, 200.0]);

        b.dt(ez::EasedDt::new(2.0, ez::Ease::SinIn));
        let child2: Node = sprite.clone().into();
        let child2 = layer.nodes.attach_child(&child1, child2);
        b.node(&child2);
        b.ease(ez::Ease::Linear).rot([0.0, 6.28]);
        b.ease(ez::Ease::SinIn).x([0.0, 100.0]);
        b.ease(ez::Ease::SinOut).y([-100.0, 0.0]);

        b.dt(ez::EasedDt::new(3.0, ez::Ease::SinIn));
        let child3: Node = sprite.clone().into();
        let child3 = layer.nodes.attach_child(&child2, child3);
        b.node(&child3);
        // FIXME: not rotating
        // b.ease(ez::Ease::SinIn).x([0.0, 50.0]).rot([0.0, -6.28]);
        b.ease(ez::Ease::Linear).rot([6.28, 0.0]);
        b.ease(ez::Ease::SinIn).x([-30.0, 0.0]);
        b.ease(ez::Ease::SinOut).y([0.0, -30.0]);

        self.tmp.push(parent);
        self.tmp.push(child1);
        self.tmp.push(child2);
        self.tmp.push(child3); // FIXME: death
    }
}
