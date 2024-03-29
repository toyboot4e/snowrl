/*!
UI scenes
*/

pub mod title;

use snow2d::{ui::Ui, Ice};

use rlbox::rl::grid2d::*;

use grue2d::game::data::res::Resources;

#[derive(Debug, PartialEq)]
pub struct Title {
    cfg: title::ColorConfig,
    state: title::TitleState,
    assets: title::TitleAssets,
    nodes: title::TitleNodes,
    anims: title::TitleAnims,
}

impl Title {
    pub fn new(ice: &mut Ice, ui: &mut Ui) -> Self {
        let assets = &mut ice.assets;

        let cfg = title::ColorConfig::default();
        let state = title::TitleState { cursor: 0 };
        let mut assets = title::TitleAssets::new(assets);

        let nodes = title::TitleNodes::new(&cfg, &mut ui.nodes, &mut assets);
        let cursor = 0;
        let anims = title::TitleAnims::init(&cfg, &mut ui.anims, &nodes, cursor);

        Self {
            cfg,
            state,
            assets,
            nodes,
            anims,
        }
    }

    pub fn handle_input(&mut self, ice: &mut Ice, res: &mut Resources) -> Option<title::Choice> {
        if let Some(dir) = res.vi.dir.dir4_pressed() {
            let y_sign = dir.y_sign();

            if y_sign != Sign::Neutral {
                ice.audio.play(&*self.assets.se_cursor.get_mut().unwrap());
            }

            let mut pos = self.state.cursor;
            match y_sign {
                Sign::Pos => {
                    pos += 1;
                    pos %= self.nodes.choices.len();
                }
                Sign::Neg => {
                    pos += self.nodes.choices.len() - 1;
                    pos %= self.nodes.choices.len();
                }
                Sign::Neutral => {}
            };

            if pos != self.state.cursor {
                self.anims
                    .select(&self.cfg, &self.nodes, &mut res.ui, self.state.cursor, pos);
                self.state.cursor = pos;
            }

            return None;
        }

        if res.vi.select.is_pressed() {
            ice.audio.play(&*self.assets.se_select.get_mut().unwrap());

            self.anims.on_exit(&mut res.ui.anims, &self.nodes);
            return Some(title::Choice::from_usize(self.state.cursor).unwrap());
        }

        None
    }
}
