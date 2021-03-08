/*!
Snow the roguelike game built on [`grue2d`]
*/

pub extern crate grue2d;

pub mod init;
pub mod utils;

pub mod play;
pub mod prelude;
pub mod scenes;
pub mod states;

use {
    grue2d::{hot_crate, render::WorldRenderFlag, GlueRl, UiLayer},
    rokol::{
        app::{Event, RApp},
        gfx as rg,
    },
    snow2d::utils::tweak::*,
};

fn sound_volume() -> f32 {
    tweak!(0.0)
}

/// The game
pub struct SnowRl {
    pub grue: GlueRl,
    pub plugin: hot_crate::HotLibrary,
    tmp: Vec<snow2d::utils::pool::Handle<snow2d::ui::node::Node>>,
}

impl SnowRl {
    pub fn new(grue: GlueRl) -> Self {
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
            tmp: vec![],
        }
    }
}

/// Lifecycle forced by `rokol`
impl RApp for SnowRl {
    fn event(&mut self, ev: &Event) {
        self.grue.gl.event(ev);
    }

    /// Create our own lifecycle
    fn frame(&mut self) {
        self.pre_update();
        self.update();
        self.render();
        self.grue.gl.on_end_frame();
        rg::commit();
    }
}

/// Our game lifecycle
impl SnowRl {
    #[inline]
    fn pre_update(&mut self) {
        // do not play sound in debug build
        #[cfg(debug_assertions)]
        self.grue.gl.ice.audio.set_global_volume(sound_volume());

        self.test_transform();

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

    // TODO: consider using systems for explicit control flow
    #[inline]
    fn update(&mut self) {
        self.grue.gl.pre_update();
        self.grue.fsm.update(&mut self.grue.gl);
        self.grue.gl.post_update();
    }

    #[inline]
    fn render(&mut self) {
        let gl = &mut self.grue.gl;
        gl.pre_render();
        let cam_mat = gl.world.cam.to_mat4();

        gl.world_render.render(
            &gl.world,
            &mut gl.ice,
            &mut gl.ui,
            WorldRenderFlag::SHADOW | WorldRenderFlag::ACTORS | WorldRenderFlag::MAP,
        );

        gl.ui.get_mut(UiLayer::Actors).render(&mut gl.ice, cam_mat);
        gl.ui
            .get_mut(UiLayer::OnActors)
            .render(&mut gl.ice, cam_mat);

        gl.world_render
            .render(&gl.world, &mut gl.ice, &mut gl.ui, WorldRenderFlag::SNOW);

        Self::test_text_style(gl);

        gl.ui.get_mut(UiLayer::Screen).render(&mut gl.ice, cam_mat);
    }

    fn test_text_style(gl: &mut grue2d::Global) {
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

        let (font_set_handle, _) = gl.ice.snow.fontbook.storage.get_by_slot(0).unwrap();
        snow2d::gfx::text::render_line(
            &layout.lines[0],
            text,
            Vec2f::new(100.0, 100.0),
            &mut gl.ice.snow,
            font_set_handle,
        );
    }

    fn test_transform(&mut self) {
        if !self.tmp.is_empty() {
            return;
        }
        use crate::prelude::*;

        let gl = &mut self.grue.gl;
        let layer = gl.ui.get_mut(grue2d::UiLayer::Screen);

        let tex: Asset<Texture2dDrop> = gl.ice.assets.load_sync(paths::img::pochi::WHAT).unwrap();
        let sprite = SpriteData::builder(tex)
            .uv_rect([0.0, 0.0, 1.0 / 6.0, 1.0 / 4.0])
            .origin([0.5, 0.5])
            .build();

        let mut center: Node = sprite.clone().into();
        center.params.pos = [640.0, 320.0].into();
        let parent = layer.nodes.add(center);

        let mut child: Node = sprite.clone().into();
        child.params.pos = [100.0, 100.0].into();
        let child = layer.nodes.attach_child(&parent, child);

        self.tmp.push(parent);
        self.tmp.push(child);
    }
}
