/*!
Text
*/

use {rlbox::utils::ez, std::time::Duration};

use snow2d::gfx::{geom2d::*, RenderPass};

use crate::utils::consts;

/// State to play text
#[derive(Debug, Default)]
pub struct PlayText {
    txt: String,
    n_chars: usize,
    pos: Vec2f,
    dt: ez::LerpDt,
}

impl PlayText {
    pub fn new(txt: impl Into<String>, pos: Vec2f) -> Self {
        let mut me = Self::default();
        me.init(txt, pos);
        me
    }

    /// Initializes `self` for next text play
    pub fn init(&mut self, txt: impl Into<String>, pos: Vec2f) {
        self.txt = txt.into();
        self.pos = pos;

        // count visible characters
        self.n_chars = self
            .txt
            .chars()
            .filter(|c| !matches!(*c, '\n' | '\t'))
            .count();

        self.dt = ez::LerpDt::new(self.n_chars as f32 / consts::CHARS_PER_SEC);
    }

    pub fn update(&mut self, dt: Duration) {
        self.dt.tick(dt);
    }

    pub fn render(&mut self, screen: &mut RenderPass<'_>) {
        // consider tween
        // let t = self.dt.get();

        // let rect = [
        //     self.rect.x + self.rect.w * (1.0 - t) / 2.0,
        //     self.rect.y + self.rect.h * (1.0 - t) / 2.0,
        //     self.rect.w * t,
        //     self.rect.h * t,
        // ];

        // let color = Color::WHITE.with_alpha((255.0 * t) as u8);

        // TODO: animate texts
        screen.txt(self.pos, &self.txt);
    }
}
