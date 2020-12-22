//! Virtual input

use std::time::Duration;

use xdl::{vi::*, Key};

/// Collection of virtual inputs
#[derive(Debug, Clone)]
pub struct VInput {
    pub dir: AxisDirButton,
    pub select: Button,
}

impl VInput {
    // TODO: use `serde`
    pub fn new() -> Self {
        let repeat = KeyRepeat::Repeat {
            // FIXME: currently these values for smooth walking animation
            first: Duration::from_nanos(1_000_000_000 / 60 * 8),
            multi: Duration::from_nanos(1_000_000_000 / 60 * 8),
        };

        Self {
            dir: AxisDirButton::new(
                repeat,
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
            select: Button::new(InputBundle { keys: vec![Key::Z] }, KeyRepeat::None),
        }
    }
}
