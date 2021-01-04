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
        tick::{AnimContext, GameLoop, TickResult},
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

enum UpdateResult {
    GotoNextFrame,
    SwitchInThisFrame(GameState),
    SwitchNextFrame(GameState),
}

#[derive(Debug)]
struct SnowRlImpl {
    wcx: WorldContext,
    world: World,
    game_loop: GameLoop,
    anims: AnimPlayer,
    state: GameState,
    current_frame_count: u64,
    last_frame_on_tick: u64,
}

impl SnowRlImpl {
    pub fn new() -> Self {
        let file = snow2d::asset::path("map/tmx/title.tmx");

        let mut wcx = WorldContext::new();
        let world = World::from_tiled_file(&mut wcx, &file).unwrap();
        let game_loop = GameLoop::new();

        Self {
            wcx,
            world,
            game_loop,
            anims: AnimPlayer::new(),
            state: GameState::Tick,
            current_frame_count: 0,
            last_frame_on_tick: 0,
        }
    }
}

impl rokol::app::RApp for SnowRlImpl {
    fn event(&mut self, ev: &ra::Event) {
        self.wcx.event(ev);
        self.world.event(&mut self.wcx, ev);
    }

    fn frame(&mut self) {
        self.current_frame_count += 1;
        // log::trace!("----- {} frame", self.current_frame_count);

        // update the internal game state
        self.wcx.update();
        self.update_scene();

        // update view states
        self.world.update_images(&mut self.wcx);
        self.world.shadow.update(self.wcx.dt);

        // finally render them all
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
                UpdateResult::SwitchInThisFrame(next_state) => {
                    switch_state(self, next_state);
                    continue;
                }
                UpdateResult::SwitchNextFrame(next_state) => {
                    switch_state(self, next_state);
                    break;
                }
            }
        }

        /// Handles `on_enter` and `on_exist` for each state
        fn switch_state(me: &mut SnowRlImpl, next_state: GameState) {
            match next_state {
                GameState::Anim => me.on_enter_anim_state(),
                _ => {}
            }

            me.state = next_state;
        }
    }

    fn update_tick(&mut self) -> UpdateResult {
        loop {
            let res = self.game_loop.tick(&mut self.world, &mut self.wcx);
            // log::trace!("{:?}", res);

            match res {
                TickResult::TakeTurn(actor) => {
                    if actor.0 == 0 {
                        // NOTE: if we handle "change direction" animation, it can results in an
                        // infinite loop:
                        // run batched walk animation if it's player's turn
                        if self.anims.any_batch() {
                            return UpdateResult::SwitchInThisFrame(GameState::Anim);
                        }

                        let is_on_same_frame = self.last_frame_on_tick == self.current_frame_count;
                        self.last_frame_on_tick = self.current_frame_count;
                        if is_on_same_frame {
                            // another player turn after all actors taking turns.
                            // maybe all actions didn't take any frame.
                            // force waiting for a frame to ensure we don't enter inifinite loop:
                            return UpdateResult::GotoNextFrame;
                        }
                    }

                    continue;
                }
                TickResult::Command(cmd) => {
                    // log::trace!("command: {:?}", cmd);

                    // try to create animation
                    let mut acx = AnimContext {
                        world: &mut self.world,
                        wcx: &mut self.wcx,
                    };

                    // play animations if any
                    if let Some(anim) = cmd.gen_anim(&mut acx) {
                        // log::trace!("command animation: {:?}", anim);

                        self.anims.enqueue_boxed(anim);

                        // run not-batched animation
                        // (batch walk animations as much as possible)
                        if self.anims.any_anim_to_run_now() {
                            return UpdateResult::SwitchInThisFrame(GameState::Anim);
                        }
                    }

                    continue;
                }
                TickResult::ProcessingCommand => {
                    return UpdateResult::GotoNextFrame;
                }
            }
        }
    }

    fn on_enter_anim_state(&mut self) {
        let mut ucx = AnimUpdateContext {
            dt: self.wcx.dt,
            world: &mut self.world,
            wcx: &mut self.wcx,
        };

        self.anims.on_start(&mut ucx);
    }

    fn update_anim(&mut self) -> UpdateResult {
        let mut ucx = AnimUpdateContext {
            dt: self.wcx.dt,
            world: &mut self.world,
            wcx: &mut self.wcx,
        };

        // TODO: handle walk animation stack
        match self.anims.update(&mut ucx) {
            AnimResult::GotoNextFrame => UpdateResult::GotoNextFrame,
            AnimResult::Finish => UpdateResult::SwitchInThisFrame(GameState::Tick),
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
