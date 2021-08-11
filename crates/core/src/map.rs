/*!
Orthogonal grid map
*/

use snow2d::utils::Inspect;

use crate::{grid2d::Vec2i, shadow::OpacityMap};

/// Roguelike game map
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Inspect)]
pub struct MapModel {
    pub size: [usize; 2],
    /// True if it's physical block
    pub body_blocks: Vec<bool>,
    /// True if it's opaque
    pub view_blocks: Vec<bool>,
}

impl MapModel {
    pub fn contains(&self, pos: impl Into<Vec2i>) -> bool {
        let pos = pos.into();
        let (x, y) = (pos.x, pos.y);
        // not outsize of the map
        !(x < 0 || y < 0 || self.size[0] as i32 <= x || self.size[1] as i32 <= y)
    }

    pub fn is_body_blocked(&self, pos: impl Into<Vec2i>) -> bool {
        let pos = pos.into();

        if !self.contains(pos) {
            return true;
        }

        let ix = pos.x + self.size[0] as i32 * pos.y;
        self.body_blocks[ix as usize]
    }

    pub fn is_view_blocked(&self, pos: impl Into<Vec2i>) -> bool {
        let pos = pos.into();

        if !self.contains(pos) {
            return true;
        }

        let ix = pos.x + self.size[0] as i32 * pos.y;
        self.view_blocks[ix as usize]
    }
}

/// FoV
impl OpacityMap for MapModel {
    fn is_opaque(&self, pos: Vec2i) -> bool {
        self.is_view_blocked(pos)
    }

    fn contains(&self, pos: Vec2i) -> bool {
        <Self>::contains(self, pos)
    }
}
