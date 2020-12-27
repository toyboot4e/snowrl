//! Animation framework for the roguelike game

use std::time::Duration;

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

    /// Push animation boxing it
    pub fn push<T: Anim + 'static>(&mut self, anim: T) {
        self.anims.push(Box::new(anim));
    }

    /// Push boxed animatio
    pub fn push_boxed(&mut self, anim: Box<dyn Anim>) {
        self.anims.push(anim);
    }

    /// Multiple walk animations should be run as a batched animation (so that player don't have to
    /// wait for unnecessary long time)
    pub fn push_walk_anim(&mut self) {
        if self.is_top_walk {
            //
        } else {
            self.is_top_walk = true;
            self.push(WalkAnim::new());
        }
    }

    pub fn on_start(&mut self) {
        assert!(
            !self.anims.is_empty(),
            "Tried to start playing stack animation while it's empty!"
        );
    }

    pub fn update(&mut self, ucx: &mut AnimUpdateContext) -> AnimResult {
        self.anims.last_mut().unwrap().update(ucx)
    }
}

pub enum AnimResult {
    Continue,
    Finished,
}

pub struct AnimUpdateContext {
    pub dt: Duration,
}

pub trait Anim {
    fn on_start(&mut self) {}
    fn update(&mut self, ucx: &mut AnimUpdateContext) -> AnimResult;
}

/// impl `Anim` for `Box<dyn Anim>`
impl<T: Anim + ?Sized> Anim for Box<T> {
    fn on_start(&mut self) {
        (**self).on_start();
    }

    fn update(&mut self, ucx: &mut AnimUpdateContext) -> AnimResult {
        (**self).update(ucx)
    }
}

/// Walk animation is currently run automatically, so we just wait for it to finish
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
