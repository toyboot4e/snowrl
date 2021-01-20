//! Virtual input

use std::time::Duration;

use snow2d::input::{vi::*, Input, Key};

use crate::utils::consts;

/// Collection of virtual inputs
#[derive(Debug, Clone)]
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
        let dir_repeat = KeyRepeat::Repeat {
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
                        keys: vec![Key::D, Key::E, Key::C],
                    },
                    // x negative
                    InputBundle {
                        keys: vec![Key::A, Key::Q, Key::Z],
                    },
                ],
                // y
                [
                    // y positive
                    InputBundle {
                        keys: vec![Key::X, Key::Z, Key::C],
                    },
                    // y negative
                    InputBundle {
                        keys: vec![Key::W, Key::Q, Key::E],
                    },
                ],
            ),
            select: Button::new(
                InputBundle {
                    keys: vec![Key::Enter],
                },
                KeyRepeat::None,
            ),
            turn: Button::new(
                InputBundle {
                    keys: vec![Key::LeftShift, Key::RightShift],
                },
                KeyRepeat::None,
            ),
            rest: Button::new(
                InputBundle {
                    keys: vec![Key::Space],
                },
                KeyRepeat::None,
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
