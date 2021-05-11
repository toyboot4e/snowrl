/*!
UI node animations

Animations hold reference-counted handles to their target nodes, so nodes will be alive until
related animations are finished.
*/

use dyn_clone::{clone_trait_object, DynClone};
use std::time::Duration;

use crate::{
    gfx::{geom2d::Vec2f, Color},
    ui::Node,
    utils::{
        enum_dispatch, ez,
        pool::{Handle, Pool},
        Inspect,
    },
};

/// Common animation lifecycle
#[enum_dispatch]
pub trait AnimImpl: std::fmt::Debug + Clone {
    /// Delayed animations are activated later
    fn is_active(&self) -> bool;
    fn set_active(&mut self, b: bool);
    fn duration(&self) -> Duration;
    fn tick(&mut self, dt: Duration);
    fn is_end(&self) -> bool;
    fn apply(&self, nodes: &mut Pool<Node>);
    fn set_accum_norm(&mut self, t: f32);
}

/// Animation function that implements basic traits
pub trait AnimFn:
    Fn(&Handle<Node>, &mut Pool<Node>, &ez::EasedDt) + std::fmt::Debug + DynClone
{
}

clone_trait_object!(AnimFn);

/// TODO: Is this needed?
impl<T> AnimFn for T where
    T: Fn(&Handle<Node>, &mut Pool<Node>, &ez::EasedDt) + std::fmt::Debug + std::clone::Clone
{
}

/// TODO: does it work?
#[derive(Debug, Clone, Inspect)]
pub struct DynAnim {
    pub is_active: bool,
    pub dt: ez::EasedDt,
    pub node: Handle<Node>,
    #[inspect(skip)]
    pub f: Box<dyn AnimFn>,
}

impl AnimImpl for DynAnim {
    fn is_active(&self) -> bool {
        self.is_active
    }

    fn set_active(&mut self, b: bool) {
        self.is_active = b;
    }

    fn duration(&self) -> Duration {
        Duration::from_secs_f32(self.dt.target)
    }

    fn tick(&mut self, dt: Duration) {
        self.dt.tick(dt);
    }

    fn is_end(&self) -> bool {
        self.dt.is_end()
    }

    fn apply(&self, nodes: &mut Pool<Node>) {
        (self.f)(&self.node, nodes, &self.dt);
    }

    fn set_accum_norm(&mut self, t: f32) {
        self.dt.set_accum_norm(t);
    }
}

macro_rules! def_tween_anim {
    ($ty:ident, $val:ident, $apply:expr) => {
        #[derive(Debug, Clone, Inspect)]
        pub struct $ty {
            pub is_active: bool,
            pub tween: ez::Tweened<$val>,
            pub node: Handle<Node>,
        }

        impl AnimImpl for $ty {
            fn is_active(&self) -> bool {
                self.is_active
            }

            fn set_active(&mut self, b: bool) {
                self.is_active = b;
            }

            fn duration(&self) -> Duration {
                Duration::from_secs_f32(self.tween.dt.target)
            }

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

def_tween_anim!(XTween, f32, |me: &Self, nodes: &mut Pool<Node>| {
    let n = &mut nodes[&me.node];
    n.params.pos.x = me.tween.get();
});

def_tween_anim!(YTween, f32, |me: &Self, nodes: &mut Pool<Node>| {
    let n = &mut nodes[&me.node];
    n.params.pos.y = me.tween.get();
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

def_tween_anim!(RotTween, f32, |me: &Self, nodes: &mut Pool<Node>| {
    let n = &mut nodes[&me.node];
    n.params.rot = Some(me.tween.get());
});
