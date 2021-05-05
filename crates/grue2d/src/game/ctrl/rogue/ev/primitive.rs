/*!
Every change to the roguelike game should happen as a primitive event. These additional steps are
also good foor both visualization and separation.
*/

use snow2d::{
    gfx::geom2d::Vec2f,
    ui::{
        anim_builder::AnimGen,
        node::{self, Node},
    },
    utils::{arena::Index, ez},
};

use rlbox::rl::grid2d::*;

use crate::game::{
    ctrl::rogue::{
        anim::{self, Anim},
        tick::{Event, EventResult, GenAnim},
    },
    data::{res::UiLayer, world::actor::Actor},
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
    fn gen_anim(&self, _data: &mut Data) -> Option<Box<dyn Anim>> {
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
    fn gen_anim(&self, _data: &mut Data) -> Option<Box<dyn Anim>> {
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
    fn gen_anim(&self, _data: &mut Data) -> Option<Box<dyn Anim>> {
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
    fn gen_anim(&self, _data: &mut Data) -> Option<Box<dyn Anim>> {
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
#[derive(Debug)]
pub struct GiveDamage {
    pub target: Index<Actor>,
    pub amount: u32,
}

impl GenAnim for GiveDamage {
    fn gen_anim(&self, data: &mut Data) -> Option<Box<dyn Anim>> {
        let actor = &data.world.entities[self.target];

        let [actors, on_actors] = data.res.ui.layers_mut([UiLayer::Actors, UiLayer::OnActors]);
        let base_pos = actors.nodes[&actor.nodes.base].params.pos;

        let text = on_actors.nodes.add({
            let text = format!("{}", self.amount);
            let mut text = Node::from(node::Text::new(text));
            // FIXME: set font texture size and align
            text.params.pos = base_pos - Vec2f::new(20.0, 20.0);
            text
        });

        let mut gen = AnimGen::default();
        gen.node(&text).dt(ez::EasedDt::linear(1.0));
        on_actors.anims.insert(gen.alpha([0, 255]));

        // FIXME: the delay should be decided externally. delay the hit anim creation itself
        let se = data
            .ice
            .assets
            .load_sync_preserve::<snow2d::audio::src::Wav, _>(crate::paths::sound::se::ATTACK)
            .unwrap();
        data.ice.audio.play_clocked(4.0 / 60.0, &*se.get().unwrap());

        // TODO: wait for reserved duration (swing animation)

        None
    }
}

impl Event for GiveDamage {
    fn run(&self, data: &mut Data) -> EventResult {
        let actor = &mut data.world.entities[self.target];

        if actor.stats.hp > self.amount {
            actor.stats.hp -= self.amount;
            EventResult::Finish
        } else {
            actor.stats.hp = 0;
            EventResult::Chain(Box::new(Death { actor: self.target }))
        }
    }
}

/// Actor died
#[derive(Debug)]
pub struct Death {
    pub actor: Index<Actor>,
}

impl GenAnim for Death {
    fn gen_anim(&self, _data: &mut Data) -> Option<Box<dyn Anim>> {
        None
    }
}

impl Event for Death {
    fn run(&self, data: &mut Data) -> EventResult {
        log::trace!("actor at slot {:?} died", self.actor.slot());

        if self.actor.slot() == PLAYER as u32 {
            todo!("implement player death");
        }

        // NOTE: deleting actors IMMEDIATELY can result in invalid indices.
        //       should we invalidate the actor AFTER finishing animation?
        // let actor = &mut data.world.entities[self.actor];
        data.world.entities.remove(self.actor).unwrap();

        EventResult::Finish
    }
}
