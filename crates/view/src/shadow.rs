/*!
Fov/FoW view
*/

use std::time::Duration;

use snow2d::utils::ez;

use core::{grid2d::*, shadow::*};

/// Double buffer
#[derive(Debug, Clone)]
pub struct Double<T> {
    /// Front
    pub a: T,
    /// Back
    pub b: T,
}

impl<T: Default> Default for Double<T> {
    fn default() -> Self {
        Self {
            a: Default::default(),
            b: Default::default(),
        }
    }
}

impl<T> Double<T> {
    /// TODO: maybe improve efficiency
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.a, &mut self.b);
    }
}

/// FoV/FoW data for visualization
#[derive(Debug)]
pub struct Shadow {
    /// Field of view
    pub fov: Double<FovData>,
    /// Fog of war (shadow on map)
    pub fow: Double<FowData>,
    /// Interpolation value
    pub dt: ez::EasedDt,
    pub is_dirty: bool,
}

impl Shadow {
    pub fn new(radius: [u32; 2], map_size: [usize; 2], anim_secs: f32, ease: ez::Ease) -> Self {
        Self {
            fov: Double {
                a: FovData::new(radius[0], radius[1]),
                b: FovData::new(radius[0], radius[1]),
            },
            fow: Double {
                a: FowData::new(map_size),
                b: FowData::new(map_size),
            },
            dt: ez::EasedDt::new(anim_secs, ease),
            is_dirty: false,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn calculate(&mut self, origin: Vec2i, map: &impl OpacityMap) {
        // FoV is always cleared so we just swap them
        self.fov.swap();

        // FoW is continued from the previous state, so we'll copy it
        self.fow.b = self.fow.a.clone();

        self.dt.reset();

        core::shadow::refresh_fov_fow(&mut self.fov.a, &mut self.fow.a, None, origin, map);
    }

    /// Call it every frame to animate FoV
    pub fn post_update(&mut self, dt: Duration, map: &impl OpacityMap, pos: Vec2i) {
        if self.is_dirty {
            self.calculate(pos, map);
            self.is_dirty = false;
        }

        self.dt.tick(dt);
    }
}
