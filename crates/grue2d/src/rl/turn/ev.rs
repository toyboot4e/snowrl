/*!

Roguelike game events

Every change to the roguelike game world should be handled as primitive event. That's good for
visualization, flexibilities and simplicities.

e.g. `MeleeAttack` -> `Attack` -> `Hit` -> `GiveDamage`

TODO: remove non-primitive events

*/

use rlbox::rl::grid2d::*;

use crate::rl::turn::{
    anim::{self, Anim},
    tick::{ActorIx, AnimContext, Event, EventContext, EventResult, GenAnim},
};

pub enum Response {
    Ignored,
    Captured,
}

impl Response {
    pub fn merge(self, b: Self) -> Self {
        match self {
            Self::Ignored => b,
            Self::Captured => Self::Captured,
        }
    }
}

/// TODO: remove
const PLAYER: usize = 0;

/// Some action resulted in a non-turn consuming action
///
/// Player should take another turn.
///
/// FIXME: unintentional side effects
#[derive(Debug)]
pub struct NotConsumeTurn {
    pub actor: ActorIx,
}

impl GenAnim for NotConsumeTurn {
    fn gen_anim(&self, _acx: &mut AnimContext) -> Option<Box<dyn Anim>> {
        if self.actor.0 == PLAYER {
            // wait for one frame so that we won't enter inifinite loop
            Some(Box::new(anim::Wait { frames: 1 }))
        } else {
            None
        }
    }
}

impl Event for NotConsumeTurn {
    fn run(&self, _ecx: &mut EventContext) -> EventResult {
        if self.actor.0 == PLAYER {
            // TODO: require one frame wait
            EventResult::chain(PlayerTurn { actor: self.actor })
        } else {
            EventResult::Finish
        }
    }
}

// --------------------------------------------------------------------------------
// Primitive events

// Every change to the roguelike game should happen as a primitive event.
// These additional steps are also good foor visualization.

#[derive(Debug)]
pub struct RestOneTurn {
    pub actor: ActorIx,
}

impl GenAnim for RestOneTurn {
    fn gen_anim(&self, _acx: &mut AnimContext) -> Option<Box<dyn Anim>> {
        None
    }
}

impl Event for RestOneTurn {
    fn run(&self, _ecx: &mut EventContext) -> EventResult {
        EventResult::Finish
    }
}

/// Just change direction
#[derive(Debug)]
pub struct ChangeDir {
    pub actor: ActorIx,
    pub dir: Dir8,
}

impl GenAnim for ChangeDir {
    fn gen_anim(&self, _acx: &mut AnimContext) -> Option<Box<dyn Anim>> {
        // TODO: play rotation and wait for it to finish
        None
    }
}

impl Event for ChangeDir {
    fn run(&self, ecx: &mut EventContext) -> EventResult {
        let actor = &mut ecx.world.entities[self.actor.0];
        actor.dir = self.dir;

        // FIXME: it's dangerous..
        EventResult::chain(NotConsumeTurn { actor: self.actor })
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
    pub actor: ActorIx,
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

impl Event for Move {
    fn run(&self, ecx: &mut EventContext) -> EventResult {
        if !ecx.world.is_blocked(self.to_pos) {
            let actor = &mut ecx.world.entities[self.actor.0];
            actor.dir = self.to_dir;
            actor.pos = self.to_pos;
            EventResult::Finish
        } else {
            EventResult::chain(ChangeDir {
                actor: self.actor,
                dir: self.to_dir,
            })
        }
    }
}

// --------------------------------------------------------------------------------
// Higher-level commands

// /// TODO: Attack in direction
// #[derive(Debug)]
// pub struct Attack {
//     pub actor: ActorIx,
//     pub dir: Dir8,
// }

// impl GenAnim for Attack {}

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
