/*!
Scenes
*/

use snow2d::audio::src::Wav;

use rlbox::ui::{
    node::{Draw, Node},
    AnimPool,
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
                selected: [24, 160, 120].into(),
                not_selected: [85, 40, 40].into(),
            },
            shadow: Phased {
                selected: [24, 24, 24].into(),
                not_selected: [16, 16, 16].into(),
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
    pub se_cursor: Asset<Wav>,
    pub se_select: Asset<Wav>,
    pub bgm: Asset<audio::src::WavStream>,
}

impl TitleAssets {
    /// TODO: use derive macro to load it
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
    /// [ [shadow, front ] ]
    pub choices: [[Handle<Node>; 2]; 3],
}

impl TitleNodes {
    pub fn new(cfg: &ItemConfig, nodes: &mut Pool<Node>, assets: &TitleAssets) -> Self {
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
            nodes[&choices[i][0]].params.color = cfg.shadow.not_selected;
            nodes[&choices[i][1]].params.color = cfg.item.not_selected;
        }

        Self { logo, choices }
    }
}

/// TODO: put animations in pool so that the animation remain when `Title` is poped
#[derive(Debug)]
pub struct TitleAnims {}

impl TitleAnims {
    const DELTA_SHADOW: [f32; 2] = [10.0, 6.0];
    const DELTA_CHOICES: [[f32; 2]; 3] = [[0.0, 0.0], [100.0, 100.0], [320.0, 200.0]];

    fn logo_pos() -> Vec2f {
        [tweak!(440.0), tweak!(12.0)].into()
    }

    fn choice_pos(i: usize) -> Vec2f {
        Vec2f::new(tweak!(80.0), tweak!(350.0)).offset(Self::DELTA_CHOICES[i])
    }

    pub fn init(cfg: &ItemConfig, anims: &mut AnimPool, nodes: &TitleNodes, cursor: usize) -> Self {
        let dt = ez::EasedDt::new(tweak!(1.2), ez::Ease::ExpOut);

        anims
            .builder()
            .node(&nodes.logo)
            .dt(dt)
            .pos([Self::logo_pos().offset([120.0, 6.0]), Self::logo_pos()])
            .color([Color::TRANSPARENT, Color::OPAQUE]);

        for i in 0..3 {
            // offset
            let from = Self::choice_pos(i).offset([tweak!(-120.0), tweak!(-30.0)]);
            let to = Self::choice_pos(i);

            let mut b = anims.builder();
            b.dt(dt);

            // shadow
            b.node(&nodes.choices[i][0])
                .pos([
                    from + Vec2f::from(Self::DELTA_SHADOW),
                    to + Vec2f::from(Self::DELTA_SHADOW),
                ])
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
            .pos([
                Self::logo_pos(),
                Self::logo_pos().offset([tweak!(-120.0), tweak!(-6.0)]),
            ])
            .color([Color::OPAQUE, Color::TRANSPARENT]);

        for i in 0..3 {
            let pos = Self::choice_pos(i);

            // text
            b.node(&nodes.choices[i][1])
                .pos([pos, pos.offset([40.0, 20.0])])
                .alpha([255, 0]);

            // shadow
            let pos = pos + Vec2f::from(Self::DELTA_SHADOW);
            b.node(&nodes.choices[i][1])
                .pos([pos, pos.offset(Self::DELTA_SHADOW).offset([40.0, 20.0])])
                .alpha([255, 0]);
        }
    }
}
