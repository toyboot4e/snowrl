/*!
High level commands
*/

use snow2d::utils::arena::Index;

use rlbox::rl::grid2d::*;

use crate::game::{
    ctrl::rogue::{
        anim::{self as rl_anim, *},
        ev,
        tick::{Event, EventResult, GenAnim},
    },
    data::world::{actor::Actor, World},
    Data,
};

use super::*;

/// TODO: rm
const SWING_SECS: f32 = 8.0 / 60.0;

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
    fn run(&self, _data: &mut Data) -> EventResult {
        EventResult::chain(GiveDamage {
            target: self.target,
            amount: 10,
        })
    }
}

impl GenAnim for Hit {
    fn gen_anim(&self, _data: &mut Data) -> Option<Box<dyn Anim>> {
        None
    }
}

#[derive(Debug)]
pub struct JustSwing {
    pub actor: Index<Actor>,
    pub dir: Option<Dir8>,
}

impl Event for JustSwing {
    fn run(&self, _data: &mut Data) -> EventResult {
        EventResult::Finish
    }
}

impl GenAnim for JustSwing {
    fn gen_anim(&self, data: &mut Data) -> Option<Box<dyn Anim>> {
        // TODO: volume 4.0
        ev::play_sound_preserve(crate::paths::sound::se::SWING, data).unwrap();

        Some(Box::new(rl_anim::SwingAnim::new(
            self.actor,
            self.dir
                .unwrap_or_else(|| data.world.entities[self.actor].dir),
            SWING_SECS,
        )))
    }
}

#[derive(Debug)]
pub struct MeleeAttack {
    pub actor: Index<Actor>,
    pub dir: Option<Dir8>,
}

impl MeleeAttack {
    fn target_dir(&self, world: &World) -> Dir8 {
        let actor = &world.entities[self.actor];
        self.dir.clone().unwrap_or(actor.dir)
    }

    fn target_pos(&self, world: &World) -> Vec2i {
        let actor = &world.entities[self.actor];
        actor.pos.offset(self.target_dir(world))
    }

    fn pull_target(&self, world: &World) -> Option<Index<Actor>> {
        let target_pos = self.target_pos(world);

        world
            .entities
            .iter()
            .find(|(_i, e)| e.pos == target_pos)
            .map(|(i, _e)| i)
    }
}

impl Event for MeleeAttack {
    fn run(&self, data: &mut Data) -> EventResult {
        if let Some(target) = self.pull_target(&data.world) {
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
    fn gen_anim(&self, data: &mut Data) -> Option<Box<dyn Anim>> {
        ev::play_sound_preserve(crate::paths::sound::se::SWING, data).unwrap();

        ev::run_dir_anim(
            "attack",
            self.target_pos(&data.world),
            self.target_dir(&data.world),
            data,
        );

        Some(Box::new(rl_anim::SwingAnim::new(
            self.actor,
            self.dir
                .unwrap_or_else(|| data.world.entities[self.actor].dir),
            SWING_SECS,
        )))
    }
}

#[derive(Debug)]
pub struct RandomWalk {
    pub actor: Index<Actor>,
}

impl GenAnim for RandomWalk {}

impl Event for RandomWalk {
    fn run(&self, _data: &mut Data) -> EventResult {
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
