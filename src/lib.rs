/*!

Snow the roguelike game

*/

// use generator (unstable Rust)
#![feature(generators, generator_trait)]

pub use {rlbox, rokol, snow2d};

pub mod turn;
pub mod utils;
pub mod world;

use rokol::{app as ra, gfx as rg};

use crate::{
    turn::{
        anim::{AnimPlayer, AnimResult, AnimUpdateContext},
        tick::{AnimContext, GameLoop, GameLoopImpl, TickResult},
    },
    world::{World, WorldContext},
};

pub fn run(app: rokol::Rokol) -> rokol::Result {
    app.run(&mut SnowRl::new())
}

#[derive(Debug)]
enum GameState {
    Tick,
    Anim,
}

enum UpdateResult {
    GotoNextFrame,
    SwitchThisFrame(GameState),
    SwitchNextFrame(GameState),
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
    anim_player: AnimPlayer,
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
            anim_player: AnimPlayer::new(),
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
            let res = match self.state {
                GameState::Tick => self.update_tick(),
                GameState::Anim => self.update_anim(),
            };

            match res {
                UpdateResult::GotoNextFrame => {
                    break;
                }
                UpdateResult::SwitchThisFrame(state) => {
                    self.state = state;
                    continue;
                }
                UpdateResult::SwitchNextFrame(state) => {
                    self.state = state;
                    break;
                }
            }
        }
    }

    fn update_tick(&mut self) -> UpdateResult {
        loop {
            // TODO: warn if every actor took turn and nothing happened
            match self.game_loop.tick(&mut self.world, &mut self.wcx) {
                TickResult::TakeTurn(_actor) => {
                    // TODO: handle walk animation stack
                    continue;
                }
                TickResult::Command(cmd) => {
                    // try to create animation
                    let mut acx = AnimContext {
                        world: &mut self.world,
                        wcx: &mut self.wcx,
                    };

                    if let Some(anim) = cmd.gen_anim(&mut acx) {
                        self.anim_player.push_boxed(anim);

                        self.anim_player.on_start();
                        return UpdateResult::SwitchThisFrame(GameState::Anim);
                    }
                }
                TickResult::ProcessingCommand => {
                    return UpdateResult::GotoNextFrame;
                }
            }
        }
    }

    fn update_anim(&mut self) -> UpdateResult {
        let mut ucx = AnimUpdateContext { dt: self.wcx.dt };

        // TODO: handle walk animation stack
        match self.anim_player.update(&mut ucx) {
            AnimResult::Continue => UpdateResult::GotoNextFrame,
            AnimResult::Finished => UpdateResult::SwitchThisFrame(GameState::Tick),
        }
    }
}

pub mod consts {
    //! Magic values (should be removed)

    /// FPS of character graphics animation
    pub const ACTOR_FPS: f32 = 4.0;

    /// Filed of view radius
    pub const FOV_R: u32 = 5;

    /// Walk duration in seconds
    pub const WALK_TIME: f32 = 8.0 / 60.0;

    /// Half frame in seconds (fixed timestep with 60 FPS)
    pub const HALF_FRAME: f32 = 1.0 / 120.0;
}
