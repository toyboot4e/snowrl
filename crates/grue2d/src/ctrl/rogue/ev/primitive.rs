/*!
Every change to the roguelike game should happen as a primitive event. These additional steps are
also good foor both visualization and separation.
*/

use snow2d::utils::arena::Index;

use rlbox::rl::grid2d::*;

use crate::{
    ctrl::rogue::{
        anim::{self, Anim},
        tick::{AnimContext, Event, EventResult, GenAnim},
    },
    data::world::actor::Actor,
    Data,
};

use super::*;

/// TODO: remove the magic number
const PLAYER: usize = 0;

/// (Primitive) Some action resulted in a non-turn consuming action
///
/// Player should take another turn on this event.
///
/// FIXME: unintentional side effects
#[derive(Debug)]
pub struct NotConsumeTurn {
    pub actor: Index<Actor>,
}

impl GenAnim for NotConsumeTurn {
    fn gen_anim(&self, _acx: &mut AnimContext) -> Option<Box<dyn Anim>> {
        // TODO: don't hard code
        if self.actor.slot() as usize == PLAYER {
            // wait for one frame so that we won't enter inifinite loop
            Some(Box::new(anim::WaitFrames { frames: 1 }))
        } else {
            None
        }
    }
}

impl Event for NotConsumeTurn {
    fn run(&self, _data: &mut Data) -> EventResult {
        if self.actor.slot() as usize == PLAYER {
            // TODO: require one frame wait
            EventResult::chain(PlayerTurn { actor: self.actor })
        } else {
            EventResult::Finish
        }
    }
}

#[derive(Debug)]
pub struct RestOneTurn {
    pub actor: Index<Actor>,
}

impl GenAnim for RestOneTurn {
    fn gen_anim(&self, _acx: &mut AnimContext) -> Option<Box<dyn Anim>> {
        None
    }
}

impl Event for RestOneTurn {
    fn run(&self, _data: &mut Data) -> EventResult {
        EventResult::Finish
    }
}

/// (Primitive) Just change the facing direction
#[derive(Debug)]
pub struct ChangeDir {
    pub actor: Index<Actor>,
    pub dir: Dir8,
}

impl GenAnim for ChangeDir {
    fn gen_anim(&self, _acx: &mut AnimContext) -> Option<Box<dyn Anim>> {
        // TODO: play rotation and wait for it to finish
        None
    }
}

impl Event for ChangeDir {
    fn run(&self, data: &mut Data) -> EventResult {
        let actor = &mut data.world.entities[self.actor];
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

/// (Primitive) Change in actor's position and direction
#[derive(Debug)]
pub struct Move {
    pub actor: Index<Actor>,
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
    fn run(&self, data: &mut Data) -> EventResult {
        if !data.world.is_blocked(self.to_pos) {
            let actor = &mut data.world.entities[self.actor];
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

/// (Primitive) Change actor's HP
pub struct GiveDamage {
    pub actor: Index<Actor>,
    pub amount: u32,
}

impl GenAnim for GiveDamage {
    fn gen_anim(&self, _acx: &mut AnimContext) -> Option<Box<dyn Anim>> {
        Some(Box::new(anim::DamageText::new(self.actor, self.amount)))
    }
}
