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
        self.world = Some(World::from_tiled_file(&file).unwrap());
    }

    fn frame(&mut self) {
        let wcx = self.wcx.as_mut().unwrap();
        wcx.update();
        let world = self.world.as_mut().unwrap();
        world.update(wcx);
        world.render(wcx);
        rg::commit();
    }

    fn event(&mut self, _ev: &ra::RAppEvent) {
        // println!("{:?}", ev);
    }
}
