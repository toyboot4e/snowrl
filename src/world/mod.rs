//! The game world

pub mod actor;
pub mod turn;
mod vi;

use {
    rokol::gfx as rg,
    std::{path::Path, time::Duration},
    xdl::Key,
};

use snow2d::{
    asset,
    gfx::{batcher::draw::*, Color},
    PassConfig, Snow2d,
};

use rlbox::rl::{
    self,
    fov::{FovData, FovWrite, OpacityMap},
    fow::FowData,
    grid2d::*,
    rlmap::{RlMap, TiledRlMap},
};

use crate::{render::FovRenderer, utils::cheat};

use self::{actor::*, turn::GameLoop, vi::VInput};

/// Powers the game [`World`]
pub struct WorldContext {
    /// 2D renderer
    pub rdr: Snow2d,
    pub soloud: soloud::Soloud,
    /// Clears target (frame buffer) with cornflower blue color
    pa_blue: rg::PassAction,
    pub fov_render: FovRenderer,
    pub input: xdl::Input,
    pub vi: VInput,
    pub dt: Duration,
}

impl WorldContext {
    pub fn new() -> Self {
        Self {
            rdr: unsafe { Snow2d::new() },
            // TODO: do not unwrap
            soloud: soloud::Soloud::default().unwrap(),
            pa_blue: rg::PassAction::clear(Color::CORNFLOWER_BLUE.to_normalized_array()),
            fov_render: FovRenderer::new(),
            input: xdl::Input::new(),
            vi: VInput::new(),
            dt: Duration::new(0, 0),
        }
    }

    pub fn event(&mut self, ev: &rokol::app::Event) {
        self.input.event(ev);
    }

    pub fn update(&mut self) {
        // FIXME: use real dt
        self.dt = std::time::Duration::from_nanos(1_000_000_000 / 60);

        // input
        self.vi.dir.update(&self.input, self.dt);
        // rendering state
        self.fov_render.update(self.dt);
    }

    pub fn render(&mut self) {
        // debug render?
    }

    pub fn on_end_frame(&mut self) {
        self.input.on_end_frame();
    }
}

#[derive(Debug)]
pub enum GameState {
    Tick,
    Anim,
    Player,
}

/// The game world
pub struct World {
    pub map: TiledRlMap,
    pub fow: FowData,
    pub entities: Vec<Player>,
    pub state: GameState,
    pub game_loop: GameLoop,
}

/// Lifecycle
impl World {
    pub fn from_tiled_file(wcx: &mut WorldContext, path: &Path) -> anyhow::Result<Self> {
        let map = TiledRlMap::from_tiled_path(path)?;

        let mut entities = Vec::with_capacity(20);

        entities.push({
            let pos = Vec2i::new(14, 12);
            let mut player = Player {
                pos,
                dir: Dir8::N,
                fov: FovData::new(crate::consts::FOV_R, 10),
                img: ActorImage::from_path(asset::path("ika-chan.png"), pos, Dir8::N)?,
            };

            self::update_fov(
                &mut player.fov,
                player.pos,
                crate::consts::FOV_R,
                &map.rlmap,
            );
            wcx.fov_render.force_set_fov(&player.fov);

            player
        });

        entities.push({
            let pos = Vec2i::new(20, 15);
            let dir = Dir8::S;
            Player {
                pos,
                dir,
                fov: FovData::empty(),
                img: ActorImage::from_path(asset::path("ika-chan.png"), pos, Dir8::N)?,
            }
        });

        let size = map.rlmap.size;
        Ok(Self {
            map,
            fow: FowData::new(size),
            entities,
            state: GameState::Tick,
            game_loop: GameLoop::new(),
        })
    }

    pub fn event(&mut self, wcx: &mut WorldContext, ev: &rokol::app::Event) {}

    pub fn update_scene(&mut self, wcx: &mut WorldContext) {
        match self.state {
            GameState::Tick => {
                let res = self.game_loop.tick();
                println!("tick() -> {:?}", res);
                self.state = GameState::Player;
            }
            GameState::Anim => unimplemented!(),
            GameState::Player => {
                self.update_player(wcx);
            }
        }
    }

    pub fn update_images(&mut self, wcx: &mut WorldContext) {
        for e in &mut self.entities {
            e.img.update(wcx.dt, e.pos, e.dir);
        }
    }

    fn update_player(&mut self, wcx: &mut WorldContext) {
        // TODO: return player command
        if let Some(dir) = wcx.vi.dir.to_dir8() {
            self::walk(self, 0, wcx, dir);
        }
    }

    pub fn render(&mut self, wcx: &mut WorldContext) {
        let mut screen = wcx.rdr.screen(PassConfig {
            pa: &wcx.pa_blue,
            tfm: None,
            pip: None,
        });

        crate::render::render_tiled(&mut screen, self);
        self.render_actors(&mut screen);

        drop(screen);

        wcx.fov_render.render_ofs(&mut wcx.rdr, self);
        wcx.fov_render.blend_to_screen(&mut wcx.rdr);

        unsafe {
            // update fontbook GPU texture
            // FIXME: it may not work on the first frame, unfortunatelly
            wcx.rdr.fontbook.update_image();
        }
    }

    fn render_actors(&mut self, draw: &mut impl DrawApi) {
        // TODO: y sort + culling
        for e in &self.entities {
            e.img.render(draw, &self.map.tiled);
        }
    }

    pub fn on_end_frame(&mut self, wcx: &mut WorldContext) {
        //
    }
}

/// API
impl World {
    pub fn player(&self) -> &Player {
        &self.entities[0]
    }

    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.entities[0]
    }

    pub fn is_blocked(&mut self, pos: Vec2i) -> bool {
        if self.map.rlmap.is_blocked(pos) {
            return true;
        }

        false
    }
}

fn walk(world: &mut World, actor_ix: usize, wcx: &mut WorldContext, dir: Dir8) {
    let player = &mut world.entities[actor_ix];

    let pos = player.pos + Vec2i::from(dir.signs_i32());

    // if rotate only
    if wcx
        .input
        .kbd
        .is_any_key_down(&[Key::LeftShift, Key::RightShift])
    {
        player.dir = dir;
        return;
    }

    drop(player); // drop mutable borrow

    // can't walk
    if world.is_blocked(pos) {
        let player = &mut world.entities[actor_ix];
        player.dir = dir;
        return;
    }

    let player = &mut world.entities[actor_ix];
    // TODO: remove this line and observe walk command
    wcx.fov_render.before_update_fov(&player.fov);

    player.pos = pos;
    player.dir = dir;

    self::update_fov(
        &mut player.fov,
        player.pos,
        crate::consts::FOV_R,
        &world.map.rlmap,
    );
}

fn update_fov(fov: &mut impl FovWrite, pos: Vec2i, r: u32, map: &impl OpacityMap) {
    rl::fov::refresh(
        fov,
        rl::fov::RefreshParams {
            r,
            origin: pos,
            opa: map,
        },
    );
}
