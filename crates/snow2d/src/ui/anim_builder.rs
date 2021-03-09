/*!
UI node animation builder
*/

use crate::{
    gfx::{geom2d::Vec2f, Color},
    ui::{anim::*, node::Node, AnimArena},
    utils::{arena::Index, ez, pool::Handle},
};

/// Internaly utility to provide with fluent API to supply two values
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Delta<T> {
    pub a: T,
    pub b: T,
}

impl<T, U: Into<T>, V: Into<T>> From<(U, V)> for Delta<T> {
    fn from(xs: (U, V)) -> Self {
        let (a, b) = xs;
        Self {
            a: a.into(),
            b: b.into(),
        }
    }
}

impl<T, U: Into<T>> From<[U; 2]> for Delta<T> {
    fn from(xs: [U; 2]) -> Self {
        let [a, b] = xs;
        Self {
            a: a.into(),
            b: b.into(),
        }
    }
}

/// Fluent API to create animation objects
pub struct AnimBuilder<'a> {
    anims: &'a mut AnimArena,
    node: Option<Handle<Node>>,
    dt: ez::EasedDt,
    /// Built animation handles
    pub built: Vec<Index<Anim>>,
}

impl<'a> AnimBuilder<'a> {
    pub fn new(anims: &'a mut AnimArena) -> Self {
        Self {
            anims,
            node: None,
            dt: ez::EasedDt::new(0.0, ez::Ease::Linear),
            built: Vec::with_capacity(4),
        }
    }

    pub fn clear_log(&mut self) {
        self.built.clear();
    }

    pub fn node<'x>(&mut self, node: &Handle<Node>) -> &mut Self {
        self.node = Some(node.clone());
        self
    }

    pub fn dt(&mut self, dt: ez::EasedDt) -> &mut Self {
        self.dt = dt;
        self
    }

    pub fn secs(&mut self, secs: f32) -> &mut Self {
        self.dt.target = secs;
        self
    }

    pub fn ease(&mut self, ease: ez::Ease) -> &mut Self {
        self.dt.ease = ease;
        self
    }
}

macro_rules! add_tween {
    ($Tween:ident, $name:ident, $data:ident) => {
        impl<'a> AnimBuilder<'a> {
            pub fn $name(&mut self, delta: impl Into<Delta<$data>>) -> &mut Self {
                let delta = delta.into();

                let index = self.anims.insert($Tween {
                    tween: ez::Tweened {
                        a: delta.a,
                        b: delta.b,
                        dt: self.dt,
                    },
                    node: self.node.clone().unwrap(),
                });
                self.built.push(index);

                self
            }
        }
    };
}

add_tween!(PosTween, pos, Vec2f);
add_tween!(XTween, x, f32);
add_tween!(YTween, y, f32);
add_tween!(SizeTween, size, Vec2f);
add_tween!(ColorTween, color, Color);
add_tween!(AlphaTween, alpha, u8);
add_tween!(RotTween, rot, f32);

