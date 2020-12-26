//! Turn-based game loop

use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
    rc::Rc,
};

use crate::{
    ev::{self, ActorIndex},
    utils::Cheat,
    world::{World, WorldContext},
};

/// [Generator](https://doc.rust-lang.org/beta/unstable-book/language-features/generators.html)
///
/// It was hard to `resume` with lifetimed parameters so, we'll cheat the borrow rules using a
/// pointer.
type Gen = Box<dyn Generator<TickContext, Yield = TickResult, Return = ()> + Unpin>;

#[derive(Clone)]
struct TickContext {
    world: Cheat<World>,
    wcx: Cheat<WorldContext>,
}

// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TickResult {
    TakeTurn,
    BeforeCommand(Rc<dyn Command>),
    ProcessingCommand,
}

pub type GameLoop = Box<GameLoopImpl>;

/// Roguelike game loop
pub struct GameLoopImpl {
    gen_stack: Vec<Gen>,
    tcx: TickContext,
}

impl GameLoopImpl {
    /// Arguments are boxed to have fixed memory position
    pub fn new() -> GameLoop {
        let mut me = Box::new(Self {
            gen_stack: Vec::with_capacity(10),
            tcx: TickContext {
                world: Cheat::empty(),
                wcx: Cheat::empty(),
            },
        });

        me.gen_stack.push(unsafe { Self::game_loop() });

        me
    }

    /// Ticks the game for "one step"
    pub fn tick(&mut self, world: &mut World, wcx: &mut WorldContext) -> TickResult {
        // // set cheat borrows here (for the generators)
        self.tcx.world = Cheat::new(world);
        self.tcx.wcx = Cheat::new(wcx);

        let gen = self.gen_stack.last_mut().expect("generator stack is null!");

        match Pin::new(gen).resume(self.tcx.clone()) {
            GeneratorState::Yielded(res) => res,
            _ => panic!("unexpected value from resume"),
        }
    }

    /// Internal game loop implemented as a generator
    unsafe fn game_loop() -> Gen {
        Box::new(|mut tcx: TickContext| {
            // TODO: separate TickState so that it can be observed
            let mut actor_ix = 0;

            loop {
                // TODO: do not hard code entity actions
                let actor = ActorIndex(actor_ix);

                let cmd: Rc<dyn Command> = match actor.0 {
                    0 => Rc::new(ev::PlayerTurn { actor }),
                    _ => Rc::new(ev::RandomWalk { actor }),
                };

                let mut ccx = CommandContext {
                    world: &mut tcx.world,
                    wcx: &mut tcx.wcx,
                };

                match Self::run_cmd(&mut ccx, &cmd) {
                    CommandResult::Continue => {
                        // wait for next frame
                        yield TickResult::ProcessingCommand;
                    }
                    CommandResult::Finish => {
                        yield TickResult::TakeTurn;

                        // go to next actor
                        actor_ix += 1;
                        actor_ix %= tcx.world.entities.len();
                    }
                    CommandResult::Chain(_) => {
                        unreachable!()
                    }
                }

                // loop
            }
        })
    }

    // TODO: allow nest and interactive command
    /// Runs a command recursively if it chains
    fn run_cmd(ccx: &mut CommandContext, cmd: &dyn Command) -> CommandResult {
        let res = cmd.run(ccx);
        if let CommandResult::Chain(cmd) = res {
            Self::run_cmd(ccx, &cmd)
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
    fn run(&self, ccx: &mut CommandContext) -> CommandResult;
}

/// impl `Command` for `Box<dyn Command>`
impl<T: Command + ?Sized> Command for Box<T> {
    fn run(&self, ccx: &mut CommandContext) -> CommandResult {
        (**self).run(ccx)
    }
}

/// impl `Command` for `Rc<dyn Command>`
impl<T: Command + ?Sized> Command for Rc<T> {
    fn run(&self, ccx: &mut CommandContext) -> CommandResult {
        (**self).run(ccx)
    }
}
