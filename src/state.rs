//! State

// TODO: use result

use std::{any::TypeId, collections::HashMap};

use rokol::app as ra;

use crate::{
    turn::{
        anim::{AnimPlayer, AnimResult, AnimUpdateContext},
        tick::{AnimContext, GameLoop, TickResult},
    },
    world::{World, WorldContext},
};

/// Shared data among [`GameState`]s
#[derive(Debug)]
pub struct Global {
    pub world: World,
    pub wcx: WorldContext,
    // Roguelike animation
    pub anims: AnimPlayer,
}

pub trait GameState: std::fmt::Debug {
    fn event(&mut self, _ev: &ra::Event, _gl: &mut Global) {}
    fn update(&mut self, _gl: &mut Global) -> StateUpdateResult;
    fn render(&mut self, _gl: &mut Global) {}
    fn on_enter(&mut self, _gl: &mut Global) {}
    fn on_exit(&mut self, _gl: &mut Global) {}
    // TODO: use proper name
    fn on_stop(&mut self, _gl: &mut Global) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateUpdateResult {
    GotoNextFrame,
    PushAndRun(TypeId),
    PopAndRun,
}

/// Stack-based finite state machine
#[derive(Debug)]
pub struct Fsm {
    states: HashMap<TypeId, Box<dyn GameState>>,
    stack: Vec<TypeId>,
}

impl Fsm {
    pub fn new() -> Self {
        Self {
            states: HashMap::with_capacity(10),
            stack: Vec::with_capacity(10),
        }
    }

    pub fn update(&mut self, gl: &mut Global) {
        loop {
            let id = self.stack.last().unwrap();
            let state = self.states.get_mut(id).unwrap();
            let res = state.update(gl);

            match res {
                StateUpdateResult::GotoNextFrame => {
                    break;
                }
                StateUpdateResult::PushAndRun(next_state) => {
                    self.push_id(next_state, gl);
                    continue;
                }
                StateUpdateResult::PopAndRun => {
                    self.pop(gl);
                    continue;
                }
            }
        }
    }
}

/// State management
impl Fsm {
    pub fn insert<T: GameState + 'static>(&mut self, state: T) -> Option<Box<dyn GameState>> {
        self.states.insert(TypeId::of::<T>(), Box::new(state))
    }

    pub fn insert_default<T: GameState + 'static + Default>(
        &mut self,
    ) -> Option<Box<dyn GameState>> {
        self.states
            .insert(TypeId::of::<T>(), Box::new(T::default()))
    }

    pub fn push<T: GameState + 'static>(&mut self, gl: &mut Global) {
        let id = TypeId::of::<T>();
        self.push_id(id, gl);
    }

    pub fn push_id(&mut self, id: TypeId, gl: &mut Global) {
        if let Some(last_id) = self.stack.last() {
            let last = self.states.get_mut(last_id).unwrap();
            last.on_stop(gl);
        }

        let new = self.states.get_mut(&id).unwrap();
        new.on_enter(gl);

        self.stack.push(id);
    }

    pub fn pop(&mut self, gl: &mut Global) {
        let last_id = self.stack.last().unwrap();
        let last = self.states.get_mut(last_id).unwrap();
        last.on_exit(gl);

        self.stack.pop();
    }
}

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
}
