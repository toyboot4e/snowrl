/*!
High level commands
*/

use snow2d::utils::{arena::Index, tweak::*};

use rlbox::rl::grid2d::*;

use crate::{
    ctrl::rogue::{
        anim::{self, Anim},
        tick::{AnimContext, Event, EventContext, EventResult, GenAnim},
    },
    data::world::actor::Actor,
};

use super::*;

#[derive(Debug)]
pub enum Attack {
    MeleeAttackFromActor { actor: Index<Actor> },
}

/// [`Attack`] applied to an actor
#[derive(Debug)]
pub struct Hit {
    pub target: Index<Actor>,
    pub attacker: Index<Actor>,
}

impl Event for Hit {
    fn run(&self, _ecx: &mut EventContext) -> EventResult {
        todo!()
    }
}

impl GenAnim for Hit {
    fn gen_anim(&self, _acx: &mut AnimContext) -> Option<Box<dyn Anim>> {
        todo!()
    }
}

#[derive(Debug)]
pub struct JustSwing {
    pub actor: Index<Actor>,
    pub dir: Option<Dir8>,
}

impl Event for JustSwing {
    fn run(&self, _ecx: &mut EventContext) -> EventResult {
        EventResult::Finish
    }
}

impl GenAnim for JustSwing {
    fn gen_anim(&self, acx: &mut AnimContext) -> Option<Box<dyn Anim>> {
        Some(Box::new(anim::SwingAnim::new(
            self.actor,
            self.dir
                .unwrap_or_else(|| acx.world.entities[self.actor].dir),
            // FIXME: magic number
            tweak!(8.0 / 60.0),
        )))
    }
}

#[derive(Debug)]
pub struct MeleeAttack {
    pub actor: Index<Actor>,
    pub dir: Option<Dir8>,
}

impl Event for MeleeAttack {
    fn run(&self, ecx: &mut EventContext) -> EventResult {
        let actor = &ecx.world.entities[self.actor];
        let actor_dir = self.dir.clone().unwrap_or(actor.dir);
        let target_pos = actor.pos.offset(actor_dir);

        if let Some((target, _target_actor)) = ecx
            .world
            .entities
            .iter()
            .find(|(_i, e)| e.pos == target_pos)
        {
            // hit entity
            EventResult::chain(Hit {
                target,
                attacker: self.actor,
            })
        } else {
            // just swing and change direction
            match self.dir {
                Some(dir) if dir != dir => EventResult::chain(ChangeDir {
                    actor: self.actor,
                    dir,
                }),
                _ => EventResult::Finish,
            }
        }
    }
}

impl GenAnim for MeleeAttack {
    fn gen_anim(&self, acx: &mut AnimContext) -> Option<Box<dyn Anim>> {
        Some(Box::new(anim::SwingAnim::new(
            self.actor,
            self.dir
                .unwrap_or_else(|| acx.world.entities[self.actor].dir),
            // FIXME: magic number
            tweak!(8.0 / 60.0),
        )))
    }
}

#[derive(Debug)]
pub struct RandomWalk {
    pub actor: Index<Actor>,
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
