/*!
UI scenes
*/

pub mod title;

use snow2d::Ice;

use rlbox::{rl::grid2d::*, ui::Ui};

use grue2d::Global;

#[derive(Debug)]
pub struct Title {
    state: title::TitleState,
    assets: title::TitleAssets,
    nodes: title::TitleNodes,
    anims: title::TitleAnims,
}

impl Title {
    pub fn new(ice: &mut Ice, ui: &mut Ui) -> Self {
        let assets = &mut ice.assets;

        let state = title::TitleState { cursor: 0 };
        let mut assets = title::TitleAssets::new(Default::default(), assets);
        let nodes = title::TitleNodes::new(&mut ui.nodes, &mut assets);
        let anims = title::TitleAnims::init(&mut ui.anims, &nodes);

        Self {
            state,
            assets,
            nodes,
            anims,
        }
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

            // self.anims.on_exit();

            return Some(title::Choice::from_usize(self.state.cursor).unwrap());
        }

        None
    }
}
