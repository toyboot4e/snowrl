/*!

Snow the roguelike game

*/

// use generator (unstable Rust)
#![feature(generators, generator_trait)]

pub use {rlbox, rokol, snow2d};

pub mod paths;

pub mod fsm;
pub mod script;
pub mod turn;
pub mod utils;
pub mod world;

use rokol::{app as ra, gfx as rg};

use {
    rlbox::rl::{fov::FovData, fow::FowData, grid2d::*, rlmap::TiledRlMap},
    snow2d::asset,
};

use crate::{
    fsm::render::WorldRenderer,
    turn::anim::AnimPlayer,
    utils::Double,
    world::{
        actor::{ActorImage, Player},
        Shadow, World, WorldContext,
    },
};

pub fn run(app: rokol::Rokol) -> rokol::Result {
    app.run(&mut SnowRl::new())
}

pub struct SnowRl {
    /// Use `Option` for lazy initialization
    x: Option<SnowRlImpl>,
}

impl SnowRl {
    pub fn new() -> Self {
        Self { x: None }
    }
}

/// Delay the initialization
impl rokol::app::RApp for SnowRl {
    fn init(&mut self) {
        rg::setup(&mut rokol::glue::app_desc());
        // FIXME: it takes too long to load textures. maybe firt show window and then
        // load resources
        self.x = Some(SnowRlImpl::new());
    }

    fn event(&mut self, ev: &ra::Event) {
        if let Some(x) = self.x.as_mut() {
            x.event(ev);
        }
    }

    fn frame(&mut self) {
        if let Some(x) = self.x.as_mut() {
            x.frame();
        }
    }
}

#[derive(Debug)]
struct SnowRlImpl {
    gl: fsm::Global,
    fsm: fsm::Fsm,
}

impl SnowRlImpl {
    pub fn new() -> Self {
        let mut gl = {
            let wcx = WorldContext::default();
            let world = self::init_world(&wcx).unwrap();

            fsm::Global {
                world,
                wcx,
                world_render: WorldRenderer::default(),
                anims: AnimPlayer::default(),
            }
        };

        let fsm = {
            let mut fsm = fsm::Fsm::default();

            fsm.insert_default::<fsm::states::Roguelike>();
            fsm.insert_default::<fsm::states::Animation>();
            fsm.insert_default::<fsm::states::Title>();

            fsm.push::<fsm::states::Roguelike>(&mut gl);
            fsm.push::<fsm::states::Title>(&mut gl);

            fsm
        };

        Self { gl, fsm }
    }
}

impl rokol::app::RApp for SnowRlImpl {
    fn event(&mut self, ev: &ra::Event) {
        self.gl.event(ev);
    }

    fn frame(&mut self) {
        self.gl.pre_update();
        self.fsm.update(&mut self.gl);
        self.gl.post_update();

        self.fsm.render(&mut self.gl);

        self.gl.on_end_frame();

        rg::commit();
    }
}

pub mod consts {
    //! Magic values (should be removed)

    /// FPS of character graphics animation
    pub const ACTOR_FPS: f32 = 4.0;

    /// Filed of view radius
    pub const FOV_R: u32 = 5;

    /// Walk duration in seconds
    pub const WALK_TIME: f32 = 8.0 / 60.0;

    /// Half frame in seconds (fixed timestep with 60 FPS)
    pub const HALF_FRAME: f32 = 1.0 / 120.0;
}

fn init_world(_wcx: &WorldContext) -> anyhow::Result<World> {
    let path = snow2d::asset::path(crate::paths::map::tmx::RL_START);
    let map = TiledRlMap::from_tiled_path(&path)?;

    let radius = [crate::consts::FOV_R, 10];
    let map_size = map.rlmap.size;

    let mut world = World {
        map,
        shadow: Shadow {
            fov: Double {
                a: FovData::new(radius[0], radius[1]),
                b: FovData::new(radius[0], radius[1]),
            },
            fow: Double {
                a: FowData::new(map_size),
                b: FowData::new(map_size),
            },
            blend_factor: 0.0,
            is_dirty: false,
        },
        entities: Vec::with_capacity(20),
    };

    self::load_actors(&mut world)?;

    // just set FoV:
    // shadow.calculate(player.pos, &map.rlmap);
    // or animate initial FoV:
    world.shadow.make_dirty();

    Ok(world)
}

fn load_actors(w: &mut World) -> anyhow::Result<()> {
    // TODO: use asset loader and make use of cache
    let img = {
        let pos = Vec2i::new(20, 16);
        let dir = Dir8::S;
        ActorImage::from_path(asset::path(crate::paths::IKA_CHAN), pos, dir)?
    };

    w.entities.push({
        let pos = Vec2i::new(20, 16);
        let dir = Dir8::S;

        let player = Player {
            pos,
            dir,
            img: {
                let mut img = img.clone();
                img.force_set(pos, dir);
                img
            },
        };

        player
    });

    w.entities.push({
        let pos = Vec2i::new(14, 12);
        let dir = Dir8::S;
        Player {
            pos,
            dir,
            img: {
                let mut img = img.clone();
                img.force_set(pos, dir);
                img
            },
        }
    });

    w.entities.push({
        let pos = Vec2i::new(25, 18);
        let dir = Dir8::S;
        Player {
            pos,
            dir,
            img: {
                let mut img = img.clone();
                img.force_set(pos, dir);
                img
            },
        }
    });

    Ok(())
}
