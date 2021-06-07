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

use std::time::Duration;

use anyhow::*;
use grue2d::{
    app::Platform,
    fsm::Fsm,
    game::{Control, Data},
    GrueRl, Plugin,
};
use rokol::gfx as rg;
use sdl2::event::Event;

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
        // it turned out don't reload C dylib with global variables that can't be swapped with
        // previous ones
        return;
    }

    fn event(&mut self, grue: &mut GrueRl, ev: &Event, platform: &mut Platform) {
        grue.event(ev, platform);
    }

    fn update(&mut self, grue: &mut GrueRl, dt: Duration, platform: &mut Platform) {
        grue.update(dt, platform);
    }

    fn render(&mut self, grue: &mut GrueRl, dt: Duration, platform: &mut Platform) {
        grue.pre_render(dt, platform);
        grue.render_default();
        grue.post_render(dt, platform);
        grue.on_end_frame();

        rg::commit();
        platform.swap_window();
    }
}

pub fn load() -> PluginA {
    PluginA {}
}
