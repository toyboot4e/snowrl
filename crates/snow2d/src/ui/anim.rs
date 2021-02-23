/*!
UI node animations

Animations hold reference-counted handles to their target nodes, so nodes will be alive until
related animations are finished.
*/

use std::time::Duration;

use crate::{
    gfx::{geom2d::Vec2f, Color},
    ui::node::Node,
    utils::{
        enum_dispatch, ez,
        pool::{Handle, Pool},
    },
};

/// Common animation lifecycle
#[enum_dispatch]
pub trait AnimImpl: std::fmt::Debug + Clone {
    fn tick(&mut self, dt: Duration);
    fn is_end(&self) -> bool;
    fn apply(&self, nodes: &mut Pool<Node>);
    fn set_accum_norm(&mut self, t: f32);
}

/// One of [`AnimImpl`] impls
#[enum_dispatch(AnimImpl)]
#[derive(Debug, Clone)]
pub enum Anim {
    PosTween,
    ColorTween,
    AlphaTween,
    SizeTween,
    // ParamsTween,
}

macro_rules! def_tween_anim {
    ($ty:ident, $val:ident, $apply:expr) => {
        #[derive(Debug, Clone)]
        pub struct $ty {
            pub tween: ez::Tweened<$val>,
            pub node: Handle<Node>,
        }

        impl AnimImpl for $ty {
            fn tick(&mut self, dt: Duration) {
                self.tween.tick(dt);
            }

            fn is_end(&self) -> bool {
                self.tween.is_end()
            }

            fn apply(&self, nodes: &mut Pool<Node>) {
                $apply(self, nodes);
            }

            fn set_accum_norm(&mut self, t: f32) {
                self.tween.set_accum_norm(t);
            }
        }
    };
}

def_tween_anim!(PosTween, Vec2f, |me: &Self, nodes: &mut Pool<Node>| {
    let n = &mut nodes[&me.node];
    n.params.pos = me.tween.get();
});

def_tween_anim!(SizeTween, Vec2f, |me: &Self, nodes: &mut Pool<Node>| {
    let n = &mut nodes[&me.node];
    n.params.size = me.tween.get();
});

def_tween_anim!(ColorTween, Color, |me: &Self, nodes: &mut Pool<Node>| {
    let n = &mut nodes[&me.node];
    n.params.color = me.tween.get();
});

def_tween_anim!(AlphaTween, u8, |me: &Self, nodes: &mut Pool<Node>| {
    let n = &mut nodes[&me.node];
    n.params.color.a = me.tween.get();
});
