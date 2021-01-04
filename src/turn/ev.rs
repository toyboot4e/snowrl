/*!

Roguelike game events

Most actor actions result in primitive events. Every change to the roguelike game world should be
handled as primitive event. That's good for both flexibility and visualization.

e.g. `MeleeAttack` -> `Attack` -> `Hit` -> `GiveDamage`

*/

use rlbox::rl::grid2d::*;

use xdl::Key;

use crate::turn::{
    anim::{self, Anim},
    tick::{ActorIndex, AnimContext, Command, CommandContext, CommandResult, GenAnim},
};

/// Some action resulted in a non-turn consuming action
///
/// Player should take another turn.
///
/// FIXME: unintentional side effects
#[derive(Debug)]
pub struct NotConsumeTurn {
    pub actor: ActorIndex,
}

impl GenAnim for NotConsumeTurn {
    fn gen_anim(&self, _acx: &mut AnimContext) -> Option<Box<dyn Anim>> {
        if self.actor.0 == 0 {
            // wait for one frame so that we won't enter inifinite loop
            Some(Box::new(anim::Wait { frames: 1 }))
        } else {
            None
        }
    }
}

impl Command for NotConsumeTurn {
    fn run(&self, _ccx: &mut CommandContext) -> CommandResult {
        if self.actor.0 == 0 {
            // TODO: require one frame wait
            CommandResult::chain(PlayerTurn { actor: self.actor })
        } else {
            CommandResult::Finish
        }
    }
}

// --------------------------------------------------------------------------------
// Primitive events

// Every change to the roguelike game should happen as a primitive event.
// These additional steps are also good foor visualization.

#[derive(Debug)]
pub struct ChangeDir {
    pub actor: ActorIndex,
    pub dir: Dir8,
}

impl GenAnim for ChangeDir {
    fn gen_anim(&self, _acx: &mut AnimContext) -> Option<Box<dyn Anim>> {
        // TODO: play rotation and wait for it to finish
        None
    }
}

impl Command for ChangeDir {
    fn run(&self, ccx: &mut CommandContext) -> CommandResult {
        let actor = &mut ccx.world.entities[self.actor.0];
        actor.dir = self.dir;

        CommandResult::chain(NotConsumeTurn { actor: self.actor })
    }
}

/// Walk | Teleport
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveContext {
    Teleport,
    Walk,
}

/// Change in actor's position and direction
#[derive(Debug)]
pub struct Move {
    pub actor: ActorIndex,
    pub mcx: MoveContext,
    pub from_pos: Vec2i,
    pub from_dir: Dir8,
    pub to_pos: Vec2i,
    pub to_dir: Dir8,
}

impl GenAnim for Move {
    fn gen_anim(&self, _acx: &mut AnimContext) -> Option<Box<dyn Anim>> {
        Some(Box::new(anim::WalkAnim::new(self.actor)))
    }
}

impl Command for Move {
    fn run(&self, ccx: &mut CommandContext) -> CommandResult {
        if !ccx.world.is_blocked(self.to_pos) {
            let actor = &mut ccx.world.entities[self.actor.0];
            actor.dir = self.to_dir;
            actor.pos = self.to_pos;
            CommandResult::Finish
        } else {
            CommandResult::chain(ChangeDir {
                actor: self.actor,
                dir: self.to_dir,
            })
        }
    }
}

// --------------------------------------------------------------------------------
// Higher-level commands

/// Attack in direction
#[derive(Debug)]
pub struct Attack {
    pub actor: ActorIndex,
    pub dir: Dir8,
}

impl GenAnim for Attack {}

#[derive(Debug)]
pub struct RandomWalk {
    pub actor: ActorIndex,
}

impl GenAnim for RandomWalk {}

impl Command for RandomWalk {
    fn run(&self, _ccx: &mut CommandContext) -> CommandResult {
        let dir = {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            Dir8::CLOCKWISE[rng.gen_range(0..8)]
        };

        CommandResult::chain(PlayerWalk {
            actor: self.actor,
            dir,
        })
    }
}

// --------------------------------------------------------------------------------
// Player control

/// Walk or change direction and chain [`PlayerTurn`]
#[derive(Debug)]
pub struct PlayerWalk {
    pub actor: ActorIndex,
    pub dir: Dir8,
}

impl GenAnim for PlayerWalk {}

impl Command for PlayerWalk {
    fn run(&self, ccx: &mut CommandContext) -> CommandResult {
        let CommandContext { world, wcx } = ccx;

        let actor = &mut world.entities[self.actor.0];
        let pos = actor.pos + Vec2i::from(self.dir.signs_i32());
        drop(actor);

        let is_rotate_only = wcx
            .input
            .kbd
            .is_any_key_down(&[Key::LeftShift, Key::RightShift]);

        if is_rotate_only || world.is_blocked(pos) {
            // TODO: change direction without consuming turn
            CommandResult::chain(ChangeDir {
                actor: self.actor,
                dir: self.dir,
            })
        } else {
            let actor = &world.entities[self.actor.0];

            CommandResult::chain(Move {
                actor: self.actor,
                mcx: MoveContext::Walk,
                from_pos: actor.pos,
                to_pos: pos,
                from_dir: actor.dir,
                to_dir: self.dir,
            })
        }
    }
}

/// Interactive command for player input
#[derive(Debug)]
pub struct PlayerTurn {
    pub actor: ActorIndex,
}

impl GenAnim for PlayerTurn {}

impl Command for PlayerTurn {
    fn run(&self, ccx: &mut CommandContext) -> CommandResult {
        if let Some(dir) = ccx.wcx.vi.dir.dir8_down() {
            CommandResult::chain(PlayerWalk {
                actor: self.actor,
                dir,
            })
        } else {
            CommandResult::GotoNextFrame
        }
    }
}
