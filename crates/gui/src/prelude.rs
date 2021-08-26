/*!
Aggressive imports
*/

pub use {
    model, rlcore,
    rokol::{self, gfx as rg},
    serde, snow2d, view,
};

pub use serde::{Deserialize, Serialize};

pub use glam::Mat4;

pub use snow2d::{
    asset::{Asset, AssetCache, AssetKey},
    gfx::{
        geom2d::*,
        tex::{SpriteData, Texture2dDrop},
        Snow2d, WindowState,
    },
    ui::{node, CoordSystem, Layer, Node, Ui},
    utils::{
        arena::{Arena, Index},
        ez,
        pool::{Handle, Pool},
        tyobj::{self, SerdeRepr, SerdeViaTyObj, TypeObject},
        Derivative, Inspect,
    },
    Ice,
};

pub use rlcore::{
    ev::hub::{DynEvent, EventHub},
    grid2d::*,
    sys::UiEvent,
};

pub use model::{
    chg,
    entity::{ActorStats, EntityModel, Relation},
    EventData, GameSystem, Model,
};

pub use view::{
    actor::{ActorImage, ActorImageType, ActorNodes, ActorView},
    anim::DirAnimType,
    camera::{Camera2d, FollowCamera2d, Transform2dParams},
    shadow::Shadow,
};

pub use crate::{
    fsm::{self, *},
    markup::{self, KbdIcons},
    renderer::WorldRenderer,
    res::*,
    spawn::*,
    spawn::{ActorSpawn, ActorType},
    window::Platform,
    Data, GameConfig, Gui, ShadowConfig, SnowConfig,
};
