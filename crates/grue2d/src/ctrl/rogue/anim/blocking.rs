/*!
Animations for the builtin events

They're created referencing rogulike events and then we forget about original events.
*/

use snow2d::{ui::anim::Anim as UiAnim, utils::arena::Index};

use crate::data::world::actor::Actor;

use super::{Anim, AnimResult, Data, Timer};

/// TODO: rm
const WALK_FRAMES: u64 = 8;

/// TODO: don't hard code player detection
const PLAYER: u32 = 0;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
            timer: Timer::from_frames(WALK_FRAMES),
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

// TODO: wait for UI animation. but we can't distinguish belonging pool.
// #[derive(Debug, Clone)]
// pub struct WaitForUiAnim {
//     anim: Index<UiAnim>,
// }
//
// impl WaitForUiAnim {
//     pub fn new(anim: Index<UiAnim>) -> Self {
//         Self { anim }
//     }
// }
//
// impl Anim for WalkAnim {
//     fn on_start(&mut self, data: &mut Data) {
//         // be sure to start animation in this frame
//         self.timer.set_started(true);
//
//         if self.actors.iter().any(|a| a.slot() == PLAYER) {
//             // update Player FoV in this frame
//             data.world.shadow.mark_dirty();
//         }
//     }
//
//     fn update(&mut self, data: &mut Data) -> AnimResult {
//         let node = data.
//         AnimResult::
//     }
// }
