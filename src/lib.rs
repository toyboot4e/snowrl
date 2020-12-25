//! SnowRL

// use generator (unstable Rust)
#![feature(generators, generator_trait)]

pub use {rlbox, rokol, snow2d};

pub mod render;
pub mod utils;
pub mod world;

use {
    rokol::{app as ra, gfx as rg},
    std::path::PathBuf,
};

use crate::world::{World, WorldContext};

pub fn run(app: rokol::Rokol) -> rokol::Result {
    app.run(&mut SnowRl::new())
}

// #[derive(Debug)]
pub struct SnowRl {
    /// Use `Option` for lazy initialization
    wcx: Option<WorldContext>,
    /// Use `Option` for lazy initialization
    world: Option<World>,
}

impl SnowRl {
    pub fn new() -> Self {
        Self {
            wcx: None,
            world: None,
        }
    }
}

impl rokol::app::RApp for SnowRl {
    fn init(&mut self) {
        rg::setup(&mut rokol::glue::app_desc());

        let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets");
        let file = root.join("map/tmx/rl_start.tmx");

        self.wcx = Some(WorldContext::new());
        self.world = Some(World::from_tiled_file(self.wcx.as_mut().unwrap(), &file).unwrap());
    }

    fn event(&mut self, ev: &ra::Event) {
        let wcx = self.wcx.as_mut().unwrap();
        let world = self.world.as_mut().unwrap();

        // print event type
        let type_ = rokol::app::EventType::from_u32(ev.type_).unwrap();
        use rokol::app::EventType as Ev;
        if !matches!(type_, Ev::MouseMove | Ev::MouseEnter | Ev::MouseLeave) {
            let key = rokol::app::Key::from_u32(ev.key_code).unwrap();
            println!("{:?}, {:?}", type_, key);
        }

        wcx.event(ev);
        world.event(wcx, ev);
    }

    fn frame(&mut self) {
        let wcx = self.wcx.as_mut().unwrap();
        let world = self.world.as_mut().unwrap();

        wcx.update();
        world.update_scene(wcx);
        world.update_images(wcx);

        wcx.render();
        world.render(wcx);

        wcx.on_end_frame();
        world.on_end_frame(wcx);

        rg::commit();
    }
}

/// Collects magic values (yes, it shuld be removed at some time)
pub mod consts {
    pub const ACTOR_FPS: f32 = 4.0;
    pub const FOV_R: u32 = 5;
    pub const WALK_TIME: f32 = 8.0 / 60.0;
}
