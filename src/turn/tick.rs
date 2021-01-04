/*!

Turn-based game loop implemented with generator in unstable Rust

*/

use std::{
    fmt,
    ops::{Generator, GeneratorState},
    pin::Pin,
    rc::Rc,
};

use crate::{
    turn::{anim::Anim, ev},
    utils::Cheat,
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
pub struct ActorIndex(pub usize);

#[derive(Debug)]
pub enum TickResult {
    /// Yielded when an actor takes turn
    TakeTurn(ActorIndex),
    /// Yielded when a new command is emitted
    Command(Rc<dyn Command>),
    /// Yielded when processing a command takes greater than or equal to one frame
    ProcessingCommand,
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
        // TODO: separate TickState so that it can be observed
        let mut actor_ix = 0;

        loop {
            // TODO: do not hard code entity actions
            let actor = ActorIndex(actor_ix);
            yield TickResult::TakeTurn(actor);

            let mut cmd: Rc<dyn Command> = match actor.0 {
                0 => Rc::new(ev::PlayerTurn { actor }),
                _ => Rc::new(ev::RandomWalk { actor }),
            };

            // process command
            loop {
                yield TickResult::Command(cmd.clone());

                let mut ccx = CommandContext {
                    world: &mut tcx.world,
                    wcx: &mut tcx.wcx,
                };

                match cmd.run(&mut ccx) {
                    CommandResult::GotoNextFrame => {
                        // wait for next frame
                        yield TickResult::ProcessingCommand;
                        break;
                    }
                    CommandResult::Finish => {
                        // go to next actor
                        actor_ix += 1;
                        actor_ix %= tcx.world.entities.len();
                        break;
                    }
                    CommandResult::Chain(new_cmd) => {
                        cmd = new_cmd.into();
                        continue;
                    }
                }
            }

            // next step for the turn-based game loop
        }
    })
}

// ----------------------------------------
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

// ----------------------------------------
// Command

/// Context for any command to process
#[derive(Debug)]
pub struct CommandContext<'a, 'b> {
    pub world: &'a mut World,
    pub wcx: &'b mut WorldContext,
}

/// Return value of command processing
#[derive(Debug)]
pub enum CommandResult {
    /// Interactive actions can take multiple frames returning this varient
    GotoNextFrame,
    Finish,
    // TODO: chain multiple commands
    Chain(Box<dyn Command>),
}

impl CommandResult {
    pub fn chain<T: Command + 'static>(cmd: T) -> Self {
        Self::Chain(Box::new(cmd))
    }
}

/// Special event that can run itself
///
/// TODO: prefer chain-of-responsibility pattern
pub trait Command: fmt::Debug + GenAnim {
    fn run(&self, ccx: &mut CommandContext) -> CommandResult;
}

// TODO: do we need them? or `get_mut` might make sense
// `impl Trait for Box<dyn trait>`

impl<T: GenAnim + ?Sized> GenAnim for Box<T> {}

impl<T: Command + ?Sized> Command for Box<T> {
    fn run(&self, ccx: &mut CommandContext) -> CommandResult {
        (**self).run(ccx)
    }
}

impl<T: GenAnim + ?Sized> GenAnim for Rc<T> {}

impl<T: Command + ?Sized> Command for Rc<T> {
    fn run(&self, ccx: &mut CommandContext) -> CommandResult {
        (**self).run(ccx)
    }
}
