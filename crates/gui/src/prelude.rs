/*!
Aggressive imports
*/

pub use {
    core, model,
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
        Inspect,
    },
    Ice,
};

pub use rlcore::grid2d::*;

pub use model::{
    entity::{ActorStats, EntityModel, Relation},
    GameSystem, Model,
};

pub use view::{
    actor::{ActorImage, ActorImageType, ActorNodes, ActorView},
    anim::DirAnimType,
    camera::{Camera2d, FollowCamera2d, Transform2dParams},
    shadow::Shadow,
};

pub use crate::{
    app::Platform,
    fsm::*,
    markup::{self, KbdIcons},
    renderer::WorldRenderer,
    res::*,
    spawn::*,
    spawn::{ActorSpawn, ActorType},
    Data, GameConfig, Gui, ShadowConfig, SnowConfig,
};
