//! Scenes

use std::time::Duration;

use snow2d::{
    asset::{Asset, AssetCacheAny},
    audio,
    gfx::{geom2d::*, tex::*, Color},
};

use rlbox::{
    ui::node::{Draw, Node},
    utils::{
        ez,
        pool::{Handle, Pool},
        tweak::*,
    },
};

use crate::utils::{
    asset_defs::{title, AssetDef},
    paths,
};

/// Return value of title screen
pub enum Choice {
    NewGame,
    Continue,
    Exit,
}

impl Choice {
    pub fn from_usize(x: usize) -> Option<Self> {
        Some(match x {
            0 => Self::NewGame,
            1 => Self::Continue,
            2 => Self::Exit,
            _ => return None,
        })
    }
}

pub struct TitleConfig {
    pub item_cfg: ItemConfig,
}

pub struct ItemConfig {
    pub item: Phased<Color>,
    pub bg: Phased<Color>,
}

pub struct Phased<T> {
    pub selected: T,
    pub not_selected: T,
}

impl Default for ItemConfig {
    fn default() -> Self {
        Self {
            item: Phased {
                selected: Color {
                    r: 24,
                    g: 160,
                    b: 120,
                    a: 255,
                },

                not_selected: Color {
                    r: 85,
                    g: 40,
                    b: 40,
                    a: 255,
                },
            },

            bg: Phased {
                selected: Color {
                    r: 32,
                    g: 32,
                    b: 32,
                    a: 255,
                },

                not_selected: Color {
                    r: 16,
                    g: 16,
                    b: 16,
                    a: 255,
                },
            },
        }
    }
}

#[derive(Debug)]
pub struct TitleState {
    pub cursor: usize,
}

#[derive(Debug)]
pub struct TitleAssets {
    pub logo: SpriteData,
    pub choices: [SpriteData; 3],
    pub se_cursor: Asset<audio::src::Wav>,
    pub se_select: Asset<audio::src::Wav>,
    pub bgm: Asset<audio::src::WavStream>,
}

impl TitleAssets {
    pub fn new(assets: &mut AssetCacheAny) -> Self {
        Self {
            logo: title::Logo::load(assets).unwrap(),
            choices: title::Choices::load(assets).unwrap(),
            se_cursor: assets.load_sync(paths::sound::se::CURSOR).unwrap(),
            se_select: assets.load_sync(paths::sound::se::SELECT).unwrap(),
            bgm: assets.load_sync(paths::sound::bgm::FOREST_02).unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct TitleNodes {
    pub logo: Handle<Node>,
    pub choices: [Handle<Node>; 3],
}

impl TitleNodes {
    pub fn new(pool: &mut Pool<Node>, assets: &mut TitleAssets) -> Self {
        let logo = pool.add(Draw::Sprite(assets.logo.clone()).into());

        let choices = [
            pool.add(Draw::Sprite(assets.logo.clone()).into()),
            pool.add(Draw::Sprite(assets.logo.clone()).into()),
            pool.add(Draw::Sprite(assets.logo.clone()).into()),
        ];

        Self { logo, choices }
    }
}

/// TODO: put animations in pool so that the animation remain when `Title` is poped
#[derive(Debug, Default)]
pub struct TitleAnims {
    logo: ez::Tweened<Vec2f>,
    choice: ez::Tweened<Vec2f>,
}

impl TitleAnims {
    pub fn init(&mut self) {
        self.logo = ez::Tweened {
            a: [tweak!(560.0), tweak!(18.0)].into(),
            b: [tweak!(440.0), tweak!(12.0)].into(),
            dt: ez::EasedDt::new(tweak!(1.2), ez::Ease::ExpOut),
        };

        self.choice = ez::Tweened {
            a: [tweak!(200.0), tweak!(380.0)].into(),
            b: [tweak!(80.0), tweak!(350.0)].into(),
            dt: ez::EasedDt::new(tweak!(1.2), ez::Ease::ExpOut),
        };
    }

    pub fn on_exit(&mut self) {
        self.logo
            .set_next_and_easing([tweak!(20.0), tweak!(400.0)].into(), ez::Ease::ExpOut);
    }

    pub fn tick(&mut self, dt: Duration) {
        self.logo.tick(dt);
        self.choice.tick(dt);
    }
}
