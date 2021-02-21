/*!

Animations for the roguelike game

They're created referencing rogulike events and then we forget about original events.

*/

use {
    downcast_rs::{impl_downcast, Downcast},
    snow2d::{utils::arena::Index, Ice},
    std::{collections::VecDeque, fmt, time::Duration},
};

use crate::rl::world::{actor::Actor, World};

/// TODO: rm
const WALK_TIME: f32 = 8.0 / 60.0;

/// Return alue of [`Anim::update`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimResult {
    GotoNextFrame,
    Finish,
}

/// Context to process any [`Anim`]
#[derive(Debug)]
pub struct AnimUpdateContext<'a, 'b> {
    pub world: &'a mut World,
    pub ice: &'b mut Ice,
}

/// Roguelike animation object
pub trait Anim: fmt::Debug + Downcast {
    fn on_start(&mut self, _ucx: &mut AnimUpdateContext) {}
    fn update(&mut self, ucx: &mut AnimUpdateContext) -> AnimResult;
    // TODO: render animation
}

// we can cast `Box<Anim>` to `Box<Any>` with `as_any`
impl_downcast!(Anim);

/// State to play roguelike animations
#[derive(Debug)]
pub struct AnimPlayer {
    /// Queue of animations
    anims: VecDeque<Box<dyn Anim>>,
    is_top_walk: bool,
}

impl Default for AnimPlayer {
    fn default() -> Self {
        Self {
            anims: VecDeque::with_capacity(10),
            is_top_walk: false,
        }
    }
}

impl AnimPlayer {
    pub fn any_anim_to_run_now(&self) -> bool {
        // more than one animation or the only animation never batches other animation
        self.anims.len() > 1 || !self.is_top_walk
    }

    /// If we have animations to batch or not (actually empty or not)
    pub fn any_batch(&self) -> bool {
        !self.anims.is_empty() && self.is_top_walk
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
                    return AnimResult::Finish;
                }
            };

            // TODO: separate `AnimResult` and `AnimPlayerResult`
            let res = front.update(ucx);

            if res == AnimResult::Finish {
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

/// Wait for n frames
#[derive(Debug, Clone)]
pub struct Wait {
    pub frames: usize,
}

impl Anim for Wait {
    fn update(&mut self, _ucx: &mut AnimUpdateContext) -> AnimResult {
        if self.frames == 0 {
            AnimResult::Finish
        } else {
            self.frames -= 1;
            AnimResult::GotoNextFrame
        }
    }
}

// do not impl `Anim` for `Box<dyn Anim>` so that the downcast works fine

/// Walk animation is currently run automatically, so we just wait for it to finish
#[derive(Debug, Clone)]
pub struct WalkAnim {
    pub dt: Duration,
    /// Batch walk animations
    pub actors: Vec<Index<Actor>>,
}

impl WalkAnim {
    pub fn new(actor: Index<Actor>) -> Self {
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

        // TODO: don't hard code player detection
        if self.actors.iter().any(|a| a.slot() == 0) {
            // update Player FoV in this frame
            ucx.world.shadow.make_dirty();
        }
    }

    fn update(&mut self, ucx: &mut AnimUpdateContext) -> AnimResult {
        self.dt += ucx.ice.dt;

        if self.dt.as_secs_f32() >= WALK_TIME {
            AnimResult::Finish
        } else {
            AnimResult::GotoNextFrame
        }
    }
}
