/*!
Non-blocking animations
*/

use {rlbox::rl::grid2d::Dir8, snow2d::utils::arena::Index, std::time::Duration};

use crate::data::world::actor::Actor;

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
}

impl SwingAnim {
    pub fn new(actor: Index<Actor>, dir: Dir8, secs: f32) -> Self {
        Self {
            actor,
            dir,
            timer: Timer::from_secs_f32(secs),
        }
    }
}

impl Anim for SwingAnim {
    fn on_start(&mut self, _ucx: &mut AnimUpdateContext) {
        // log::trace!("{:?}", self.actors);
    }

    fn update(&mut self, ucx: &mut AnimUpdateContext) -> AnimResult {
        self.timer.tick_as_result(ucx.ice.dt())
    }
}
