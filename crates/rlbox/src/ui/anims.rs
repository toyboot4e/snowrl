/*!

OMG animations

Animations hold reference-counted handles to their target nodes, so nodes will be alive until
related animations are finished.

*/

use snow2d::gfx::{geom2d::Vec2f, Color};
use std::time::Duration;

use crate::{
    ui::node::{Geom, Node},
    utils::{
        ez,
        pool::{Handle, Pool},
    },
};

/// Any kind of animation
#[derive(Debug)]
pub enum Anim {
    Seq(SeqAnims),
    Parallel(ParallelAnims),
    PosTween(PosTween),
    // PatternAnim(
}

impl Anim {
    pub fn tick(&mut self, dt: Duration) {
        match self {
            Self::Seq(x) => x.tick(dt),
            Self::Parallel(x) => x.tick(dt),
            Self::PosTween(x) => x.tick(dt),
        }
    }

    pub fn is_end(&self) -> bool {
        match self {
            Self::Seq(x) => x.is_end(),
            Self::Parallel(x) => x.is_end(),
            Self::PosTween(x) => x.is_end(),
        }
    }
}

#[derive(Debug)]
pub struct SeqAnims {
    anims: Box<Vec<Anim>>,
    pos: usize,
}

impl SeqAnims {
    fn tick(&mut self, dt: Duration) {
        for a in &mut *self.anims {
            a.tick(dt);
        }
    }

    fn is_end(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct ParallelAnims {
    anims: Box<Vec<Anim>>,
}

impl ParallelAnims {
    fn tick(&mut self, dt: Duration) {
        for a in &mut *self.anims {
            a.tick(dt);
        }
    }

    fn is_end(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct PosTween {
    pub tween: ez::Tweened<Vec2f>,
    pub node: Handle<Node>,
}

impl PosTween {
    fn tick(&mut self, _dt: Duration) {
        //
    }

    fn is_end(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct ColorTween {
    pub tween: ez::Tweened<Color>,
    pub node: Handle<Node>,
}

// pub struct PatternAnim<T>

#[derive(Debug)]
pub struct Tween<T> {
    pub target: *mut T,
    pub target_handle: Handle<Node>,
}
