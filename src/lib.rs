/*!
Snow the roguelike game

2D framework:

| Crate      | Description                             |
|------------|-----------------------------------------|
| [`rokol`]  | Window and lower-level graphics         |
| [`snow2d`] | 2D rendering and asset management       |

SnowRL framework:

| Crate      | Description                             |
|------------|-----------------------------------------|
| [`rlbox`]  | Toolkit to power 2D GUI roguelike games |
| [`grue2d`] | Game states for SnowRL                  |

And `snowrl` is a set of plugins to [`grue2d`].
*/

// use generator (unstable Rust)
#![feature(generators, generator_trait)]

pub mod utils;

pub mod play;
pub mod prelude;
pub mod scenes;
pub mod states;

use {
    grue2d::{hot_crate, render::WorldRenderFlag, GlueRl},
    rlbox::utils::tweak::*,
    rokol::{
        app::{Event, RApp},
        gfx as rg,
    },
};

pub struct SnowRl {
    pub grue: GlueRl,
    pub plugin: hot_crate::HotLibrary,
}

fn sound_volume() -> f32 {
    tweak!(0.0)
}

// Lifecycle
impl RApp for SnowRl {
    fn event(&mut self, ev: &Event) {
        self.grue.gl.event(ev);
    }

    fn frame(&mut self) {
        self.pre_update();
        self.update();
        self.render();
        self.grue.gl.on_end_frame();
        rg::commit();
    }
}

impl SnowRl {
    #[inline]
    fn pre_update(&mut self) {
        #[cfg(debug_assertions)]
        self.grue.gl.ice.audio.set_global_volume(sound_volume());

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
    fn update(&mut self) {
        self.grue.gl.pre_update();
        self.grue.fsm.update(&mut self.grue.gl);

        self.grue.gl.post_update();
    }

    #[inline]
    fn render(&mut self) {
        let gl = &mut self.grue.gl;

        gl.pre_render();
        gl.world_render
            .render(&gl.world, &mut gl.ice, WorldRenderFlag::ALL);

        let cam_mat = gl.world.cam.to_mat4();
        for (_ix, layer) in &mut gl.ui.layers {
            layer.render(&mut gl.ice, cam_mat);
        }
    }
}
