//! SnowRL

// use generator (unstable Rust)
#![feature(generators, generator_trait)]

pub use {rlbox, rokol, snow2d};

pub mod ev;
pub mod render;
pub mod utils;
pub mod world;

use {
    rokol::{app as ra, gfx as rg},
    std::path::PathBuf,
};

use crate::world::{
    turn::{GameLoop, GameLoopImpl},
    World, WorldContext,
};

pub fn run(app: rokol::Rokol) -> rokol::Result {
    app.run(&mut SnowRl::new())
}

#[derive(Debug)]
enum GameState {
    Tick,
    Anim,
    Player,
}

pub struct SnowRl {
    /// Use `Option` for lazy initialization
    x: Option<SnowRlImpl>,
}

impl SnowRl {
    pub fn new() -> Self {
        Self { x: None }
    }
}

/// Delay the initialization
impl rokol::app::RApp for SnowRl {
    fn init(&mut self) {
        rg::setup(&mut rokol::glue::app_desc());
        self.x = Some(SnowRlImpl::new());
    }

    fn event(&mut self, ev: &ra::Event) {
        if let Some(x) = self.x.as_mut() {
            x.event(ev);
        }
    }

    fn frame(&mut self) {
        if let Some(x) = self.x.as_mut() {
            x.frame();
        }
    }
}

// #[derive(Debug)]
struct SnowRlImpl {
    wcx: WorldContext,
    world: World,
    game_loop: GameLoop,
    state: GameState,
}

impl SnowRlImpl {
    pub fn new() -> Self {
        let file = snow2d::asset::path("map/tmx/rl_start.tmx");

        let mut wcx = WorldContext::new();
        let world = World::from_tiled_file(&mut wcx, &file).unwrap();
        let game_loop = GameLoopImpl::new();

        Self {
            wcx,
            world,
            game_loop,
            state: GameState::Tick,
        }
    }
}

impl rokol::app::RApp for SnowRlImpl {
    fn event(&mut self, ev: &ra::Event) {
        // print event type
        let type_ = rokol::app::EventType::from_u32(ev.type_).unwrap();
        use rokol::app::EventType as Ev;
        if !matches!(type_, Ev::MouseMove | Ev::MouseEnter | Ev::MouseLeave) {
            let key = rokol::app::Key::from_u32(ev.key_code).unwrap();
            println!("{:?}, {:?}", type_, key);
        }

        self.wcx.event(ev);
        self.world.event(&mut self.wcx, ev);
    }

    fn frame(&mut self) {
        self.wcx.update();
        self.update_scene();
        self.world.update_images(&mut self.wcx);

        self.wcx.render();
        self.world.render(&mut self.wcx);

        self.wcx.on_end_frame();
        self.world.on_end_frame(&mut self.wcx);

        rg::commit();
    }
}

impl SnowRlImpl {
    /// Updates the game depending on the state
    fn update_scene(&mut self) {
        loop {
            match self.state {
                // tick the game (take turn and process actions)
                GameState::Tick => {
                    let res = self.game_loop.tick(&mut self.world, &mut self.wcx);
                    // println!("tick() -> {:?}", res);

                    // TODO: handle player action
                    break;

                    self.state = GameState::Player;
                    continue;
                }

                // GameState::Player => {
                //     if self.update_player(wcx) {
                //         self.state = GameState::Tick;
                //     }
                // }
                // play animation and wait for it to finish
                // GameState::Anim => unimplemented!(),
                _ => unimplemented!(),
            }
            break;
        }
    }
}

/// Collects magic values (yes, it shuld be removed at some time)
pub mod consts {
    pub const ACTOR_FPS: f32 = 4.0;
    pub const FOV_R: u32 = 5;
    pub const WALK_TIME: f32 = 8.0 / 60.0;
}
