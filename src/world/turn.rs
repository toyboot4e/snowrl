//! Turn-based game loop

use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
};

use crate::{utils::Cheat, world::World};

type Gen = Box<dyn Generator<TickContext, Yield = TickResult, Return = ()>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TickResult {
    TakeTurn,
}

pub struct TickContext {
    world: Cheat<World>,
}

pub struct GameLoop {
    gen_stack: Vec<Gen>,
}

impl GameLoop {
    pub fn new() -> Self {
        let mut stack = Vec::with_capacity(10);
        stack.push(Self::game_loop());

        Self { gen_stack: stack }
    }

    fn game_loop() -> Gen {
        Box::new(|tcx: TickContext| {
            let tcx = yield TickResult::TakeTurn;
            unreachable!()
        })
    }

    pub fn tick(&mut self, tcx: TickContext) -> TickResult {
        TickResult::TakeTurn
    }

    // fn process_action() -> GameLoop {
    // }
}
