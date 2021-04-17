/*!
Controls of the game based on a stack-based finite state machine
*/

use std::{any::TypeId, collections::HashMap};

use crate::game::{Control, Data};

// type Event = rokol::app::Event;
type Event = sdl2::event::Event;

/// Game state lifecycle
pub trait GameState: std::fmt::Debug {
    fn on_enter(&mut self, _data: &mut Data, _ctrl: &mut Control) {}
    fn on_exit(&mut self, _data: &mut Data, _ctrl: &mut Control) {}
    // TODO: use proper name
    fn on_stop(&mut self, _data: &mut Data, _ctrl: &mut Control) {}
    fn event(&mut self, _ev: &Event, _data: &mut Data, _ctrl: &mut Control) {}
    fn update(&mut self, _data: &mut Data, _ctrl: &mut Control) -> StateReturn;
}

/// Return value of [`GameState::update`]
#[derive(Debug)]
pub enum StateReturn {
    /// Run every command in this frame. Call update in next frame
    NextFrame(Vec<StateCommand>),
    /// Run every command in this frame. Call update in this frame
    ThisFrame(Vec<StateCommand>),
}

impl StateReturn {
    pub fn into_cmds(self) -> Vec<StateCommand> {
        match self {
            Self::NextFrame(cmds) => cmds,
            Self::ThisFrame(cmds) => cmds,
        }
    }
}

/// Command in [`StateReturn`]
#[derive(Debug)]
pub enum StateCommand {
    Insert(TypeId, Box<dyn GameState>),
    Pop,
    PopAndRemove,
    Push(TypeId),
}

impl StateCommand {
    pub fn insert<T: GameState + 'static>(state: T) -> Self {
        Self::Insert(TypeId::of::<T>(), Box::new(state))
    }
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
    pub fn update(&mut self, data: &mut Data, ctrl: &mut Control) {
        loop {
            let id = self.stack.last().unwrap();
            let state = self.states.get_mut(id).unwrap();
            let res = state.update(data, ctrl);

            let finish = matches!(res, StateReturn::NextFrame(_));

            for cmd in res.into_cmds() {
                self.run_cmd(cmd, data, ctrl);
            }

            if finish {
                break;
            } else {
                continue;
            }
        }
    }

    fn run_cmd(&mut self, cmd: StateCommand, data: &mut Data, ctrl: &mut Control) {
        match cmd {
            StateCommand::Insert(typeid, state) => {
                self.states.insert(typeid, state);
            }
            StateCommand::Pop => {
                let _ = self.stack.pop().unwrap();
            }
            StateCommand::Push(typeid) => {
                self.push_id(typeid, data, ctrl);
            }
            StateCommand::PopAndRemove => {
                let typeid = self.stack.pop().unwrap();
                self.states.remove(&typeid).unwrap();
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

    pub fn push<T: GameState + 'static>(&mut self, data: &mut Data, ctrl: &mut Control) {
        let id = TypeId::of::<T>();
        self.push_id(id, data, ctrl);
    }

    pub fn push_id(&mut self, id: TypeId, data: &mut Data, ctrl: &mut Control) {
        if let Some(last_id) = self.stack.last() {
            let last = self.states.get_mut(last_id).unwrap();
            last.on_stop(data, ctrl);
        }

        let new = self.states.get_mut(&id).unwrap();
        new.on_enter(data, ctrl);

        self.stack.push(id);
    }

    pub fn pop(&mut self, data: &mut Data, ctrl: &mut Control) {
        let last_id = self.stack.last().unwrap();
        let last = self.states.get_mut(last_id).unwrap();
        last.on_exit(data, ctrl);

        self.stack.pop();
    }
}
