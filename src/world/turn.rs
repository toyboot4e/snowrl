//! Turn-based game loop

use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
};

use crate::{
    utils::Cheat,
    world::{World, WorldContext},
};

/// [Generator](https://doc.rust-lang.org/beta/unstable-book/language-features/generators.html)
///
/// It was hard to `resume` with lifetimed parameters so, we'll cheat the borrow rules using a
/// pointer.
type Gen = Box<dyn Generator<Yield = TickResult, Return = ()> + Unpin>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TickResult {
    TakeTurn,
}

pub type GameLoop = Box<GameLoopImpl>;

/// Roguelike game loop
pub struct GameLoopImpl {
    /// Stack of generators
    gen_stack: Vec<Gen>,
    /// Currently processing actor
    actor_ix: usize,
    world: Cheat<World>,
    wcx: Cheat<WorldContext>,
}

impl GameLoopImpl {
    /// Arguments are boxed to have fixed memory position
    pub fn new(world: &Box<World>, wcx: &Box<WorldContext>) -> GameLoop {
        // give fixed memory position
        let mut me = Box::new(Self {
            gen_stack: Vec::with_capacity(10),
            actor_ix: 0,
            world: Cheat::new(world),
            wcx: Cheat::new(wcx),
        });

        let ptr: *mut Self = (&*me) as *const Self as *mut Self;
        me.gen_stack.push(unsafe { Self::game_loop(ptr) });

        me
    }

    /// Ticks the game for "one step"
    ///
    /// `World` and `WorldContext` are semantic parameters and not actually used.
    pub fn tick(&mut self, _world: &mut World, _wcx: &mut WorldContext) -> TickResult {
        // // set cheat borrows here (for the generators)
        // self.world = Cheat::new(world);
        // self.wcx = Cheat::new(wcx);

        let gen = self.gen_stack.last_mut().expect("generator stack is null!");

        match Pin::new(gen).resume(()) {
            GeneratorState::Yielded(res) => res,
            _ => panic!("unexpected value from resume"),
        }
    }

    /// Internal game loop implemented as a generator
    ///
    /// The pointer has to be valid for a while.
    unsafe fn game_loop(ptr: *mut Self) -> Gen {
        Box::new(move || {
            use crate::ev::{ActorIndex, PlayerTurn};

            // get illegally create mutable borrow to self = tick context
            let tcx = { &mut *ptr }; // unsafe

            loop {
                // TODO: decide command depending on actor
                let mut cmd = PlayerTurn {
                    actor: ActorIndex(tcx.actor_ix),
                };

                let mut ccx = CommandContext {
                    world: &mut tcx.world,
                    wcx: &mut tcx.wcx,
                };

                match Self::run_cmd(&mut ccx, &mut cmd) {
                    // TODO: allow interactive/blocking command
                    CommandResult::Continue => {
                        // wait for next frame
                    }
                    CommandResult::Finish => {
                        // process next actor
                    }
                    CommandResult::Chain(cmd) => {
                        unreachable!()
                    }
                }

                yield TickResult::TakeTurn;
            }
        })
    }

    // TODO: allow nest
    /// Runs command recursively
    fn run_cmd(ccx: &mut CommandContext, cmd: &mut dyn Command) -> CommandResult {
        let res = cmd.run(ccx);
        if let CommandResult::Chain(mut cmd) = res {
            Self::run_cmd(ccx, &mut cmd)
        } else {
            res
        }
    }

    // fn process_action() -> GameLoop {
    // }
}

/// Bindings for any command to process
pub struct CommandContext<'a, 'b> {
    pub world: &'a mut World,
    pub wcx: &'b mut WorldContext,
}

/// Return value of command processing
pub enum CommandResult {
    /// Interactive actions can take multiple frames returning this varient
    Continue,
    Finish,
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
pub trait Command {
    fn run(&mut self, ccx: &mut CommandContext) -> CommandResult;
}

/// impl `Command` for `Box<dyn Command>`
impl<T: Command + ?Sized> Command for Box<T> {
    fn run(&mut self, ccx: &mut CommandContext) -> CommandResult {
        (**self).run(ccx)
    }
}
