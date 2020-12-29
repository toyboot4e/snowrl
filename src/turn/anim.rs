//! Animation framework for the roguelike game

use downcast_rs::{impl_downcast, Downcast};

use std::{
    any::{Any, TypeId},
    fmt,
    time::Duration,
};

use crate::turn::ev;

pub struct AnimPlayer {
    anims: Vec<Box<dyn Anim>>,
    is_top_walk: bool,
}

impl AnimPlayer {
    pub fn new() -> Self {
        Self {
            anims: Vec::with_capacity(10),
            is_top_walk: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.anims.is_empty()
    }

    /// Push animation boxing it
    pub fn push<T: Anim + 'static>(&mut self, anim: T) {
        self.anims.push(Box::new(anim));
    }

    /// Push boxed animation
    pub fn push_boxed(&mut self, anim: Box<dyn Anim>) {
        if (*anim).as_any().is::<WalkAnim>() {
            self.push_walk_anim();
            self.is_top_walk = true;
        } else {
            self.anims.push(anim);
            self.is_top_walk = false;
        }
    }

    /// Multiple walk animations should be run as a batched animation (so that player don't have to
    /// wait for unnecessary long time)
    fn push_walk_anim(&mut self) {
        if self.is_top_walk {
            // parallelize the walk animation
        } else {
            self.push(WalkAnim::new());
        }
    }

    pub fn on_start(&mut self) {
        assert!(
            !self.anims.is_empty(),
            "Tried to start playing stack animation while it's empty!"
        );

        let last = self.anims.last_mut().unwrap();
        last.on_start();
    }

    pub fn update(&mut self, ucx: &mut AnimUpdateContext) -> AnimResult {
        loop {
            let last = match self.anims.last_mut() {
                Some(a) => a,
                None => return AnimResult::Finished,
            };

            let res = last.update(ucx);
            if res == AnimResult::Finished {
                // TODO: wait for one frame or not?
                continue; // next animation
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
