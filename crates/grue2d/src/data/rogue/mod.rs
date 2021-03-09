/*!

Turn-based game state

*/

pub mod anim;
pub mod ev;
pub mod script;
pub mod tick;

use self::{anim::AnimPlayer, script::ScriptRef};

#[derive(Debug)]
pub struct Rogue {
    pub anims: AnimPlayer,
    pub script_to_play: Option<ScriptRef>,
}

impl Rogue {
    pub fn new() -> Self {
        Self {
            anims: AnimPlayer::default(),
            script_to_play: None,
        }
    }
}
