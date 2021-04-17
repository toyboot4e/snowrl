/*!
High level commands
*/

use snow2d::{
    gfx::geom2d::Vec2f,
    ui::{
        anim_builder::AnimGen,
        node::{self, Node},
    },
    utils::{arena::Index, ez, tweak::*},
};

use rlbox::rl::grid2d::*;

use crate::grue::{
    ctrl::rogue::{
        anim::{self as rl_anim, *},
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
    fn run(&self, _data: &mut Data) -> EventResult {
        EventResult::chain(GiveDamage {
            actor: self.target,
            amount: 10,
        })
    }
}

impl GenAnim for Hit {
    fn gen_anim(&self, data: &mut Data) -> Option<Box<dyn Anim>> {
        let actor = &data.world.entities[self.target];

        let [actors, on_actors] = data.res.ui.layers_mut([UiLayer::Actors, UiLayer::OnActors]);
        let base_pos = actors.nodes[&actor.nodes.base].params.pos;

        let text = on_actors.nodes.add({
            let mut text = Node::from(node::Text::new("HIT"));
            // FIXME: set font texture size and align
            text.params.pos = base_pos - Vec2f::new(20.0, 20.0);
            text
        });

        let mut gen = AnimGen::default();
        gen.node(&text).dt(ez::EasedDt::linear(1.0));
        on_actors.anims.insert(gen.alpha([0, 255]));

        // TODO: wait for reserved duration (swing animation)
        // Some(Box::new(WaitForUiAnim::new(
        //     on_actors.anims.insert(gen.alpha([0, 255])),
        //     UiLayer::OnActors,
        // )))

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
