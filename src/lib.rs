/*!

Snow the roguelike game

*/

// use generator (unstable Rust)
#![feature(generators, generator_trait)]

// re-export mainly dependent crates
pub extern crate rlbox;
pub extern crate rokol;
pub extern crate snow2d;

pub mod consts;
pub mod paths;

pub mod fsm;
pub mod script;
pub mod turn;
pub mod world;

use rokol::{app as ra, gfx as rg};

use {
    rlbox::{
        render::actor::ActorImage,
        rl::{grid2d::*, rlmap::TiledRlMap},
    },
    snow2d::{
        asset::AssetCacheT,
        audio,
        gfx::tex::{Texture2dDrop, TextureLoader},
    },
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
            let mut wcx = WorldContext::new(title);

            wcx.assets
                .add_cache::<Texture2dDrop>(AssetCacheT::new(TextureLoader));

            audio::asset::register_asset_loaders(&mut wcx.assets, &wcx.audio.clone());

            let world = self::init_world(&mut wcx).unwrap();

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

            fsm.insert(fsm::states::Title::new(&mut gl.wcx));
            fsm.insert(fsm::states::PlayScript::new(&mut gl.wcx.assets));

            fsm.push::<fsm::states::Roguelike>(&mut gl);
            fsm.push::<fsm::states::Title>(&mut gl);

            fsm
        };

        Self { gl, fsm }
    }
}

// Lifecycle
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

fn init_world(wcx: &mut WorldContext) -> anyhow::Result<World> {
    let map = TiledRlMap::new(
        paths::map::tmx::RL_START,
        wcx.assets.cache_mut::<Texture2dDrop>().unwrap(),
    )?;

    let radius = [consts::FOV_R, 10];
    let map_size = map.rlmap.size;

    let mut world = World {
        map,
        shadow: Shadow::new(radius, map_size, consts::WALK_TIME, consts::FOV_EASE),
        entities: Vec::with_capacity(20),
    };

    self::load_actors(&mut world, wcx)?;

    // just set FoV:
    // shadow.calculate(player.pos, &map.rlmap);
    // or animate initial FoV:
    world.shadow.make_dirty();

    Ok(world)
}

fn load_actors(w: &mut World, wcx: &mut WorldContext) -> anyhow::Result<()> {
    let cache = wcx.assets.cache_mut::<Texture2dDrop>().unwrap();

    // player

    let tex = cache.load_sync(paths::IKA_CHAN).unwrap();
    let img = {
        let pos = Vec2i::new(20, 16);
        let dir = Dir8::S;

        ActorImage::new(
            tex,
            consts::ACTOR_FPS,
            consts::WALK_TIME,
            consts::WALK_EASE,
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

    let tex = cache.load_sync(paths::img::pochi::WHAT).unwrap();
    let img = {
        let pos = Vec2i::new(20, 16);
        let dir = Dir8::S;

        ActorImage::new(
            tex,
            consts::ACTOR_FPS,
            consts::WALK_TIME,
            consts::WALK_EASE,
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
