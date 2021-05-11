/*!
Aggressive imports for SnowRL
*/

pub use snow2d::{
    asset::{Asset, AssetCacheAny, AssetCacheT},
    audio,
    gfx::{draw::*, geom2d::*, tex::*, Color, Snow2d},
    input::{Dir8, Key, Sign},
    ui::{Layer, Node, Ui},
    utils::{
        arena::{Arena, Index},
        ez::{self, Ease, EasedDt, LinearDt, Tweened},
        pool::{Handle, Pool},
        tweak::*,
        ArrayTools,
    },
    Ice,
};

pub use grue2d::{
    game::{
        data::res::{Fonts, UiLayer, VInput},
        Data,
    },
    GrueRl,
};

pub use crate::utils::{consts, paths};
