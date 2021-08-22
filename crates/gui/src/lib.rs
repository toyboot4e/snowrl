/*!
SnowRL GUI application framework ([`core`] > [`model`] > [`view`] > `gui`)

TODO: rename rlview to view
*/

#![feature(drain_filter)]

pub use model;
pub use rlcore;
pub use view;

pub mod content;
pub mod fsm;
pub mod markup;
pub mod prelude;
pub mod renderer;
pub mod res;
pub mod spawn;
pub mod window;

use snow2d::{
    utils::{arena::Arena, Inspect},
    Ice,
};

use model::{GameSystem, Model};

use view::{
    actor::ActorView,
    camera::{Camera2d, FollowCamera2d},
    map::MapView,
    shadow::Shadow,
};

use crate::res::Resources;

/// Passive data to be operated on
#[derive(Debug)]
pub struct Data {
    /// Roguelike game system and the internal game state
    pub system: GameSystem,
    /// Generic game context
    pub ice: Ice,
    /// SnowRL GUi
    pub gui: Gui,
    /// Data specific to SnowRL
    pub res: Resources,
    /// How we run the game
    pub cfg: GameConfig,
}

/// Collection of GUI
#[derive(Debug)]
pub struct Gui {
    /// View model synced to the game model
    pub vm: Model,
    pub entities: Arena<ActorView>,
    pub map: MapView,
    /// Double buffer of FoV/FoW with interpolation value
    pub shadow: Shadow,
    /// Where we see
    pub cam: Camera2d,
    /// State for the camera to follow the player
    pub cam_follow: FollowCamera2d,
}

/// Lifecycle
impl Gui {
    pub fn update(&mut self, ice: &mut Ice) {
        // FIXME: impl Into itermut
        for (_ix, view) in &mut self.entities {
            let model = &self.vm.entities[view.model];
            view.img.update(ice.dt(), model.pos, model.dir);
        }
    }
}

#[derive(Debug, Clone, Inspect)]
pub struct GameConfig {
    /// Global sound volume
    pub vol: f32,
    pub shadow_cfg: ShadowConfig,
    pub snow_cfg: SnowConfig,
}

impl GameConfig {
    //
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Inspect)]
pub enum ShadowConfig {
    Blur,
    Raw,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Inspect)]
pub enum SnowConfig {
    Blizzard,
    // Light,
    None,
}
