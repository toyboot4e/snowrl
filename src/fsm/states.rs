//! Stack-based game states

use {
    rokol::gfx as rg,
    std::{any::TypeId, borrow::Cow},
};

use snow2d::{
    asset::{Asset, AssetCacheAny},
    audio,
    gfx::{draw::*, geom2d::*, tex::*, Color, PassConfig},
    Ice,
};

use rlbox::rl::grid2d::*;

use crate::{
    fsm::{render::WorldRenderFlag, GameState, Global, StateCommand, StateReturn},
    play,
    script::ScriptRef,
    turn::{
        anim::{AnimResult, AnimUpdateContext},
        ev,
        tick::{ActorIx, AnimContext, GameLoop, TickResult},
    },
    utils::paths,
};

/// Roguelike game state
#[derive(Debug, Default)]
pub struct Roguelike {
    game_loop: GameLoop,
    current_frame_count: u64,
    last_frame_on_tick: u64,
}

impl GameState for Roguelike {
    fn update(&mut self, gl: &mut Global) -> StateReturn {
        loop {
            let res = self.game_loop.tick(&mut gl.world, &mut gl.vi);

            match res {
                TickResult::TakeTurn(actor) => {
                    if actor.0 == 0 {
                        // NOTE: if we handle "change direction" animation, it can results in an
                        // infinite loop:
                        // run batched walk animation if it's player's turn
                        if gl.anims.any_batch() {
                            return StateReturn::ThisFrame(vec![StateCommand::Push(TypeId::of::<
                                Animation,
                            >(
                            ))]);
                        }

                        if self.last_frame_on_tick == self.current_frame_count {
                            // another player turn after all actors taking turns.
                            // maybe all actions didn't take any frame.
                            // force waiting for a frame to ensure we don't enter inifinite loop:
                            // log::trace!("avoid player inifinite loop");
                            return StateReturn::ThisFrame(vec![]);
                        }

                        self.last_frame_on_tick = self.current_frame_count;
                    }

                    continue;
                }
                TickResult::Event(ev) => {
                    // play animations if any
                    if let Some(anim) = ev.gen_anim(&mut AnimContext {
                        world: &mut gl.world,
                        ice: &mut gl.ice,
                    }) {
                        // log::trace!("event animation: {:?}", anim);

                        gl.anims.enqueue_boxed(anim);

                        // run not-batched animation
                        // (batch walk animations as much as possible)
                        if gl.anims.any_anim_to_run_now() {
                            return StateReturn::ThisFrame(vec![StateCommand::Push(TypeId::of::<
                                Animation,
                            >(
                            ))]);
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

                        // enter PlayScript state in NEXT frame because interact key is still pressed
                        return StateReturn::NextFrame(vec![StateCommand::Push(TypeId::of::<
                            PlayScript,
                        >(
                        ))]);
                    }

                    continue;
                }
                TickResult::ProcessingEvent => {
                    return StateReturn::NextFrame(vec![]);
                }
            }
        }
    }

    fn render(&mut self, gl: &mut Global) {
        let flags = WorldRenderFlag::ALL;
        gl.world_render.render(&gl.world, &mut gl.ice, flags);
    }
}

/// Roguelike game animation state
///
/// TODO: Animation should have the anmation queue and handle PushAnim event (if possible)
#[derive(Debug, Default)]
pub struct Animation {}

impl GameState for Animation {
    fn update(&mut self, gl: &mut Global) -> StateReturn {
        let mut ucx = AnimUpdateContext {
            world: &mut gl.world,
            ice: &mut gl.ice,
        };

        match gl.anims.update(&mut ucx) {
            AnimResult::GotoNextFrame => StateReturn::NextFrame(vec![]),
            AnimResult::Finish => StateReturn::ThisFrame(vec![StateCommand::Pop]),
        }
    }

    fn on_enter(&mut self, gl: &mut Global) {
        let mut ucx = AnimUpdateContext {
            world: &mut gl.world,
            ice: &mut gl.ice,
        };

        gl.anims.on_start(&mut ucx);
    }

    fn render(&mut self, gl: &mut Global) {
        let flags = WorldRenderFlag::ALL;
        gl.world_render.render(&gl.world, &mut gl.ice, flags);
    }
}

/// Title screen
#[derive(Debug)]
pub struct Title {
    logo: SpriteData,
    // TODO: impl selections
    choices: [SpriteData; 3],
    cursor: usize,
    se_cursor: Asset<audio::src::Wav>,
    se_select: Asset<audio::src::Wav>,
}

impl Title {
    pub fn new(ice: &mut Ice) -> Self {
        let cache = &mut ice.assets;
        Self {
            logo: SpriteData {
                tex: cache.load_sync(paths::img::title::SNOWRL).unwrap(),
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                rot: 0.0,
                origin: [0.0, 0.0],
                scales: [0.5, 0.5],
            },
            choices: {
                let tex = cache.load_sync(paths::img::title::CHOICES).unwrap();

                let scale = [0.5, 0.5];
                let origin = [0.0, 0.0];
                let rot = 0.0;

                let unit = 1.0 / 3.0;

                // FIXME: slow, so use async. Which is slow, CPU part or GPU part?
                [
                    SpriteData {
                        tex: tex.clone(),
                        uv_rect: [0.0, unit * 0.0, 1.0, unit],
                        rot,
                        origin,
                        scales: scale,
                    },
                    SpriteData {
                        tex: tex.clone(),
                        uv_rect: [0.0, unit * 1.0, 1.0, unit],
                        rot,
                        origin,
                        scales: scale,
                    },
                    SpriteData {
                        tex: tex.clone(),
                        uv_rect: [0.0, unit * 2.0, 1.0, unit],
                        rot,
                        origin,
                        scales: scale,
                    },
                ]
            },
            cursor: 0,
            se_cursor: cache.load_sync(paths::sound::se::CURSOR).unwrap(),
            se_select: cache.load_sync(paths::sound::se::SELECT).unwrap(),
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
    fn on_enter(&mut self, gl: &mut Global) {
        let ice = &mut gl.ice;
        let cache = &mut ice.assets;

        let song = cache.load_sync(paths::sound::bgm::FOREST_02).unwrap();
        ice.music_player.play_song(song);
    }

    fn update(&mut self, gl: &mut Global) -> StateReturn {
        if let Some(dir) = gl.vi.dir.dir4_pressed() {
            match dir.y_sign() {
                Sign::Pos => {
                    self.cursor += self.choices.len() - 1;
                    self.cursor %= self.choices.len();
                    gl.ice.audio.play(&*self.se_cursor.get_mut().unwrap());
                }
                Sign::Neg => {
                    self.cursor += 1;
                    self.cursor %= self.choices.len();
                    gl.ice.audio.play(&*self.se_cursor.get_mut().unwrap());
                }
                Sign::Neutral => {}
            }
        }

        if gl.vi.select.is_pressed() {
            gl.ice.audio.play(&*self.se_select.get_mut().unwrap());
            StateReturn::ThisFrame(vec![StateCommand::Pop])
        } else {
            // TODO: fade out
            StateReturn::NextFrame(vec![])
        }
    }

    fn render(&mut self, gl: &mut Global) {
        let flags = WorldRenderFlag::ALL;
        gl.world_render.render(&gl.world, &mut gl.ice, flags);

        let mut screen = gl.ice.rdr.screen(PassConfig {
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
}

/// Just plays hard-coded script (for now)
#[derive(Debug)]
pub struct PlayScript {
    window: NineSliceSprite,
    baloon: SpriteData,
}

impl PlayScript {
    pub fn new(cache: &mut AssetCacheAny) -> Self {
        Self {
            window: NineSliceSprite {
                tex: cache.load_sync(paths::img::sourve::A).unwrap(),
            },
            baloon: SpriteData {
                tex: cache.load_sync(paths::img::sourve::BALOON).unwrap(),
                uv_rect: [0.0, 0.0, 0.5, 0.5],
                // REMARK:
                origin: [0.5, 0.0],
                ..Default::default()
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

    fn update(&mut self, gl: &mut Global) -> StateReturn {
        // we assume interact script (for now)
        let script = gl.script_to_play.as_ref().unwrap();
        let (from, to) = match script {
            ScriptRef::Interact { from, to } => (from.clone(), to.clone()),
        };

        // TODO: allow any script
        let txt = "石焼き芋！　焼き芋〜〜\n二行目のテキストです。あいうえお、かきくけこ。\n\n4 行目です！";
        let play_text = PlayTextState::new(gl, txt.to_string(), from, to);

        StateReturn::ThisFrame(vec![
            StateCommand::insert(play_text),
            StateCommand::Push(TypeId::of::<PlayTextState>()),
        ])
    }

    fn render(&mut self, _gl: &mut Global) {
        // unreachable!()
    }
}

#[derive(Debug)]
pub struct PlayTextState {
    data: play::talk::PlayTalk,
}

impl PlayTextState {
    pub fn new(gl: &mut Global, txt: String, from: ActorIx, to: ActorIx) -> Self {
        let (a, b) = (&gl.world.entities[from.0], &gl.world.entities[to.0]);

        let talk = play::talk::TalkCommand {
            txt: Cow::Owned(txt),
            from,
            to,
            cfg: play::talk::TalkConfig {
                // let the window not overwrap actors
                dir: if a.pos.y >= b.pos.y {
                    play::talk::TalkDirection::Up
                } else {
                    play::talk::TalkDirection::Down
                },
                kind: play::talk::TalkKind::Speak,
            },
        };

        Self {
            data: play::talk::PlayTalk::new(talk, &mut gl.ice, &gl.world),
        }
    }
}

impl GameState for PlayTextState {
    fn update(&mut self, gl: &mut Global) -> StateReturn {
        self.data.update(gl.ice.dt);

        if gl.vi.select.is_pressed() {
            // Exit on enter
            StateReturn::NextFrame(vec![StateCommand::PopAndRemove, StateCommand::Pop])
        } else {
            StateReturn::NextFrame(vec![])
        }
    }

    fn render(&mut self, gl: &mut Global) {
        let flags = WorldRenderFlag::ALL;
        gl.world_render.render(&gl.world, &mut gl.ice, flags);
        let mut screen = gl.ice.rdr.screen(Default::default());
        self.data.render(&mut screen);
    }
}
