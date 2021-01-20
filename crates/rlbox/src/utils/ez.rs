/*!

Easing and tweening

Easing functions are visualized in [this site] for example. They map `[0.0, 1.0]` to a different
curve in the same range `[0.0, 1.0]`. They can be applied to other types that are linearly
interpolatable:

[this site]: https://easings.net/

```no_run
pub fn tween<T: Lerp>(a: T, b: T, ease: Ease, t: f32) -> T {
    T::lerp(a, b, ease.map(t))
}
```

Tweens are applied based on time.

*/

use std::{
    f32::consts::{FRAC_PI_2, PI},
    time::Duration,
};

/// Linearly interpolatable, which can be [`Tweened`]
pub trait Lerp {
    fn lerp(a: Self, b: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    /// t: [0.0, 1.0] → t': [a, b]
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        a + t * (b - a)
    }
}

pub fn tween<T: Lerp>(a: T, b: T, ease: Ease, t: f32) -> T {
    T::lerp(a, b, ease.map(t))
}

/// Generates tweened values
#[derive(Debug)]
pub struct Tweened<T: Lerp + Clone> {
    pub a: T,
    pub b: T,
    pub dt: EasedDt,
}

impl<T: Lerp + Clone> Tweened<T> {
    pub fn tick(&mut self, dt: Duration) {
        self.dt.tick(dt);
    }

    pub fn get(&self) -> T {
        T::lerp(self.a.clone(), self.b.clone(), self.dt.get())
    }
}

/// Delta time `[0.0, target]` mapped to `[0.0, 1.0]` with easing on `get`
#[derive(Debug, Clone)]
pub struct EasedDt {
    target: f32,
    accum: f32,
    pub ease: Ease,
}

impl EasedDt {
    pub fn new(target_secs: f32, ease: Ease) -> Self {
        Self {
            target: target_secs,
            accum: 0.0,
            ease,
        }
    }

    pub fn reset(&mut self) {
        self.accum = 0.0;
    }

    pub fn tick(&mut self, dt: Duration) {
        self.accum += dt.as_secs_f32();
        if self.accum > self.target {
            self.accum = self.target;
        }
    }

    pub fn get(&self) -> f32 {
        self.ease.map(self.accum / self.target)
    }
}

/// Delta time `[0.0, target]` mapped to `[0.0, 1.0]` on `get`
#[derive(Debug, Clone, Default)]
pub struct LerpDt {
    target: f32,
    accum: f32,
}

impl LerpDt {
    pub fn new(target_secs: f32) -> Self {
        Self {
            target: target_secs,
            accum: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.accum = 0.0;
    }

    pub fn tick(&mut self, dt: Duration) {
        self.accum += dt.as_secs_f32();
        if self.accum > self.target {
            self.accum = self.target;
        }
    }

    /// Interpolation vaule in range `[0.0, 1.0]`
    pub fn get(&self) -> f32 {
        self.accum / self.target
    }

    pub fn eased(&self, ease: Ease) -> f32 {
        ease.map(self.get())
    }
}

/// Easing function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ease {
    Linear,
    //
    QuadIn,
    QuadOut,
    QuadIo,
    //
    CubicInt,
    CubicOut,
    CubicIo,
    //
    QuartIn,
    QuartOut,
    QuartIo,
    //
    QuintIn,
    QuintOut,
    QuintIo,
    //
    SinIn,
    SinOut,
    SinIo,
    //
    CircIn,
    CircOut,
    CircIo,
    //
    ExpoIn,
    ExpoOut,
    ExpoIo,
    //
    ElasticIn,
    ElasticOut,
    ElasticIo,
    //
    BackIn,
    BackOut,
    BackIo,
    //
    BounceIn,
    BounceOut,
    BounceIo,
}

impl Ease {
    /// t: [0.0, 1.0] → t': [0.0 1.0]
    pub fn map(&self, t: f32) -> f32 {
        match self {
            Self::Linear => self::linear(t),
            //
            Self::QuadIn => self::quad_in(t),
            Self::QuadOut => self::quad_out(t),
            Self::QuadIo => self::quad_inout(t),
            //
            Self::CubicInt => self::cubic_in(t),
            Self::CubicOut => self::cubic_out(t),
            Self::CubicIo => self::cubic_io(t),
            //
            Self::QuartIn => self::quart_in(t),
            Self::QuartOut => self::quart_out(t),
            Self::QuartIo => self::quart_io(t),
            //
            Self::QuintIn => self::quint_in(t),
            Self::QuintOut => self::quint_out(t),
            Self::QuintIo => self::quint_io(t),
            //
            Self::SinIn => self::sin_in(t),
            Self::SinOut => self::sin_out(t),
            Self::SinIo => self::sin_io(t),
            //
            Self::CircIn => self::circ_in(t),
            Self::CircOut => self::circ_out(t),
            Self::CircIo => self::circ_io(t),
            //
            Self::ExpoIn => self::exp_in(t),
            Self::ExpoOut => self::exp_out(t),
            Self::ExpoIo => self::exp_io(t),
            //
            Self::ElasticIn => self::elastic_in(t),
            Self::ElasticOut => self::elastic_out(t),
            Self::ElasticIo => self::elastic_io(t),
            //
            Self::BackIn => self::back_in(t),
            Self::BackOut => self::back_out(t),
            Self::BackIo => self::back_io(t),
            //
            Self::BounceIn => self::bounce_in(t),
            Self::BounceOut => self::bounce_out(t),
            Self::BounceIo => self::bounce_io(t),
        }
    }
}

// --------------------------------------------------------------------------------
// functions

#[inline]
pub fn linear(t: f32) -> f32 {
    t
}

#[inline]
pub fn quad_in(t: f32) -> f32 {
    t * t
}

#[inline]
pub fn quad_out(t: f32) -> f32 {
    -t * (t - 2.0)
}

#[inline]
pub fn quad_inout(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        (-2.0 * t * t) + (4.0 * t) - 1.0
    }
}

#[inline]
pub fn cubic_in(t: f32) -> f32 {
    t * t * t
}

#[inline]
pub fn cubic_out(t: f32) -> f32 {
    let f = t - 1.0;
    f * f * f + 1.0
}

#[inline]
pub fn cubic_io(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let f = (2.0 * t) - 2.0;
        0.5 * f * f * f + 1.0
    }
}

#[inline]
pub fn quart_in(t: f32) -> f32 {
    t * t * t * t
}

#[inline]
pub fn quart_out(t: f32) -> f32 {
    let f = t - 1.0;
    f * f * f * (1.0 - t) + 1.0
}

#[inline]
pub fn quart_io(t: f32) -> f32 {
    if t < 0.5 {
        8.0 * t * t * t * t
    } else {
        let f = t - 1.0;
        -8.0 * f * f * f * f + 1.0
    }
}

#[inline]
pub fn quint_in(t: f32) -> f32 {
    t * t * t * t * t
}

#[inline]
pub fn quint_out(t: f32) -> f32 {
    let f = t - 1.0;
    f * f * f * f * f + 1.0
}

#[inline]
pub fn quint_io(t: f32) -> f32 {
    if t < 0.5 {
        16.0 * t * t * t * t * t
    } else {
        let f = (2.0 * t) - 2.0;
        0.5 * f * f * f * f * f + 1.0
    }
}

#[inline]
pub fn sin_in(t: f32) -> f32 {
    ((t - 1.0) * FRAC_PI_2).sin() + 1.0
}

#[inline]
pub fn sin_out(t: f32) -> f32 {
    (t * FRAC_PI_2).sin()
}

#[inline]
pub fn sin_io(t: f32) -> f32 {
    0.5 * (1.0 - (t * PI).cos())
}

#[inline]
pub fn circ_in(t: f32) -> f32 {
    1.0 - (1.0 - t * t).sqrt()
}

#[inline]
pub fn circ_out(t: f32) -> f32 {
    ((2.0 - t) * t).sqrt()
}

#[inline]
pub fn circ_io(t: f32) -> f32 {
    if t < 0.5 {
        0.5 * (1.0 - (1.0 - 4.0 * t * t).sqrt())
    } else {
        0.5 * ((-(2.0 * t - 3.0) * (2.0 * t - 1.0)).sqrt() + 1.0)
    }
}

#[inline]
pub fn exp_in(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else {
        2.0f32.powf(10.0 * (t - 1.0))
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(float_cmp))]
#[inline]
pub fn exp_out(t: f32) -> f32 {
    if t == 1.0 {
        1.0
    } else {
        1.0 - 2.0f32.powf(-10.0 * t)
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(float_cmp))]
#[inline]
pub fn exp_io(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else if t < 0.5 {
        0.5 * 2.0f32.powf(20.0 * t - 10.0)
    } else {
        -0.5 * 2.0f32.powf(-20.0 * t + 10.0) + 1.0
    }
}

#[inline]
pub fn elastic_in(t: f32) -> f32 {
    (13.0 * FRAC_PI_2 * t).sin() * 2.0f32.powf(10.0 * (t - 1.0))
}

#[inline]
pub fn elastic_out(t: f32) -> f32 {
    (-13.0 * FRAC_PI_2 * (t + 1.0)).sin() * 2.0f32.powf(-10.0 * t) + 1.0
}

#[inline]
pub fn elastic_io(t: f32) -> f32 {
    if t < 0.5 {
        0.5 * (13.0 * FRAC_PI_2 * 2.0 * t).sin() * 2.0f32.powf(10.0 * (2.0 * t - 1.0))
    } else {
        0.5 * ((-13.0 * FRAC_PI_2 * 2.0 * t).sin() * 2.0f32.powf(-10.0 * (2.0 * t - 1.0)) + 2.0)
    }
}

#[inline]
pub fn back_in(t: f32) -> f32 {
    t * t * t - t * (t * PI).sin()
}

#[inline]
pub fn back_out(t: f32) -> f32 {
    let f = 1.0 - t;
    1.0 - f * f * f + f * (f * PI).sin()
}

#[inline]
pub fn back_io(t: f32) -> f32 {
    if t < 0.5 {
        let f = 2.0 * t;
        0.5 * (f * f * f - f * (f * PI).sin())
    } else {
        let f = 2.0 - 2.0 * t;
        0.5 * (1.0 - (f * f * f - f * (f * PI).sin())) + 0.5
    }
}

#[inline]
pub fn bounce_in(t: f32) -> f32 {
    1.0 - bounce_out(1.0 - t)
}

#[inline]
pub fn bounce_out(t: f32) -> f32 {
    if t < 4.0 / 11.0 {
        121.0 / 16.0 * t * t
    } else if t < 8.0 / 11.0 {
        363.0 / 40.0 * t * t - 99.0 / 10.0 * t + 17.0 / 5.0
    } else if t < 9.0 / 10.0 {
        4356.0 / 361.0 * t * t - 35442.0 / 1805.0 * t + 16061.0 / 1805.0
    } else {
        54.0 / 5.0 * t * t - 513.0 / 25.0 * t + 268.0 / 25.0
    }
}

#[inline]
pub fn bounce_io(t: f32) -> f32 {
    if t < 0.5 {
        0.5 * bounce_in(t * 2.0)
    } else {
        0.5 * bounce_out(t * 2.0 - 1.0) + 0.5
    }
}
