use std::any::TypeId;

use snow2d::{
    asset,
    gfx::tex::{SpriteData, Texture2dDrop},
};

use crate::{
    fsm::{render::WorldRenderFlag, GameState, Global, StateUpdateResult},
    turn::{
        anim::{AnimResult, AnimUpdateContext},
        tick::{AnimContext, GameLoop, TickResult},
    },
};

/// Roguelike game state
#[derive(Debug, Default)]
pub struct Roguelike {
    game_loop: GameLoop,
    current_frame_count: u64,
    last_frame_on_tick: u64,
}

impl GameState for Roguelike {
    fn update(&mut self, gl: &mut Global) -> StateUpdateResult {
        loop {
            let res = self.game_loop.tick(&mut gl.world, &mut gl.wcx);
            // log::trace!("{:?}", res);

            match res {
                TickResult::TakeTurn(actor) => {
                    if actor.0 == 0 {
                        // NOTE: if we handle "change direction" animation, it can results in an
                        // infinite loop:
                        // run batched walk animation if it's player's turn
                        if gl.anims.any_batch() {
                            return StateUpdateResult::PushAndRun(TypeId::of::<Animation>());
                        }

                        let is_on_same_frame = self.last_frame_on_tick == self.current_frame_count;
                        self.last_frame_on_tick = self.current_frame_count;
                        if is_on_same_frame {
                            // another player turn after all actors taking turns.
                            // maybe all actions didn't take any frame.
                            // force waiting for a frame to ensure we don't enter inifinite loop:
                            return StateUpdateResult::GotoNextFrame;
                        }
                    }

                    continue;
                }
                TickResult::Command(cmd) => {
                    // log::trace!("command: {:?}", cmd);

                    // try to create animation
                    let mut acx = AnimContext {
                        world: &mut gl.world,
                        wcx: &mut gl.wcx,
                    };

                    // play animations if any
                    if let Some(anim) = cmd.gen_anim(&mut acx) {
                        // log::trace!("command animation: {:?}", anim);

                        gl.anims.enqueue_boxed(anim);

                        // run not-batched animation
                        // (batch walk animations as much as possible)
                        if gl.anims.any_anim_to_run_now() {
                            return StateUpdateResult::PushAndRun(TypeId::of::<Animation>());
                        }
                    }

                    continue;
                }
                TickResult::ProcessingCommand => {
                    return StateUpdateResult::GotoNextFrame;
                }
            }
        }
    }

    fn render(&mut self, gl: &mut Global) {
        let flags = WorldRenderFlag::ALL;
        gl.world_render.render(&gl.world, &mut gl.wcx, flags);
    }
}

/// Roguelike game animation state
#[derive(Debug, Default)]
pub struct Animation {}

impl GameState for Animation {
    fn update(&mut self, gl: &mut Global) -> StateUpdateResult {
        let mut ucx = AnimUpdateContext {
            world: &mut gl.world,
            wcx: &mut gl.wcx,
        };

        match gl.anims.update(&mut ucx) {
            AnimResult::GotoNextFrame => StateUpdateResult::GotoNextFrame,
            AnimResult::Finish => StateUpdateResult::PopAndRun,
        }
    }

    fn on_enter(&mut self, gl: &mut Global) {
        let mut ucx = AnimUpdateContext {
            world: &mut gl.world,
            wcx: &mut gl.wcx,
        };

        gl.anims.on_start(&mut ucx);
    }

    fn render(&mut self, gl: &mut Global) {
        let flags = WorldRenderFlag::ALL;
        gl.world_render.render(&gl.world, &mut gl.wcx, flags);
    }
}

/// Title screen
#[derive(Debug)]
pub struct Title {
    title: SpriteData,
    choices: [SpriteData; 3],
}

impl Default for Title {
    fn default() -> Self {
        Self {
            title: SpriteData {
                sub_tex: Texture2dDrop::from_path(asset::path("img/title/title.png"))
                    .unwrap()
                    .into_shared()
                    .split([0.0, 0.0, 1.0, 1.0]),
                rot: 0.0,
                origin: [0.0, 0.0],
            },
            choices: {
                let tex = Texture2dDrop::from_path(asset::path("img/title/choices.png"))
                    .unwrap()
                    .into_shared();

                let unit = 1.0 / 3.0;

                // FIXME: slow, so use async. Which is slow, CPU part or GPU part?
                [
                    SpriteData {
                        sub_tex: tex.split([0.0, 0.0 * unit, 1.0, unit]),
                        rot: 0.0,
                        origin: [0.0, 0.0],
                    },
                    SpriteData {
                        sub_tex: tex.split([0.0, 1.0 * unit, 1.0, unit]),
                        rot: 0.0,
                        origin: [0.0, 0.0],
                    },
                    SpriteData {
                        sub_tex: tex.split([0.0, 2.0 * unit, 1.0, unit]),
                        rot: 0.0,
                        origin: [0.0, 0.0],
                    },
                ]
            },
        }
    }
}

impl GameState for Title {
    fn update(&mut self, _gl: &mut Global) -> StateUpdateResult {
        StateUpdateResult::GotoNextFrame
    }

    fn render(&mut self, gl: &mut Global) {
        let flags = WorldRenderFlag::ALL;
        gl.world_render.render(&gl.world, &mut gl.wcx, flags);
    }

    fn on_enter(&mut self, _gl: &mut Global) {
        // let mut ucx = AnimUpdateContext {
        //     world: &mut gl.world,
        //     wcx: &mut gl.wcx,
        // };

        // gl.anims.on_start(&mut ucx);
    }
}
