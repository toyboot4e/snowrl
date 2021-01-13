/*!

Turn-based game loop implemented with generator in unstable Rust

# Borrow rules

It _seems_ like a [generator] holds the value passed to it when they `resume` and we basically have
to use `Rc<RefCell<T>>` for our game data. However, it is uncomfortable and I'm using a pointer to
cheat the borrow rules ([`rlbox::utils::Cheat`]).

[generator]: (https://doc.rust-lang.org/beta/unstable-book/language-features/generators.html)

*/

use std::{
    fmt,
    ops::{Generator, GeneratorState},
    pin::Pin,
    rc::Rc,
};

use downcast_rs::{impl_downcast, Downcast};

use rlbox::utils::Cheat;

use crate::{
    turn::{anim::Anim, ev},
    world::{World, WorldContext},
};

/// Boxed [generator]
///
/// [gemerator]: (https://doc.rust-lang.org/beta/unstable-book/language-features/generators.html)
///
/// It was hard to `resume` with lifetimed parameters so, we'll cheat the borrow rules using a
/// pointer.
type Gen = Box<dyn Generator<TickContext, Yield = TickResult, Return = ()> + Unpin>;

#[derive(Debug, Clone)]
struct TickContext {
    world: Cheat<World>,
    wcx: Cheat<WorldContext>,
}

// do we actually need it?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActorIx(pub usize);

#[derive(Debug)]
pub enum TickResult {
    /// Yielded when an actor takes turn
    TakeTurn(ActorIx),
    /// Yielded when a new [`Event`] is emitted
    Event(Rc<dyn Event>),
    /// Yielded when processing a command takes greater than or equal to one frame
    ProcessingEvent,
}

/// Roguelike game loop
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
            tcx: TickContext {
                world: Cheat::empty(),
                wcx: Cheat::empty(),
            },
        }
    }
}

impl GameLoop {
    /// Ticks the game for "one step"
    pub fn tick(&mut self, world: &mut World, wcx: &mut WorldContext) -> TickResult {
        // set cheat borrows here (for the generators)
        self.tcx.world = Cheat::new(world);
        self.tcx.wcx = Cheat::new(wcx);

        match Pin::new(&mut self.gen).resume(self.tcx.clone()) {
            GeneratorState::Yielded(res) => res,
            _ => panic!("unexpected value from resume"),
        }
    }
}

/// Internal game loop implemented as a generator
fn game_loop() -> Gen {
    Box::new(|mut tcx: TickContext| {
        let mut actor_ix = 0;

        loop {
            let actor = ActorIx(actor_ix);
            yield TickResult::TakeTurn(actor);

            // TODO: do not hard code entity actions
            let mut ev: Rc<dyn Event> = match actor.0 {
                crate::consts::PLAYER => Rc::new(ev::PlayerTurn { actor }),
                _ => Rc::new(ev::RandomWalk { actor }),
            };

            // process command
            yield TickResult::Event(ev.clone());

            loop {
                let mut ecx = EventContext {
                    world: &mut tcx.world,
                    wcx: &mut tcx.wcx,
                };

                match ev.run(&mut ecx) {
                    EventResult::GotoNextFrame => {
                        // wait for next frame
                        yield TickResult::ProcessingEvent;
                        continue;
                    }
                    EventResult::Finish => {
                        // go to next actor
                        actor_ix += 1;
                        actor_ix %= tcx.world.entities.len();
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

/// Context for making animation
#[derive(Debug)]
pub struct AnimContext<'a, 'b> {
    pub world: &'a mut World,
    pub wcx: &'b mut WorldContext,
}

pub trait GenAnim {
    fn gen_anim(&self, _acx: &mut AnimContext) -> Option<Box<dyn Anim>> {
        None
    }
}

// --------------------------------------------------------------------------------
// Event

/// Context for event handling, both internals ang GUI
#[derive(Debug)]
pub struct EventContext<'a, 'b> {
    pub world: &'a mut World,
    pub wcx: &'b mut WorldContext,
}

/// Return value of event handling
#[derive(Debug)]
pub enum EventResult {
    /// Interactive actions can take multiple frames returning this varient
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
    fn run(&self, ecx: &mut EventContext) -> EventResult;
}

impl_downcast!(Event);

// TODO: do we need them? or `get_mut` might make sense
// `impl Trait for Box<dyn trait>`

impl<T: GenAnim + ?Sized> GenAnim for Box<T> {}

impl<T: Event + ?Sized> Event for Box<T> {
    fn run(&self, ecx: &mut EventContext) -> EventResult {
        (**self).run(ecx)
    }
}

impl<T: GenAnim + ?Sized> GenAnim for Rc<T> {}

impl<T: Event + ?Sized> Event for Rc<T> {
    fn run(&self, ecx: &mut EventContext) -> EventResult {
        (**self).run(ecx)
    }
}
