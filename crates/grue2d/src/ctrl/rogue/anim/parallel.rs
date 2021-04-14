/*!
Non-blocking animations
*/

use std::time::Duration;

use {
    rlbox::rl::grid2d::{Dir8, Vec2i},
    snow2d::{
        gfx::geom2d::Vec2f,
        ui::{anim::AnimIndex, anim_builder::AnimSeq},
        utils::arena::Index,
    },
};

use crate::data::{res::UiLayer, world::actor::Actor};

use super::{Anim, AnimResult, AnimUpdateContext, Timer};

#[derive(Debug, Clone)]
pub struct DamageText {
    pub actor: Index<Actor>,
    pub amount: u32,
    timer: Timer,
}

impl DamageText {
    pub fn new(actor: Index<Actor>, amount: u32) -> Self {
        let ms = 1000.0 * 20.0 / 60.0;
        Self {
            actor,
            amount,
            timer: Timer::from_duration(Duration::from_millis(ms as u64)),
        }
    }
}

impl Anim for DamageText {
    fn on_start(&mut self, _ucx: &mut AnimUpdateContext) {
        // log::trace!("{:?}", self.actors);
    }

    fn update(&mut self, ucx: &mut AnimUpdateContext) -> AnimResult {
        self.timer.tick_as_result(ucx.ice.dt())
    }
}

#[derive(Debug, Clone)]
pub struct SwingAnim {
    pub actor: Index<Actor>,
    pub dir: Dir8,
    timer: Timer,
    anims: Option<Vec<AnimIndex>>,
}

impl SwingAnim {
    pub fn new(actor: Index<Actor>, dir: Dir8, secs: f32) -> Self {
        Self {
            actor,
            dir,
            timer: Timer::from_secs_f32(secs),
            anims: None,
        }
    }
}

impl Anim for SwingAnim {
    fn on_start(&mut self, ucx: &mut AnimUpdateContext) {
        let actor = &ucx.world.entities[self.actor];
        let actor_layer = ucx.res.ui.layer_mut(UiLayer::Actors);

        // parameters
        let dpos = {
            let size = Vec2f::new(
                ucx.world.map.tiled.tile_width as f32,
                ucx.world.map.tiled.tile_height as f32,
            );
            size * Vec2i::from(self.dir).to_vec2f()
        };
        let img_offset = actor.view.img_offset();

        // animation sequence
        actor_layer.anims.insert_seq({
            let (mut seq, mut gen) = AnimSeq::begin();
            gen.node(&actor.nodes.img)
                .secs(self.timer.target().as_secs_f32() / 2.0);
            seq.append(gen.pos([img_offset, img_offset + dpos]));
            seq.append(gen.pos([img_offset + dpos, img_offset]));
            seq
        });
    }

    fn update(&mut self, ucx: &mut AnimUpdateContext) -> AnimResult {
        self.timer.tick_as_result(ucx.ice.dt())
    }
}
