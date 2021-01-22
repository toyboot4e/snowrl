/*!

Snow the roguelike game

SnowRL is a set of plugins to [`grue2d`]. There are multiple crates under SnowRL:

| Crate      | Description                             |
|------------|-----------------------------------------|
| [`rokol`]  | Window and lower-level graphics         |
| [`snow2d`] | 2D rendering and asset management       |
| [`rlbox`]  | Toolkit to power 2D GUI roguelike games |
| [`grue2d`] | Game states for SnowRL                  |

*/

// use generator (unstable Rust)
#![feature(generators, generator_trait)]

// re-export mainly dependent crates
pub extern crate rokol;

pub extern crate rlbox;

pub extern crate snow2d;

pub extern crate grue2d;

pub mod utils;

pub mod play;
pub mod states;
