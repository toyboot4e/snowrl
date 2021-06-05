/*!
Resource types specific for SnowRL
*/

use std::time::Duration;

use serde::{Deserialize, Serialize};

use snow2d::{
    gfx::text::FontFamilyHandle,
    input::{vi::*, Dir8, Input, Key},
    ui::{CoordSystem, Layer, Node, Ui},
    utils::{arena::Index, pool::Handle, Inspect},
};

use rlbox::view::anim::DirAnimState;

use crate::markup::KbdIcons;

/// TODO: rm
const REPEAT_FIRST_FRAMES: u64 = 10;
/// TODO: rm
const REPEAT_MULTI_FRAMES: u64 = 6;

/// Can be converted to [`Layer`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Inspect)]
pub enum UiLayer {
    Actors,
    OnActors,
    OnShadow,
    Screen,
}

impl UiLayer {
    pub fn to_layer(&self) -> Layer {
        match self {
            Self::Actors => Layer {
                coord: CoordSystem::World,
                z_order: 0.20,
            },
            Self::OnActors => Layer {
                coord: CoordSystem::World,
                z_order: 0.50,
            },
            Self::OnShadow => Layer {
                coord: CoordSystem::World,
                z_order: 0.75,
            },
            Self::Screen => Layer {
                coord: CoordSystem::Screen,
                z_order: 0.90,
            },
        }
    }

    /// Returns inclusive range of z orders of nodes to draw
    ///
    /// `z_order` of nodes should be in range from `0.0` to `1.0`.
    pub fn to_draw_range(&self) -> std::ops::RangeInclusive<f32> {
        let low = self.to_layer().z_order;
        let hi = low + 0.1;
        low..=hi
    }
}

/// SnowRL font collection
#[derive(Debug)]
pub struct Fonts {
    pub default: Index<FontFamilyHandle>,
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

#[derive(Debug, Clone)]
pub struct DirAnimEntry {
    /// Animation node
    pub node: Handle<Node>,
    pub dir: Dir8,
    pub state: DirAnimState,
}

/// Runs directionary animations performed by entities
#[derive(Debug, Default)]
pub struct DirAnimRunner {
    entries: Vec<DirAnimEntry>,
}

impl DirAnimRunner {
    /// Ticks the animation states and applies those animations to target `Node`
    pub fn update(&mut self, dt: Duration, ui: &mut Ui) {
        // drain (remove) finished animation nodes
        let _ = self
            .entries
            .drain_filter(|e| e.state.is_stopped())
            .collect::<Vec<_>>();

        // update
        for e in &mut self.entries {
            let node = &mut ui.nodes[&e.node];
            node.surface = e.state.current_frame_with_dir(e.dir).into();

            // NOTE: we don't tick in the first frame
            e.state.tick(dt);
        }
    }

    pub fn add(&mut self, entry: DirAnimEntry) {
        self.entries.push(entry);
    }
}

#[derive(Debug)]
pub struct Resources {
    pub fonts: Fonts,
    /// TODO: re-consider the data layout
    pub kbd_icons: KbdIcons,
    pub vi: VInput,
    pub ui: Ui,
    /// Directional animations over UI nodes
    pub dir_anims: DirAnimRunner,
}
