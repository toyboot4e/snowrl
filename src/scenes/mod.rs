/*!
UI scenes
*/

pub mod title;

use snow2d::Ice;

use rlbox::{
    rl::grid2d::*,
    ui::{Layer, Ui},
    utils::arena::Index,
};

use grue2d::Global;

#[derive(Debug)]
pub struct Title {
    cfg: title::ColorConfig,
    state: title::TitleState,
    assets: title::TitleAssets,
    nodes: title::TitleNodes,
    anims: title::TitleAnims,
    /// Temporary layer for title
    layer_ix: Index<Layer>,
}

impl Title {
    pub fn new(ice: &mut Ice, ui: &mut Ui) -> Self {
        let assets = &mut ice.assets;

        let cfg = title::ColorConfig::default();
        let state = title::TitleState { cursor: 0 };
        let mut assets = title::TitleAssets::new(assets);

        let layer_ix = ui.layers.insert(Layer::default());
        let layer = &mut ui.layers[layer_ix];

        let nodes = title::TitleNodes::new(&cfg, &mut layer.nodes, &mut assets);
        let cursor = 0;
        let anims = title::TitleAnims::init(&cfg, &mut layer.anims, &nodes, cursor);

        Self {
            cfg,
            state,
            assets,
            nodes,
            anims,
            layer_ix,
        }
    }

    pub fn handle_input(&mut self, gl: &mut Global) -> Option<title::Choice> {
        let layer = &mut gl.ui.layers[self.layer_ix];

        if let Some(dir) = gl.vi.dir.dir4_pressed() {
            let y_sign = dir.y_sign();

            if y_sign != Sign::Neutral {
                gl.ice
                    .audio
                    .play(&*self.assets.se_cursor.get_mut().unwrap());
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
                    .select(&self.cfg, &self.nodes, layer, self.state.cursor, pos);
                self.state.cursor = pos;
            }

            return None;
        }

        if gl.vi.select.is_pressed() {
            gl.ice
                .audio
                .play(&*self.assets.se_select.get_mut().unwrap());

            self.anims.on_exit(&mut layer.anims, &self.nodes);
            return Some(title::Choice::from_usize(self.state.cursor).unwrap());
        }

        None
    }
}
