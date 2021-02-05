/*!
Scenes
*/

use snow2d::audio::src::Wav;

use rlbox::ui::{
    anims::*,
    builder::animate,
    node::{Draw, Node},
    AnimPool, Ui,
};

use crate::utils::asset_defs::{title, AssetDef};

use crate::prelude::*;

/// Return value of title screen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Choice {
    NewGame,
    Continue,
    Exit,
}

#[derive(Debug, Clone)]
pub struct Phased<T> {
    pub selected: T,
    pub not_selected: T,
}

impl<T: Clone> Phased<T> {
    fn get(&self, is_selected: bool) -> T {
        if is_selected {
            self.selected.clone()
        } else {
            self.not_selected.clone()
        }
    }
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

#[derive(Debug, Clone)]
pub struct ItemConfig {
    pub item: Phased<Color>,
    pub shadow: Phased<Color>,
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

            shadow: Phased {
                selected: Color {
                    r: 24,
                    g: 24,
                    b: 24,
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
    pub cfg: ItemConfig,
    pub logo: SpriteData,
    pub choices: [SpriteData; 3],
    pub se_cursor: Asset<Wav>,
    pub se_select: Asset<Wav>,
    pub bgm: Asset<audio::src::WavStream>,
}

impl TitleAssets {
    pub fn new(cfg: ItemConfig, assets: &mut AssetCacheAny) -> Self {
        Self {
            cfg,
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
    /// [ [shadow, front ] ]
    pub choices: [[Handle<Node>; 2]; 3],
}

impl TitleNodes {
    pub fn new(nodes: &mut Pool<Node>, assets: &TitleAssets) -> Self {
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

        // overwrite initial colors
        for i in 0..choices.len() {
            nodes[&choices[i][0]].params.color = assets.cfg.shadow.not_selected;
            nodes[&choices[i][1]].params.color = assets.cfg.item.not_selected;
        }

        Self { logo, choices }
    }
}

/// TODO: put animations in pool so that the animation remain when `Title` is poped
#[derive(Debug)]
pub struct TitleAnims {}

impl TitleAnims {
    pub fn init(cfg: &ItemConfig, anims: &mut AnimPool, nodes: &TitleNodes, cursor: usize) -> Self {
        let dt = ez::EasedDt::new(tweak!(1.2), ez::Ease::ExpOut);

        anims
            .builder()
            .node(&nodes.logo)
            .dt(dt)
            .pos([(tweak!(560.0), tweak!(18.0)), (tweak!(440.0), tweak!(12.0))])
            .color([Color::TRANSPARENT, Color::OPAQUE]);

        let from = Vec2f::new(tweak!(200.0), tweak!(380.0));
        let to = Vec2f::new(tweak!(80.0), tweak!(350.0));

        let delta_shadow = Vec2f::new(10.0, 6.0);
        let delta_choices: [Vec2f; 3] = [
            [0.0, 0.0].into(),
            [100.0, 100.0].into(),
            [320.0, 200.0].into(),
        ];

        for i in 0..3 {
            // offset
            let from = from + delta_choices[i];
            let to = to + delta_choices[i];

            let mut b = anims.builder();
            b.dt(dt);

            // shadow
            b.node(&nodes.choices[i][0])
                .pos([from + delta_shadow, to + delta_shadow])
                .color([Color::TRANSPARENT, cfg.shadow.get(i == cursor)]);

            // text
            b.node(&nodes.choices[i][1])
                .pos([from, to])
                .color([Color::TRANSPARENT, cfg.item.get(i == cursor)]);
        }

        Self {}
    }

    pub fn select(
        &mut self,
        cfg: &ItemConfig,
        nodes: &TitleNodes,
        anims: &mut AnimPool,
        from: usize,
        to: usize,
    ) {
        let mut b = anims.builder();
        b.dt(EasedDt::new(6.0 / 60.0, Ease::Linear));

        // items
        b.node(&nodes.choices[from][1])
            .color([cfg.item.selected, cfg.item.not_selected]);
        b.node(&nodes.choices[to][1])
            .color([cfg.item.not_selected, cfg.item.selected]);

        // shadows
        b.node(&nodes.choices[from][0])
            .color([cfg.shadow.selected, cfg.shadow.not_selected]);
        b.node(&nodes.choices[to][0])
            .color([cfg.shadow.not_selected, cfg.shadow.selected]);
    }

    pub fn on_exit(&mut self, anims: &mut AnimPool, nodes: &TitleNodes) {
        let mut b = anims.builder();
        b.dt(ez::EasedDt::new(24.0 / 60.0, ez::Ease::SinOut));

        b.node(&nodes.logo)
            .pos([(tweak!(440.0), tweak!(18.0)), (tweak!(320.0), tweak!(6.0))])
            .color([Color::OPAQUE, Color::TRANSPARENT]);

        // TODO: tween texts
    }
}
