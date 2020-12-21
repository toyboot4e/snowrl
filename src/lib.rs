//! SnowRL

pub use {rlbox, rokol, snow2d};

pub mod render;
pub mod world;

use {
    rokol::{app as ra, gfx as rg},
    std::path::PathBuf,
};

use self::world::{World, WorldContext};

pub fn run(app: rokol::Rokol) -> rokol::Result {
    app.run(&mut SnowRl::new())
}

#[derive(Debug)]
pub struct SnowRl {
    // use `Option` for lazily initialization
    wcx: Option<WorldContext>,
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

        wcx.event(ev);
        world.event(wcx, ev);
    }

    fn frame(&mut self) {
        let wcx = self.wcx.as_mut().unwrap();
        let world = self.world.as_mut().unwrap();

        wcx.update();
        world.update(wcx);

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
}
