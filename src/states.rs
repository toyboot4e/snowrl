/*!
Stack-based game states
*/

use std::{any::TypeId, borrow::Cow};

use rlbox::{ui::Ui, utils::arena::Index};

use grue2d::{
    rl::{
        script::ScriptRef,
        turn::{
            anim::{AnimResult, AnimUpdateContext},
            ev,
            tick::{AnimContext, GameLoop, TickResult},
        },
        world::actor::Actor,
    },
    GameState, Global, StateCommand, StateReturn,
};

use crate::{play, prelude::*, utils::paths};

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
                    // TODO: don't hard code player detection
                    if actor.slot() == 0 {
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
}

/// Title screen
#[derive(Debug)]
pub struct Title {
    title: crate::scenes::Title,
}

impl Title {
    pub fn new(ice: &mut Ice, ui: &mut Ui) -> Self {
        Self {
            title: crate::scenes::Title::new(ice, ui),
        }
    }
}

impl GameState for Title {
    fn on_enter(&mut self, gl: &mut Global) {
        // self.title.init();

        let song = gl
            .ice
            .assets
            .load_sync(paths::sound::bgm::FOREST_02)
            .unwrap();
        gl.ice.music_player.play_song(song);
    }

    fn update(&mut self, gl: &mut Global) -> StateReturn {
        // // if debug
        // #[cfg(debug_assertions)]
        // if gl.ice.input.kbd.is_key_pressed(snow2d::input::Key::R) {
        //     self.init();
        // }

        let res = match self.title.handle_input(gl) {
            Some(res) => res,
            None => return StateReturn::NextFrame(vec![]),
        };

        use crate::scenes::title::Choice::*;
        match res {
            NewGame => StateReturn::NextFrame(vec![StateCommand::PopAndRemove]),
            Continue => {
                println!("unimplemented");
                return StateReturn::NextFrame(vec![]);
            }
            Exit => {
                println!("unimplemented");
                return StateReturn::NextFrame(vec![]);
            }
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
        let txt = "OMG!\nYou're too big, Ika-chan.\n\nHallo hallo haa~~♪";
        let play_text = PlayTalkState::new(gl, txt.to_string(), from, to);

        StateReturn::ThisFrame(vec![
            StateCommand::insert(play_text),
            StateCommand::Push(TypeId::of::<PlayTalkState>()),
        ])
    }
}

#[derive(Debug)]
pub struct PlayTalkState {
    data: play::talk::PlayTalk,
}

impl PlayTalkState {
    pub fn new(gl: &mut Global, txt: String, from: Index<Actor>, to: Index<Actor>) -> Self {
        let (a, b) = (&gl.world.entities[from], &gl.world.entities[to]);

        let talk = play::talk::TalkViewCommand {
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
            data: play::talk::PlayTalk::new(talk, gl),
        }
    }
}

impl GameState for PlayTalkState {
    fn update(&mut self, gl: &mut Global) -> StateReturn {
        if gl.vi.select.is_pressed() {
            // Exit on enter
            StateReturn::NextFrame(vec![StateCommand::PopAndRemove, StateCommand::Pop])
        } else {
            StateReturn::NextFrame(vec![])
        }
    }
}
