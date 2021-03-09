/*!
Controls of the game with stack-based finite state machine
*/

use std::{any::TypeId, collections::HashMap};

use rokol::app as ra;

use crate::Data;

/// Game state lifecycle
pub trait GameState: std::fmt::Debug {
    fn on_enter(&mut self, _data: &mut Data) {}
    fn on_exit(&mut self, _data: &mut Data) {}
    // TODO: use proper name
    fn on_stop(&mut self, _data: &mut Data) {}

    fn event(&mut self, _ev: &ra::Event, _data: &mut Data) {}
    fn update(&mut self, _data: &mut Data) -> StateReturn;
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
    pub fn update(&mut self, update: &mut Data) {
        loop {
            let id = self.stack.last().unwrap();
            let state = self.states.get_mut(id).unwrap();
            let res = state.update(update);

            let finish = matches!(res, StateReturn::NextFrame(_));

            for cmd in res.into_cmds() {
                self.run_cmd(cmd, update);
            }

            if finish {
                break;
            }
        }
    }

    fn run_cmd(&mut self, cmd: StateCommand, data: &mut Data) {
        match cmd {
            StateCommand::Insert(typeid, state) => {
                self.states.insert(typeid, state);
            }
            StateCommand::Pop => {
                let _ = self.stack.pop().unwrap();
            }
            StateCommand::Push(typeid) => {
                self.push_id(typeid, data);
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

    pub fn push<T: GameState + 'static>(&mut self, gl: &mut Data) {
        let id = TypeId::of::<T>();
        self.push_id(id, gl);
    }

    pub fn push_id(&mut self, id: TypeId, gl: &mut Data) {
        if let Some(last_id) = self.stack.last() {
            let last = self.states.get_mut(last_id).unwrap();
            last.on_stop(gl);
        }

        let new = self.states.get_mut(&id).unwrap();
        new.on_enter(gl);

        self.stack.push(id);
    }

    pub fn pop(&mut self, gl: &mut Data) {
        let last_id = self.stack.last().unwrap();
        let last = self.states.get_mut(last_id).unwrap();
        last.on_exit(gl);

        self.stack.pop();
    }
}
