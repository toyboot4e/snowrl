/*!

Snow the roguelike game

*/

// use generator (unstable Rust)
#![feature(generators, generator_trait)]

// re-export mainly dependent crates
pub extern crate rlbox;
pub extern crate rokol;
pub extern crate snow2d;

pub mod utils;

pub mod fsm;
pub mod script;
pub mod turn;
pub mod world;

use rokol::{
    app::{self as ra, RApp},
    gfx as rg, Result, Rokol,
};

pub fn run<T: RApp, F: FnOnce(Rokol) -> T>(rokol: Rokol, init: F) -> Result {
    rokol.run(&mut Runner {
        init_rokol: Some(rokol.clone()),
        init: Some(init),
        x: None,
    })
}

/// Runs [`RApp`], which provides 60 FPS fixed-timestep game loop
pub struct Runner<T: RApp, F: FnOnce(Rokol) -> T> {
    init_rokol: Option<Rokol>,
    init: Option<F>,
    /// Use `Option` for lazy initialization
    x: Option<T>,
}

impl<T: RApp, F: FnOnce(Rokol) -> T> rokol::app::RApp for Runner<T, F> {
    fn init(&mut self) {
        rg::setup(&mut rokol::glue::app_desc());
        let f = self.init.take().unwrap();
        self.x = Some(f(self.init_rokol.take().unwrap()));
    }

    fn event(&mut self, ev: &ra::Event) {
        if let Some(x) = self.x.as_mut() {
            x.event(ev);
        }
    }

    fn frame(&mut self) {
        if let Some(x) = self.x.as_mut() {
            x.frame();
        }
    }
}
