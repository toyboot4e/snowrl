/*!

Animations for the roguelike game

They're created referencing rogulike events and then we forget about original events.

*/

use downcast_rs::{impl_downcast, Downcast};

use std::{collections::VecDeque, fmt, time::Duration};

use crate::{
    turn::tick::ActorIndex,
    world::{World, WorldContext},
};

#[derive(Debug)]
pub struct AnimPlayer {
    /// Queue of animations
    anims: VecDeque<Box<dyn Anim>>,
    is_top_walk: bool,
}

impl AnimPlayer {
    pub fn new() -> Self {
        Self {
            anims: VecDeque::with_capacity(10),
            is_top_walk: false,
        }
    }

    /// If the animation should be batched or not
    pub fn should_batch_top_anim(&self) -> bool {
        self.is_top_walk
    }

    /// If we have animations to batch or not (actually empty or not)
    pub fn any_batch(&self) -> bool {
        !self.anims.is_empty()
    }

    /// Add animation boxing it
    pub fn enqueue<T: Anim + 'static>(&mut self, anim: T) {
        self.anims.push_back(Box::new(anim));
    }

    /// Add boxed animation
    pub fn enqueue_boxed(&mut self, anim: Box<dyn Anim>) {
        if let Some(walk) = (*anim).as_any().downcast_ref::<WalkAnim>() {
            self.push_walk_anim(walk);
            self.is_top_walk = true;
        } else {
            self.anims.push_back(anim);
            self.is_top_walk = false;
        }
    }

    /// Multiple walk animations should be run as a batched animation (so that player don't have to
    /// wait for unnecessary long time)
    fn push_walk_anim(&mut self, walk: &WalkAnim) {
        if self.is_top_walk {
            // parallelize the walk animation
            let anim = self.anims.front_mut().unwrap();
            let batch = anim.downcast_mut::<WalkAnim>().unwrap();
            batch.merge(walk);
            // log::trace!("batch: {:?} || {:?}", batch, walk);
        } else {
            self.enqueue(walk.clone());
        }
    }

    pub fn on_start(&mut self, ucx: &mut AnimUpdateContext) {
        assert!(
            !self.anims.is_empty(),
            "Tried to start playing animation stack while it's empty!"
        );

        let front = self.anims.front_mut().unwrap();
        front.on_start(ucx);

        // log::trace!("animation queue: {:?}", self.anims);
    }

    fn on_exit(&mut self) {
        self.is_top_walk = false;
    }

    pub fn update(&mut self, ucx: &mut AnimUpdateContext) -> AnimResult {
        loop {
            let front = match self.anims.front_mut() {
                Some(a) => a,
                None => {
                    self.on_exit();
                    return AnimResult::Finished;
                }
            };

            // TODO: separate `AnimResult` and `AnimPlayerResult`
            let res = front.update(ucx);

            if res == AnimResult::Finished {
                self.anims.pop_front();

                if let Some(front) = self.anims.front_mut() {
                    front.on_start(ucx);
                }

                continue; // process next animation
            }

            return res;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimResult {
    Continue,
    Finished,
}

#[derive(Debug)]
pub struct AnimUpdateContext<'a, 'b> {
    pub dt: Duration,
    pub world: &'a mut World,
    pub wcx: &'b mut WorldContext,
}

pub trait Anim: fmt::Debug + Downcast {
    fn on_start(&mut self, ucx: &mut AnimUpdateContext);
    fn update(&mut self, ucx: &mut AnimUpdateContext) -> AnimResult;
}

impl_downcast!(Anim);

// do not impl `Anim` for `Box<dyn Anim>`

/// Walk animation is currently run automatically, so we just wait for it to finish
#[derive(Debug, Clone)]
pub struct WalkAnim {
    pub dt: Duration,
    /// Batch walk animations
    pub actors: Vec<ActorIndex>,
}

impl WalkAnim {
    pub fn new(actor: ActorIndex) -> Self {
        Self {
            dt: Duration::new(0, 0),
            actors: {
                let mut xs = Vec::with_capacity(4);
                xs.push(actor);
                xs
            },
        }
    }

    /// Merge other walk animation into one
    pub fn merge(&mut self, other: &Self) {
        self.actors.extend(&other.actors);
        // TODO ensure no duplicates
    }
}

impl Anim for WalkAnim {
    fn on_start(&mut self, ucx: &mut AnimUpdateContext) {
        // log::trace!("{:?}", self.actors);

        if self.actors.iter().all(|a| a.0 != 0) {
            // return if the player do not walk
            return;
        }

        // update FoV animation
        // FIXME: there are other chances that can change FoV, so
        // we should store two FoV and track their states
        ucx.wcx.fov_render.on_fov_change(&ucx.world.entities[0].fov);

        // update Player FoV
        let actor = &mut ucx.world.entities[0];
        let pos = actor.pos;
        let r = actor.fov.radius;
        crate::world::update_fov(&mut actor.fov, pos, r, &ucx.world.map.rlmap);
    }

    fn update(&mut self, ucx: &mut AnimUpdateContext) -> AnimResult {
        self.dt += ucx.dt;
        if self.dt.as_secs_f32() >= (crate::consts::WALK_TIME - crate::consts::HALF_FRAME) {
            AnimResult::Finished
        } else {
            AnimResult::Continue
        }
    }
}
