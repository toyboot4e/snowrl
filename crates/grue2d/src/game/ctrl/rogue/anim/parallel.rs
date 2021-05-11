/*!
Non-blocking animations
*/

use std::time::Duration;

use snow2d::utils::{arena::Index, Inspect};

use crate::game::data::world::actor::Actor;

use super::{Anim, AnimResult, Data, Timer};

#[derive(Debug, Clone, Inspect)]
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
    fn on_start(&mut self, _data: &mut Data) {
        // log::trace!("{:?}", self.actors);
    }

    fn update(&mut self, data: &mut Data) -> AnimResult {
        self.timer.tick_as_result(data.ice.dt())
    }
}
