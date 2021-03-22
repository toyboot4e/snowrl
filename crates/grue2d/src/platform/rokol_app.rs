//! `rokol:app` support

use rokol::app::{self as ra, glue::Rokol, RApp};

/// Runs [`RApp`], which provides 60 FPS fixed-timestep game loop
pub fn run<App: RApp, AppConstructor: FnOnce(Rokol) -> App>(
    rokol: Rokol,
    constructor: AppConstructor,
) -> ra::glue::Result {
    rokol.run(&mut Runner {
        init_rokol: Some(rokol.clone()),
        init: Some(constructor),
        x: None,
    })
}

/// Creates [`RApp`] _after_ creating `rokol::gfx` contexts
struct Runner<T: RApp, F: FnOnce(Rokol) -> T> {
    init_rokol: Option<Rokol>,
    /// Use `Option` for lazy initialization
    init: Option<F>,
    /// Use `Option` for lazy initialization
    x: Option<T>,
}

impl<T: RApp, F: FnOnce(Rokol) -> T> rokol::app::RApp for Runner<T, F> {
    fn init(&mut self) {
        rg::setup(&mut rokol::app::glue::app_desc());
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

/// Lifecycle methods to be used by a driver of `Fsm`
impl GrueRl {
    pub fn event(&mut self, ev: &Event) {
        self.data.ice.event(ev);
    }

    pub fn update(&mut self) {
        self.pre_update();
        self.fsm.update(&mut self.data);
        self.post_update();
    }

    pub fn pre_render(&mut self) {
        self.data.ice.pre_render(snow2d::input::WindowState {
            w: rokol::app: w(),
            h: rokol::app: h(),
        });
    }

    pub fn on_end_frame(&mut self) {
        self.data.ice.on_end_frame();
    }
}
