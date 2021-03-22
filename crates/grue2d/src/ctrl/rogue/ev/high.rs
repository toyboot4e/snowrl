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

/// TODO: Attack in direction
#[derive(Debug)]
pub struct MeleeAttack {
    pub actor: Index<Actor>,
    pub dir: Option<Dir8>,
}

impl Event for MeleeAttack {
    fn run(&self, ecx: &mut EventContext) -> EventResult {
        let actor = &ecx.world.entities[self.actor];
        let target_dir = self.dir.clone().unwrap_or(actor.dir);
        let target_pos = actor.pos.offset(target_dir);

        if let Some(_target) = ecx
            .world
            .entities
            .iter()
            .find(|(_i, e)| e.pos == target_pos)
        {
            // hit entity
            // EventResult::chain(Hit {
            // })
            EventResult::Finish
        } else {
            // just swing
            EventResult::Finish
            // EventResult::chain(ChangeDir {
            //     actor: self.actor,
            //     dir: self.to_dir,
            // })
        }
    }
}

impl GenAnim for MeleeAttack {
    fn gen_anim(&self, acx: &mut AnimContext) -> Option<Box<dyn Anim>> {
        Some(Box::new(anim::SwingAnim::new(
            self.actor,
            self.dir
                .unwrap_or_else(|| acx.world.entities[self.actor].dir),
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
