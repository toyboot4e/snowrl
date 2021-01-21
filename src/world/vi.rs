//! Virtual input

use std::time::Duration;

use serde::{Deserialize, Serialize};

use snow2d::input::{
    vi::{
        KeyEntry::{Key1, Key2},
        *,
    },
    Input, Key,
};

use crate::utils::consts;

/// Collection of virtual inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VInput {
    /// Diractional input
    pub dir: AxisDirButton,
    /// Enter
    pub select: Button,
    /// Change direction without changin position
    pub turn: Button,
    /// Rest one turn
    pub rest: Button,
}

impl VInput {
    /// TODO: use serde
    pub fn new() -> Self {
        let dir_repeat = KeyRepeatConfig::Repeat {
            first: Duration::from_nanos(1_000_000_000 / 60 * consts::REPEAT_FIRST_FRAMES),
            multi: Duration::from_nanos(1_000_000_000 / 60 * consts::REPEAT_MULTI_FRAMES),
        };

        Self {
            dir: AxisDirButton::new(
                dir_repeat,
                // x
                [
                    // x positive
                    InputBundle {
                        keys: vec![Key1([Key::D]), Key1([Key::E]), Key1([Key::C])],
                    },
                    // x negative
                    InputBundle {
                        keys: vec![Key1([Key::A]), Key1([Key::Q]), Key1([Key::Z])],
                    },
                ],
                // y
                [
                    // y positive
                    InputBundle {
                        keys: vec![Key1([Key::X]), Key1([Key::Z]), Key1([Key::C])],
                    },
                    // y negative
                    InputBundle {
                        keys: vec![Key1([Key::W]), Key1([Key::Q]), Key1([Key::E])],
                    },
                ],
            ),
            select: Button::new(
                InputBundle {
                    keys: vec![Key1([Key::Enter])],
                },
                KeyRepeatConfig::NoRepeat,
            ),
            turn: Button::new(
                InputBundle {
                    keys: vec![Key1([Key::LeftShift]), Key1([Key::RightShift])],
                },
                KeyRepeatConfig::NoRepeat,
            ),
            rest: Button::new(
                InputBundle {
                    keys: vec![Key1([Key::Space])],
                },
                KeyRepeatConfig::NoRepeat,
            ),
        }
    }

    pub fn update(&mut self, input: &Input, dt: Duration) {
        self.dir.update(input, dt);
        for bt in &mut [&mut self.select, &mut self.turn, &mut self.rest] {
            bt.update(input, dt);
        }
    }
}
