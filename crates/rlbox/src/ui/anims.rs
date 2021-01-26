/*!

Animations

Animations hold reference-counted handles to their target nodes, so nodes will be alive until
related animations are finished.

*/

use std::time::Duration;

use crate::utils::enum_dispatch;
use snow2d::gfx::{geom2d::Vec2f, Color};

use crate::{
    ui::node::Node,
    utils::{ez, pool::Handle},
};

#[enum_dispatch]
pub trait AnimImpl: std::fmt::Debug + Clone {
    fn tick(&mut self, dt: Duration);
    /// TODO: remove end animations automatically
    fn is_end(&self) -> bool;
}

/// Any kind of animation
#[enum_dispatch(AnimImpl)]
#[derive(Debug, Clone)]
pub enum Anim {
    Seq,
    Parallel,
    PosTween,
}

#[derive(Debug, Clone)]
pub struct Seq {
    anims: Box<Vec<Anim>>,
    pos: usize,
}

impl AnimImpl for Seq {
    fn tick(&mut self, dt: Duration) {
        for a in &mut *self.anims {
            a.tick(dt);
        }
    }

    fn is_end(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub struct Parallel {
    anims: Box<Vec<Anim>>,
}

impl AnimImpl for Parallel {
    fn tick(&mut self, dt: Duration) {
        for a in &mut *self.anims {
            a.tick(dt);
        }
    }

    fn is_end(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub struct PosTween {
    pub tween: ez::Tweened<Vec2f>,
    pub node: Handle<Node>,
}

impl AnimImpl for PosTween {
    fn tick(&mut self, dt: Duration) {
        self.tween.tick(dt);
    }

    fn is_end(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub struct ColorTween {
    pub tween: ez::Tweened<Color>,
    pub node: Handle<Node>,
}

#[derive(Debug, Clone)]
pub struct AlphaTween {
    pub tween: ez::Tweened<f32>,
    pub node: Handle<Node>,
}

// pub struct PatternAnim<T>

#[derive(Debug)]
pub struct Tween<T> {
    pub target: *mut T,
    pub target_handle: Handle<Node>,
}
