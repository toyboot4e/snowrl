//! TODO: filter debug/error log on release build

use rokol::{
    app as ra,
    fons::{Align, FontConfig},
    gfx as rg, Rokol,
};

use snow2d::{
    asset::AssetCacheT,
    audio,
    gfx::{
        tex::{Texture2dDrop, TextureLoader},
        Snow2d,
    },
    Ice,
};

use rlbox::{
    render::actor::ActorImage,
    rl::{grid2d::*, rlmap::TiledRlMap},
    utils::tweak::*,
};

use grue2d::{
    render::WorldRenderer,
    rl::{
        turn::anim::AnimPlayer,
        world::{actor::Actor, Shadow, World},
    },
    vi::VInput,
    Fsm, Global,
};

use snowrl::{
    states,
    utils::{consts, paths},
};

fn main() -> rokol::Result {
    env_logger::init();

    let rokol = rokol::Rokol {
        w: 1280,
        h: 720,
        title: "SnowRL".to_string(),
        use_high_dpi: false,
        ..Default::default()
    };

    grue2d::run(rokol, SnowRl::new)
}

/// Runs a [`Fsm`] with some [`Global`] data
///
/// [`Fsm`] fsm::Fsm
/// [`Global`] fsm::Global
#[derive(Debug)]
struct SnowRl {
    gl: Global,
    fsm: Fsm,
}

impl SnowRl {
    pub fn new(rokol: Rokol) -> Self {
        let title = rokol.title;
        let mut gl = {
            let mut snow = unsafe { Snow2d::new() };
            let font_cfg = FontConfig {
                font: {
                    // FIXME: font path
                    let font = include_bytes!("../assets_embeded/mplus-1p-regular.ttf");
                    let ix = snow
                        .fontbook
                        .stash()
                        .add_font_mem("mplus-1p-regular", font)
                        .unwrap();
                    snow.fontbook.stash().set_align(Align::TOP | Align::LEFT);
                    ix
                },
                fontsize: crate::consts::DEFAULT_FONT_SIZE,
                line_spacing: crate::consts::DEFAULT_LINE_SPACE,
            };
            snow.fontbook.apply_cfg(&font_cfg);

            let mut ice = Ice::new(title, snow, font_cfg);

            ice.assets
                .add_cache::<Texture2dDrop>(AssetCacheT::new(TextureLoader));

            audio::asset::register_asset_loaders(&mut ice.assets, &ice.audio.clone());

            let world = self::init_world(&mut ice).unwrap();

            Global {
                world,
                ice,
                vi: VInput::new(),
                world_render: WorldRenderer::default(),
                anims: AnimPlayer::default(),
                script_to_play: None,
            }
        };

        {
            let x = ron::ser::to_string_pretty(&gl.vi, Default::default()).unwrap();
            println!("{}", x);
            let _y: VInput = ron::de::from_str(&x).unwrap();
        }

        let fsm = {
            let mut fsm = grue2d::Fsm::default();

            fsm.insert_default::<states::Roguelike>();
            fsm.insert_default::<states::Animation>();

            fsm.insert(states::Title::new(&mut gl.ice));
            fsm.insert(states::PlayScript::new(&mut gl.ice.assets));

            fsm.push::<states::Roguelike>(&mut gl);
            fsm.push::<states::Title>(&mut gl);

            fsm
        };

        Self { gl, fsm }
    }
}

// Lifecycle
impl rokol::app::RApp for SnowRl {
    fn event(&mut self, ev: &ra::Event) {
        self.gl.event(ev);
    }

    fn frame(&mut self) {
        // if debug
        #[cfg(debug_assertions)]
        self.gl.ice.audio.set_global_volume(tweak!(0.0));

        self.gl.pre_update();
        self.fsm.update(&mut self.gl);
        self.gl.post_update();

        self.gl.pre_render();
        self.fsm.render(&mut self.gl);

        self.gl.on_end_frame();

        rg::commit();
    }
}

fn init_world(ice: &mut Ice) -> anyhow::Result<World> {
    let map = TiledRlMap::new(
        paths::map::tmx::RL_START,
        ice.assets.cache_mut::<Texture2dDrop>().unwrap(),
    )?;

    let radius = [consts::FOV_R, 10];
    let map_size = map.rlmap.size;

    let mut world = World {
        map,
        shadow: Shadow::new(radius, map_size, consts::WALK_TIME, consts::FOV_EASE),
        entities: Vec::with_capacity(20),
    };

    self::load_actors(&mut world, ice)?;

    // just set FoV:
    // shadow.calculate(player.pos, &map.rlmap);
    // or animate initial FoV:
    world.shadow.make_dirty();

    Ok(world)
}

fn load_actors(w: &mut World, ice: &mut Ice) -> anyhow::Result<()> {
    let cache = ice.assets.cache_mut::<Texture2dDrop>().unwrap();

    // player

    let tex = cache.load_sync(paths::CHICKEN).unwrap();

    let img = {
        let pos = Vec2i::new(20, 16);
        let dir = Dir8::S;

        let mut img = ActorImage::new(
            tex,
            consts::ACTOR_FPS,
            consts::WALK_TIME,
            consts::WALK_EASE,
            pos,
            dir,
        )?;

        for frame_sprite in img.frames_mut() {
            frame_sprite.scales[0] = 2.0;
            frame_sprite.scales[1] = 2.0;
        }

        img
    };

    w.entities.push({
        let pos = Vec2i::new(20, 16);
        let dir = Dir8::S;

        let player = Actor {
            pos,
            dir,
            img: {
                let mut img = img.clone();
                img.warp(pos, dir);
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
                img.warp(pos, dir);
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
                img.warp(pos, dir);
                img
            },
        }
    });

    Ok(())
}
