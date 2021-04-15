/*!
High level commands
*/

use snow2d::{
    ui::{
        anim_builder::AnimGen,
        node::{self, Node},
    },
    utils::{arena::Index, tweak::*},
};

use rlbox::rl::grid2d::*;

use crate::{
    ctrl::rogue::{
        anim::{self as rl_anim, Anim},
        tick::{Event, EventResult, GenAnim},
    },
    data::{res::UiLayer, world::actor::Actor},
    Data,
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
    fn run(&self, data: &mut Data) -> EventResult {
        let target = &data.world.entities[self.target];
        // let [on_shadow, on_actors] = data.res.ui.layer_mut([UiLayer::OnShadow, UiLayer::OnActors]);
        // let mut gen = AnimGen::default();
        // let mut text = Node::from(node::Text::new("HIT"));
        // text.pos = e
        // gen.node(&self.target).dt(ez::EasedDt::linear(
        // layer.anims.insert(gen.text
        EventResult::Finish
    }
}

impl GenAnim for Hit {
    fn gen_anim(&self, _data: &mut Data) -> Option<Box<dyn Anim>> {
        todo!()
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
        Some(Box::new(rl_anim::SwingAnim::new(
            self.actor,
            self.dir
                .unwrap_or_else(|| data.world.entities[self.actor].dir),
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
    fn run(&self, data: &mut Data) -> EventResult {
        let actor = &data.world.entities[self.actor];
        let actor_dir = self.dir.clone().unwrap_or(actor.dir);
        let target_pos = actor.pos.offset(actor_dir);

        if let Some((target, _target_actor)) = data
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
    fn gen_anim(&self, data: &mut Data) -> Option<Box<dyn Anim>> {
        Some(Box::new(rl_anim::SwingAnim::new(
            self.actor,
            self.dir
                .unwrap_or_else(|| data.world.entities[self.actor].dir),
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
