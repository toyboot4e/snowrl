//! Turn-based game loop

use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
};

use crate::{utils::Cheat, world::World};

/// [Generator](https://doc.rust-lang.org/beta/unstable-book/language-features/generators.html)
///
/// It was hard to `resume` with lifetimed parameters so we'll cheat the borrow rules using a
/// pointer.
type Gen = Box<dyn Generator<Yield = TickResult, Return = ()> + Unpin>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TickResult {
    TakeTurn,
}

pub struct GameLoop {
    world: Cheat<World>,
    gen_stack: Vec<Gen>,
}

struct TickContext {
    world: Cheat<World>,
}

impl GameLoop {
    pub fn new() -> Self {
        let mut me = Self {
            world: Cheat::empty(),
            gen_stack: Vec::with_capacity(10),
        };

        let ptr: *mut Self = &me as *const _ as *mut _;
        me.gen_stack.push(Self::game_loop(ptr));

        me
    }

    /// Internal game loop implemented as a generator
    fn game_loop(ptr: *mut Self) -> Gen {
        Box::new(move || {
            // illegally create mutable borrow to self = tick context
            let tcx = unsafe { &mut *ptr };

            yield TickResult::TakeTurn;
            unreachable!()
        })
    }

    pub fn tick(&mut self) -> TickResult {
        let gen = self.gen_stack.last_mut().expect("generator stack is null!");

        match Pin::new(gen).resume(()) {
            GeneratorState::Yielded(res) => res,
            _ => panic!("unexpected value from resume"),
        }
    }

    // fn process_action() -> GameLoop {
    // }
}
