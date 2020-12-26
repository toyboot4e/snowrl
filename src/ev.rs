//! Event-based roguelike game system

use rlbox::rl::grid2d::*;

use xdl::Key;

use crate::world::{
    turn::{Command, CommandContext, CommandResult},
    World, WorldContext,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActorIndex(pub usize);

// --------------------------------------------------------------------------------
// Primitive events

// Every change to the roguelike game should happen as a primitive event.
// These additional steps are also good foor visualization.

#[derive(Debug)]
pub struct ChangeDir {
    pub actor: ActorIndex,
    pub dir: Dir8,
}

impl Command for ChangeDir {
    fn run(&self, ccx: &mut CommandContext) -> CommandResult {
        let actor = &mut ccx.world.entities[self.actor.0];
        actor.dir = self.dir;

        CommandResult::Finish
    }
}

#[derive(Debug)]
pub struct Move {
    pub actor: ActorIndex,
    pub mcx: MoveContext,
    pub from_pos: Vec2i,
    pub from_dir: Dir8,
    pub to_pos: Vec2i,
    pub to_dir: Dir8,
}

#[derive(Debug)]
pub enum MoveContext {
    Teleport,
    Walk,
}

impl Command for Move {
    fn run(&self, ccx: &mut CommandContext) -> CommandResult {
        let actor = &mut ccx.world.entities[self.actor.0];
        actor.dir = self.to_dir;
        actor.pos = self.to_pos;

        // TODO: update FoV. mark dirty subscribing walk event
        // wcx.fov_render.before_update_fov(&player.fov);

        // TODO: update FoV. mark dirty subscribing walk event
        // self::update_fov(
        //     &mut player.fov,
        //     player.pos,
        //     crate::consts::FOV_R,
        //     &world.map.rlmap,
        // );

        CommandResult::Finish
    }
}

// --------------------------------------------------------------------------------
// Events

#[derive(Debug)]
pub struct Attack {
    pub actor: ActorIndex,
    pub dir: Dir8,
}

#[derive(Debug)]
pub struct Walk {
    pub actor: ActorIndex,
    pub dir: Dir8,
}

// --------------------------------------------------------------------------------
// Higher-level events

/// Interactive command for player input
#[derive(Debug)]
pub struct PlayerTurn {
    pub actor: ActorIndex,
}

impl Command for PlayerTurn {
    fn run(&self, ccx: &mut CommandContext) -> CommandResult {
        if let Some(dir) = ccx.wcx.vi.dir.to_dir8() {
            CommandResult::chain(Walk {
                actor: self.actor,
                dir,
            })
        } else {
            CommandResult::Continue
        }
    }
}

#[derive(Debug)]
pub struct RandomWalk {
    pub actor: ActorIndex,
}

impl Command for RandomWalk {
    fn run(&self, _ccx: &mut CommandContext) -> CommandResult {
        let dir = {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            Dir8::CLOCKWISE[rng.gen_range(0..8)]
        };

        CommandResult::chain(Walk {
            actor: self.actor,
            dir,
        })
    }
}

impl Command for Walk {
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
            return CommandResult::chain(ChangeDir {
                actor: self.actor,
                dir: self.dir,
            });
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
