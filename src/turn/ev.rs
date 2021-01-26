/*!
Roguelike game events

Every change to the roguelike game world should be handled as primitive event. That's good for
visualization, flexibilities and simplicities.

e.g. `MeleeAttack` -> `Attack` -> `Hit` -> `GiveDamage`
*/

use rlbox::rl::grid2d::*;

use crate::{
    turn::{
        anim::{self, Anim},
        tick::{ActorIx, AnimContext, Event, EventContext, EventResult, GenAnim},
    },
    utils::consts,
};

// --------------------------------------------------------------------------------
// Interactive commands

#[derive(Debug)]
pub struct Talk {
    pub from: ActorIx,
    pub to: ActorIx,
}

impl GenAnim for Talk {}

impl Event for Talk {
    /// [`Talk`] event should be handled exterally by GUI
    fn run(&self, _ecx: &mut EventContext) -> EventResult {
        EventResult::Finish
    }
}

// --------------------------------------------------------------------------------
// Player commands

/// Walk or change direction and chain [`PlayerTurn`]
#[derive(Debug)]
pub struct PlayerWalk {
    pub actor: ActorIx,
    pub dir: Dir8,
}

impl GenAnim for PlayerWalk {}

impl Event for PlayerWalk {
    fn run(&self, ecx: &mut EventContext) -> EventResult {
        let EventContext { world, vi } = ecx;

        let actor = &mut world.entities[self.actor.0];
        let pos = actor.pos + Vec2i::from(self.dir.signs_i32());
        drop(actor);

        let is_rotate_only = vi.turn.is_down();

        if is_rotate_only || world.is_blocked(pos) {
            EventResult::chain(ChangeDir {
                actor: self.actor,
                dir: self.dir,
            })
        } else {
            let actor = &world.entities[self.actor.0];

            EventResult::chain(Move {
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

// TODO: impl Interact delegating the process to FSM
#[derive(Debug)]
pub struct Interact {
    pub actor: ActorIx,
    pub dir: Dir8,
}

impl GenAnim for Interact {}

impl Event for Interact {
    fn run(&self, ecx: &mut EventContext) -> EventResult {
        let actor = &ecx.world.entities[self.actor.0];
        let pos = actor.pos + Vec2i::from(self.dir);

        if let Some(target) = ecx
            .world
            .entities
            .iter()
            .enumerate()
            .find(|(_i, e)| e.pos == pos)
            .map(|(i, _e)| i)
        {
            EventResult::chain(Talk {
                from: self.actor,
                to: ActorIx(target),
            })
        } else {
            EventResult::chain(NotConsumeTurn { actor: self.actor })
        }
    }
}

// --------------------------------------------------------------------------------
// Entity control

/// Interactive command for player input
#[derive(Debug)]
pub struct PlayerTurn {
    pub actor: ActorIx,
}

impl GenAnim for PlayerTurn {}

impl PlayerTurn {
    /// Find he only actor that is at an adjacent cell to the controlled actor
    fn find_only_neighbor(&self, ecx: &EventContext) -> Option<Dir8> {
        let mut res = Option::<Dir8>::None;

        let origin = ecx.world.entities[self.actor.0].pos;
        for e in &ecx.world.entities {
            let dvec = e.pos - origin;
            if dvec.len_king() != 1 {
                continue;
            }

            if res.is_some() {
                return None;
            }

            res = Dir8::from_signs([Sign::from_i32(dvec.x), Sign::from_i32(dvec.y)]);
        }

        res
    }
}

impl Event for PlayerTurn {
    fn run(&self, ecx: &mut EventContext) -> EventResult {
        let (select, turn, rest, dir) = (
            ecx.vi.select.is_pressed(),
            ecx.vi.turn.is_pressed(),
            ecx.vi.rest.is_pressed(),
            ecx.vi.dir.dir8_down(),
        );

        if select {
            let dir = ecx.world.entities[self.actor.0].dir;

            return EventResult::chain(Interact {
                actor: self.actor,
                dir,
            });
        }

        if turn {
            if let Some(dir) = self.find_only_neighbor(ecx) {
                return EventResult::chain(ChangeDir {
                    actor: self.actor,
                    dir,
                });
            }
        }

        if rest {
            return EventResult::chain(RestOneTurn { actor: self.actor });
        }

        if let Some(dir) = dir {
            return EventResult::chain(PlayerWalk {
                actor: self.actor,
                dir,
            });
        }

        EventResult::GotoNextFrame
    }
}

#[derive(Debug)]
pub struct RandomWalk {
    pub actor: ActorIx,
}

impl GenAnim for RandomWalk {}

impl Event for RandomWalk {
    fn run(&self, _ecx: &mut EventContext) -> EventResult {
        let dir = {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            Dir8::CLOCKWISE[rng.gen_range(0..8)]
        };

        EventResult::chain(PlayerWalk {
            actor: self.actor,
            dir,
        })
    }
}
