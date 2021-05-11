/*!
Animations for the builtin events

They're created referencing rogulike events and then we forget about original events.
*/

use std::time::Duration;

use {
    rlbox::rl::grid2d::{Dir8, Vec2i},
    snow2d::{
        gfx::geom2d::Vec2f,
        ui::{anim::AnimImpl, anim_builder::AnimSeq, Anim as UiAnim, AnimIndex},
        utils::{arena::Index, Inspect},
    },
};

use crate::game::data::{res::UiLayer, world::actor::Actor};

use super::{Anim, AnimResult, Data, Timer};

/// TODO: rm
const WALK_SECS: f32 = 8.0 / 60.0;

/// TODO: don't hard code player detection
const PLAYER: u32 = 0;

#[derive(Debug, Clone, Inspect)]
pub struct WaitFrames {
    pub frames: usize,
}

impl Anim for WaitFrames {
    fn update(&mut self, _data: &mut Data) -> AnimResult {
        if self.frames == 0 {
            AnimResult::Finish
        } else {
            self.frames -= 1;
            AnimResult::GotoNextFrame
        }
    }
}

#[derive(Debug, Clone, Inspect)]
pub struct WaitSecs {
    timer: Timer,
}

impl WaitSecs {
    pub fn new(secs: f32) -> Self {
        Self {
            timer: Timer::from_secs_f32(secs),
        }
    }
}

impl Anim for WaitSecs {
    fn update(&mut self, data: &mut Data) -> AnimResult {
        self.timer.tick_as_result(data.ice.dt())
    }
}

/// Walk animation is currently run automatically, so we just wait for it to finish
#[derive(Debug, Clone, Inspect)]
pub struct WalkAnim {
    /// Batch walk animations
    pub actors: Vec<Index<Actor>>,
    timer: Timer,
}

impl WalkAnim {
    pub fn new(actor: Index<Actor>) -> Self {
        Self {
            actors: {
                let mut xs = Vec::with_capacity(4);
                xs.push(actor);
                xs
            },
            timer: Timer::from_duration(Duration::from_secs_f32(WALK_SECS)),
        }
    }

    /// Merge other walk animation into one
    pub fn merge(&mut self, other: &Self) {
        self.actors.extend(&other.actors);
        // TODO ensure no duplicate exists
    }
}

impl Anim for WalkAnim {
    fn on_start(&mut self, data: &mut Data) {
        // be sure to start animation in this frame
        self.timer.set_started(true);

        if self.actors.iter().any(|a| a.slot() == PLAYER) {
            // update Player FoV in this frame
            data.world.shadow.mark_dirty();
        }
    }

    fn update(&mut self, data: &mut Data) -> AnimResult {
        self.timer.tick_as_result(data.ice.dt())
    }
}

#[derive(Debug, Clone, Inspect)]
pub struct WaitForUiAnim {
    anim: Index<UiAnim>,
    layer: UiLayer,
}

impl WaitForUiAnim {
    pub fn new(anim: Index<UiAnim>, layer: UiLayer) -> Self {
        Self { anim, layer }
    }
}

impl Anim for WaitForUiAnim {
    fn on_start(&mut self, _data: &mut Data) {}

    fn update(&mut self, data: &mut Data) -> AnimResult {
        let anim = match data.res.ui.layer(self.layer).anims.get(self.anim) {
            Some(node) => node,
            None => return AnimResult::Finish,
        };

        if anim.is_end() {
            AnimResult::Finish
        } else {
            AnimResult::GotoNextFrame
        }
    }
}

#[derive(Debug, Clone, Inspect)]
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
    fn on_start(&mut self, data: &mut Data) {
        let actor = &data.world.entities[self.actor];
        let actor_layer = data.res.ui.layer_mut(UiLayer::Actors);

        // parameters
        let dpos = {
            let size = Vec2f::new(
                data.world.map.tiled.tile_width as f32,
                data.world.map.tiled.tile_height as f32,
            );
            size * Vec2i::from(self.dir).to_vec2f()
        };
        let img_offset = actor.view.img_offset();

        // sequence of animations
        actor_layer.anims.insert_seq({
            let (mut seq, mut gen) = AnimSeq::begin();
            gen.node(&actor.nodes.img)
                .secs(self.timer.target().as_secs_f32() / 2.0);
            seq.append(gen.pos([img_offset, img_offset + dpos]));
            seq.append(gen.pos([img_offset + dpos, img_offset]));
            seq
        });
    }

    fn update(&mut self, data: &mut Data) -> AnimResult {
        self.timer.tick_as_result(data.ice.dt())
    }
}
