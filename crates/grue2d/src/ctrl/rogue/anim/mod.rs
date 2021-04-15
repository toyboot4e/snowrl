/*!
Animations for the builtin events

Animations are created referencing rogulike events and original events will be forgot.
*/

mod blocking;
pub use blocking::*;
mod parallel;
pub use parallel::*;

use {
    downcast_rs::{impl_downcast, Downcast},
    std::{collections::VecDeque, fmt, time::Duration},
};

use crate::Data;

/// Utility for implementing animations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Timer {
    /// Do not tick on first frame
    is_started: bool,
    dt: Duration,
    target_duration: Duration,
}

/// Creation
impl Timer {
    pub fn new(dt: Duration, duration: Duration) -> Self {
        Self {
            is_started: false,
            dt,
            target_duration: duration,
        }
    }

    pub fn from_duration(duration: Duration) -> Self {
        Self::new(Duration::new(0, 0), duration)
    }

    pub fn from_secs_f32(secs: f32) -> Self {
        let ns = 1_000_000_000.0 * secs;
        Self::from_duration(Duration::from_nanos(ns as u64))
    }

    pub fn from_frames(frames: u64) -> Self {
        // FIXME: hard-coded FPS
        let ns = 1_000_000_000 * frames / 60;
        Self::from_duration(Duration::from_nanos(ns))
    }
}

impl Timer {
    /// Target duration
    pub fn target(&self) -> Duration {
        self.target_duration
    }

    pub fn set_started(&mut self, b: bool) {
        self.is_started = b;
    }
}

/// Lifecycle
impl Timer {
    /// Ticks the timer and returns if it's finished
    pub fn tick(&mut self, dt: Duration) -> bool {
        if !self.is_started {
            // on the first frame, no duration has passed
            self.is_started = true;
        } else {
            self.dt += dt;
        }
        self.dt > self.target_duration
    }

    pub fn tick_as_result(&mut self, dt: Duration) -> AnimResult {
        if self.tick(dt) {
            AnimResult::Finish
        } else {
            AnimResult::GotoNextFrame
        }
    }
}

/// Return value of [`Anim::update`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimResult {
    GotoNextFrame,
    Finish,
}

/// Roguelike animation object
pub trait Anim: fmt::Debug + Downcast {
    fn on_start(&mut self, _ucx: &mut Data) {}
    fn update(&mut self, ucx: &mut Data) -> AnimResult;
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
    /// If we have queued animation that should be played immediately without batching
    pub fn any_anim_to_run_now(&self) -> bool {
        // more than one animation or the only animation never batches other animation
        self.anims.len() > 1 || !self.is_top_walk
    }

    /// If we should play queued animations that should be batched and run at once
    pub fn any_batch(&self) -> bool {
        !self.anims.is_empty() && self.is_top_walk
    }

    /// Add animation boxing it
    pub fn enqueue<T: Anim + 'static>(&mut self, anim: T) {
        self.anims.push_back(Box::new(anim));
    }

    /// Add pre-boxed animation
    pub fn enqueue_box(&mut self, anim: Box<dyn Anim>) {
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

    pub fn on_start(&mut self, data: &mut Data) {
        assert!(
            !self.anims.is_empty(),
            "Tried to start playing animation stack while it's empty!"
        );

        let front = self.anims.front_mut().unwrap();
        front.on_start(data);

        // log::trace!("animation queue: {:?}", self.anims);
    }

    fn on_exit(&mut self) {
        self.is_top_walk = false;
    }
}

/// Lifecycle
impl AnimPlayer {
    pub fn update(&mut self, data: &mut Data) -> AnimResult {
        loop {
            let front = match self.anims.front_mut() {
                Some(a) => a,
                None => {
                    self.on_exit();
                    return AnimResult::Finish;
                }
            };

            // TODO: separate `AnimResult` and `AnimPlayerResult`
            let res = front.update(data);

            if res == AnimResult::Finish {
                self.anims.pop_front();

                if let Some(front) = self.anims.front_mut() {
                    front.on_start(data);
                }

                continue; // process next animation
            }

            return res;
        }
    }
}
