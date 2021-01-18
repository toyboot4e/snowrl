/*!

Stack-based finite state machine and a scheduler of renderers

*/

// TODO: use result

pub mod render;
pub mod states;

use {
    rokol::app as ra,
    std::{any::TypeId, collections::HashMap},
};

use crate::{
    fsm::render::WorldRenderer,
    script::ScriptRef,
    turn::anim::AnimPlayer,
    world::{World, WorldContext},
};

/// Thread-local global variables
#[derive(Debug)]
pub struct Global {
    pub world: World,
    // TODO: rename it to global context
    pub wcx: WorldContext,
    pub world_render: WorldRenderer,
    /// Roguelike game animations (TODO: remove it. use event to pass animation object)
    pub anims: AnimPlayer,
    pub script_to_play: Option<ScriptRef>,
    // TODO: add AssetCache
}

impl Global {
    pub fn event(&mut self, ev: &ra::Event) {
        self.wcx.event(ev);
    }

    /// Called before updating the FSM (game state)
    pub fn pre_update(&mut self) {
        self.wcx.pre_update();
        self.world.update(&mut self.wcx);
    }

    /// Called after updating the FSM (game state)
    pub fn post_update(&mut self) {
        self.wcx.post_update();

        self.world
            .shadow
            .post_update(self.wcx.dt, &self.world.map.rlmap, &self.world.entities[0]);

        self.world_render.post_update(&self.world, self.wcx.dt);
    }

    pub fn on_end_frame(&mut self) {
        self.wcx.on_end_frame();
    }
}

/// TODO: consider parameter on push and pass using downcast
pub trait GameState: std::fmt::Debug {
    fn on_enter(&mut self, _gl: &mut Global) {}
    fn on_exit(&mut self, _gl: &mut Global) {}
    // TODO: use proper name
    fn on_stop(&mut self, _gl: &mut Global) {}

    fn event(&mut self, _ev: &ra::Event, _gl: &mut Global) {}
    fn update(&mut self, _gl: &mut Global) -> StateUpdateResult;
    fn render(&mut self, gl: &mut Global) {
        use crate::fsm::render::WorldRenderFlag;
        let flags = WorldRenderFlag::ALL;
        gl.world_render.render(&gl.world, &mut gl.wcx, flags);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateUpdateResult {
    GotoNextFrame,
    PushAndRun(TypeId),
    PushAndRunNextFrame(TypeId),
    PopAndRun,
}

/// Stack-based finite state machine
#[derive(Debug)]
pub struct Fsm {
    states: HashMap<TypeId, Box<dyn GameState>>,
    stack: Vec<TypeId>,
}

impl Default for Fsm {
    fn default() -> Self {
        Self {
            states: HashMap::with_capacity(10),
            stack: Vec::with_capacity(10),
        }
    }
}

impl Fsm {
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
                StateUpdateResult::PushAndRunNextFrame(next_state) => {
                    self.push_id(next_state, gl);
                    break;
                }
                StateUpdateResult::PopAndRun => {
                    self.pop(gl);
                    continue;
                }
            }
        }
    }

    pub fn render(&mut self, gl: &mut Global) {
        let id = self.stack.last().unwrap();
        let state = self.states.get_mut(id).unwrap();
        state.render(gl);
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
