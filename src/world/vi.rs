//! Virtual input

use std::time::Duration;

use serde::{Deserialize, Serialize};

use snow2d::input::{vi::*, Input, Key};

use crate::utils::consts;

/// Collection of virtual inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VInput {
    /// Diractional input
    pub dir: AxisDirButton,
    /// Enter
    #[serde(with = "snow2d::input::vi::button_serde_with")]
    pub select: Button,
    /// Change direction without changin position
    #[serde(with = "snow2d::input::vi::button_serde_with")]
    pub turn: Button,
    /// Rest one turn
    #[serde(with = "snow2d::input::vi::button_serde_with")]
    pub rest: Button,
}

impl VInput {
    /// TODO: use serde
    pub fn new() -> Self {
        let dir_repeat = KeyRepeatConfig::Repeat {
            first: Duration::from_nanos(1_000_000_000 / 60 * consts::REPEAT_FIRST_FRAMES),
            multi: Duration::from_nanos(1_000_000_000 / 60 * consts::REPEAT_MULTI_FRAMES),
        };

        use Key::*;

        Self {
            dir: AxisDirButton::new(
                dir_repeat,
                // x
                [
                    // x positive (right)
                    InputBundle {
                        keys: keys![D, E, C],
                    },
                    // x negative (left)
                    InputBundle {
                        keys: keys![A, Q, Z],
                    },
                ],
                // y
                [
                    // y positive (down)
                    InputBundle {
                        keys: keys![Z, X, C],
                    },
                    // y negative (up)
                    InputBundle {
                        keys: keys![W, Q, E],
                    },
                ],
            ),
            select: Button::new(
                InputBundle { keys: keys![Enter] },
                KeyRepeatConfig::NoRepeat,
            ),
            turn: Button::new(
                InputBundle {
                    keys: keys![LShift, RShift],
                },
                KeyRepeatConfig::NoRepeat,
            ),
            rest: Button::new(
                InputBundle { keys: keys![Space] },
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
