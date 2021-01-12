use {rokol::gfx as rg, std::any::TypeId};

use snow2d::{
    asset,
    gfx::{
        batcher::draw::*,
        geom2d::*,
        tex::{NineSliceSprite, SpriteData, Texture2dDrop},
        Color,
    },
    PassConfig,
};

use rlbox::rl::grid2d::*;

use crate::{
    fsm::{render::WorldRenderFlag, GameState, Global, StateUpdateResult},
    paths,
    script::ScriptRef,
    turn::{
        anim::{AnimResult, AnimUpdateContext},
        ev,
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
        if gl.wcx.vi.select.is_pressed() {
            log::trace!("ENTER");
        }
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

                        if self.last_frame_on_tick == self.current_frame_count {
                            // another player turn after all actors taking turns.
                            // maybe all actions didn't take any frame.
                            // force waiting for a frame to ensure we don't enter inifinite loop:
                            log::trace!("avoid loop");
                            return StateUpdateResult::GotoNextFrame;
                        }

                        self.last_frame_on_tick = self.current_frame_count;
                    }

                    continue;
                }
                TickResult::Event(ev) => {
                    // play animations if any
                    if let Some(anim) = ev.gen_anim(&mut AnimContext {
                        world: &mut gl.world,
                        wcx: &mut gl.wcx,
                    }) {
                        // log::trace!("event animation: {:?}", anim);

                        gl.anims.enqueue_boxed(anim);

                        // run not-batched animation
                        // (batch walk animations as much as possible)
                        if gl.anims.any_anim_to_run_now() {
                            return StateUpdateResult::PushAndRun(TypeId::of::<Animation>());
                        }
                    }

                    // handle delegated event
                    // FIXME: don't use downcast to handle events
                    let any = (*ev).as_any();

                    if let Some(talk) = any.downcast_ref::<ev::Talk>() {
                        gl.script_to_play = Some(ScriptRef::Interact {
                            from: talk.from,
                            to: talk.to,
                        });

                        return StateUpdateResult::PushAndRunNextFrame(TypeId::of::<PlayScript>());
                    }

                    continue;
                }
                TickResult::ProcessingEvent => {
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
///
/// TODO: Animation should have the anmation queue and handle PushAnim event (if possible)
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
    logo: SpriteData,
    // TODO: impl selections
    choices: [SpriteData; 3],
    cursor: usize,
}

impl Default for Title {
    fn default() -> Self {
        Self {
            logo: SpriteData {
                sub_tex: Texture2dDrop::from_path(asset::path(paths::img::title::SNOWRL))
                    .unwrap()
                    .into_shared()
                    .split([0.0, 0.0, 1.0, 1.0]),
                rot: 0.0,
                origin: [0.0, 0.0],
                scale: [0.5, 0.5],
            },
            choices: {
                let tex = Texture2dDrop::from_path(asset::path(paths::img::title::CHOICES))
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

        // title logo
        screen.sprite(&self.logo).dst_pos_px([400.0, 32.0]);

        // choices (TODO: animate selection transition)
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

#[derive(Debug)]
pub struct PlayScript {
    window: NineSliceSprite,
}

impl Default for PlayScript {
    fn default() -> Self {
        Self {
            window: NineSliceSprite {
                tex: Texture2dDrop::from_path(asset::path(paths::img::sourve::A))
                    .unwrap()
                    .into_shared(),
            },
        }
    }
}

impl GameState for PlayScript {
    fn on_enter(&mut self, gl: &mut Global) {
        assert!(gl.script_to_play.is_some());
    }

    fn on_exit(&mut self, gl: &mut Global) {
        gl.script_to_play = None;
    }

    fn update(&mut self, gl: &mut Global) -> StateUpdateResult {
        if gl.wcx.vi.select.is_pressed() {
            // Exit on enter
            StateUpdateResult::PopAndRun
        } else {
            StateUpdateResult::GotoNextFrame
        }
    }

    fn render(&mut self, gl: &mut Global) {
        let flags = WorldRenderFlag::ALL;
        gl.world_render.render(&gl.world, &mut gl.wcx, flags);

        let pos = Vec2f::new(80.0, 80.0);

        let mut screen = gl.wcx.rdr.screen(Default::default());

        let text = "石焼き芋！　焼き芋〜〜\n二行目のテキストです。あいうえお、かきくけこ。";

        let mut bounds = screen.fontbook().text_bounds(pos, text);
        bounds[0] -= crate::consts::MESSAGE_PAD_EACH[0];
        bounds[1] -= crate::consts::MESSAGE_PAD_EACH[1];
        bounds[2] += crate::consts::MESSAGE_PAD_EACH[0] * 2.0;
        bounds[3] += crate::consts::MESSAGE_PAD_EACH[1] * 2.0;

        screen.sprite(&self.window).dst_rect_px(bounds);
        screen.text(pos, text);
    }
}
