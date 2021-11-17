/*!
SnowRL GUI application framework

Dependencies: `gui` → [`view`] → [`model`] → [`rlcore`] → [`snow2d`]
*/

#![feature(drain_filter)]

pub extern crate model;
pub extern crate rlcore;
pub extern crate view;

pub extern crate snow2d;

pub mod content;
pub mod fsm;
pub mod markup;
pub mod prelude;
pub mod renderer;
pub mod res;
pub mod spawn;
pub mod window;

#[cfg(debug_assertions)]
pub mod debug;

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
pub struct God {
    /// Roguelike game system and the internal game state
    pub sys: GameSystem,
    /// Generic 2D game context
    pub ice: Ice,
    /// SnowRL GUi
    pub gui: Gui,
    /// Data specific to SnowRL
    pub res: Resources,
    /// How we run the game
    pub cfg: GameConfig,
    /// (Debug-only) ImGUI
    #[cfg(debug_assertions)]
    pub imgui: debug::Backend,
    /// (Debug-only) Debug state
    #[cfg(debug_assertions)]
    pub debug_states: debug::DebugState,
}

impl God {
    #[cfg(debug_assertions)]
    pub fn debug_render(&mut self, window: &mut sdl2::video::Window) {
        let mut ui = self.imgui.begin_frame(window);
        self.debug_states.render(&mut ui, &mut self.gui);
        ui.end_frame(window, &mut ()).unwrap();
    }
}

/// Collection of GUI
#[derive(Debug)]
pub struct Gui {
    /// View model synced to the game model
    pub vm: Model,
    pub actors: Arena<ActorView>,
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
        for (_ix, view) in &mut self.actors {
            let mdl = &self.vm.ents[view.mdl];
            view.img.update(ice.dt(), mdl.pos, mdl.dir);
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
