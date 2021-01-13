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
pub mod world;

use rokol::{app as ra, gfx as rg};

use {
    rlbox::{
        render::actor::ActorImage,
        rl::{fov::FovData, fow::FowData, grid2d::*, rlmap::TiledRlMap},
        utils::Double,
    },
    snow2d::asset,
};

use crate::{
    fsm::render::WorldRenderer,
    turn::anim::AnimPlayer,
    world::{actor::Actor, Shadow, World, WorldContext},
};

pub fn run(app: rokol::Rokol) -> rokol::Result {
    app.run(&mut SnowRl {
        x: None,
        default_window_title: app.title.clone(),
    })
}

pub struct SnowRl {
    /// Use `Option` for lazy initialization
    x: Option<SnowRlImpl>,
    default_window_title: String,
}

/// `sokol_app.h` provides with a 60 FPS game loop!
impl rokol::app::RApp for SnowRl {
    fn init(&mut self) {
        rg::setup(&mut rokol::glue::app_desc());
        // Now we have access to `rokol::gfx`!
        // So we can initialize the actual game:
        self.x = Some(SnowRlImpl::new(self.default_window_title.clone()));
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

/// Runs a [`Fsm`] with some [`Global`] data
///
/// [`Fsm`] fsm::Fsm
/// [`Global`] fsm::Global
#[derive(Debug)]
struct SnowRlImpl {
    gl: fsm::Global,
    fsm: fsm::Fsm,
}

impl SnowRlImpl {
    pub fn new(title: String) -> Self {
        let mut gl = {
            let wcx = WorldContext::new(title);
            let world = self::init_world(&wcx).unwrap();

            fsm::Global {
                world,
                wcx,
                world_render: WorldRenderer::default(),
                anims: AnimPlayer::default(),
                script_to_play: None,
            }
        };

        let fsm = {
            let mut fsm = fsm::Fsm::default();

            fsm.insert_default::<fsm::states::Roguelike>();
            fsm.insert_default::<fsm::states::Animation>();
            fsm.insert_default::<fsm::states::Title>();
            fsm.insert_default::<fsm::states::PlayScript>();

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

    /// Temporary way to identify player
    pub const PLAYER: usize = 0;

    /// FPS of character graphics animation
    pub const ACTOR_FPS: f32 = 4.0;

    /// Filed of view radius
    pub const FOV_R: u32 = 5;

    /// Walk duration in seconds
    pub const WALK_TIME: f32 = 8.0 / 60.0;

    /// Half frame in seconds (fixed timestep with 60 FPS)
    pub const HALF_FRAME: f32 = 1.0 / 120.0;

    /// Key repeat duration for virtual directionaly key
    pub const REPEAT_FIRST_FRAMES: u64 = 8;

    /// Key repeat duration for virtual directionaly key
    pub const REPEAT_MULTI_FRAMES: u64 = 6;

    /// [x, y]
    pub const MESSAGE_PAD_EACH: [f32; 2] = [12.0, 8.0];
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
    // player

    let img = {
        let pos = Vec2i::new(20, 16);
        let dir = Dir8::S;

        ActorImage::from_path(
            asset::path(crate::paths::IKA_CHAN),
            crate::consts::ACTOR_FPS,
            crate::consts::WALK_TIME,
            pos,
            dir,
        )?
    };

    w.entities.push({
        let pos = Vec2i::new(20, 16);
        let dir = Dir8::S;

        let player = Actor {
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

    // non-player characters

    let img = {
        let pos = Vec2i::new(20, 16);
        let dir = Dir8::S;

        ActorImage::from_path(
            asset::path(crate::paths::img::pochi::WHAT),
            crate::consts::ACTOR_FPS,
            crate::consts::WALK_TIME,
            pos,
            dir,
        )?
    };

    w.entities.push({
        let pos = Vec2i::new(14, 12);
        let dir = Dir8::S;
        Actor {
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
        Actor {
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
