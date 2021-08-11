/*!
Stack-based finite state machine

[`Fsm`] is `HashMap<TypeId, Box<dyn State>>` + `VecDeque<TypeId>`. States can communiate with via [`StateCell`].
*/

use std::{
    any::{self, TypeId},
    collections::HashMap,
};

use downcast_rs::*;
use smallvec::SmallVec;

/// Game state lifecycle
pub trait State: std::fmt::Debug + DowncastSync {
    type Data;
    fn on_enter(&mut self, _cell: &StateCell<Self::Data>, _data: &mut Self::Data)
    // where
    //     Self: Sized,
    {
    }
    fn on_exit(&mut self, _cell: &StateCell<Self::Data>, _data: &mut Self::Data)
    // where
    //     Self: Sized,
    {
    }
    // TODO: use proper name
    fn on_stop(&mut self, _cell: &StateCell<Self::Data>, _data: &mut Self::Data)
    // where
    //     Self: Sized,
    {
    }
    fn event(&mut self, _cell: &StateCell<Self::Data>, _data: &mut Self::Data)
    // where
    //     Self: Sized,
    {
    }
    fn update(
        &mut self,
        _cell: &StateCell<Self::Data>,
        _data: &mut Self::Data,
    ) -> StateReturn<Self::Data>
// where
    //     Self: Sized
    ;
}

impl_downcast!(sync State assoc Data);

/// Mutable access to multiple states. Panices when trying to borrow the same state twice (mutably
/// or immutable). TODO: Tolerant the rule to match to the standard Rust.
#[derive(Debug)]
pub struct StateCell<'a, D> {
    inner: CellInner<'a, D>,
}

#[derive(Debug)]
struct CellInner<'a, D> {
    states: &'a mut HashMap<TypeId, Box<dyn State<Data = D>>>,
    log: SmallVec<[TypeId; 2]>,
}

impl<'a, D: 'static> StateCell<'a, D> {
    pub fn get<S: State>(&self) -> Option<&S> {
        let inner: &mut CellInner<D> = unsafe { &mut *(self as *const _ as *mut _) };

        let id = TypeId::of::<S>();
        assert!(
            inner.log.iter().find(|x| **x == id).is_none(),
            "Tried to pull the same state twice: {}",
            any::type_name::<S>()
        );

        inner.log.push(id);
        inner.states.get(&id)?.as_any().downcast_ref()
    }

    pub fn get_mut<S: State>(&self) -> Option<&mut S> {
        let inner: &mut CellInner<D> = unsafe { &mut *(self as *const _ as *mut _) };

        let id = TypeId::of::<S>();
        assert!(
            inner.log.iter().find(|x| **x == id).is_none(),
            "Tried to pull the same state twice: {}",
            any::type_name::<S>()
        );

        inner.log.push(id);
        inner.states.get_mut(&id)?.as_any_mut().downcast_mut()
    }

    pub fn get_by_id(&self, id: &TypeId) -> Option<&dyn State<Data = D>> {
        let inner: &mut CellInner<D> = unsafe { &mut *(self as *const _ as *mut _) };

        assert!(
            inner.log.iter().find(|x| *x == id).is_none(),
            "Tried to pull the same state twice",
        );

        inner.log.push(id.clone());
        inner.states.get(id).map(|b| b.as_ref())
    }

    pub fn get_mut_by_id(&self, id: &TypeId) -> Option<&mut dyn State<Data = D>> {
        let inner: &mut CellInner<D> = unsafe { &mut *(self as *const _ as *mut _) };

        assert!(
            inner.log.iter().find(|x| *x == id).is_none(),
            "Tried to pull the same state twice",
        );

        inner.log.push(id.clone());
        inner.states.get_mut(id).map(|b| b.as_mut())
    }
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
    Insert(TypeId, Box<dyn State<Data = P>>),
    Pop,
    PopAndRemove,
    Push(TypeId),
}

impl<P> StateCommand<P> {
    pub fn insert<S: State<Data = P> + 'static + Sized>(state: S) -> Self {
        Self::Insert(TypeId::of::<S>(), Box::new(state))
    }
}

/// Stack-based finite state machine
#[derive(Debug)]
pub struct Fsm<P> {
    states: HashMap<TypeId, Box<dyn State<Data = P>>>,
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

impl<P: 'static> Fsm<P> {
    fn cell(states: &mut HashMap<TypeId, Box<dyn State<Data = P>>>) -> StateCell<P> {
        StateCell {
            inner: CellInner {
                states,
                log: Default::default(),
            },
        }
    }

    pub fn update(&mut self, params: &mut P) {
        loop {
            // TODO: maybe return error
            let id = self.stack.last().expect("No state in stack");

            let res = {
                let cell = Self::cell(&mut self.states);
                let state = cell.get_mut_by_id(id).unwrap();
                state.update(&cell, params)
            };

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
    pub fn insert<S: State<Data = P> + 'static + Sized>(
        &mut self,
        state: S,
    ) -> Option<Box<dyn State<Data = P>>> {
        self.states.insert(TypeId::of::<S>(), Box::new(state))
    }

    /// Inserts a state into the storage
    pub fn insert_default<S: State<Data = P> + 'static + Sized + Default>(
        &mut self,
    ) -> Option<Box<dyn State<Data = P>>> {
        self.states
            .insert(TypeId::of::<S>(), Box::new(S::default()))
    }

    /// Pushes an existing state to the stack
    pub fn push<S: State + Sized + 'static>(&mut self, params: &mut P) {
        let id = TypeId::of::<S>();
        self.push_id(id, params);
    }

    /// Pushes an existing state to the stack by type ID
    pub fn push_id(&mut self, id: TypeId, params: &mut P) {
        let cell = Self::cell(&mut self.states);

        if let Some(last_id) = self.stack.last() {
            let last = cell.get_mut_by_id(last_id).unwrap();
            last.on_stop(&cell, params);
        }

        let new = cell.get_mut_by_id(&id).unwrap();
        new.on_enter(&cell, params);

        self.stack.push(id);
    }

    /// Pushes a state from the stack
    pub fn pop(&mut self, params: &mut P) -> TypeId {
        let cell = Self::cell(&mut self.states);
        let last_id = self
            .stack
            .last()
            .expect("Tried to pop state but there's none!");

        let last = cell.get_mut_by_id(last_id).unwrap();
        last.on_exit(&cell, params);

        self.stack.pop().unwrap()
    }
}
