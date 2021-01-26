//! UI scenes

pub mod title;

use std::time::Duration;

use snow2d::Ice;

use rlbox::{rl::grid2d::*, ui::node::Node, utils::pool::Pool};

use grue2d::Global;

#[derive(Debug)]
pub struct Title {
    state: title::TitleState,
    assets: title::TitleAssets,
    nodes: title::TitleNodes,
    anims: title::TitleAnims,
}

impl Title {
    pub fn new(ice: &mut Ice, pool: &mut Pool<Node>) -> Self {
        let assets = &mut ice.assets;

        let state = title::TitleState { cursor: 0 };
        let mut assets = title::TitleAssets::new(assets);
        let nodes = title::TitleNodes::new(pool, &mut assets);
        let mut anims = title::TitleAnims::default();
        anims.init();

        Self {
            state,
            assets,
            nodes,
            anims,
        }
    }

    // pub fn init(&mut self) {
    //     //
    // }

    pub fn tick(&mut self, dt: Duration) {
        self.anims.tick(dt);
    }

    pub fn handle_input(&mut self, gl: &mut Global) -> Option<title::Choice> {
        if let Some(dir) = gl.vi.dir.dir4_pressed() {
            let y_sign = dir.y_sign();

            if y_sign != Sign::Neutral {
                gl.ice
                    .audio
                    .play(&*self.assets.se_cursor.get_mut().unwrap());
            }

            match y_sign {
                Sign::Pos => {
                    self.state.cursor += self.nodes.choices.len() - 1;
                    self.state.cursor %= self.nodes.choices.len();
                }
                Sign::Neg => {
                    self.state.cursor += 1;
                    self.state.cursor %= self.nodes.choices.len();
                }
                Sign::Neutral => {}
            }
        }

        if gl.vi.select.is_pressed() {
            gl.ice
                .audio
                .play(&*self.assets.se_select.get_mut().unwrap());

            self.anims.on_exit();

            return Some(title::Choice::from_usize(self.state.cursor).unwrap());
        }

        None
    }
}
