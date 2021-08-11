/*!
Stack-based finite state machine
*/

use std::{any::TypeId, collections::HashMap};

use snow2d::utils::Inspect;

// type Event = rokol::app::Event;
type Event = sdl2::event::Event;

/// Game state lifecycle
pub trait State: std::fmt::Debug {
    type Params;
    fn on_enter(&mut self, _params: &mut Self::Params) {}
    fn on_exit(&mut self, _params: &mut Self::Params) {}
    // TODO: use proper name
    fn on_stop(&mut self, _params: &mut Self::Params) {}
    fn event(&mut self, _params: &mut Self::Params) {}
    fn update(&mut self, params: &mut Self::Params) -> StateReturn<Self::Params>;
}

/// Return value of [`State::update`]
#[derive(Debug)]
pub enum StateReturn<P> {
    /// Run every command in this frame. Call update in next frame
    NextFrame(Vec<StateCommand<P>>),
    /// Run every command in this frame. Call update in this frame
    ThisFrame(Vec<StateCommand<P>>),
}

impl<P> StateReturn<P> {
    pub fn into_cmds(self) -> Vec<StateCommand<P>> {
        match self {
            Self::NextFrame(cmds) => cmds,
            Self::ThisFrame(cmds) => cmds,
        }
    }
}

/// Command in [`StateReturn`]
#[derive(Debug)]
pub enum StateCommand<P> {
    Insert(TypeId, Box<dyn State<Params = P>>),
    Pop,
    PopAndRemove,
    Push(TypeId),
}

impl<P> StateCommand<P> {
    pub fn insert<T: State<Params = P> + 'static>(state: T) -> Self {
        Self::Insert(TypeId::of::<T>(), Box::new(state))
    }
}

/// Stack-based finite state machine
#[derive(Debug)]
pub struct Fsm<P> {
    states: HashMap<TypeId, Box<dyn State<Params = P>>>,
    stack: Vec<TypeId>,
}

impl<P> Default for Fsm<P> {
    fn default() -> Self {
        Self {
            states: HashMap::with_capacity(10),
            stack: Vec::with_capacity(10),
        }
    }
}

impl<P> Fsm<P> {
    pub fn update(&mut self, params: &mut P) {
        loop {
            let id = self.stack.last().expect("No state in stack");
            let state = self.states.get_mut(id).unwrap();
            let res = state.update(params);

            let finish = matches!(res, StateReturn::NextFrame(_));

            for cmd in res.into_cmds() {
                self.run_cmd(cmd, params);
            }

            if finish {
                break;
            } else {
                continue;
            }
        }
    }

    fn run_cmd(&mut self, cmd: StateCommand<P>, params: &mut P) {
        match cmd {
            StateCommand::Insert(typeid, state) => {
                self.states.insert(typeid, state);
            }
            StateCommand::Pop => {
                let _ = self.stack.pop().unwrap();
            }
            StateCommand::Push(typeid) => {
                self.push_id(typeid, params);
            }
            StateCommand::PopAndRemove => {
                let typeid = self.pop(params);
                self.states.remove(&typeid).unwrap();
            }
        }
    }

    /// Inserts a state into the storage
    pub fn insert<T: State<Params = P> + 'static>(
        &mut self,
        state: T,
    ) -> Option<Box<dyn State<Params = P>>> {
        self.states.insert(TypeId::of::<T>(), Box::new(state))
    }

    /// Inserts a state into the storage
    pub fn insert_default<T: State<Params = P> + 'static + Default>(
        &mut self,
    ) -> Option<Box<dyn State<Params = P>>> {
        self.states
            .insert(TypeId::of::<T>(), Box::new(T::default()))
    }

    /// Pushes an existing state to the stack
    pub fn push<T: State + 'static>(&mut self, params: &mut P) {
        let id = TypeId::of::<T>();
        self.push_id(id, params);
    }

    /// Pushes an existing state to the stack by type ID
    pub fn push_id(&mut self, id: TypeId, params: &mut P) {
        if let Some(last_id) = self.stack.last() {
            let last = self.states.get_mut(last_id).unwrap();
            last.on_stop(params);
        }

        let new = self.states.get_mut(&id).unwrap();
        new.on_enter(params);

        self.stack.push(id);
    }

    /// Pushes a state from the stack
    pub fn pop(&mut self, params: &mut P) -> TypeId {
        let last_id = self
            .stack
            .last()
            .expect("Tried to pop state but there's none!");

        let last = self.states.get_mut(last_id).unwrap();
        last.on_exit(params);

        self.stack.pop().unwrap()
    }
}
