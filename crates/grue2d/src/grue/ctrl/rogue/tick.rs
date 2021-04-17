/*!

Turn-based game loop implemented with generator in unstable Rust

*/

use std::{
    fmt,
    ops::{Generator, GeneratorState},
    pin::Pin,
    rc::Rc,
};

use {
    downcast_rs::{impl_downcast, Downcast},
    snow2d::utils::{arena::Index, Cheat},
};

use crate::grue::{
    ctrl::rogue::{anim::Anim, ev},
    data::world::actor::Actor,
    Data,
};

/// TODO: remove me
const PLAYER: usize = 0;

/// Boxed [generator]
///
/// [gemerator]: (https://doc.rust-lang.org/beta/unstable-book/language-features/generators.html)
///
/// It was hard to `resume` with lifetimed parameters so, we'll cheat the borrow rules using a
/// pointer.
type Gen = Box<dyn Generator<TickContext, Yield = TickResult, Return = ()> + Unpin>;

type TickContext = Cheat<crate::Data>;

/// Return value of [`GameLoop::tick`]
#[derive(Debug)]
pub enum TickResult {
    /// Yielded when an actor takes turn
    TakeTurn(Index<Actor>),
    /// Yielded when a new [`Event`] is emitted
    Event(Rc<dyn Event>),
    /// Yielded when processing a command takes greater than or equal to one frame
    ProcessingEvent,
}

/// Roguelike game loop
///
/// Internally, it's using [generator] (unstable Rust).
///
/// [generator]: (https://doc.rust-lang.org/beta/unstable-book/language-features/generators.html)
///
/// # Internals: cheating the borrow checker
///
/// Generator _holds_ the value passed to it on `resume`, and we have to _share_ data with generator,
/// but [`GameLoop::tick`] is not lying about lifetimes of the parameters; the generator uses them
/// only while it's running. Therefore, cheating the borrow checker with pointers so that we can
/// relax the restriction to the ordinary borrow rules in rust.
pub struct GameLoop {
    gen: Gen,
    tcx: TickContext,
}

impl std::fmt::Debug for GameLoop {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "GameLoopImpl {{ gen: <cant print for now>, tcx: {:?} }}",
            self.tcx,
        )
    }
}

impl Default for GameLoop {
    fn default() -> Self {
        Self {
            gen: self::game_loop(),
            tcx: unsafe { Cheat::null() },
        }
    }
}

impl GameLoop {
    /// Ticks the game for "one step"
    pub fn tick(&mut self, data: &mut Data) -> TickResult {
        // set cheat borrows here for the generator
        unsafe {
            self.tcx = Cheat::new(data);
        }

        match Pin::new(&mut self.gen).resume(self.tcx.clone()) {
            GeneratorState::Yielded(res) => res,
            _ => panic!("unexpected value from resume"),
        }
    }
}

/// Internal game loop implemented as a generator
fn game_loop() -> Gen {
    Box::new(|tcx: TickContext| {
        let mut actor_slot = 0;

        loop {
            let (actor_index, _) = match tcx.world.entities.get_by_slot(actor_slot) {
                Some(index) => index,
                None => continue,
            };

            yield TickResult::TakeTurn(actor_index);

            // TODO: do not hard code entity actions
            let mut ev: Rc<dyn Event> = match actor_index.slot() as usize {
                PLAYER => Rc::new(ev::PlayerTurn { actor: actor_index }),
                _ => Rc::new(ev::RandomWalk { actor: actor_index }),
            };

            // process command
            yield TickResult::Event(ev.clone());

            loop {
                match ev.run(tcx.as_mut()) {
                    EventResult::GotoNextFrame => {
                        // wait for next frame
                        yield TickResult::ProcessingEvent;
                        continue;
                    }
                    EventResult::Finish => {
                        // go to next actor
                        actor_slot += 1;
                        actor_slot %= tcx.world.entities.len() as u32;
                        break;
                    }
                    EventResult::Chain(new_ev) => {
                        ev = new_ev.into();
                        yield TickResult::Event(ev.clone());
                        continue;
                    }
                }
            }

            // next step for the turn-based game loop
        }
    })
}

// --------------------------------------------------------------------------------
// Animation

/// TODO: generate animations externally
pub trait GenAnim {
    fn gen_anim(&self, _data: &mut Data) -> Option<Box<dyn Anim>> {
        None
    }
}

// --------------------------------------------------------------------------------
// Event

/// Return value of event handling
#[derive(Debug)]
pub enum EventResult {
    /// Need another frame to process this action (often interactive actions)
    GotoNextFrame,
    Finish,
    Chain(Box<dyn Event>),
}

impl EventResult {
    pub fn chain<T: Event + 'static>(ev: T) -> Self {
        Self::Chain(Box::new(ev))
    }
}

/// TODO: prefer chain-of-responsibility pattern
pub trait Event: fmt::Debug + Downcast + GenAnim {
    fn run(&self, ecx: &mut Data) -> EventResult;
}

impl_downcast!(Event);

// TODO: do we need them? or `get_mut` might make sense
// `impl Trait for Box<dyn trait>`

impl<T: GenAnim + ?Sized> GenAnim for Box<T> {}

impl<T: Event + ?Sized> Event for Box<T> {
    fn run(&self, ecx: &mut Data) -> EventResult {
        (**self).run(ecx)
    }
}

impl<T: GenAnim + ?Sized> GenAnim for Rc<T> {}

impl<T: Event + ?Sized> Event for Rc<T> {
    fn run(&self, ecx: &mut Data) -> EventResult {
        (**self).run(ecx)
    }
}
