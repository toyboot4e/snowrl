/*!
Stack-based game states
*/

use std::{any::TypeId, borrow::Cow};

use snow2d::utils::arena::Index;

use grue2d::{
    ctrl::{
        rogue::{
            anim::AnimResult,
            ev,
            script::ScriptRef,
            tick::{AnimContext, GameLoop, TickResult},
        },
        Control,
    },
    data::{res::Ui, world::actor::Actor, Data},
    fsm::{GameState, StateCommand, StateReturn},
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
    fn update(&mut self, data: &mut Data, ctrl: &mut Control) -> StateReturn {
        loop {
            let res = self
                .game_loop
                .tick(&mut data.world, &mut data.ice, &mut data.res.vi);

            match res {
                TickResult::TakeTurn(actor) => {
                    // TODO: don't hard code player detection
                    if actor.slot() == 0 {
                        // NOTE: if we handle "change direction" animation, it can results in an
                        // infinite loop:
                        // run batched walk animation if it's player's turn
                        if ctrl.rogue.anims.any_batch() {
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
                        world: &mut data.world,
                        ice: &mut data.ice,
                    }) {
                        // log::trace!("event animation: {:?}", anim);

                        ctrl.rogue.anims.enqueue_box(anim);

                        // run not-batched animation
                        // (batch walk animations as much as possible)
                        if ctrl.rogue.anims.any_anim_to_run_now() {
                            return StateReturn::ThisFrame(vec![StateCommand::Push(TypeId::of::<
                                Animation,
                            >(
                            ))]);
                        }
                    }

                    // handle delegated event
                    // FIXME: don't use downcast to handle events
                    let any = (*ev).as_any();

                    if let Some(talk) = any.downcast_ref::<ev::InteractWithActor>() {
                        ctrl.rogue.script_to_play = Some(ScriptRef::Interact {
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
    fn update(&mut self, data: &mut Data, ctrl: &mut Control) -> StateReturn {
        match ctrl.rogue.anims.update(data) {
            AnimResult::GotoNextFrame => StateReturn::NextFrame(vec![]),
            AnimResult::Finish => StateReturn::ThisFrame(vec![StateCommand::Pop]),
        }
    }

    fn on_enter(&mut self, data: &mut Data, ctrl: &mut Control) {
        ctrl.rogue.anims.on_start(data);
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
    fn on_enter(&mut self, data: &mut Data, _ctrl: &mut Control) {
        // self.title.init();

        let song = data
            .ice
            .assets
            .load_sync(paths::sound::bgm::FOREST_02)
            .unwrap();
        data.ice.music_player.play_song(song);
    }

    fn update(&mut self, data: &mut Data, _ctrl: &mut Control) -> StateReturn {
        // // if debug
        // #[cfg(debug_assertions)]
        // if gl.ice.input.kbd.is_key_pressed(snow2d::input::Key::R) {
        //     self.init();
        // }

        let res = match self.title.handle_input(&mut data.ice, &mut data.res) {
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
            baloon: {
                SpriteData::builder(cache.load_sync(paths::img::sourve::BALOON).unwrap())
                    .uv_rect([0.0, 0.0, 0.5, 0.5])
                    .origin([0.5, 0.0])
                    .build()
            },
        }
    }
}

impl GameState for PlayScript {
    fn on_enter(&mut self, _data: &mut Data, ctrl: &mut Control) {
        assert!(ctrl.rogue.script_to_play.is_some());
    }

    fn on_exit(&mut self, _data: &mut Data, ctrl: &mut Control) {
        ctrl.rogue.script_to_play = None;
    }

    fn update(&mut self, data: &mut Data, ctrl: &mut Control) -> StateReturn {
        // we assume interact script (for now)
        let script = ctrl.rogue.script_to_play.as_ref().unwrap();
        let (from, to) = match script {
            ScriptRef::Interact { from, to } => (from.clone(), to.clone()),
        };

        // TODO: allow any script
        let txt = "OMG!\nYou're too big, Ika-chan.\n\nHallo hallo haa~~♪";
        let play_text = PlayTalkState::new(data, txt.to_string(), from, to);

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
    pub fn new(data: &mut Data, txt: String, from: Index<Actor>, to: Index<Actor>) -> Self {
        let (a, b) = (&data.world.entities[from], &data.world.entities[to]);

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
            data: play::talk::PlayTalk::new(talk, data),
        }
    }
}

impl GameState for PlayTalkState {
    fn update(&mut self, data: &mut Data, _ctrl: &mut Control) -> StateReturn {
        if data.res.vi.select.is_pressed() {
            // Exit on enter
            StateReturn::NextFrame(vec![StateCommand::PopAndRemove, StateCommand::Pop])
        } else {
            StateReturn::NextFrame(vec![])
        }
    }
}
