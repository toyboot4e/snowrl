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
pub mod scenes;
pub mod states;

use {
    grue2d::{hot_crate, GlueRl},
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

        self.grue.gl.pre_update();
        self.grue.fsm.update(&mut self.grue.gl);
        self.grue.gl.post_update();

        self.grue.gl.ui.update(self.grue.gl.ice.dt);

        self.grue.gl.pre_render();
        self.grue.fsm.render(&mut self.grue.gl);

        self.grue.gl.on_end_frame();

        rg::commit();
    }
}
