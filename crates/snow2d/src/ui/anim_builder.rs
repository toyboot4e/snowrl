/*!
UI node animation builder
*/

use std::time::Duration;

use crate::{
    gfx::{geom2d::Vec2f, Color},
    ui::{anim::*, node::Node, AnimStorage, DelayedAnim},
    utils::{arena::Index, ez, pool::Handle},
};

/// Internaly utility for providing fluent API of tweens
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

#[derive(Debug, Clone, Default)]
pub struct AnimSeq {
    pub(crate) anims: Vec<DelayedAnim>,
    tot_delay: Duration,
}

impl AnimSeq {
    pub fn begin() -> (Self, AnimGen) {
        (Self::default(), AnimGen::default())
    }

    pub fn delay_at(&self, slot: usize) -> Duration {
        let mut delay = Duration::default();
        for i in 0..slot {
            delay += self.anims[i].anim.duration();
        }
        delay
    }
}

impl AnimSeq {
    pub fn append(&mut self, anim: impl Into<Anim>) -> &mut Self {
        let anim = anim.into();
        let duration = anim.duration();
        self.anims.push(DelayedAnim::new(self.tot_delay, anim));
        self.tot_delay += duration;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct AnimGen {
    pub node: Option<Handle<Node>>,
    pub dt: ez::EasedDt,
}

impl AnimGen {
    pub fn node(&mut self, node: &Handle<Node>) -> &mut Self {
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

/// Fluent API to create animation objects
#[derive(Debug)]
pub struct AnimBuilder<'a> {
    anims: &'a mut AnimStorage,
    node: Option<Handle<Node>>,
    dt: ez::EasedDt,
    /// Built animation handles
    pub built: Vec<Index<Anim>>,
}

impl<'a> AnimBuilder<'a> {
    /// Make sure to call [`node`](Self::node) after creating this builder
    pub fn new(anims: &'a mut AnimStorage) -> Self {
        Self {
            anims,
            node: None,
            dt: ez::EasedDt::new(0.0, ez::Ease::Linear),
            built: Vec::with_capacity(4),
        }
    }

    pub fn with_node(self, node: &Handle<Node>) -> Self {
        Self {
            node: Some(node.clone()),
            ..self
        }
    }

    pub fn node<'x>(&mut self, node: &Handle<Node>) -> &mut Self {
        self.node = Some(node.clone());
        self
    }

    pub fn clear_log(&mut self) {
        self.built.clear();
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
                    is_active: true,
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

        impl AnimGen {
            pub fn $name(&self, delta: impl Into<Delta<$data>>) -> $Tween {
                let delta = delta.into();

                $Tween {
                    is_active: false,
                    tween: ez::Tweened {
                        a: delta.a,
                        b: delta.b,
                        dt: self.dt,
                    },
                    node: self.node.clone().unwrap(),
                }
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
