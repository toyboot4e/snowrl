/*!
SnowRL implemented as set of plugins for `grue2d`
*/

pub extern crate grue2d;

pub mod init;
pub mod utils;

pub mod play;
pub mod prelude;
pub mod scenes;
pub mod states;

use anyhow::*;
use grue2d::{
    app::Platform,
    fsm::Fsm,
    game::{Control, Data},
    GrueRl, Plugin,
};

#[derive(Debug)]
pub struct PluginA {}

impl Plugin for PluginA {
    fn init_game(&mut self) -> Result<(Platform, (Data, Control, Fsm))> {
        let [w, h] = [1280, 720];

        let platform = crate::init::gen_app(w, h)?;
        let (data, ctrl, fsm) = crate::init::new_game(w, h)?;

        Ok((platform, (data, ctrl, fsm)))
    }

    fn on_load(&mut self, _grue: &mut GrueRl, _platfrom: &mut Platform) {
        // do something
    }
}

#[no_mangle]
pub extern "C" fn load() -> Box<dyn Plugin> {
    Box::new(PluginA {})
}
