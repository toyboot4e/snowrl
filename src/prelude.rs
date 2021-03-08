/*!
Aggressive imports for SnowRL
*/

pub use snow2d::{
    asset::{Asset, AssetCacheAny, AssetCacheT},
    audio,
    gfx::{draw::*, geom2d::*, tex::*, Color, PassConfig, Snow2d},
    input::{Dir8, Key, Sign},
    ui::{node::Node, Layer},
    utils::{
        arena::{Arena, Index},
        ez::{self, Ease, EasedDt, LinearDt, Tweened},
        pool::{Handle, Pool},
        tweak::*,
        ArrayTools,
    },
    Ice,
};

pub use grue2d::{Fonts, Global, GlueRl, Ui, UiLayer, VInput};

pub use crate::utils::{consts, paths};
