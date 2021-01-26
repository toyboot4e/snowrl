/*!
Fog of war, shadow on the map
*/

use crate::rl::{
    fov::{self, FovData, FovWrite, OpacityMap},
    grid2d::Vec2i,
};

/// Fog of war, shadow on the map
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FowData {
    /// [w, h]
    map_size: [usize; 2],
    /// True if uncovered
    shadows: Vec<bool>,
}

impl FowData {
    pub fn new(size: [usize; 2]) -> Self {
        FowData {
            map_size: size,
            shadows: vec![false; size[0] * size[1]],
        }
    }

    pub fn clear(&mut self) {
        let area = self.map_size[0] * self.map_size[1];
        for i in 0..area {
            self.shadows[i] = false;
        }
    }

    fn ix(&self, pos: [usize; 2]) -> usize {
        pos[0] + pos[1] * self.map_size[0]
    }

    pub fn cover(&mut self, pos: [usize; 2]) {
        let ix = self.ix(pos);
        if ix >= self.shadows.len() {
            log::warn!("tried to  position out of the map: {:?}", pos);
        } else {
            self.shadows[ix] = false;
        }
    }

    pub fn uncover(&mut self, pos: [usize; 2]) {
        let ix = self.ix(pos);
        if ix >= self.shadows.len() {
            log::warn!("tried to uncover position out of the map: {:?}", pos);
        } else {
            self.shadows[ix] = true;
        }
    }

    // TODO: maybe prefer u32
    pub fn is_visible(&self, pos: [usize; 2]) -> bool {
        let ix = self.ix(pos);
        if ix >= self.shadows.len() {
            false
        } else {
            self.shadows[ix]
        }
    }
}

pub fn calculate_fov_fow<'a, 'b>(
    fov: &'a mut FovData,
    fow: &'b mut FowData,
    r: Option<u32>,
    origin: Vec2i,
    opa: &impl fov::OpacityMap,
) {
    let r = r.unwrap_or(fov.radius());

    let mut bind = FovFowWrite { fov, fow };
    let params = fov::RefreshParams { r, origin, opa };

    fov::refresh(&mut bind, params);
}

struct FovFowWrite<'a, 'b> {
    fov: &'a mut FovData,
    fow: &'b mut FowData,
}

impl<'a, 'b> FovWrite for FovFowWrite<'a, 'b> {
    fn on_refresh<T: OpacityMap>(&mut self, params: &fov::RefreshParams<T>) {
        self.fov.on_refresh(params);
    }

    fn light(&mut self, pos: Vec2i) {
        self.fov.light(pos);
        self.fow.uncover([pos.x as usize, pos.y as usize]);
    }
}
