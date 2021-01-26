/*!
Scenes
*/

use snow2d::{
    asset::{Asset, AssetCacheAny},
    audio,
    gfx::{geom2d::*, tex::*, Color},
};

use rlbox::{
    ui::{
        anims::*,
        node::{Draw, Node},
        AnimPool,
    },
    utils::{
        arena::Index,
        ez,
        pool::{Handle, Pool},
        tweak::*,
        ArrayTools,
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

pub struct Phased<T> {
    pub selected: T,
    pub not_selected: T,
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

pub struct ItemConfig {
    pub item: Phased<Color>,
    pub bg: Phased<Color>,
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
    /// [ [front, shadow] ]
    pub choices: [[Handle<Node>; 2]; 3],
}

impl TitleNodes {
    pub fn new(nodes: &mut Pool<Node>, assets: &mut TitleAssets) -> Self {
        let logo = nodes.add(Draw::Sprite(assets.logo.clone()).into());

        let choices = [
            [
                nodes.add(Draw::Sprite(assets.choices[0].clone()).into()),
                nodes.add(Draw::Sprite(assets.choices[0].clone()).into()),
            ],
            [
                nodes.add(Draw::Sprite(assets.choices[1].clone()).into()),
                nodes.add(Draw::Sprite(assets.choices[1].clone()).into()),
            ],
            [
                nodes.add(Draw::Sprite(assets.choices[2].clone()).into()),
                nodes.add(Draw::Sprite(assets.choices[2].clone()).into()),
            ],
        ];

        Self { logo, choices }
    }
}

/// TODO: put animations in pool so that the animation remain when `Title` is poped
#[derive(Debug)]
pub struct TitleAnims {
    logo: Index,
    choices: [[Index; 2]; 3],
}

impl TitleAnims {
    pub fn init(anims: &mut AnimPool, nodes: &TitleNodes) -> Self {
        let logo = anims.insert(Anim::PosTween(PosTween {
            tween: ez::Tweened {
                a: [tweak!(560.0), tweak!(18.0)].into(),
                b: [tweak!(440.0), tweak!(12.0)].into(),
                dt: ez::EasedDt::new(tweak!(1.2), ez::Ease::ExpOut),
            },
            node: nodes.logo.clone(),
        }));

        let a = Vec2f::new(tweak!(200.0), tweak!(380.0));
        let b = Vec2f::new(tweak!(80.0), tweak!(350.0));
        let dt = ez::EasedDt::new(tweak!(1.2), ez::Ease::ExpOut);

        let choices = [
            Self::choice_anim(
                nodes,
                (a, Color::TRANSPARENT),
                (b, Color::WHITE),
                dt.clone(),
                0,
            ),
            Self::choice_anim(
                nodes,
                (a, Color::TRANSPARENT),
                (b, Color::WHITE),
                dt.clone(),
                1,
            ),
            Self::choice_anim(
                nodes,
                (a, Color::TRANSPARENT),
                (b, Color::WHITE),
                dt.clone(),
                2,
            ),
        ];

        Self {
            logo,
            // thanks, ArrayTools
            choices: choices.map(|[a, b]| [anims.insert(a), anims.insert(b)]),
        }
    }

    fn choice_anim(
        nodes: &TitleNodes,
        from: (Vec2f, Color),
        to: (Vec2f, Color),
        dt: ez::EasedDt,
        i: usize,
    ) -> [Anim; 2] {
        let delta_shadow = Vec2f::new(10.0, 6.0);
        let delta_choices: [Vec2f; 3] = [
            [0.0, 0.0].into(),
            [100.0, 100.0].into(),
            [320.0, 200.0].into(),
        ];

        let item = Anim::PosTween(PosTween {
            tween: ez::Tweened {
                a: from.0 + delta_choices[i],
                b: to.0 + delta_choices[i],
                dt,
            },
            node: nodes.choices[i][0].clone(),
        });

        let shadow = Anim::PosTween(PosTween {
            tween: ez::Tweened {
                a: delta_shadow + from.0 + delta_choices[i],
                b: delta_shadow + to.0 + delta_choices[i],
                dt,
            },
            node: nodes.choices[i][1].clone(),
        });

        // TODO: tween alpha only

        [item, shadow]
    }

    // pub fn on_exit(&mut self) {
    //     self.logo
    //         .set_next_and_easing([tweak!(20.0), tweak!(400.0)].into(), ez::Ease::ExpOut);
    // }
}
