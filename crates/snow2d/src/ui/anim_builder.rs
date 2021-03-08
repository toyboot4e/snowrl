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

    pub fn color(&mut self, delta: impl Into<Delta<Color>>) -> &mut Self {
        let delta = delta.into();

        let index = self.anims.insert(ColorTween {
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

    pub fn alpha(&mut self, delta: impl Into<Delta<u8>>) -> &mut Self {
        let delta = delta.into();

        let index = self.anims.insert(AlphaTween {
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

    pub fn pos(&mut self, delta: impl Into<Delta<Vec2f>>) -> &mut Self {
        let delta = delta.into();

        let index = self.anims.insert(PosTween {
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

    pub fn size(&mut self, delta: impl Into<Delta<Vec2f>>) -> &mut Self {
        let delta = delta.into();

        let index = self.anims.insert(SizeTween {
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