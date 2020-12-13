use rokol::{app as ra, gfx as rg};

fn main() -> rokol::Result {
    env_logger::init();

    let rokol = rokol::Rokol {
        w: 1280,
        h: 720,
        title: "SnowRL".to_string(),
        ..Default::default()
    };

    let mut app = SnowRl::new();

    rokol.run(&mut app)
}

#[derive(Debug)]
struct SnowRl {
    /// Clears the frame color buffer on starting screen rendering pass
    pa: rg::PassAction,
}

impl SnowRl {
    pub fn new() -> Self {
        let pa = rg::PassAction::clear([100.0 / 255.0, 149.0 / 255.0, 237.0 / 255.0, 1.0]);

        Self { pa }
    }
}

impl rokol::app::RApp for SnowRl {
    fn init(&mut self) {
        rg::setup(&mut rokol::glue::app_desc());
    }

    fn frame(&mut self) {
        rg::begin_default_pass(&self.pa, ra::width(), ra::height());
        rg::end_pass();
        rg::commit();
    }
}
