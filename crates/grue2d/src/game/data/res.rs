/*!
Resource types specific for SnowRL
*/

use std::time::Duration;

use serde::{Deserialize, Serialize};

use snow2d::{
    gfx::text::font::FontSetHandle,
    input::{vi::*, Input, Key},
    ui::{CoordSystem, Layer},
    utils::arena::Index,
};

/// TODO: rm
const REPEAT_FIRST_FRAMES: u64 = 10;
/// TODO: rm
const REPEAT_MULTI_FRAMES: u64 = 6;

/// SnowRL UI layer collection
#[derive(Debug)]
pub struct Ui {
    actors: Layer,
    on_actors: Layer,
    on_shadow: Layer,
    screen: Layer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiLayer {
    Actors,
    OnActors,
    OnShadow,
    Screen,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            actors: Layer::new(CoordSystem::World),
            on_actors: Layer::new(CoordSystem::World),
            on_shadow: Layer::new(CoordSystem::World),
            screen: Layer::new(CoordSystem::Screen),
        }
    }

    pub fn layer(&self, layer: UiLayer) -> &Layer {
        match layer {
            UiLayer::Actors => &self.actors,
            UiLayer::OnActors => &self.on_actors,
            UiLayer::OnShadow => &self.on_shadow,
            UiLayer::Screen => &self.screen,
        }
    }

    pub fn layer_mut(&mut self, layer: UiLayer) -> &mut Layer {
        match layer {
            UiLayer::Actors => &mut self.actors,
            UiLayer::OnActors => &mut self.on_actors,
            UiLayer::OnShadow => &mut self.on_shadow,
            UiLayer::Screen => &mut self.screen,
        }
    }

    pub fn layers<const N: usize>(&self, layers: [UiLayer; N]) -> [&Layer; N] {
        layers.map(|l| self.layer(l))
    }

    pub fn layers_mut<const N: usize>(&mut self, layers: [UiLayer; N]) -> [&mut Layer; N] {
        layers.map(|l| unsafe { (&mut *(self as *mut Self)).layer_mut(l) })
    }

    pub fn update(&mut self, dt: Duration) {
        self.actors.update(dt);
        self.on_actors.update(dt);
        self.on_shadow.update(dt);
        self.screen.update(dt);
    }
}

/// SnowRL font collection
#[derive(Debug)]
pub struct Fonts {
    pub default: Index<FontSetHandle>,
}

/// SnowRL virtual input collection
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
    /// TODO: load from serde
    pub fn new() -> Self {
        let dir_repeat = KeyRepeatConfig::Repeat {
            first: Duration::from_nanos(1_000_000_000 / 60 * REPEAT_FIRST_FRAMES),
            multi: Duration::from_nanos(1_000_000_000 / 60 * REPEAT_MULTI_FRAMES),
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

#[derive(Debug)]
pub struct Resources {
    pub fonts: Fonts,
    pub vi: VInput,
    pub ui: Ui,
}
