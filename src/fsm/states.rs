use std::any::TypeId;

use rokol::gfx as rg;

use snow2d::{
    asset,
    gfx::{
        batcher::draw::*,
        geom2d::*,
        tex::{SpriteData, Texture2dDrop},
        Color,
    },
    PassConfig,
};

use rlbox::rl::grid2d::*;

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
    // TODO: impl selections
    choices: [SpriteData; 3],
    cursor: usize,
}

impl Default for Title {
    fn default() -> Self {
        Self {
            title: SpriteData {
                sub_tex: Texture2dDrop::from_path(asset::path(crate::paths::img::title::SNOWRL))
                    .unwrap()
                    .into_shared()
                    .split([0.0, 0.0, 1.0, 1.0]),
                rot: 0.0,
                origin: [0.0, 0.0],
                scale: [0.5, 0.5],
            },
            choices: {
                let tex = Texture2dDrop::from_path(asset::path(crate::paths::img::title::CHOICES))
                    .unwrap()
                    .into_shared();

                let scale = [0.5, 0.5];
                let origin = [0.0, 0.0];
                let rot = 0.0;

                let unit = 1.0 / 3.0;

                // FIXME: slow, so use async. Which is slow, CPU part or GPU part?
                [
                    SpriteData {
                        sub_tex: tex.split([0.0, unit * 0.0, 1.0, unit]),
                        rot,
                        origin,
                        scale,
                    },
                    SpriteData {
                        sub_tex: tex.split([0.0, unit * 1.0, 1.0, unit]),
                        rot,
                        origin,
                        scale,
                    },
                    SpriteData {
                        sub_tex: tex.split([0.0, unit * 2.0, 1.0, unit]),
                        rot,
                        origin,
                        scale,
                    },
                ]
            },
            cursor: 0,
        }
    }
}

impl Title {
    const SELECTED: Color = Color {
        r: 24,
        g: 160,
        b: 120,
        a: 255,
    };

    const UNSELECTED: Color = Color {
        r: 85,
        g: 40,
        b: 40,
        a: 255,
    };

    const SHADOW_SELECTED: Color = Color {
        r: 32,
        g: 32,
        b: 32,
        a: 255,
    };

    const SHADOW_UNSELECTED: Color = Color {
        r: 16,
        g: 16,
        b: 16,
        a: 255,
    };
}

impl GameState for Title {
    fn update(&mut self, gl: &mut Global) -> StateUpdateResult {
        if let Some(dir) = gl.wcx.vi.dir.dir4_pressed() {
            match dir.y_sign() {
                Sign::Pos => {
                    self.cursor += self.choices.len() - 1;
                    self.cursor %= self.choices.len();
                }
                Sign::Neg => {
                    self.cursor += 1;
                    self.cursor %= self.choices.len();
                }
                Sign::Neutral => {}
            }
        }

        if gl.wcx.vi.select.is_pressed() {
            StateUpdateResult::PopAndRun
        } else {
            // TODO: fade out
            StateUpdateResult::GotoNextFrame
        }
    }

    fn render(&mut self, gl: &mut Global) {
        let flags = WorldRenderFlag::ALL;
        gl.world_render.render(&gl.world, &mut gl.wcx, flags);

        let mut screen = gl.wcx.rdr.screen(PassConfig {
            pa: &rg::PassAction::NONE,
            tfm: None,
            pip: None,
        });

        screen.sprite(&self.title).dst_pos_px([400.0, 32.0]);

        let pos = {
            let pos = Vec2f::new(100.0, 340.0);
            let delta_y = 100.0;
            [
                pos,
                pos + Vec2f::new(100.0, delta_y),
                pos + Vec2f::new(320.0, delta_y * 2.0),
            ]
        };

        for i in 0..3 {
            let color = if i == self.cursor {
                Self::SHADOW_SELECTED
            } else {
                Self::SHADOW_UNSELECTED
            };

            // shadow
            screen
                .sprite(&self.choices[i])
                .dst_pos_px(pos[i] + Vec2f::new(8.0, 6.0))
                .color(color);

            let color = if i == self.cursor {
                Self::SELECTED
            } else {
                Self::UNSELECTED
            };

            // logo
            screen
                .sprite(&self.choices[i])
                .dst_pos_px(pos[i])
                .color(color);
        }
    }

    fn on_enter(&mut self, _gl: &mut Global) {
        // let mut ucx = AnimUpdateContext {
        //     world: &mut gl.world,
        //     wcx: &mut gl.wcx,
        // };

        // gl.anims.on_start(&mut ucx);
    }
}
