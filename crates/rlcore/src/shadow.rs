/*!
Field of view & fog of war
*/

mod fov;
mod fow;

pub use self::{fov::*, fow::*};

use crate::grid2d::Vec2i;

/// Updates FoV and FoW in one iteration
pub fn refresh_fov_fow<'a, 'b>(
    fov: &'a mut FovData,
    fow: &'b mut FowData,
    r: Option<u32>,
    origin: Vec2i,
    opa: &impl fov::OpacityMap,
) {
    let r = r.unwrap_or(fov.radius());

    let mut bind = FovFowWrite { fov, fow };
    let params = fov::FovRefreshParams { r, origin, opa };

    fov::refresh_fov(&mut bind, params);
}

struct FovFowWrite<'a, 'b> {
    fov: &'a mut FovData,
    fow: &'b mut FowData,
}

impl<'a, 'b> FovWrite for FovFowWrite<'a, 'b> {
    fn on_refresh<T: OpacityMap>(&mut self, params: &fov::FovRefreshParams<T>) {
        self.fov.on_refresh(params);
    }

    fn light(&mut self, pos: Vec2i) {
        self.fov.light(pos);
        self.fow.uncover([pos.x as usize, pos.y as usize]);
    }
}
