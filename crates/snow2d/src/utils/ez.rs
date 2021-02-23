/*!
[Easing] and tweening

[easing]: https://easings.net/

# TIP

Exponential tween can be replaced with simple calculation every frame:

```no_run
fn update_follow_camera() {
    camera_position += target_delta_position * lerp_speed;
    // P(t) := target_delta_position
    // P(t + dt) = (1.0 - lerp_speed) * P(t)
    // ^ This is actually a derivative function of `exp(-Ct)`
}
```
*/

use std::{
    f32::consts::{FRAC_PI_2, PI},
    time::Duration,
};

use crate::input::Dir8;

/// Linearly interpolatable; can be [`Tweened`]
pub trait Lerp {
    /// t: [0.0, 1.0] → t': [a, b]
    fn lerp(a: Self, b: Self, t: f32) -> Self;
}

macro_rules! impl_simple_lerp {
    ($($ty:ident),* $(,)?) => {
        $(
            impl Lerp for $ty {
                fn lerp(a: Self, b: Self, t: f32) -> Self {
                    (a as f32 * (1.0-t) + b as f32 * t) as Self
                }
            }
        )*
    };
}

impl_simple_lerp!(u8, f32);

impl Lerp for [f32; 2] {
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        [f32::lerp(a[0], b[0], t), f32::lerp(a[1], b[1], t)]
    }
}

impl Lerp for crate::gfx::geom2d::Vec2f {
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        Self::new(f32::lerp(a.x, b.x, t), f32::lerp(a.y, b.y, t))
    }
}

impl Lerp for crate::gfx::Color {
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        Self::rgba(
            u8::lerp(a.r, b.r, t),
            u8::lerp(a.g, b.g, t),
            u8::lerp(a.b, b.b, t),
            u8::lerp(a.a, b.a, t),
        )
    }
}

impl crate::utils::ez::Lerp for Dir8 {
    fn lerp(a: Self, b: Self, t: f32) -> Dir8 {
        let n_steps = (b as u8 + 8 - a as u8) % 8;

        let (n_steps, step_size) = if n_steps <= 4 {
            (n_steps, 1)
        } else {
            (8 - n_steps, 8 - 1)
        };

        let current_step = (n_steps as f32 * t) as u8;

        let slot = (a as u8 + current_step * step_size) % 8;
        Self::CLOCKWISE[slot as usize]
    }
}

/// Interpolates [`Dir8`] with smallast number of frames considering rotation
pub fn tween_dirs(a: Dir8, b: Dir8, time_per_pattern: f32) -> Tweened<Dir8> {
    let n_steps = (b as u8 + 8 - a as u8) % 8;

    // we'll use shorter rotation on `lerp`
    let n_steps = u8::max(n_steps, 4);

    Tweened {
        a,
        b,
        dt: EasedDt::new(time_per_pattern * n_steps as f32, Ease::Linear),
    }
}

/// Interpolates [`Lerp`] types
pub fn tween<T: Lerp>(a: T, b: T, ease: Ease, t: f32) -> T {
    T::lerp(a, b, ease.map(t))
}

/// Generates tweened values
#[derive(Debug, Clone)]
pub struct Tweened<T: Lerp + Clone> {
    pub a: T,
    pub b: T,
    pub dt: EasedDt,
}

impl<T: Lerp + Clone + Default> Default for Tweened<T> {
    fn default() -> Self {
        Self {
            a: Default::default(),
            b: Default::default(),
            dt: Default::default(),
        }
    }
}

impl<T: Lerp + Clone> Tweened<T> {
    pub fn tick(&mut self, dt: Duration) {
        self.dt.tick(dt);
    }

    pub fn get(&self) -> T {
        T::lerp(self.a.clone(), self.b.clone(), self.dt.get())
    }

    /// Interpolation value
    pub fn t(&self) -> f32 {
        self.dt.get()
    }

    pub fn is_end(&self) -> bool {
        self.dt.is_end()
    }

    /// Begins new tween from the ending value (meaning `t = 1.0`) of the last tween
    pub fn set_next(&mut self, x: T) {
        self.a = self.b.clone();
        self.b = x;
        self.dt.reset();
    }

    pub fn set_next_and_easing(&mut self, x: T, ease: Ease) {
        self.set_next(x);
        self.dt.ease = ease;
    }

    /// Overwrites the target duration in seconds
    pub fn set_duration_secs(&mut self, target: f32) {
        self.dt.target = target;
    }

    /// Overwrites the accumulated duration
    pub fn set_accum_norm(&mut self, t: f32) {
        self.dt.accum = self.dt.target * t;
    }
}

/// Delta time `[0.0, target]` mapped to `[0.0, 1.0]` with easing on `get`
#[derive(Debug, Clone, Copy)]
pub struct EasedDt {
    target: f32,
    accum: f32,
    pub ease: Ease,
}

impl Default for EasedDt {
    fn default() -> Self {
        Self {
            target: Default::default(),
            accum: Default::default(),
            ease: Ease::Linear,
        }
    }
}

impl EasedDt {
    pub fn new(target_secs: f32, ease: Ease) -> Self {
        Self {
            target: target_secs,
            accum: 0.0,
            ease,
        }
    }

    pub fn completed() -> Self {
        Self {
            target: 1.0,
            accum: 1.0,
            ease: Ease::Linear,
        }
    }

    pub fn tick(&mut self, dt: Duration) {
        self.accum += dt.as_secs_f32();
        if self.accum > self.target {
            self.accum = self.target;
        }
    }

    pub fn reset(&mut self) {
        self.accum = 0.0;
    }

    pub fn is_end(&self) -> bool {
        self.accum >= self.target
    }

    pub fn get(&self) -> f32 {
        self.ease.map(self.accum / self.target)
    }

    pub fn set_target(&mut self, secs: f32) {
        self.target = secs;
    }
}

/// Delta time `[0.0, target]` mapped to `[0.0, 1.0]` on `get`
#[derive(Debug, Clone, Default)]
pub struct LinearDt {
    target: f32,
    accum: f32,
}

impl LinearDt {
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

    pub fn is_end(&self) -> bool {
        self.accum > self.target
    }

    /// Interpolation vaule in range `[0.0, 1.0]`
    pub fn get(&self) -> f32 {
        self.accum / self.target
    }

    pub fn get_eased(&self, ease: Ease) -> f32 {
        ease.map(self.get())
    }
}

/// Easing function dispatched dynamically
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
    ExpIn,
    ExpOut,
    ExpIo,
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
            Self::ExpIn => self::exp_in(t),
            Self::ExpOut => self::exp_out(t),
            Self::ExpIo => self::exp_io(t),
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
