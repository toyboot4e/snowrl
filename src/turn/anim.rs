/*!

Animations for the roguelike game

They're created referencing rogulike events and then we forget about original events.

*/

use downcast_rs::{impl_downcast, Downcast};

use std::{collections::VecDeque, fmt, time::Duration};

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
        if (*anim).as_any().is::<WalkAnim>() {
            self.push_walk_anim();
            self.is_top_walk = true;
        } else {
            self.anims.push_back(anim);
            self.is_top_walk = false;
        }
    }

    /// Multiple walk animations should be run as a batched animation (so that player don't have to
    /// wait for unnecessary long time)
    fn push_walk_anim(&mut self) {
        if self.is_top_walk {
            // parallelize the walk animation
        } else {
            self.enqueue(WalkAnim::new());
        }
    }

    pub fn on_start(&mut self) {
        assert!(
            !self.anims.is_empty(),
            "Tried to start playing animation stack while it's empty!"
        );

        let last = self.anims.front_mut().unwrap();
        last.on_start();

        log::trace!("animation queue: {:?}", self.anims);
    }

    fn on_exit(&mut self) {
        self.is_top_walk = false;
    }

    pub fn update(&mut self, ucx: &mut AnimUpdateContext) -> AnimResult {
        loop {
            let last = match self.anims.front_mut() {
                Some(a) => a,
                None => {
                    self.on_exit();
                    return AnimResult::Finished;
                }
            };

            // TODO: separate `AnimResult` and `AnimPlayerResult`
            let res = last.update(ucx);
            if res == AnimResult::Finished {
                self.anims.pop_front();

                if let Some(last) = self.anims.front_mut() {
                    last.on_start();
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
pub struct AnimUpdateContext {
    pub dt: Duration,
}

pub trait Anim: fmt::Debug + Downcast {
    fn on_start(&mut self) {}
    fn update(&mut self, ucx: &mut AnimUpdateContext) -> AnimResult;
}

impl_downcast!(Anim);

// do not impl `Anim` for `Box<dyn Anim>`

/// Walk animation is currently run automatically, so we just wait for it to finish
#[derive(Debug)]
pub struct WalkAnim {
    pub dt: Duration,
}

impl WalkAnim {
    pub fn new() -> Self {
        Self {
            dt: Duration::new(0, 0),
        }
    }
}

impl Anim for WalkAnim {
    fn update(&mut self, ucx: &mut AnimUpdateContext) -> AnimResult {
        self.dt += ucx.dt;
        if self.dt.as_secs_f32() >= (crate::consts::WALK_TIME - crate::consts::HALF_FRAME) {
            AnimResult::Finished
        } else {
            AnimResult::Continue
        }
    }
}
