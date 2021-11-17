/*!
Stack-based finite state machine

[`Fsm`] is `HashMap<TypeId, Box<dyn State>>` + `VecDeque<TypeId>`. States can communiate with via [`StateCell`].
*/

// FIXME: Better StateReturn API

#[cfg(test)]
mod test;

use std::{
    any::{self, TypeId},
    cell::UnsafeCell,
    collections::HashMap,
};

use downcast_rs::*;
use smallvec::SmallVec;

pub type BoxState<D> = Box<dyn State<Data = D>>;

type States<D> = HashMap<TypeId, BoxState<D>>;

/// Game state lifecycle
pub trait State: std::fmt::Debug + Downcast {
    type Data;
    fn on_enter(&mut self, _god: &mut Self::Data, _cell: &StateCell<Self::Data>) {}
    fn on_exit(&mut self, _god: &mut Self::Data, _cell: &StateCell<Self::Data>) {}
    fn on_stop(&mut self, _god: &mut Self::Data, _cell: &StateCell<Self::Data>) {}
    fn event(&mut self, _god: &mut Self::Data, _cell: &StateCell<Self::Data>) {}
    fn update(
        &mut self,
        _god: &mut Self::Data,
        _cell: &StateCell<Self::Data>,
    ) -> StateReturn<Self::Data>;
}

impl_downcast!(State assoc Data);

/// Mutable access to multiple states. Panics when trying to borrow the same state twice (mutably
/// or immutably). TODO: Tolerant the rule to match to the Rust's standard.
#[derive(Debug)]
pub struct StateCell<'a, D> {
    inner: UnsafeCell<CellInner<'a, D>>,
}

impl<'a, D> StateCell<'a, D> {
    fn cast(states: &'a mut States<D>) -> Self {
        Self {
            inner: UnsafeCell::new(CellInner {
                states,
                log: Default::default(),
            }),
        }
    }
}

/// Given interior mutability by wrapeper
#[derive(Debug)]
struct CellInner<'a, D> {
    // hashmap
    states: &'a mut States<D>,
    log: SmallVec<[TypeId; 2]>,
}

impl<'a, D: 'static> StateCell<'a, D> {
    pub fn get<S: State<Data = D> + 'static + Sized>(&self) -> Option<&S> {
        // SAFE: StateCell is unique, not cloneable
        let inner: &mut CellInner<D> = unsafe { &mut *self.inner.get() };

        // access to the internal item must follow the aliasing rules:
        let id = TypeId::of::<S>();
        assert!(
            inner.log.iter().find(|x| **x == id).is_none(),
            "Tried to pull the same state twice: {}",
            any::type_name::<S>()
        );

        inner.log.push(id);
        inner
            .states
            .get(&id)
            .map(|state| state.as_any().downcast_ref().unwrap())
    }

    pub fn get_mut<S: State<Data = D> + 'static + Sized>(&self) -> Option<&mut S> {
        // SAFE: StateCell is unique, not cloneable
        let inner: &mut CellInner<D> = unsafe { &mut *self.inner.get() };

        // access to the internal item must follow the aliasing rules:
        let id = TypeId::of::<S>();
        assert!(
            inner.log.iter().find(|x| **x == id).is_none(),
            "Tried to pull the same state twice: {}",
            any::type_name::<S>()
        );

        inner.log.push(id);
        inner
            .states
            .get_mut(&id)
            .map(|state| state.downcast_mut().unwrap())
    }

    pub fn get_by_id(&self, id: &TypeId) -> Option<&dyn State<Data = D>> {
        // SAFE: StateCell is unique, not cloneable
        let inner: &mut CellInner<D> = unsafe { &mut *self.inner.get() };

        // access to the internal item must follow the aliasing rules:
        assert!(
            inner.log.iter().find(|x| *x == id).is_none(),
            "Tried to pull the same state twice",
        );

        inner.log.push(id.clone());
        inner.states.get(id).map(|b| b.as_ref())
    }

    pub fn get_mut_by_id(&self, id: &TypeId) -> Option<&mut dyn State<Data = D>> {
        // SAFE: StateCell is unique, not cloneable
        let inner: &mut CellInner<D> = unsafe { &mut *self.inner.get() };

        // access to the internal item must follow the aliasing rules:
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
pub enum StateReturn<D> {
    /// Run every command in this frame. Call update in next frame
    NextFrame(Vec<StateCommand<D>>),
    /// Run every command in this frame. Call update in this frame
    ThisFrame(Vec<StateCommand<D>>),
}

impl<D> StateReturn<D> {
    pub fn into_cmds(self) -> Vec<StateCommand<D>> {
        match self {
            Self::NextFrame(cmds) => cmds,
            Self::ThisFrame(cmds) => cmds,
        }
    }
}

/// Command in [`StateReturn`]
#[derive(Debug)]
pub enum StateCommand<D> {
    Insert(TypeId, BoxState<D>),
    Pop,
    PopAndRemove,
    Push(TypeId),
}

impl<D> StateCommand<D> {
    pub fn insert<S: State<Data = D> + 'static + Sized>(state: S) -> Self {
        Self::Insert(TypeId::of::<S>(), Box::new(state))
    }

    pub fn push<S: State<Data = D> + 'static + Sized>() -> Self {
        Self::Push(TypeId::of::<S>())
    }
}

/// Stack-based finite state machine
#[derive(Debug)]
pub struct Fsm<D> {
    states: States<D>,
    stack: Vec<TypeId>,
}

impl<D> Default for Fsm<D> {
    fn default() -> Self {
        Self {
            states: States::with_capacity(10),
            stack: Vec::with_capacity(10),
        }
    }
}

impl<D: 'static> Fsm<D> {
    pub fn update(&mut self, params: &mut D) {
        loop {
            // TODO: maybe return error
            let id = self.stack.last().expect("No state in stack");

            let res = {
                let cell = StateCell::cast(&mut self.states);
                let state = cell.get_mut_by_id(id).unwrap();
                state.update(params, &cell)
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

    fn run_cmd(&mut self, cmd: StateCommand<D>, params: &mut D) {
        match cmd {
            StateCommand::Insert(typeid, box_state) => {
                self.states.insert(typeid, box_state);
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
    pub fn insert<S: State<Data = D> + 'static + Sized>(
        &mut self,
        state: S,
    ) -> Option<BoxState<D>> {
        self.states.insert(TypeId::of::<S>(), Box::new(state))
    }

    /// Tries to retrieve the state of type `S` from the storage
    pub fn get<S: State<Data = D> + 'static + Sized>(&self) -> Option<&S> {
        self.states
            .get(&TypeId::of::<S>())
            .and_then(|b| b.downcast_ref())
    }

    /// Tries to retrieve the state of type `S` from the storage
    pub fn get_mut<S: State<Data = D> + 'static + Sized>(&mut self) -> Option<&mut S> {
        self.states
            .get_mut(&TypeId::of::<S>())
            .and_then(|b| b.downcast_mut())
    }

    /// Pushes an existing state to the stack
    pub fn push<S: State + Sized + 'static>(&mut self, params: &mut D) {
        let id = TypeId::of::<S>();
        self.push_id(id, params);
    }

    /// Pushes an existing state to the stack by type ID
    pub fn push_id(&mut self, id: TypeId, data: &mut D) {
        let cell = StateCell::cast(&mut self.states);

        if let Some(last_id) = self.stack.last() {
            let last = cell.get_mut_by_id(last_id).unwrap();
            last.on_stop(data, &cell);
        }

        let new = cell
            .get_mut_by_id(&id)
            .expect("Unable to find pushed type in storage");
        new.on_enter(data, &cell);

        self.stack.push(id);
    }

    /// Pushes a state from the stack
    pub fn pop(&mut self, data: &mut D) -> TypeId {
        let cell = StateCell::cast(&mut self.states);
        let last_id = self
            .stack
            .last()
            .expect("Tried to pop state but there's none!");

        let last = cell.get_mut_by_id(last_id).unwrap();
        last.on_exit(data, &cell);

        self.stack.pop().unwrap()
    }
}
